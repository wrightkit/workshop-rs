//! Catalog-backed validation of Workshop-origin WIR.
//!
//! WIR builtin references are locale-independent canonical catalog ids. This
//! module validates a Workshop IR program against the canonical catalog so an
//! unknown, misspelled, or locale-tainted builtin is rejected deterministically
//! instead of being stored as opaque unchecked text.

use crate::wir;

use crate::catalog::{Catalog, Kind};
use crate::error::{Result, WorkshopError};

/// Validate every builtin reference in a Workshop-origin WIR program against
/// the catalog: action/value call names must be known canonical ids, and enum
/// references must resolve to a canonical member of a known domain.
pub fn validate_canonical_ids(program: &wir::Program, catalog: &Catalog) -> Result<()> {
    let mut errors = Vec::new();
    for (index, _) in program.rules.iter().enumerate() {
        let rule = wir::RuleId::from_index(index);
        let Some(rule_data) = program.rules.get(rule) else {
            continue;
        };
        validate_event(&rule_data.event, rule_data.span, catalog, &mut errors);
        for action in &rule_data.actions {
            validate_action(program, catalog, *action, &mut errors);
        }
        for condition in &rule_data.conditions {
            validate_value(program, catalog, *condition, &mut errors);
        }
    }
    errors.into_iter().next().map_or(Ok(()), Err)
}

fn validate_event(
    event: &wir::Event,
    span: Option<crate::source::Span>,
    catalog: &Catalog,
    errors: &mut Vec<WorkshopError>,
) {
    let (id, filters) = match event {
        wir::Event::Global => ("global", None),
        wir::Event::EachPlayer => ("eachPlayer", None),
        wir::Event::EachPlayerWithFilters { team, target } => ("eachPlayer", Some((*team, target))),
        wir::Event::Player { kind, team, target } => (kind.catalog_id(), Some((*team, target))),
        wir::Event::Subroutine(_) => ("subroutine", None),
    };
    if catalog.entry(Kind::Event, id).is_none() {
        errors.push(WorkshopError::Unknown {
            kind: "event",
            spelling: id.to_string(),
            locale: crate::catalog::Locale::new("en-US"),
            span,
        });
        return;
    }
    let Some((team, target)) = filters else {
        return;
    };
    let en = crate::catalog::Locale::new("en-US");
    let team_member = match team {
        wir::EventTeam::All => "ALL",
        wir::EventTeam::Team1 => "TEAM_1",
        wir::EventTeam::Team2 => "TEAM_2",
    };
    if catalog
        .enum_spelling("EventTeam", &en, team_member)
        .is_none()
    {
        errors.push(WorkshopError::Unknown {
            kind: "event team",
            spelling: team_member.to_string(),
            locale: en.clone(),
            span,
        });
    }
    let target_member = match target {
        wir::EventTarget::All => Some("ALL".to_string()),
        wir::EventTarget::Slot(slot) => Some(format!("SLOT_{slot}")),
        wir::EventTarget::Hero(hero) => {
            if catalog.enum_spelling("Hero", &en, hero).is_none() {
                errors.push(WorkshopError::Unknown {
                    kind: "event player",
                    spelling: hero.clone(),
                    locale: en.clone(),
                    span,
                });
            }
            None
        }
    };
    if let Some(target_member) = target_member {
        if catalog
            .enum_spelling("EventPlayer", &en, &target_member)
            .is_none()
        {
            errors.push(WorkshopError::Unknown {
                kind: "event player",
                spelling: target_member,
                locale: en,
                span,
            });
        }
    }
}

