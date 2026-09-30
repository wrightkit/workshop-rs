use super::super::*;
use super::{malformed_id, wir_value_children};
use crate::core::error::Result;
use crate::wir;

impl Program {
    pub(crate) fn from_wir(storage: wir::Program) -> Result<Self> {
        let mut program = Self {
            settings: storage.settings.clone(),
            global_variables: storage
                .global_variables
                .iter()
                .map(|variable| Variable::with_index(variable.name.clone(), variable.index))
                .collect(),
            player_variables: storage
                .player_variables
                .iter()
                .map(|variable| Variable::with_index(variable.name.clone(), variable.index))
                .collect(),
            subroutines: storage
                .subroutines
                .iter()
                .map(|subroutine| Subroutine::with_index(subroutine.name.clone(), subroutine.index))
                .collect(),
            rules: Vec::with_capacity(storage.rules.len()),
            files: storage.files.iter().cloned().collect(),
            provenance: Some(Box::new(ProgramProvenance {
                global_variables: storage
                    .global_variables
                    .iter()
                    .map(|variable| DeclarationProvenance {
                        span: variable.span,
                        name_span: variable.name_span,
                    })
                    .collect(),
                player_variables: storage
                    .player_variables
                    .iter()
                    .map(|variable| DeclarationProvenance {
                        span: variable.span,
                        name_span: variable.name_span,
                    })
                    .collect(),
                subroutines: storage
                    .subroutines
                    .iter()
                    .map(|subroutine| DeclarationProvenance {
                        span: subroutine.span,
                        name_span: subroutine.name_span,
                    })
                    .collect(),
                rules: Vec::with_capacity(storage.rules.len()),
            })),
        };
        for rule in storage.rules.iter() {
            let event = public_event(&storage, &rule.event)?;
            let conditions = rule
                .conditions
                .iter()
                .map(|condition| {
                    Ok(Condition {
                        value: public_value(&storage, condition.value)?,
                        disabled: condition.disabled,
                    })
                })
                .collect::<Result<Vec<_>>>()?;
            let mut actions = Vec::new();
            let mut action_provenance = Vec::new();
            for action in &rule.actions {
                public_actions(&storage, *action, &mut actions)?;
                public_action_provenance(&storage, *action, &mut action_provenance)?;
            }
            let event_name = match &rule.event {
                wir::Event::Subroutine { name_span, .. } => *name_span,
                _ => None,
            };
            program
                .provenance
                .as_mut()
                .expect("parsed programs retain provenance")
                .rules
                .push(RuleProvenance {
                    span: rule.span,
                    name: rule.name_span,
                    event_name,
                    conditions: rule
                        .conditions
                        .iter()
                        .map(|condition| value_provenance(&storage, condition.value))
                        .collect(),
                    actions: action_provenance,
                });
            program.rules.push(Rule {
                name: rule.name.clone(),
                disabled: rule.disabled,
                event,
                conditions,
                actions,
            });
        }
        Ok(program)
    }
}

fn public_event(storage: &wir::Program, event: &wir::Event) -> Result<Event> {
    Ok(match event {
        wir::Event::Global => Event::Global,
        wir::Event::EachPlayer => Event::EachPlayer,
        wir::Event::EachPlayerWithFilters { team, target } => Event::EachPlayerWithFilters {
            team: public_team(*team),
            target: public_target(target),
        },
        wir::Event::Player { kind, team, target } => Event::Player {
            kind: public_player_event(*kind),
            team: public_team(*team),
            target: public_target(target),
        },
        wir::Event::Subroutine { subroutine, .. } => Event::Subroutine(
            storage
                .subroutines
                .get(*subroutine)
                .ok_or_else(|| malformed_id("subroutine", subroutine.index()))?
                .name
                .clone(),
        ),
    })
}

fn public_team(team: wir::EventTeam) -> EventTeam {
    match team {
        wir::EventTeam::All => EventTeam::All,
        wir::EventTeam::Team1 => EventTeam::Team1,
        wir::EventTeam::Team2 => EventTeam::Team2,
    }
}

fn public_target(target: &wir::EventTarget) -> EventTarget {
    match target {
        wir::EventTarget::All => EventTarget::All,
        wir::EventTarget::Slot(slot) => EventTarget::Slot(*slot),
        wir::EventTarget::Hero(hero) => EventTarget::Hero(hero.clone()),
    }
}

