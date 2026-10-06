use std::collections::HashMap;

use super::super::identity::{NodeIdentity, action_identity, condition_identity, value_identity};
use super::super::*;
use super::{malformed_id, wir_value_children};
use crate::core::error::{Result, WorkshopError};
use crate::wir;

impl Program {
    pub(crate) fn to_wir(&self) -> Result<wir::Program> {
        let mut storage = wir::Program {
            settings: self.settings.clone(),
            ..Default::default()
        };

        for file in &self.files {
            storage.add_file(file.clone());
        }

        let mut globals = HashMap::new();
        for (position, variable) in self.global_variables.iter().enumerate() {
            let declaration = self.declaration_provenance(
                |provenance| &provenance.global_variables,
                &self.global_variables,
                position,
            );
            let id = storage.global_variables.push(wir::WorkshopVariable {
                name: variable.name.clone(),
                index: variable.index.unwrap_or(position as u32),
                span: declaration.span,
                name_span: declaration.name_span,
            });
            globals.insert(variable.name.clone(), id);
        }
        let mut players = HashMap::new();
        for (position, variable) in self.player_variables.iter().enumerate() {
            let declaration = self.declaration_provenance(
                |provenance| &provenance.player_variables,
                &self.player_variables,
                position,
            );
            let id = storage.player_variables.push(wir::WorkshopVariable {
                name: variable.name.clone(),
                index: variable.index.unwrap_or(position as u32),
                span: declaration.span,
                name_span: declaration.name_span,
            });
            players.insert(variable.name.clone(), id);
        }
        let mut subroutines = HashMap::new();
        for (position, subroutine) in self.subroutines.iter().enumerate() {
            let declaration = self.declaration_provenance(
                |provenance| &provenance.subroutines,
                &self.subroutines,
                position,
            );
            let id = storage.subroutines.push(wir::WorkshopSubroutine {
                name: subroutine.name.clone(),
                index: subroutine.index.unwrap_or(position as u32),
                span: declaration.span,
                name_span: declaration.name_span,
            });
            subroutines.insert(subroutine.name.clone(), id);
        }

        for (rule_index, rule) in self.rules.iter().enumerate() {
            let mut event = wir_event(&rule.event, &subroutines)?;
            if let wir::Event::Subroutine { name_span, .. } = &mut event {
                *name_span = self
                    .rule_provenance(rule_index)
                    .and_then(|provenance| provenance.event_name);
            }
            let conditions = rule
                .conditions
                .iter()
                .enumerate()
                .map(|(condition_index, condition)| {
                    wir_value(
                        &condition.value,
                        &mut storage,
                        &globals,
                        &players,
                        &subroutines,
                    )
                    .inspect(|&value| {
                        if let Some(provenance) = self.condition_record(rule_index, condition_index)
                        {
                            apply_value_provenance(
                                &mut storage,
                                value,
                                &condition.value,
                                provenance,
                                condition_identity(condition),
                            );
                        }
                    })
                    .map(|value| wir::Condition {
                        value,
                        disabled: condition.disabled,
                    })
                })
                .collect::<Result<Vec<_>>>()?;
            let mut actions = Vec::new();
            let mut position = 0;
            lower_actions(
                &rule.actions,
                &mut position,
                &mut actions,
                &mut storage,
                &globals,
                &players,
                &subroutines,
            )?;
            if let Some(provenance) = self
                .rule_record(rule_index)
                .filter(|provenance| provenance.actions.len() == rule.actions.len())
            {
                let mut public_position = 0;
                apply_action_provenance(
                    &mut storage,
                    &actions,
                    &provenance.actions,
                    &rule.actions,
                    &mut public_position,
                )?;
            }
            if position != rule.actions.len() {
                return Err(WorkshopError::malformed(
                    "unexpected control-flow terminator in rule actions".to_string(),
                    None,
                ));
            }
            storage.rules.push(wir::Rule {
                name: rule.name.clone(),
                span: self.rule_span(rule_index),
                name_span: self
                    .rule_provenance(rule_index)
                    .and_then(|provenance| provenance.name),
                disabled: rule.disabled,
                event,
                conditions,
                actions,
            });
        }
        Ok(storage)
    }
}

