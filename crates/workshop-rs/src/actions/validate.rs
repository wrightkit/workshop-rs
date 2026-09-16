use crate::catalog::{Catalog, Kind};
use crate::core::error::{Result, WorkshopError};
use crate::wir;

pub(crate) fn validate_action(
    program: &wir::Program,
    catalog: &Catalog,
    action_id: wir::ActionId,
) -> Result<()> {
    let Some(action) = program.actions.get(action_id) else {
        return Ok(());
    };
    match action {
        wir::Action::Call { name, args, span } => {
            let Some(entry) = catalog.entry(Kind::Action, name) else {
                return Err(WorkshopError::Unknown {
                    kind: "action",
                    spelling: name.clone(),
                    locale: crate::catalog::Locale::new("en-US"),
                    span: *span,
                });
            };
            crate::values::validate::validate_call_signature(entry, args, *span, program, catalog)?;
            for arg in args {
                crate::values::validate::validate_value(program, catalog, *arg)?;
            }
        }
        wir::Action::SetGlobalVariable { value, .. }
        | wir::Action::ModifyGlobalVariable { value, .. } => {
            crate::values::validate::validate_value(program, catalog, *value)?;
        }
        wir::Action::SetPlayerVariable { player, value, .. }
        | wir::Action::ModifyPlayerVariable { player, value, .. } => {
            crate::values::validate::validate_value(program, catalog, *player)?;
            crate::values::validate::validate_value(program, catalog, *value)?;
        }
        wir::Action::AssignMember {
            target,
            value,
            span,
            ..
        } => {
            if !is_member_assignment_target(program, *target) {
                return Err(WorkshopError::Malformed {
                    message: "AssignMember target must be a memberAccess value".to_string(),
                    span: *span,
                });
            }
            crate::values::validate::validate_value(program, catalog, *target)?;
            crate::values::validate::validate_value(program, catalog, *value)?;
        }
        wir::Action::If {
            branches,
            else_body,
            ..
        } => {
            for branch in branches {
                crate::values::validate::validate_value(program, catalog, branch.condition)?;
                for action in &branch.body {
                    validate_action(program, catalog, *action)?;
                }
            }
            if let Some(else_body) = else_body {
                for action in else_body {
                    validate_action(program, catalog, *action)?;
                }
            }
        }
        wir::Action::While {
            condition, body, ..
        } => {
            crate::values::validate::validate_value(program, catalog, *condition)?;
            for action in body {
                validate_action(program, catalog, *action)?;
            }
        }
        wir::Action::ForGlobalVariable {
            start,
            stop,
            step,
            body,
            ..
        } => {
            crate::values::validate::validate_value(program, catalog, *start)?;
            crate::values::validate::validate_value(program, catalog, *stop)?;
            crate::values::validate::validate_value(program, catalog, *step)?;
            for action in body {
                validate_action(program, catalog, *action)?;
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
            crate::values::validate::validate_value(program, catalog, *player)?;
            crate::values::validate::validate_value(program, catalog, *start)?;
            crate::values::validate::validate_value(program, catalog, *stop)?;
            crate::values::validate::validate_value(program, catalog, *step)?;
            for action in body {
                validate_action(program, catalog, *action)?;
            }
        }
        wir::Action::CallSubroutine { .. } => {}
    }
    Ok(())
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
