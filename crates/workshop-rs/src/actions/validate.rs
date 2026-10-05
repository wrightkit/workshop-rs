use crate::catalog::{Catalog, Kind};
use crate::core::error::{Result, WorkshopError};
use crate::wir;

pub(crate) fn validate_action(
    program: &wir::Program,
    catalog: &Catalog,
    action_id: wir::ActionId,
    tolerate_residuals: bool,
) -> Result<()> {
    let Some(action) = program.actions.get(action_id) else {
        return Ok(());
    };
    match action {
        wir::Action::Call { name, args, span } => {
            let Some(entry) = catalog.entry(Kind::Action, name) else {
                // A catalog-unknown call is a completeness residual reported
                // by `semantic_issues`; its signature is unknowable, but its
                // argument values are still validated so a nested known
                // violation is not masked.
                if tolerate_residuals {
                    for arg in args {
                        crate::values::validate::validate_value(
                            program,
                            catalog,
                            *arg,
                            tolerate_residuals,
                        )?;
                    }
                    return Ok(());
                }
                return Err(WorkshopError::unknown(
                    "action",
                    name.clone(),
                    crate::catalog::Locale::new("en-US"),
                    *span,
                ));
            };
            crate::values::validate::validate_call_signature(entry, args, *span, program, catalog)?;
            for arg in args {
                crate::values::validate::validate_value(
                    program,
                    catalog,
                    *arg,
                    tolerate_residuals,
                )?;
            }
        }
        wir::Action::SetGlobalVariable { value, .. }
        | wir::Action::ModifyGlobalVariable { value, .. } => {
            crate::values::validate::validate_value(program, catalog, *value, tolerate_residuals)?;
        }
        wir::Action::SetPlayerVariable { player, value, .. }
        | wir::Action::ModifyPlayerVariable { player, value, .. } => {
            crate::values::validate::validate_value(program, catalog, *player, tolerate_residuals)?;
            crate::values::validate::validate_value(program, catalog, *value, tolerate_residuals)?;
        }
        wir::Action::AssignMember {
            target,
            value,
            span,
            ..
        } => {
            if !is_member_assignment_target(program, *target) {
                return Err(WorkshopError::malformed(
                    "AssignMember target must be a memberAccess value".to_string(),
                    *span,
                ));
            }
            crate::values::validate::validate_value(program, catalog, *target, tolerate_residuals)?;
            crate::values::validate::validate_value(program, catalog, *value, tolerate_residuals)?;
        }
        wir::Action::If {
            branches,
            else_body,
            ..
        } => {
            for branch in branches {
                crate::values::validate::validate_value(
                    program,
                    catalog,
                    branch.condition,
                    tolerate_residuals,
                )?;
                for action in &branch.body {
                    validate_action(program, catalog, *action, tolerate_residuals)?;
                }
            }
            if let Some(else_body) = else_body {
                for action in else_body {
                    validate_action(program, catalog, *action, tolerate_residuals)?;
                }
            }
        }
        wir::Action::While {
            condition, body, ..
        } => {
            crate::values::validate::validate_value(
                program,
                catalog,
                *condition,
                tolerate_residuals,
            )?;
            for action in body {
                validate_action(program, catalog, *action, tolerate_residuals)?;
            }
        }
        wir::Action::ForGlobalVariable {
            start,
            stop,
            step,
            body,
            ..
        } => {
            crate::values::validate::validate_value(program, catalog, *start, tolerate_residuals)?;
            crate::values::validate::validate_value(program, catalog, *stop, tolerate_residuals)?;
            crate::values::validate::validate_value(program, catalog, *step, tolerate_residuals)?;
            for action in body {
                validate_action(program, catalog, *action, tolerate_residuals)?;
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
            crate::values::validate::validate_value(program, catalog, *player, tolerate_residuals)?;
            crate::values::validate::validate_value(program, catalog, *start, tolerate_residuals)?;
            crate::values::validate::validate_value(program, catalog, *stop, tolerate_residuals)?;
            crate::values::validate::validate_value(program, catalog, *step, tolerate_residuals)?;
            for action in body {
                validate_action(program, catalog, *action, tolerate_residuals)?;
            }
        }
        wir::Action::Disabled { action, .. } => {
            validate_action(program, catalog, *action, tolerate_residuals)?;
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