fn lower_actions(
    actions: &[Action],
    position: &mut usize,
    output: &mut Vec<wir::ActionId>,
    storage: &mut wir::Program,
    globals: &HashMap<String, wir::GlobalVarId>,
    players: &HashMap<String, wir::PlayerVarId>,
    subroutines: &HashMap<String, wir::SubroutineId>,
) -> Result<()> {
    while *position < actions.len() {
        let (disabled, current) = match &actions[*position] {
            Action::Disabled { action } => (true, action.as_ref()),
            action => (false, action),
        };
        if disabled
            && matches!(
                current,
                Action::ElseIf { .. } | Action::Else | Action::End | Action::Disabled { .. }
            )
        {
            return Err(WorkshopError::unsupported(
                "the disabled modifier applies to a single executable action",
                None,
            ));
        }
        match current {
            Action::ElseIf { .. } | Action::Else | Action::End => return Ok(()),
            Action::If { condition } => {
                *position += 1;
                let mut branches = vec![wir::IfBranch {
                    condition: wir_value(condition, storage, globals, players, subroutines)?,
                    body: Vec::new(),
                }];
                lower_actions(
                    actions,
                    position,
                    &mut branches[0].body,
                    storage,
                    globals,
                    players,
                    subroutines,
                )?;
                while let Some(Action::ElseIf { condition }) = actions.get(*position) {
                    *position += 1;
                    let mut body = Vec::new();
                    lower_actions(
                        actions,
                        position,
                        &mut body,
                        storage,
                        globals,
                        players,
                        subroutines,
                    )?;
                    branches.push(wir::IfBranch {
                        condition: wir_value(condition, storage, globals, players, subroutines)?,
                        body,
                    });
                }
                let else_body = if matches!(actions.get(*position), Some(Action::Else)) {
                    *position += 1;
                    let mut body = Vec::new();
                    lower_actions(
                        actions,
                        position,
                        &mut body,
                        storage,
                        globals,
                        players,
                        subroutines,
                    )?;
                    Some(body)
                } else {
                    None
                };
                if !matches!(actions.get(*position), Some(Action::End)) {
                    return Err(WorkshopError::malformed(
                        "control-flow action is missing End".to_string(),
                        None,
                    ));
                }
                *position += 1;
                output.push(storage.actions.push(wir::Action::If {
                    branches,
                    else_body,
                    span: None,
                }));
            }
            Action::While { condition } => {
                *position += 1;
                let mut body = Vec::new();
                lower_actions(
                    actions,
                    position,
                    &mut body,
                    storage,
                    globals,
                    players,
                    subroutines,
                )?;
                require_end(actions, position)?;
                let condition = wir_value(condition, storage, globals, players, subroutines)?;
                output.push(storage.actions.push(wir::Action::While {
                    condition,
                    body,
                    span: None,
                }));
            }
            Action::ForGlobalVariable {
                variable,
                start,
                stop,
                step,
            } => {
                *position += 1;
                let mut body = Vec::new();
                lower_actions(
                    actions,
                    position,
                    &mut body,
                    storage,
                    globals,
                    players,
                    subroutines,
                )?;
                require_end(actions, position)?;
                let variable = *globals
                    .get(variable)
                    .ok_or_else(|| unknown_name("global variable", variable))?;
                let start = wir_value(start, storage, globals, players, subroutines)?;
                let stop = wir_value(stop, storage, globals, players, subroutines)?;
                let step = wir_value(step, storage, globals, players, subroutines)?;
                output.push(storage.actions.push(wir::Action::ForGlobalVariable {
                    variable,
                    start,
                    stop,
                    step,
                    body,
                    span: None,
                    target_span: None,
                }));
            }
            Action::ForPlayerVariable {
                player,
                variable,
                start,
                stop,
                step,
            } => {
                *position += 1;
                let mut body = Vec::new();
                lower_actions(
                    actions,
                    position,
                    &mut body,
                    storage,
                    globals,
                    players,
                    subroutines,
                )?;
                require_end(actions, position)?;
                let player = wir_value(player, storage, globals, players, subroutines)?;
                let variable = *players
                    .get(variable)
                    .ok_or_else(|| unknown_name("player variable", variable))?;
                let start = wir_value(start, storage, globals, players, subroutines)?;
                let stop = wir_value(stop, storage, globals, players, subroutines)?;
                let step = wir_value(step, storage, globals, players, subroutines)?;
                output.push(storage.actions.push(wir::Action::ForPlayerVariable {
                    player,
                    variable,
                    start,
                    stop,
                    step,
                    body,
                    span: None,
                    target_span: None,
                }));
            }
            action => {
                *position += 1;
                let lowered = wir_action(action, storage, globals, players, subroutines)?;
                output.push(lowered);
            }
        }
        if disabled {
            let action = output.pop().expect("a lowered action was just pushed");
            output.push(
                storage
                    .actions
                    .push(wir::Action::Disabled { action, span: None }),
            );
        }
    }
    Ok(())
}

