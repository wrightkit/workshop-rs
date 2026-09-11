//! Canonical Workshop program concepts.

use std::collections::HashMap;

use crate::core::error::{Result, WorkshopError};
use crate::settings::Settings;
use crate::wir;

/// A complete Workshop program built from Workshop concepts.
#[derive(Debug, Clone, Default)]
pub struct Program {
    pub settings: Option<Settings>,
    pub global_variables: Vec<Variable>,
    pub player_variables: Vec<Variable>,
    pub subroutines: Vec<Subroutine>,
    pub rules: Vec<Rule>,
    /// Raw source metadata is retained by parsed programs without becoming a
    /// required field for independently constructed programs.
    source: Option<Box<wir::Program>>,
}

impl Program {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn global_variable(&mut self, variable: Variable) -> &mut Self {
        self.global_variables.push(variable);
        self
    }

    pub fn player_variable(&mut self, variable: Variable) -> &mut Self {
        self.player_variables.push(variable);
        self
    }

    pub fn subroutine(&mut self, subroutine: Subroutine) -> &mut Self {
        self.subroutines.push(subroutine);
        self
    }

    pub fn rule(&mut self, rule: Rule) -> &mut Self {
        self.rules.push(rule);
        self
    }

    /// Return the retained source document for a parsed file.
    pub fn source(&self, file: crate::source::FileId) -> Option<&crate::source::SourceDocument> {
        self.source
            .as_deref()
            .and_then(|program| program.source(file))
    }

    /// Validate the structural invariants of the canonical program.
    pub fn validate(&self) -> std::result::Result<(), WorkshopError> {
        let storage = self.to_wir()?;
        storage
            .validate()
            .map_err(|error| WorkshopError::Malformed {
                message: error.to_string(),
                span: None,
            })
    }

    /// Report constructs that are structurally preserved but not fully
    /// understood by the canonical catalog.
    pub fn semantic_issues(
        &self,
        catalog: &crate::catalog::Catalog,
    ) -> Vec<crate::rules::SemanticIssue> {
        crate::analysis::semantic::inspect(self, catalog)
    }

    /// Render the program through the canonical Workshop debug representation.
    pub fn dump(&self) -> String {
        self.to_wir().map_or_else(
            |error| format!("invalid program: {error}"),
            |program| program.dump(),
        )
    }

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
            source: Some(Box::new(storage.clone())),
        };
        for rule in storage.rules.iter() {
            let event = public_event(&storage, &rule.event)?;
            let conditions = rule
                .conditions
                .iter()
                .map(|condition| public_value(&storage, *condition))
                .collect::<Result<Vec<_>>>()?;
            let mut actions = Vec::new();
            for action in &rule.actions {
                public_actions(&storage, *action, &mut actions)?;
            }
            program.rules.push(Rule {
                name: rule.name.clone(),
                disabled: rule.disabled,
                event,
                conditions: conditions.into_iter().map(Condition::new).collect(),
                actions,
            });
        }
        Ok(program)
    }

    pub(crate) fn to_wir(&self) -> Result<wir::Program> {
        let mut storage = wir::Program {
            settings: self.settings.clone(),
            ..Default::default()
        };

        if let Some(source) = &self.source {
            for file in source.files.iter() {
                storage.add_file(file.clone());
            }
        }

        let mut globals = HashMap::new();
        for (position, variable) in self.global_variables.iter().enumerate() {
            let id = storage.global_variables.push(wir::WorkshopVariable {
                name: variable.name.clone(),
                index: variable.index.unwrap_or(position as u32),
                span: None,
                name_span: None,
            });
            globals.insert(variable.name.clone(), id);
        }
        let mut players = HashMap::new();
        for (position, variable) in self.player_variables.iter().enumerate() {
            let id = storage.player_variables.push(wir::WorkshopVariable {
                name: variable.name.clone(),
                index: variable.index.unwrap_or(position as u32),
                span: None,
                name_span: None,
            });
            players.insert(variable.name.clone(), id);
        }
        let mut subroutines = HashMap::new();
        for (position, subroutine) in self.subroutines.iter().enumerate() {
            let id = storage.subroutines.push(wir::WorkshopSubroutine {
                name: subroutine.name.clone(),
                index: subroutine.index.unwrap_or(position as u32),
                span: None,
                name_span: None,
            });
            subroutines.insert(subroutine.name.clone(), id);
        }

        for rule in &self.rules {
            let event = wir_event(&rule.event, &subroutines)?;
            let conditions = rule
                .conditions
                .iter()
                .map(|condition| {
                    wir_value(
                        &condition.value,
                        &mut storage,
                        &globals,
                        &players,
                        &subroutines,
                    )
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
            if position != rule.actions.len() {
                return Err(WorkshopError::Malformed {
                    message: "unexpected control-flow terminator in rule actions".to_string(),
                    span: None,
                });
            }
            storage.rules.push(wir::Rule {
                name: rule.name.clone(),
                span: None,
                name_span: None,
                disabled: rule.disabled,
                event,
                conditions,
                actions,
            });
        }
        Ok(storage)
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
        wir::Event::Subroutine(id) => Event::Subroutine(
            storage
                .subroutines
                .get(*id)
                .ok_or_else(|| malformed_id("subroutine", id.index()))?
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
        match &actions[*position] {
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
                    return Err(WorkshopError::Malformed {
                        message: "control-flow action is missing End".to_string(),
                        span: None,
                    });
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
                }));
            }
            action => {
                *position += 1;
                let lowered = wir_action(action, storage, globals, players, subroutines)?;
                output.push(lowered);
            }
        }
    }
    Ok(())
}