fn public_player_event(kind: wir::PlayerEventKind) -> PlayerEventKind {
    match kind {
        wir::PlayerEventKind::DealtDamage => PlayerEventKind::DealtDamage,
        wir::PlayerEventKind::DealtFinalBlow => PlayerEventKind::DealtFinalBlow,
        wir::PlayerEventKind::DealtHealing => PlayerEventKind::DealtHealing,
        wir::PlayerEventKind::DealtKnockback => PlayerEventKind::DealtKnockback,
        wir::PlayerEventKind::Died => PlayerEventKind::Died,
        wir::PlayerEventKind::EarnedElimination => PlayerEventKind::EarnedElimination,
        wir::PlayerEventKind::Joined => PlayerEventKind::Joined,
        wir::PlayerEventKind::Left => PlayerEventKind::Left,
        wir::PlayerEventKind::ReceivedHealing => PlayerEventKind::ReceivedHealing,
        wir::PlayerEventKind::ReceivedKnockback => PlayerEventKind::ReceivedKnockback,
        wir::PlayerEventKind::TookDamage => PlayerEventKind::TookDamage,
    }
}

fn public_value(storage: &wir::Program, id: wir::ValueId) -> Result<Value> {
    let node = storage
        .values
        .get(id)
        .ok_or_else(|| malformed_id("value", id.index()))?;
    Ok(match &node.value {
        wir::Value::Number { value, .. } => Value::Number(*value),
        wir::Value::String(value) => Value::String(value.clone()),
        wir::Value::LocalizedString(value) => Value::LocalizedString(value.clone()),
        wir::Value::Bool(value) => Value::Bool(*value),
        wir::Value::Null => Value::Null,
        wir::Value::Array(values) => Value::Array(
            values
                .iter()
                .map(|value| public_value(storage, *value))
                .collect::<Result<Vec<_>>>()?,
        ),
        wir::Value::Vector { x, y, z } => Value::Vector {
            x: Box::new(public_value(storage, *x)?),
            y: Box::new(public_value(storage, *y)?),
            z: Box::new(public_value(storage, *z)?),
        },
        wir::Value::Enum { value_type, value } => Value::Enum {
            value_type: value_type.clone(),
            value: value.clone(),
        },
        wir::Value::GlobalVariable(id) => Value::GlobalVariable(
            storage
                .global_variables
                .get(*id)
                .ok_or_else(|| malformed_id("global variable", id.index()))?
                .name
                .clone(),
        ),
        wir::Value::PlayerVariable { player, variable } => Value::PlayerVariable {
            player: Box::new(public_value(storage, *player)?),
            variable: storage
                .player_variables
                .get(*variable)
                .ok_or_else(|| malformed_id("player variable", variable.index()))?
                .name
                .clone(),
        },
        wir::Value::Subroutine(id) => Value::Subroutine(
            storage
                .subroutines
                .get(*id)
                .ok_or_else(|| malformed_id("subroutine", id.index()))?
                .name
                .clone(),
        ),
        wir::Value::EventPlayer => Value::EventPlayer,
        wir::Value::Call { name, args } => Value::Call {
            name: name.clone(),
            args: args
                .iter()
                .map(|arg| public_value(storage, *arg))
                .collect::<Result<Vec<_>>>()?,
        },
    })
}