fn validate_action(
    program: &wir::Program,
    catalog: &Catalog,
    action_id: wir::ActionId,
    errors: &mut Vec<WorkshopError>,
) {
    let Some(action) = program.actions.get(action_id) else {
        return;
    };
    match action {
        wir::Action::Call { name, args, span } => {
            let entry = catalog.entry(Kind::Action, name);
            if entry.is_none() {
                errors.push(WorkshopError::Unknown {
                    kind: "action",
                    spelling: name.clone(),
                    locale: crate::catalog::Locale::new("en-US"),
                    span: *span,
                });
            } else if let Some(entry) = entry {
                validate_call_signature(entry, args, *span, program, catalog, errors);
            }
            for arg in args {
                validate_value(program, catalog, *arg, errors);
            }
        }
        wir::Action::SetGlobalVariable { value, .. }
        | wir::Action::ModifyGlobalVariable { value, .. }
        | wir::Action::Debug { value, .. }
        | wir::Action::Print { message: value, .. } => {
            validate_value(program, catalog, *value, errors);
        }
        wir::Action::SetPlayerVariable { player, value, .. }
        | wir::Action::ModifyPlayerVariable { player, value, .. } => {
            validate_value(program, catalog, *player, errors);
            validate_value(program, catalog, *value, errors);
        }
        wir::Action::AssignMember { target, value, .. } => {
            validate_value(program, catalog, *target, errors);
            validate_value(program, catalog, *value, errors);
        }
        wir::Action::If {
            branches,
            else_body,
            ..
        } => {
            for branch in branches {
                validate_value(program, catalog, branch.condition, errors);
                for action in &branch.body {
                    validate_action(program, catalog, *action, errors);
                }
            }
            if let Some(else_body) = else_body {
                for action in else_body {
                    validate_action(program, catalog, *action, errors);
                }
            }
        }
        wir::Action::While {
            condition, body, ..
        } => {
            validate_value(program, catalog, *condition, errors);
            for action in body {
                validate_action(program, catalog, *action, errors);
            }
        }
        wir::Action::ForGlobalVariable {
            start,
            stop,
            step,
            body,
            ..
        } => {
            validate_value(program, catalog, *start, errors);
            validate_value(program, catalog, *stop, errors);
            validate_value(program, catalog, *step, errors);
            for action in body {
                validate_action(program, catalog, *action, errors);
            }
        }
        wir::Action::ForPlayerVariable {
            player,
            start,
            stop,
            step,
            body,
            ..
        } => {
            validate_value(program, catalog, *player, errors);
            validate_value(program, catalog, *start, errors);
            validate_value(program, catalog, *stop, errors);
            validate_value(program, catalog, *step, errors);
            for action in body {
                validate_action(program, catalog, *action, errors);
            }
        }
        wir::Action::CallSubroutine { .. } => {}
    }
}