fn require_end(actions: &[Action], position: &mut usize) -> Result<()> {
    if !matches!(actions.get(*position), Some(Action::End)) {
        return Err(WorkshopError::Malformed {
            message: "control-flow action is missing End".to_string(),
            span: None,
        });
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
            op: wir_modify(*op),
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
            op: wir_modify(*op),
            value: wir_value(value, storage, globals, players, subroutines)?,
            span: None,
            target_span: None,
        },
        Action::AssignMember { target, op, value } => wir::Action::AssignMember {
            target: wir_value(target, storage, globals, players, subroutines)?,
            op: op.map(wir_modify),
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
        Action::Disabled { action } => {
            return wir_action(action, storage, globals, players, subroutines);
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
            team: wir_team(*team),
            target: wir_target(target),
        },
        Event::Player { kind, team, target } => wir::Event::Player {
            kind: wir_player_event(*kind),
            team: wir_team(*team),
            target: wir_target(target),
        },
        Event::Subroutine(name) => wir::Event::Subroutine(
            *subroutines
                .get(name)
                .ok_or_else(|| unknown_name("subroutine", name))?,
        ),
    })
}

fn wir_team(team: EventTeam) -> wir::EventTeam {
    match team {
        EventTeam::All => wir::EventTeam::All,
        EventTeam::Team1 => wir::EventTeam::Team1,
        EventTeam::Team2 => wir::EventTeam::Team2,
    }
}

fn wir_target(target: &EventTarget) -> wir::EventTarget {
    match target {
        EventTarget::All => wir::EventTarget::All,
        EventTarget::Slot(slot) => wir::EventTarget::Slot(*slot),
        EventTarget::Hero(hero) => wir::EventTarget::Hero(hero.clone()),
    }
}

fn wir_player_event(kind: PlayerEventKind) -> wir::PlayerEventKind {
    match kind {
        PlayerEventKind::DealtDamage => wir::PlayerEventKind::DealtDamage,
        PlayerEventKind::DealtFinalBlow => wir::PlayerEventKind::DealtFinalBlow,
        PlayerEventKind::DealtHealing => wir::PlayerEventKind::DealtHealing,
        PlayerEventKind::DealtKnockback => wir::PlayerEventKind::DealtKnockback,
        PlayerEventKind::Died => wir::PlayerEventKind::Died,
        PlayerEventKind::EarnedElimination => wir::PlayerEventKind::EarnedElimination,
        PlayerEventKind::Joined => wir::PlayerEventKind::Joined,
        PlayerEventKind::Left => wir::PlayerEventKind::Left,
        PlayerEventKind::ReceivedHealing => wir::PlayerEventKind::ReceivedHealing,
        PlayerEventKind::ReceivedKnockback => wir::PlayerEventKind::ReceivedKnockback,
        PlayerEventKind::TookDamage => wir::PlayerEventKind::TookDamage,
    }
}

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
        wir::ModifyOp::RemoveFromArray => ModifyOp::RemoveFromArray,
        wir::ModifyOp::RemoveFromArrayByIndex => ModifyOp::RemoveFromArrayByIndex,
    }
}