fn public_actions(
    storage: &wir::Program,
    id: wir::ActionId,
    output: &mut Vec<Action>,
) -> Result<()> {
    let action = storage
        .actions
        .get(id)
        .ok_or_else(|| malformed_id("action", id.index()))?;
    match action {
        wir::Action::SetGlobalVariable {
            variable, value, ..
        } => output.push(Action::SetGlobalVariable {
            variable: storage
                .global_variables
                .get(*variable)
                .ok_or_else(|| malformed_id("global variable", variable.index()))?
                .name
                .clone(),
            value: public_value(storage, *value)?,
        }),
        wir::Action::ModifyGlobalVariable {
            variable,
            op,
            value,
            ..
        } => output.push(Action::ModifyGlobalVariable {
            variable: storage
                .global_variables
                .get(*variable)
                .ok_or_else(|| malformed_id("global variable", variable.index()))?
                .name
                .clone(),
            op: public_modify(*op),
            value: public_value(storage, *value)?,
        }),
        wir::Action::SetPlayerVariable {
            player,
            variable,
            value,
            ..
        } => output.push(Action::SetPlayerVariable {
            player: public_value(storage, *player)?,
            variable: storage
                .player_variables
                .get(*variable)
                .ok_or_else(|| malformed_id("player variable", variable.index()))?
                .name
                .clone(),
            value: public_value(storage, *value)?,
        }),
        wir::Action::ModifyPlayerVariable {
            player,
            variable,
            op,
            value,
            ..
        } => output.push(Action::ModifyPlayerVariable {
            player: public_value(storage, *player)?,
            variable: storage
                .player_variables
                .get(*variable)
                .ok_or_else(|| malformed_id("player variable", variable.index()))?
                .name
                .clone(),
            op: public_modify(*op),
            value: public_value(storage, *value)?,
        }),
        wir::Action::AssignMember {
            target, op, value, ..
        } => output.push(Action::AssignMember {
            target: public_value(storage, *target)?,
            op: op.map(public_modify),
            value: public_value(storage, *value)?,
        }),
        wir::Action::CallSubroutine { subroutine, .. } => output.push(Action::CallSubroutine {
            subroutine: storage
                .subroutines
                .get(*subroutine)
                .ok_or_else(|| malformed_id("subroutine", subroutine.index()))?
                .name
                .clone(),
        }),
        wir::Action::If {
            branches,
            else_body,
            ..
        } => {
            for (index, branch) in branches.iter().enumerate() {
                output.push(if index == 0 {
                    Action::If {
                        condition: public_value(storage, branch.condition)?,
                    }
                } else {
                    Action::ElseIf {
                        condition: public_value(storage, branch.condition)?,
                    }
                });
                for action in &branch.body {
                    public_actions(storage, *action, output)?;
                }
            }
            if let Some(body) = else_body {
                output.push(Action::Else);
                for action in body {
                    public_actions(storage, *action, output)?;
                }
            }
            output.push(Action::End);
        }
        wir::Action::While {
            condition, body, ..
        } => {
            output.push(Action::While {
                condition: public_value(storage, *condition)?,
            });
            for action in body {
                public_actions(storage, *action, output)?;
            }
            output.push(Action::End);
        }
        wir::Action::ForGlobalVariable {
            variable,
            start,
            stop,
            step,
            body,
            ..
        } => {
            output.push(Action::ForGlobalVariable {
                variable: storage
                    .global_variables
                    .get(*variable)
                    .ok_or_else(|| malformed_id("global variable", variable.index()))?
                    .name
                    .clone(),
                start: public_value(storage, *start)?,
                stop: public_value(storage, *stop)?,
                step: public_value(storage, *step)?,
            });
            for action in body {
                public_actions(storage, *action, output)?;
            }
            output.push(Action::End);
        }
        wir::Action::ForPlayerVariable {
            player,
            variable,
            start,
            stop,
            step,
            body,
            ..
        } => {
            output.push(Action::ForPlayerVariable {
                player: public_value(storage, *player)?,
                variable: storage
                    .player_variables
                    .get(*variable)
                    .ok_or_else(|| malformed_id("player variable", variable.index()))?
                    .name
                    .clone(),
                start: public_value(storage, *start)?,
                stop: public_value(storage, *stop)?,
                step: public_value(storage, *step)?,
            });
            for action in body {
                public_actions(storage, *action, output)?;
            }
            output.push(Action::End);
        }
        wir::Action::Disabled { action, .. } => {
            let first = output.len();
            public_actions(storage, *action, output)?;
            let inner = output.remove(first);
            output.insert(first, Action::disabled(inner));
        }
        wir::Action::Call { name, args, .. } => output.push(Action::Call {
            name: name.clone(),
            args: args
                .iter()
                .map(|arg| public_value(storage, *arg))
                .collect::<Result<Vec<_>>>()?,
        }),
    }
    Ok(())
}