fn validate_value(
    program: &wir::Program,
    catalog: &Catalog,
    value_id: wir::ValueId,
    errors: &mut Vec<WorkshopError>,
) {
    let Some(node) = program.values.get(value_id) else {
        return;
    };
    match &node.value {
        wir::Value::Call { name, args } => {
            // Comparison operators are represented as call names (`==`, `<`,
            // …) following the `Compare(a, op, b)` convention, so both value
            // and operator identities are valid call names.
            let canonical_helper = matches!(
                name.as_str(),
                "memberAccess"
                    | "+"
                    | "-"
                    | "*"
                    | "/"
                    | "%"
                    | "add"
                    | "subtract"
                    | "multiply"
                    | "divide"
                    | "modulo"
                    | "raiseToPower"
                    | "appendToArray"
                    | "removeFromArray"
                    | "removeFromArrayByIndex"
            ) && (args.is_empty()
                || matches!(name.as_str(), "memberAccess" | "+" | "-" | "*" | "/" | "%"));
            let known = canonical_helper
                || catalog.entry(Kind::Value, name).is_some()
                || catalog.entry(Kind::Operator, name).is_some();
            if !known {
                errors.push(WorkshopError::Unknown {
                    kind: "value",
                    spelling: name.clone(),
                    locale: crate::catalog::Locale::new("en-US"),
                    span: node.span,
                });
            } else if name == "memberAccess" {
                if !(2..=3).contains(&args.len()) {
                    errors.push(WorkshopError::Malformed {
                        message: "memberAccess expects two or three arguments".to_string(),
                        span: node.span,
                    });
                } else if !matches!(
                    program.values.get(args[1]),
                    Some(wir::ValueNode {
                        value: wir::Value::String(_),
                        ..
                    })
                ) {
                    errors.push(WorkshopError::Malformed {
                        message: "memberAccess member must be a string".to_string(),
                        span: node.span,
                    });
                }
            } else if !canonical_helper {
                if let Some(entry) = catalog.entry(Kind::Value, name) {
                    validate_call_signature(entry, args, node.span, program, catalog, errors);
                }
            }
            for arg in args {
                validate_value(program, catalog, *arg, errors);
            }
        }
        wir::Value::Enum {
            value_type, value, ..
        } => {
            if catalog.enum_domain(value_type).is_none() {
                errors.push(WorkshopError::Unknown {
                    kind: "enum domain",
                    spelling: value_type.clone(),
                    locale: crate::catalog::Locale::new("en-US"),
                    span: node.span,
                });
            } else if catalog
                .enum_spelling(value_type, &crate::catalog::Locale::new("en-US"), value)
                .is_none()
            {
                errors.push(WorkshopError::Unknown {
                    kind: "enum member",
                    spelling: value.clone(),
                    locale: crate::catalog::Locale::new("en-US"),
                    span: node.span,
                });
            }
        }
        wir::Value::Array(elements) => {
            for element in elements {
                validate_value(program, catalog, *element, errors);
            }
        }
        wir::Value::Vector { x, y, z } => {
            validate_value(program, catalog, *x, errors);
            validate_value(program, catalog, *y, errors);
            validate_value(program, catalog, *z, errors);
        }
        wir::Value::PlayerVariable { player, .. } => {
            validate_value(program, catalog, *player, errors);
        }
        wir::Value::Subroutine(subroutine) => {
            if !program.subroutines.contains(*subroutine) {
                errors.push(WorkshopError::Malformed {
                    message: format!("dangling subroutine value {}", subroutine.index()),
                    span: node.span,
                });
            }
        }
        wir::Value::Number { .. }
        | wir::Value::String(_)
        | wir::Value::Bool(_)
        | wir::Value::Null
        | wir::Value::GlobalVariable(_)
        | wir::Value::EventPlayer => {}
    }
}

fn validate_call_signature(
    entry: &crate::catalog::CatalogEntry,
    args: &[wir::ValueId],
    span: Option<crate::source::Span>,
    program: &wir::Program,
    catalog: &Catalog,
    errors: &mut Vec<WorkshopError>,
) {
    // An empty signature in the current inventory means that arity is not
    // declared, not that the builtin is a zero-argument function. This is
    // important for documented variadic calls such as Custom String.
    if entry.param_count() == 0 && entry.required_param_count() == 0 {
        return;
    }
    // Existing WIR callers may intentionally construct a partial generic
    // call while probing one argument's semantics. Reject the unambiguous
    // boundary violations (no arguments for a required signature, or too
    // many arguments) while leaving partial calls available for the explicit
    // evidence-insufficient path.
    if (args.is_empty() && entry.required_param_count() > 0) || args.len() > entry.param_count() {
        errors.push(WorkshopError::Unsupported {
            message: format!(
                "{} '{}' expects {}..{} argument(s), got {}",
                entry.kind.as_str(),
                entry.id,
                entry.required_param_count(),
                entry.param_count(),
                args.len()
            ),
            span,
        });
        return;
    }

    let complete_signature = args.len() == entry.param_count();
    for (index, arg_id) in args.iter().enumerate() {
        if let Some(expected) = entry.param_type(index) {
            if !value_matches_type(program, catalog, *arg_id, expected) {
                let actual = value_type_name(program, catalog, *arg_id);
                errors.push(WorkshopError::Unsupported {
                    message: format!(
                        "{} '{}' argument {} must have semantic type '{}', got {}",
                        entry.kind.as_str(),
                        entry.id,
                        index + 1,
                        expected,
                        actual
                    ),
                    span: program.values.get(*arg_id).and_then(|node| node.span),
                });
            }
        }
        let Some(domain) = entry.param_domain(index) else {
            continue;
        };
        let Some(node) = program.values.get(*arg_id) else {
            continue;
        };
        // A declared enum domain constrains enum literals. Dynamic values,
        // Null, and defaults remain valid expressions for the same position;
        // their runtime value cannot be proven from WIR alone.
        let valid = match &node.value {
            wir::Value::Enum {
                value_type, value, ..
            } => {
                let member_is_known = catalog
                    .enum_spelling(value_type, catalog.primary_locale(), value)
                    .is_some();
                if complete_signature {
                    value_type == domain
                        && catalog
                            .enum_spelling(domain, catalog.primary_locale(), value)
                            .is_some()
                } else {
                    // Partial generic calls do not provide enough evidence
                    // to prove the positional domain, but an explicitly
                    // unknown enum literal must still fail closed.
                    member_is_known
                }
            }
            _ => true,
        };
        if !valid {
            let actual = match &node.value {
                wir::Value::Enum {
                    value_type, value, ..
                } => {
                    format!("{value_type}.{value}")
                }
                _ => "non-enum expression".to_string(),
            };
            errors.push(WorkshopError::Unsupported {
                message: format!(
                    "{} '{}' argument {} must be a member of enum domain '{}', got {}",
                    entry.kind.as_str(),
                    entry.id,
                    index + 1,
                    domain,
                    actual
                ),
                span: node.span,
            });
        }
    }
}

