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
            if catalog.entry(Kind::Action, name).is_none() {
                errors.push(WorkshopError::Unknown {
                    kind: "action",
                    spelling: name.clone(),
                    locale: crate::catalog::Locale::new("en-US"),
                    span: *span,
                });
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
            let known = catalog.entry(Kind::Value, name).is_some()
                || catalog.entry(Kind::Operator, name).is_some();
            if !known {
                errors.push(WorkshopError::Unknown {
                    kind: "value",
                    spelling: name.clone(),
                    locale: crate::catalog::Locale::new("en-US"),
                    span: node.span,
                });
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
        wir::Value::Number { .. }
        | wir::Value::String(_)
        | wir::Value::Bool(_)
        | wir::Value::Null
        | wir::Value::GlobalVariable(_)
        | wir::Value::EventPlayer => {}
    }
}