fn apply_action_provenance(
    storage: &mut wir::Program,
    actions: &[wir::ActionId],
    provenance: &[ActionProvenance],
    public: &[Action],
    position: &mut usize,
) -> Result<()> {
    for id in actions {
        let action = storage
            .actions
            .get(*id)
            .cloned()
            .ok_or_else(|| malformed_id("action", id.index()))?;
        match action {
            wir::Action::If {
                branches,
                else_body,
                ..
            } => {
                let record = provenance.get(*position);
                let public_action = public.get(*position);
                *position += 1;
                if let Some(public_action) = public_action {
                    apply_action_source(storage, *id, public_action, record);
                }
                for (branch_index, branch) in branches.iter().enumerate() {
                    if branch_index > 0 {
                        let record = provenance.get(*position);
                        let condition = record.and_then(|record| {
                            public
                                .get(*position)
                                .and_then(|action| argument_value(record, action, 0))
                        });
                        *position += 1;
                        if let Some((record, value)) = condition {
                            apply_value_provenance(
                                storage,
                                branch.condition,
                                value,
                                record,
                                value_identity(value),
                            );
                        }
                    }
                    apply_action_provenance(storage, &branch.body, provenance, public, position)?;
                }
                if let Some(body) = else_body {
                    *position += 1;
                    apply_action_provenance(storage, &body, provenance, public, position)?;
                }
                *position += 1;
            }
            wir::Action::While { body, .. }
            | wir::Action::ForGlobalVariable { body, .. }
            | wir::Action::ForPlayerVariable { body, .. } => {
                let record = provenance.get(*position);
                let public_action = public.get(*position);
                *position += 1;
                if let Some(public_action) = public_action {
                    apply_action_source(storage, *id, public_action, record);
                }
                apply_action_provenance(storage, &body, provenance, public, position)?;
                *position += 1;
            }
            wir::Action::Disabled { action, .. } => {
                let start = *position;
                apply_action_provenance(storage, &[action], provenance, public, position)?;
                let record = provenance.get(start);
                if let Some(wir::Action::Disabled { span, .. }) = storage.actions.get_mut(*id) {
                    *span = match record {
                        Some(record) if row_matches(record, public, start) => record.span,
                        _ => None,
                    };
                }
            }
            // Leaf actions consume one row; new block-shaped variants also
            // need an explicit arm in `public_action_provenance`.
            _ => {
                let record = provenance.get(*position);
                let public_action = public.get(*position);
                *position += 1;
                if let Some(public_action) = public_action {
                    apply_action_source(storage, *id, public_action, record);
                }
            }
        }
    }
    Ok(())
}

/// Whether a recorded row still describes the public action at `position`.
fn row_matches(record: &ActionProvenance, public: &[Action], position: usize) -> bool {
    public
        .get(position)
        .is_some_and(|action| record.identity == action_identity(action))
}

/// The provenance record and public value for the direct argument at
/// `argument`, while the recorded table still matches the action's argument
/// count. Whether the record also describes the value's content is decided
/// by [`apply_value_provenance`].
fn argument_value<'a>(
    source: &'a ActionProvenance,
    public: &'a Action,
    argument: usize,
) -> Option<(&'a ValueProvenance, &'a Value)> {
    let values = action_argument_values(public);
    if source.arguments.len() != values.len() {
        return None;
    }
    source
        .arguments
        .get(argument)
        .zip(values.get(argument).copied())
}