fn public_action_provenance(
    storage: &wir::Program,
    id: wir::ActionId,
    output: &mut Vec<ActionProvenance>,
) -> Result<()> {
    let action = storage
        .actions
        .get(id)
        .ok_or_else(|| malformed_id("action", id.index()))?;
    let identifier = action_identifier(action);
    let push = |output: &mut Vec<ActionProvenance>, arguments: &[wir::ValueId]| {
        output.push(ActionProvenance {
            span: action.span(),
            identifier,
            arguments: arguments
                .iter()
                .map(|value| value_provenance(storage, *value))
                .collect(),
        });
    };
    let push_without_span = |output: &mut Vec<ActionProvenance>, arguments: &[wir::ValueId]| {
        output.push(ActionProvenance {
            span: None,
            identifier: None,
            arguments: arguments
                .iter()
                .map(|value| value_provenance(storage, *value))
                .collect(),
        });
    };
    match action {
        wir::Action::SetGlobalVariable { value, .. }
        | wir::Action::ModifyGlobalVariable { value, .. } => push(output, &[*value]),
        wir::Action::SetPlayerVariable { player, value, .. }
        | wir::Action::ModifyPlayerVariable { player, value, .. } => {
            push(output, &[*player, *value])
        }
        wir::Action::AssignMember { target, value, .. } => push(output, &[*target, *value]),
        wir::Action::CallSubroutine { .. } => push(output, &[]),
        wir::Action::If {
            branches,
            else_body,
            ..
        } => {
            for (index, branch) in branches.iter().enumerate() {
                if index == 0 {
                    push(output, &[branch.condition]);
                } else {
                    push_without_span(output, &[branch.condition]);
                }
                for action in &branch.body {
                    public_action_provenance(storage, *action, output)?;
                }
                if index + 1 == branches.len() && else_body.is_none() {
                    push_without_span(output, &[]);
                }
            }
            if let Some(body) = else_body {
                push_without_span(output, &[]);
                for action in body {
                    public_action_provenance(storage, *action, output)?;
                }
                push_without_span(output, &[]);
            }
        }
        wir::Action::While {
            condition, body, ..
        } => {
            push(output, &[*condition]);
            for action in body {
                public_action_provenance(storage, *action, output)?;
            }
            push_without_span(output, &[]);
        }
        wir::Action::ForGlobalVariable {
            start,
            stop,
            step,
            body,
            ..
        } => {
            push(output, &[*start, *stop, *step]);
            for action in body {
                public_action_provenance(storage, *action, output)?;
            }
            push_without_span(output, &[]);
        }
        wir::Action::ForPlayerVariable {
            player,
            start,
            stop,
            step,
            body,
            ..
        } => {
            push(output, &[*player, *start, *stop, *step]);
            for action in body {
                public_action_provenance(storage, *action, output)?;
            }
            push_without_span(output, &[]);
        }
        wir::Action::Disabled { action, .. } => {
            public_action_provenance(storage, *action, output)?;
        }
        wir::Action::Call { args, .. } => push(output, args),
    }
    Ok(())
}

/// The recorded span of the variable or subroutine a WIR action names, when
/// the parse produced one.
fn action_identifier(action: &wir::Action) -> Option<Span> {
    match action {
        wir::Action::SetGlobalVariable { target_span, .. }
        | wir::Action::ModifyGlobalVariable { target_span, .. }
        | wir::Action::SetPlayerVariable { target_span, .. }
        | wir::Action::ModifyPlayerVariable { target_span, .. }
        | wir::Action::ForGlobalVariable { target_span, .. }
        | wir::Action::ForPlayerVariable { target_span, .. } => *target_span,
        wir::Action::CallSubroutine { callee_span, .. } => *callee_span,
        _ => None,
    }
}

/// The recorded provenance of a WIR value node and its children, mirroring
/// the public [`Value`] tree.
fn value_provenance(storage: &wir::Program, id: wir::ValueId) -> ValueProvenance {
    let Some(node) = storage.values.get(id) else {
        return ValueProvenance::default();
    };
    ValueProvenance {
        span: node.span,
        identifier: node.identifier,
        children: wir_value_children(&node.value)
            .into_iter()
            .map(|child| value_provenance(storage, child))
            .collect(),
    }
}

/// The child value ids of a WIR value, in the order the public [`Value`]
/// exposes them.
fn public_modify(op: wir::ModifyOp) -> ModifyOp {
    match op {
        wir::ModifyOp::Add => ModifyOp::Add,
        wir::ModifyOp::Subtract => ModifyOp::Subtract,
        wir::ModifyOp::Multiply => ModifyOp::Multiply,
        wir::ModifyOp::Divide => ModifyOp::Divide,
        wir::ModifyOp::Modulo => ModifyOp::Modulo,
        wir::ModifyOp::Min => ModifyOp::Min,
        wir::ModifyOp::Max => ModifyOp::Max,
        wir::ModifyOp::RaiseToPower => ModifyOp::RaiseToPower,
        wir::ModifyOp::AppendToArray => ModifyOp::AppendToArray,
        wir::ModifyOp::RemoveFromArrayByValue => ModifyOp::RemoveFromArrayByValue,
        wir::ModifyOp::RemoveFromArrayByIndex => ModifyOp::RemoveFromArrayByIndex,
    }
}
