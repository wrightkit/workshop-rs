use crate::catalog::{Catalog, Kind};
use crate::core::error::{Result, WorkshopError};
use crate::wir;

pub(crate) fn validate_value(
    program: &wir::Program,
    catalog: &Catalog,
    value_id: wir::ValueId,
) -> Result<()> {
    let Some(node) = program.values.get(value_id) else {
        return Ok(());
    };
    match &node.value {
        wir::Value::Call { name, args } => {
            if name == wir::AMBIGUOUS_ENUM_CALL {
                return validate_ambiguous_enum(program, catalog, value_id, node.span);
            }
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
                    | "min"
                    | "max"
                    | "raiseToPower"
                    | "appendToArray"
                    | "removeFromArray"
                    | "removeFromArrayByValue"
                    | "removeFromArrayByIndex"
            ) && (args.is_empty()
                || matches!(name.as_str(), "memberAccess" | "+" | "-" | "*" | "/" | "%"));
            let known = canonical_helper
                || catalog.entry(Kind::Value, name).is_some()
                || catalog.entry(Kind::Operator, name).is_some();
            if !known {
                return Err(WorkshopError::Unknown {
                    kind: "value",
                    spelling: name.clone(),
                    locale: crate::catalog::Locale::new("en-US"),
                    span: node.span,
                });
            } else if name == "memberAccess" {
                if !(2..=3).contains(&args.len()) {
                    return Err(WorkshopError::Malformed {
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
                    return Err(WorkshopError::Malformed {
                        message: "memberAccess member must be a string".to_string(),
                        span: node.span,
                    });
                }
            } else if !canonical_helper {
                if let Some(entry) = catalog.entry(Kind::Value, name) {
                    validate_call_signature(entry, args, node.span, program, catalog)?;
                }
            }
            for arg in args {
                validate_value(program, catalog, *arg)?;
            }
        }
        wir::Value::Enum {
            value_type, value, ..
        } => {
            if catalog.enum_domain(value_type).is_none() {
                return Err(WorkshopError::Unknown {
                    kind: "enum domain",
                    spelling: value_type.clone(),
                    locale: crate::catalog::Locale::new("en-US"),
                    span: node.span,
                });
            } else if catalog
                .enum_spelling(value_type, &crate::catalog::Locale::new("en-US"), value)
                .is_none()
            {
                return Err(WorkshopError::Unknown {
                    kind: "enum member",
                    spelling: value.clone(),
                    locale: crate::catalog::Locale::new("en-US"),
                    span: node.span,
                });
            }
        }
        wir::Value::Array(elements) => {
            for element in elements {
                validate_value(program, catalog, *element)?;
            }
        }
        wir::Value::Vector { x, y, z } => {
            validate_value(program, catalog, *x)?;
            validate_value(program, catalog, *y)?;
            validate_value(program, catalog, *z)?;
        }
        wir::Value::PlayerVariable { player, .. } => {
            validate_value(program, catalog, *player)?;
        }
        wir::Value::Subroutine(subroutine) => {
            if !program.subroutines.contains(*subroutine) {
                return Err(WorkshopError::Malformed {
                    message: format!("dangling subroutine value {}", subroutine.index()),
                    span: node.span,
                });
            }
        }
        wir::Value::Number { .. }
        | wir::Value::String(_)
        | wir::Value::LocalizedString(_)
        | wir::Value::Bool(_)
        | wir::Value::Null
        | wir::Value::GlobalVariable(_)
        | wir::Value::EventPlayer => {}
    }
    Ok(())
}

fn validate_ambiguous_enum(
    program: &wir::Program,
    catalog: &Catalog,
    value_id: wir::ValueId,
    span: Option<crate::source::Span>,
) -> Result<()> {
    let Some((_spelling, candidate_ids)) = wir::ambiguous_enum_parts(program, value_id) else {
        return Err(WorkshopError::Malformed {
            message: "ambiguous enum value has an invalid shape".to_string(),
            span,
        });
    };
    if candidate_ids.is_empty() {
        return Err(WorkshopError::Malformed {
            message: "ambiguous enum value has no candidates".to_string(),
            span,
        });
    }
    for candidate_id in candidate_ids {
        let Some(wir::ValueNode {
            value: wir::Value::Enum { value_type, value },
            ..
        }) = program.values.get(*candidate_id)
        else {
            return Err(WorkshopError::Malformed {
                message: "ambiguous enum candidate is not an enum value".to_string(),
                span,
            });
        };
        if catalog.enum_domain(value_type).is_none() {
            return Err(WorkshopError::Unknown {
                kind: "enum domain",
                spelling: value_type.clone(),
                locale: crate::catalog::Locale::new("en-US"),
                span,
            });
        } else if catalog
            .enum_spelling(value_type, &crate::catalog::Locale::new("en-US"), value)
            .is_none()
        {
            return Err(WorkshopError::Unknown {
                kind: "enum member",
                spelling: format!("{value_type}.{value}"),
                locale: crate::catalog::Locale::new("en-US"),
                span,
            });
        }
    }
    Ok(())
}

pub(crate) fn validate_call_signature(
    entry: &crate::catalog::CatalogEntry,
    args: &[wir::ValueId],
    span: Option<crate::core::source::Span>,
    program: &wir::Program,
    catalog: &Catalog,
) -> Result<()> {
    // An empty signature in the current inventory means that arity is not
    // declared, not that the builtin is a zero-argument function. This is
    // important for documented variadic calls such as Custom String.
    if entry.param_count() == 0 && entry.required_param_count() == 0 {
        return Ok(());
    }
    // Trailing defaults may make a signature partial, but every supplied
    // argument is still checked against its declared position.
    if (args.is_empty() && entry.required_param_count() > 0)
        || (!entry.variadic && args.len() > entry.param_count())
    {
        return Err(WorkshopError::Unsupported {
            message: format!(
                "{} '{}' expects {}..{}{} argument(s), got {}",
                entry.kind.as_str(),
                entry.id,
                entry.required_param_count(),
                entry.param_count(),
                if entry.variadic { "+" } else { "" },
                args.len()
            ),
            span,
        });
    }

    for (index, arg_id) in args.iter().enumerate() {
        if entry.id == "string"
            && index == 0
            && !matches!(
                program.values.get(*arg_id).map(|node| &node.value),
                Some(wir::Value::LocalizedString(_))
            )
        {
            return Err(WorkshopError::Unsupported {
                message: "value 'string' argument 1 must be localized string text".to_string(),
                span: program.values.get(*arg_id).and_then(|node| node.span),
            });
        }
        if let Some(expected) = entry.param_type(index) {
            if !value_matches_type(program, catalog, *arg_id, expected)
                && !contextual_value_matches(entry, index, program, *arg_id, expected)
            {
                let actual = value_type_name(program, catalog, *arg_id);
                return Err(WorkshopError::Unsupported {
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
                value_type == domain
                    && catalog
                        .enum_spelling(domain, catalog.primary_locale(), value)
                        .is_some()
            }
            wir::Value::Call { name, .. } if name == wir::AMBIGUOUS_ENUM_CALL => {
                wir::ambiguous_enum_parts(program, *arg_id).is_some_and(|(_, candidate_ids)| {
                    candidate_ids.iter().any(|candidate_id| {
                        matches!(
                            program.values.get(*candidate_id),
                            Some(wir::ValueNode {
                                value: wir::Value::Enum { value_type, value },
                                ..
                            }) if value_type == domain
                                && catalog
                                    .enum_spelling(domain, catalog.primary_locale(), value)
                                    .is_some()
                        )
                    })
                })
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
                wir::Value::Call { name, .. } if name == wir::AMBIGUOUS_ENUM_CALL => {
                    wir::ambiguous_enum_parts(program, *arg_id)
                        .map(|(spelling, _)| format!("ambiguous '{spelling}'"))
                        .unwrap_or_else(|| "malformed ambiguous enum".to_string())
                }
                _ => "non-enum expression".to_string(),
            };
            return Err(WorkshopError::Unsupported {
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
    Ok(())
}

fn contextual_value_matches(
    entry: &crate::catalog::CatalogEntry,
    index: usize,
    program: &wir::Program,
    value_id: wir::ValueId,
    expected: &str,
) -> bool {
    let Some(coercions) = entry.param_coercions(index) else {
        return false;
    };
    let Some(node) = program.values.get(value_id) else {
        return false;
    };
    let expected_number = expected
        .split('|')
        .any(|alternative| matches!(alternative, "Number" | "Any" | "Unknown"));
    let expected_string = expected
        .split('|')
        .any(|alternative| matches!(alternative, "String" | "Text"));
    match &node.value {
        wir::Value::Bool(false) => coercions.false_as_number && expected_number,
        wir::Value::Bool(true) => coercions.true_as_number && expected_number,
        wir::Value::Number { value, .. } => coercions.zero_as_null && *value == 0.0,
        wir::Value::Vector { x, y, z } => {
            coercions.null_vector_as_null
                && is_zero_number(program, *x)
                && is_zero_number(program, *y)
                && is_zero_number(program, *z)
        }
        wir::Value::Call { name, args } => {
            (coercions.null_vector_as_null
                && name == "vector"
                && args.len() == 3
                && args
                    .iter()
                    .all(|value_id| is_zero_number(program, *value_id)))
                || (coercions.empty_array_as_string
                    && expected_string
                    && name == "emptyArray"
                    && args.is_empty())
        }
        wir::Value::Array(elements) => {
            coercions.empty_array_as_string && expected_string && elements.is_empty()
        }
        _ => false,
    }
}

fn is_zero_number(program: &wir::Program, value_id: wir::ValueId) -> bool {
    matches!(
        program.values.get(value_id),
        Some(wir::ValueNode {
            value: wir::Value::Number { value, .. },
            ..
        }) if *value == 0.0
    )
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
    if expected
        .split('|')
        .any(|alternative| alternative == "Player Variable")
    {
        if let wir::Value::PlayerVariable { player, .. } = &node.value {
            if !value_matches_type(program, catalog, *player, "Player|Array") {
                return false;
            }
        }
    }
    if let wir::Value::Call { name, .. } = &node.value {
        if name == wir::AMBIGUOUS_ENUM_CALL {
            return expected.split('|').any(|alternative| {
                matches!(alternative, "Any" | "Unknown" | "Object")
                    || wir::ambiguous_enum_parts(program, value_id).is_some_and(
                        |(_, candidate_ids)| {
                            candidate_ids.iter().any(|candidate_id| {
                                matches!(
                                    program.values.get(*candidate_id),
                                    Some(wir::ValueNode {
                                        value: wir::Value::Enum { value_type, .. },
                                        ..
                                    }) if value_type == alternative
                                )
                            })
                        },
                    )
            });
        }
    }
    expected
        .split('|')
        .any(|alternative| value_matches_single_type(catalog, &node.value, alternative))
}

fn value_matches_single_type(catalog: &Catalog, value: &wir::Value, expected: &str) -> bool {
    match (value, expected) {
        (_, "Any" | "Unknown") => true,
        (wir::Value::Number { .. }, "Number") => true,
        (wir::Value::String(_) | wir::Value::LocalizedString(_), "String" | "Text") => true,
        (wir::Value::Bool(_), "Boolean") => true,
        (wir::Value::Vector { .. }, "Vector") => true,
        (wir::Value::Array(_), "Array") => true,
        (
            wir::Value::Number { .. }
            | wir::Value::String(_)
            | wir::Value::LocalizedString(_)
            | wir::Value::Bool(_)
            | wir::Value::Vector { .. },
            "Object",
        ) => true,
        (wir::Value::Enum { value_type, .. }, domain) => {
            matches!(domain, "Any" | "Unknown" | "Object") || value_type == domain
        }
        (wir::Value::Call { name, .. }, expected) => {
            if expected == "Operation"
                && matches!(
                    name.as_str(),
                    "add"
                        | "subtract"
                        | "multiply"
                        | "divide"
                        | "modulo"
                        | "min"
                        | "max"
                        | "raiseToPower"
                        | "appendToArray"
                        | "removeFromArray"
                        | "removeFromArrayByValue"
                        | "removeFromArrayByIndex"
                )
            {
                return true;
            }
            catalog
                .entry(crate::catalog::Kind::Value, name)
                .and_then(|entry| entry.return_type())
                .is_none_or(|return_type| {
                    return_type
                        .split('|')
                        .any(|actual| semantic_types_compatible(actual, expected))
                })
        }
        // Null is a valid Workshop placeholder for every value contract;
        // its runtime meaning is resolved by the enclosing builtin.
        (wir::Value::Null, _) => true,
        (wir::Value::GlobalVariable(_), "Global Variable") => true,
        (wir::Value::PlayerVariable { .. }, "Player Variable") => true,
        (wir::Value::Subroutine(_), "Subroutine") => true,
        (wir::Value::EventPlayer, "Player") => true,
        // Variables and other runtime expressions are intentionally accepted
        // for value contracts whose runtime contents are not statically
        // knowable, but their statically known reference category must not be
        // coerced into another variable/reference category.
        (
            wir::Value::GlobalVariable(_)
            | wir::Value::PlayerVariable { .. }
            | wir::Value::Subroutine(_)
            | wir::Value::EventPlayer,
            expected,
        ) => !matches!(
            expected,
            "Global Variable" | "Player Variable" | "Subroutine"
        ),
        _ => false,
    }
}

fn semantic_types_compatible(actual: &str, expected: &str) -> bool {
    matches!(actual, "Any" | "Unknown")
        || matches!(expected, "Any" | "Unknown")
        || actual == expected
        || actual == "Object"
        || (expected == "Object" && actual != "Array" && actual != "Void")
        || (actual == "Object" && expected == "Object")
}

fn value_type_name(program: &wir::Program, catalog: &Catalog, value_id: wir::ValueId) -> String {
    let Some(node) = program.values.get(value_id) else {
        return "missing".to_string();
    };
    match &node.value {
        wir::Value::Number { .. } => "Number".to_string(),
        wir::Value::String(_) | wir::Value::LocalizedString(_) => "String".to_string(),
        wir::Value::Bool(_) => "Boolean".to_string(),
        wir::Value::Vector { .. } => "Vector".to_string(),
        wir::Value::Array(_) => "Array".to_string(),
        wir::Value::Enum { value_type, .. } => value_type.clone(),
        wir::Value::Call { name, .. } if name == wir::AMBIGUOUS_ENUM_CALL => {
            "AmbiguousEnum".to_string()
        }
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