fn apply_action_source(
    storage: &mut wir::Program,
    id: wir::ActionId,
    public: &Action,
    source: Option<&ActionProvenance>,
) {
    let Some(source) = source else {
        return;
    };
    if source.identity == action_identity(public) {
        if let Some(action) = storage.actions.get_mut(id) {
            *action.span_mut() = source.span;
            if let Some(identifier) = action.identifier_span_mut() {
                *identifier = source.identifier;
            }
        }
    }
    let value_ids = storage
        .actions
        .get(id)
        .map(wir::Action::value_args)
        .unwrap_or_default();
    for (argument, value) in value_ids.into_iter().enumerate() {
        if let Some((record, public_value)) = argument_value(source, public, argument) {
            apply_value_provenance(
                storage,
                value,
                public_value,
                record,
                value_identity(public_value),
            );
        }
    }
}

/// Write a recorded value-provenance tree back onto a WIR value subtree.
/// `public` is the public value `value` was lowered from and
/// `root_identity` the identity `source` was recorded for — a condition
/// record keeps the condition's identity, every other record keeps its
/// value's. A node's own fields are written only while it still has the
/// recorded identity; descending only requires each level's child table to
/// match, so a stale node does not hide unaffected siblings.
fn apply_value_provenance(
    storage: &mut wir::Program,
    value: wir::ValueId,
    public: &Value,
    source: &ValueProvenance,
    root_identity: NodeIdentity,
) {
    if source.identity == root_identity {
        if let Some(node) = storage.values.get_mut(value) {
            node.span = source.span;
            node.identifier = source.identifier;
        }
    }
    let public_children = value_children(public);
    if source.children.len() != public_children.len() {
        return;
    }
    let Some(node) = storage.values.get(value) else {
        return;
    };
    let children = wir_value_children(&node.value);
    for ((child, public_child), source) in children
        .into_iter()
        .zip(public_children)
        .zip(&source.children)
    {
        apply_value_provenance(
            storage,
            child,
            public_child,
            source,
            value_identity(public_child),
        );
    }
}

fn require_end(actions: &[Action], position: &mut usize) -> Result<()> {
    if !matches!(actions.get(*position), Some(Action::End)) {
        return Err(WorkshopError::malformed(
            "control-flow action is missing End".to_string(),
            None,
        ));
    }
    *position += 1;
    Ok(())
}

fn wir_action(
    action: &Action,
    storage: &mut wir::Program,
    globals: &HashMap<String, wir::GlobalVarId>,
    players: &HashMap<String, wir::PlayerVarId>,
    subroutines: &HashMap<String, wir::SubroutineId>,
) -> Result<wir::ActionId> {
    let action = match action {
        Action::SetGlobalVariable { variable, value } => wir::Action::SetGlobalVariable {
            variable: *globals
                .get(variable)
                .ok_or_else(|| unknown_name("global variable", variable))?,
            value: wir_value(value, storage, globals, players, subroutines)?,
            span: None,
            target_span: None,
        },
        Action::ModifyGlobalVariable {
            variable,
            op,
            value,
        } => wir::Action::ModifyGlobalVariable {
            variable: *globals
                .get(variable)
                .ok_or_else(|| unknown_name("global variable", variable))?,
            op: *op,
            value: wir_value(value, storage, globals, players, subroutines)?,
            span: None,
            target_span: None,
        },
        Action::SetPlayerVariable {
            player,
            variable,
            value,
        } => wir::Action::SetPlayerVariable {
            player: wir_value(player, storage, globals, players, subroutines)?,
            variable: *players
                .get(variable)
                .ok_or_else(|| unknown_name("player variable", variable))?,
            value: wir_value(value, storage, globals, players, subroutines)?,
            span: None,
            target_span: None,
        },
        Action::ModifyPlayerVariable {
            player,
            variable,
            op,
            value,
        } => wir::Action::ModifyPlayerVariable {
            player: wir_value(player, storage, globals, players, subroutines)?,
            variable: *players
                .get(variable)
                .ok_or_else(|| unknown_name("player variable", variable))?,
            op: *op,
            value: wir_value(value, storage, globals, players, subroutines)?,
            span: None,
            target_span: None,
        },
        Action::AssignMember { target, op, value } => wir::Action::AssignMember {
            target: wir_value(target, storage, globals, players, subroutines)?,
            op: *op,
            value: wir_value(value, storage, globals, players, subroutines)?,
            span: None,
        },
        Action::CallSubroutine { subroutine } => wir::Action::CallSubroutine {
            subroutine: *subroutines
                .get(subroutine)
                .ok_or_else(|| unknown_name("subroutine", subroutine))?,
            span: None,
            callee_span: None,
        },
        Action::Disabled { .. } => {
            unreachable!("disabled actions are lowered by lower_actions")
        }
        Action::Call { name, args } => wir::Action::Call {
            name: name.clone(),
            args: args
                .iter()
                .map(|arg| wir_value(arg, storage, globals, players, subroutines))
                .collect::<Result<Vec<_>>>()?,
            span: None,
        },
        Action::ElseIf { .. }
        | Action::Else
        | Action::End
        | Action::If { .. }
        | Action::While { .. }
        | Action::ForGlobalVariable { .. }
        | Action::ForPlayerVariable { .. } => {
            unreachable!("structured actions are lowered by lower_actions")
        }
    };
    Ok(storage.actions.push(action))
}