fn value_matches_type(
    program: &wir::Program,
    catalog: &Catalog,
    value_id: wir::ValueId,
    expected: &str,
) -> bool {
    let Some(node) = program.values.get(value_id) else {
        return false;
    };
    match (&node.value, expected) {
        (_, "Any" | "Unknown") => true,
        (wir::Value::Number { .. }, "Number") => true,
        (wir::Value::String(_), "String" | "Text") => true,
        (wir::Value::Bool(_), "Boolean") => true,
        (wir::Value::Vector { .. }, "Vector") => true,
        (wir::Value::Array(_), "Array") => true,
        (wir::Value::Enum { value_type, .. }, domain) => value_type == domain,
        (wir::Value::Call { name, .. }, expected) => catalog
            .entry(crate::catalog::Kind::Value, name)
            .and_then(|entry| entry.return_type())
            .is_some_and(|return_type| return_type == expected),
        // Variables and other runtime expressions are intentionally accepted
        // only for broad contracts; their value is not statically knowable.
        (
            wir::Value::GlobalVariable(_)
            | wir::Value::PlayerVariable { .. }
            | wir::Value::Subroutine(_)
            | wir::Value::EventPlayer
            | wir::Value::Null,
            _,
        ) => matches!(expected, "Any" | "Object" | "Player" | "Variable"),
        _ => false,
    }
}

fn value_type_name(program: &wir::Program, catalog: &Catalog, value_id: wir::ValueId) -> String {
    let Some(node) = program.values.get(value_id) else {
        return "missing".to_string();
    };
    match &node.value {
        wir::Value::Number { .. } => "Number".to_string(),
        wir::Value::String(_) => "String".to_string(),
        wir::Value::Bool(_) => "Boolean".to_string(),
        wir::Value::Vector { .. } => "Vector".to_string(),
        wir::Value::Array(_) => "Array".to_string(),
        wir::Value::Enum { value_type, .. } => value_type.clone(),
        wir::Value::Call { name, .. } => catalog
            .entry(crate::catalog::Kind::Value, name)
            .and_then(|entry| entry.return_type())
            .unwrap_or("dynamic")
            .to_string(),
        wir::Value::Null => "Null".to_string(),
        wir::Value::GlobalVariable(_) | wir::Value::PlayerVariable { .. } => "Variable".to_string(),
        wir::Value::Subroutine(_) => "Subroutine".to_string(),
        wir::Value::EventPlayer => "Player".to_string(),
    }
}
