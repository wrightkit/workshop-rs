//! Canonical validation owned by the Workshop action domain.

use crate::catalog::{Catalog, Kind};
use crate::core::error::WorkshopError;
use crate::wir;

pub(crate) fn validate_action(
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
                crate::values::validate::validate_call_signature(
                    entry, args, *span, program, catalog, errors,
                );
            }
            for arg in args {
                crate::values::validate::validate_value(program, catalog, *arg, errors);
            }
        }
        wir::Action::SetGlobalVariable { value, .. }
        | wir::Action::ModifyGlobalVariable { value, .. } => {
            crate::values::validate::validate_value(program, catalog, *value, errors);
        }
        wir::Action::SetPlayerVariable { player, value, .. }
        | wir::Action::ModifyPlayerVariable { player, value, .. } => {
            crate::values::validate::validate_value(program, catalog, *player, errors);
            crate::values::validate::validate_value(program, catalog, *value, errors);
        }
        wir::Action::AssignMember {
            target,
            value,
            span,
            ..
        } => {
            if !is_member_assignment_target(program, *target) {
                errors.push(WorkshopError::Malformed {
                    message: "AssignMember target must be a memberAccess value".to_string(),
                    span: *span,
                });
            }
            crate::values::validate::validate_value(program, catalog, *target, errors);
            crate::values::validate::validate_value(program, catalog, *value, errors);
        }
        wir::Action::If {
            branches,
            else_body,
            ..
        } => {
            for branch in branches {
                crate::values::validate::validate_value(program, catalog, branch.condition, errors);
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
            crate::values::validate::validate_value(program, catalog, *condition, errors);
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
            crate::values::validate::validate_value(program, catalog, *start, errors);
            crate::values::validate::validate_value(program, catalog, *stop, errors);
            crate::values::validate::validate_value(program, catalog, *step, errors);
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
            crate::values::validate::validate_value(program, catalog, *player, errors);
            crate::values::validate::validate_value(program, catalog, *start, errors);
            crate::values::validate::validate_value(program, catalog, *stop, errors);
            crate::values::validate::validate_value(program, catalog, *step, errors);
            for action in body {
                validate_action(program, catalog, *action, errors);
            }
        }
        wir::Action::CallSubroutine { .. } => {}
    }
}

pub(crate) fn is_member_assignment_target(program: &wir::Program, target: wir::ValueId) -> bool {
    match program.values.get(target) {
        Some(wir::ValueNode {
            value: wir::Value::Call { name, args },
            ..
        }) if name == "memberAccess" => (2..=3).contains(&args.len()),
        _ => false,
    }
}