fn wir_value(
    value: &Value,
    storage: &mut wir::Program,
    globals: &HashMap<String, wir::GlobalVarId>,
    players: &HashMap<String, wir::PlayerVarId>,
    subroutines: &HashMap<String, wir::SubroutineId>,
) -> Result<wir::ValueId> {
    let value = match value {
        Value::Number(value) => wir::Value::Number {
            value: *value,
            text: crate::core::format::format_number(*value),
        },
        Value::String(value) => wir::Value::String(value.clone()),
        Value::LocalizedString(value) => wir::Value::LocalizedString(value.clone()),
        Value::Bool(value) => wir::Value::Bool(*value),
        Value::Null => wir::Value::Null,
        Value::Empty => wir::Value::Empty,
        Value::Array(values) => wir::Value::Array(
            values
                .iter()
                .map(|value| wir_value(value, storage, globals, players, subroutines))
                .collect::<Result<Vec<_>>>()?,
        ),
        Value::Vector { x, y, z } => wir::Value::Vector {
            x: wir_value(x, storage, globals, players, subroutines)?,
            y: wir_value(y, storage, globals, players, subroutines)?,
            z: wir_value(z, storage, globals, players, subroutines)?,
        },
        Value::Enum { value_type, value } => wir::Value::Enum {
            value_type: value_type.clone(),
            value: value.clone(),
        },
        Value::GlobalVariable(name) => wir::Value::GlobalVariable(
            *globals
                .get(name)
                .ok_or_else(|| unknown_name("global variable", name))?,
        ),
        Value::PlayerVariable { player, variable } => wir::Value::PlayerVariable {
            player: wir_value(player, storage, globals, players, subroutines)?,
            variable: *players
                .get(variable)
                .ok_or_else(|| unknown_name("player variable", variable))?,
        },
        Value::Subroutine(name) => wir::Value::Subroutine(
            *subroutines
                .get(name)
                .ok_or_else(|| unknown_name("subroutine", name))?,
        ),
        Value::EventPlayer => wir::Value::EventPlayer,
        Value::Call { name, args } => wir::Value::Call {
            name: name.clone(),
            args: args
                .iter()
                .map(|arg| wir_value(arg, storage, globals, players, subroutines))
                .collect::<Result<Vec<_>>>()?,
        },
    };
    Ok(storage.values.push(wir::ValueNode::new(value, None)))
}

fn wir_event(
    event: &Event,
    subroutines: &HashMap<String, wir::SubroutineId>,
) -> Result<wir::Event> {
    Ok(match event {
        Event::Global => wir::Event::Global,
        Event::EachPlayer => wir::Event::EachPlayer,
        Event::EachPlayerWithFilters { team, target } => wir::Event::EachPlayerWithFilters {
            team: *team,
            target: target.clone(),
        },
        Event::Player { kind, team, target } => wir::Event::Player {
            kind: *kind,
            team: *team,
            target: target.clone(),
        },
        Event::Subroutine(name) => wir::Event::Subroutine {
            subroutine: *subroutines
                .get(name)
                .ok_or_else(|| unknown_name("subroutine", name))?,
            name_span: None,
        },
    })
}

fn unknown_name(kind: &str, name: &str) -> WorkshopError {
    WorkshopError::malformed(format!("unknown {kind} '{name}'"), None)
}