fn wir_modify(op: ModifyOp) -> wir::ModifyOp {
    match op {
        ModifyOp::Add => wir::ModifyOp::Add,
        ModifyOp::Subtract => wir::ModifyOp::Subtract,
        ModifyOp::Multiply => wir::ModifyOp::Multiply,
        ModifyOp::Divide => wir::ModifyOp::Divide,
        ModifyOp::Modulo => wir::ModifyOp::Modulo,
        ModifyOp::Min => wir::ModifyOp::Min,
        ModifyOp::Max => wir::ModifyOp::Max,
        ModifyOp::RaiseToPower => wir::ModifyOp::RaiseToPower,
        ModifyOp::AppendToArray => wir::ModifyOp::AppendToArray,
        ModifyOp::RemoveFromArray => wir::ModifyOp::RemoveFromArray,
        ModifyOp::RemoveFromArrayByIndex => wir::ModifyOp::RemoveFromArrayByIndex,
    }
}

fn malformed_id(kind: &str, index: usize) -> WorkshopError {
    WorkshopError::Malformed {
        message: format!("dangling {kind} {index}"),
        span: None,
    }
}

fn unknown_name(kind: &str, name: &str) -> WorkshopError {
    WorkshopError::Malformed {
        message: format!("unknown {kind} '{name}'"),
        span: None,
    }
}

/// A Workshop global or player variable declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Variable {
    pub name: String,
    /// The raw Workshop declaration index, when the declaration has one.
    pub index: Option<u32>,
}

impl Variable {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            index: None,
        }
    }

    pub fn with_index(name: impl Into<String>, index: u32) -> Self {
        Self {
            name: name.into(),
            index: Some(index),
        }
    }
}

/// A Workshop subroutine declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Subroutine {
    pub name: String,
    /// The raw Workshop declaration index, when the declaration has one.
    pub index: Option<u32>,
}

impl Subroutine {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            index: None,
        }
    }

    pub fn with_index(name: impl Into<String>, index: u32) -> Self {
        Self {
            name: name.into(),
            index: Some(index),
        }
    }
}

/// A Workshop rule with explicit conditions and a linear action stream.
#[derive(Debug, Clone)]
pub struct Rule {
    pub name: String,
    pub disabled: bool,
    pub event: Event,
    pub conditions: Vec<Condition>,
    pub actions: Vec<Action>,
}

impl Rule {
    pub fn new(name: impl Into<String>, event: Event) -> Self {
        Self {
            name: name.into(),
            disabled: false,
            event,
            conditions: Vec::new(),
            actions: Vec::new(),
        }
    }

    pub fn condition(mut self, condition: impl Into<Condition>) -> Self {
        self.conditions.push(condition.into());
        self
    }

    pub fn action(mut self, action: Action) -> Self {
        self.actions.push(action);
        self
    }
}

/// A rule condition. Conditions remain distinct from general value expressions.
#[derive(Debug, Clone)]
pub struct Condition {
    pub value: Value,
    pub disabled: bool,
}

impl Condition {
    pub fn new(value: Value) -> Self {
        Self {
            value,
            disabled: false,
        }
    }

    pub fn disabled(value: Value) -> Self {
        Self {
            value,
            disabled: true,
        }
    }
}

impl From<Value> for Condition {
    fn from(value: Value) -> Self {
        Self::new(value)
    }
}

/// A Workshop event identity and its native filters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    Global,
    EachPlayer,
    EachPlayerWithFilters {
        team: EventTeam,
        target: EventTarget,
    },
    Player {
        kind: PlayerEventKind,
        team: EventTeam,
        target: EventTarget,
    },
    Subroutine(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventTeam {
    All,
    Team1,
    Team2,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventTarget {
    All,
    Slot(u8),
    Hero(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerEventKind {
    DealtDamage,
    DealtFinalBlow,
    DealtHealing,
    DealtKnockback,
    Died,
    EarnedElimination,
    Joined,
    Left,
    ReceivedHealing,
    ReceivedKnockback,
    TookDamage,
}

/// A Workshop action line. Control flow is represented in the same order as
/// the Workshop source, including its explicit `End` lines.
#[derive(Debug, Clone)]
pub enum Action {
    SetGlobalVariable {
        variable: String,
        value: Value,
    },
    ModifyGlobalVariable {
        variable: String,
        op: ModifyOp,
        value: Value,
    },
    SetPlayerVariable {
        player: Value,
        variable: String,
        value: Value,
    },
    ModifyPlayerVariable {
        player: Value,
        variable: String,
        op: ModifyOp,
        value: Value,
    },
    AssignMember {
        target: Value,
        op: Option<ModifyOp>,
        value: Value,
    },
    CallSubroutine {
        subroutine: String,
    },
    If {
        condition: Value,
    },
    ElseIf {
        condition: Value,
    },
    Else,
    While {
        condition: Value,
    },
    ForGlobalVariable {
        variable: String,
        start: Value,
        stop: Value,
        step: Value,
    },
    ForPlayerVariable {
        player: Value,
        variable: String,
        start: Value,
        stop: Value,
        step: Value,
    },
    End,
    Disabled {
        action: Box<Action>,
    },
    Call {
        name: String,
        args: Vec<Value>,
    },
}

impl Action {
    /// Mark an action as disabled.
    pub fn disabled(action: Action) -> Self {
        Self::Disabled {
            action: Box::new(action),
        }
    }

    /// Construct a dynamic action call by canonical Workshop id.
    pub fn call(name: impl Into<String>, args: impl IntoIterator<Item = Value>) -> Self {
        Self::Call {
            name: name.into(),
            args: args.into_iter().collect(),
        }
    }
}

/// The operation used by a Workshop variable modification action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModifyOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Min,
    Max,
    RaiseToPower,
    AppendToArray,
    RemoveFromArray,
    RemoveFromArrayByIndex,
}

/// A composable Workshop value expression.
#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    String(String),
    LocalizedString(String),
    Bool(bool),
    Null,
    Array(Vec<Value>),
    Vector {
        x: Box<Value>,
        y: Box<Value>,
        z: Box<Value>,
    },
    Enum {
        value_type: String,
        value: String,
    },
    GlobalVariable(String),
    PlayerVariable {
        player: Box<Value>,
        variable: String,
    },
    Subroutine(String),
    EventPlayer,
    Call {
        name: String,
        args: Vec<Value>,
    },
}

impl Value {
    /// Construct a numeric Workshop literal.
    pub fn number(value: f64) -> Self {
        Self::Number(value)
    }

    /// Construct a custom Workshop string literal.
    pub fn string(value: impl Into<String>) -> Self {
        Self::String(value.into())
    }

    pub fn global_variable(name: impl Into<String>) -> Self {
        Self::GlobalVariable(name.into())
    }

    pub fn player_variable(player: Value, name: impl Into<String>) -> Self {
        Self::PlayerVariable {
            player: Box::new(player),
            variable: name.into(),
        }
    }

    /// Construct a dynamic value call by canonical Workshop id.
    pub fn call(name: impl Into<String>, args: impl IntoIterator<Item = Value>) -> Self {
        Self::Call {
            name: name.into(),
            args: args.into_iter().collect(),
        }
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Self::Number(value)
    }
}

impl From<f32> for Value {
    fn from(value: f32) -> Self {
        Self::Number(f64::from(value))
    }
}

macro_rules! impl_integer_value {
    ($($type:ty),+ $(,)?) => {
        $(
            impl From<$type> for Value {
                fn from(value: $type) -> Self {
                    Self::Number(value as f64)
                }
            }
        )+
    };
}

impl_integer_value!(i8, i16, i32, i64, isize, u8, u16, u32, u64, usize);

impl From<String> for Value {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Self::String(value.to_string())
    }
}

impl<T: Into<Value>> From<Vec<T>> for Value {
    fn from(values: Vec<T>) -> Self {
        Self::Array(values.into_iter().map(Into::into).collect())
    }
}

impl<T: Into<Value>, const N: usize> From<[T; N]> for Value {
    fn from(values: [T; N]) -> Self {
        Self::Array(values.into_iter().map(Into::into).collect())
    }
}

include!(concat!(env!("OUT_DIR"), "/typed_api.rs"));
