// Emitter behavior owned by the Workshop actions domain.

use super::*;

impl Emitter<'_> {
    /// Emit one rule action; `rule_final` marks the last action of the rule,
    /// for which an `if`/`if-else` closes without the trailing `End;`
    /// (the pinned oracle's spelling, #87).
    pub(super) fn action(
        &mut self,
        id: wir::ActionId,
        level: usize,
        rule_final: bool,
    ) -> Result<()> {
        let Some(action) = self.program.actions.get(id) else {
            return Err(WorkshopError::Malformed {
                message: format!("dangling action {id}"),
                span: None,
            });
        };
        match action {
            wir::Action::SetGlobalVariable {
                variable, value, ..
            } => {
                let name = self.global_name(*variable)?;
                let mut value_text = String::new();
                self.value(*value, &mut value_text)?;
                let keyword = self.spelling(Kind::Structural, "setGlobalVariable")?;
                self.line(level, &format!("{keyword}({name}, {value_text});"))?;
            }
            wir::Action::ModifyGlobalVariable {
                variable,
                op,
                value,
                ..
            } => {
                let name = self.global_name(*variable)?;
                let op = self.modify_op_spelling(*op)?;
                let mut value_text = String::new();
                self.value(*value, &mut value_text)?;
                let keyword = self.spelling(Kind::Structural, "modifyGlobalVariable")?;
                self.line(level, &format!("{keyword}({name}, {op}, {value_text});"))?;
            }
            wir::Action::SetPlayerVariable {
                player,
                variable,
                value,
                ..
            } => {
                let mut player_text = String::new();
                self.value(*player, &mut player_text)?;
                let name = self.player_name(*variable)?;
                let mut value_text = String::new();
                self.value(*value, &mut value_text)?;
                let keyword = self.spelling(Kind::Structural, "setPlayerVariable")?;
                self.line(
                    level,
                    &format!("{keyword}({player_text}, {name}, {value_text});"),
                )?;
            }
            wir::Action::ModifyPlayerVariable {
                player,
                variable,
                op,
                value,
                ..
            } => {
                let mut player_text = String::new();
                self.value(*player, &mut player_text)?;
                let name = self.player_name(*variable)?;
                let op = self.modify_op_spelling(*op)?;
                let mut value_text = String::new();
                self.value(*value, &mut value_text)?;
                let keyword = self.spelling(Kind::Structural, "modifyPlayerVariable")?;
                self.line(
                    level,
                    &format!("{keyword}({player_text}, {name}, {op}, {value_text});"),
                )?;
            }
            wir::Action::CallSubroutine { subroutine, .. } => {
                let name = self
                    .program
                    .subroutines
                    .get(*subroutine)
                    .map(|s| s.name.clone())
                    .ok_or_else(|| WorkshopError::Unknown {
                        kind: "subroutine",
                        spelling: format!("<{subroutine}>"),
                        locale: self.locale.clone(),
                        span: None,
                    })?;
                let keyword = self.spelling(Kind::Structural, "callSubroutine")?;
                self.line(level, &format!("{keyword}({name});"))?;
            }
            wir::Action::If {
                branches,
                else_body,
                ..
            } => {
                for (index, branch) in branches.iter().enumerate() {
                    let mut condition = String::new();
                    self.value(branch.condition, &mut condition)?;
                    let keyword =
                        self.spelling(Kind::Structural, if index == 0 { "if" } else { "elseIf" })?;
                    self.line(level, &format!("{keyword}({condition});"))?;
                    for action in &branch.body {
                        self.action(*action, level + 1, false)?;
                    }
                }
                if let Some(else_body) = else_body {
                    let keyword = self.spelling(Kind::Structural, "else")?;
                    self.line(level, &format!("{keyword};"))?;
                    for action in else_body {
                        self.action(*action, level + 1, false)?;
                    }
                }
                // A rule-final if closes the rule without `End;` (oracle
                // spelling); nested and middle-of-rule ifs keep it.
                if !rule_final {
                    let keyword = self.spelling(Kind::Structural, "end")?;
                    self.line(level, &format!("{keyword};"))?;
                }
            }
            wir::Action::While {
                condition, body, ..
            } => {
                let mut text = String::new();
                self.value(*condition, &mut text)?;
                let keyword = self.spelling(Kind::Structural, "while")?;
                self.line(level, &format!("{keyword}({text});"))?;
                for action in body {
                    self.action(*action, level + 1, false)?;
                }
                let end = self.spelling(Kind::Structural, "end")?;
                self.line(level, &format!("{end};"))?;
            }
            wir::Action::ForGlobalVariable {
                variable,
                start,
                stop,
                step,
                body,
                ..
            } => {
                let name = self.global_name(*variable)?;
                let mut start_text = String::new();
                let mut stop_text = String::new();
                let mut step_text = String::new();
                self.value(*start, &mut start_text)?;
                self.value(*stop, &mut stop_text)?;
                self.value(*step, &mut step_text)?;
                let keyword = self.spelling(Kind::Structural, "forGlobalVariable")?;
                self.line(
                    level,
                    &format!("{keyword}({name}, {start_text}, {stop_text}, {step_text});"),
                )?;
                for action in body {
                    self.action(*action, level + 1, false)?;
                }
                let end = self.spelling(Kind::Structural, "end")?;
                self.line(level, &format!("{end};"))?;
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
                let keyword = self.structural("forPlayerVariable")?;
                let mut player_text = String::new();
                let mut start_text = String::new();
                let mut stop_text = String::new();
                let mut step_text = String::new();
                self.value(*player, &mut player_text)?;
                self.value(*start, &mut start_text)?;
                self.value(*stop, &mut stop_text)?;
                self.value(*step, &mut step_text)?;
                let name = self.player_name(*variable)?;
                self.line(
                    level,
                    &format!(
                        "{}({player_text}, {name}, {start_text}, {stop_text}, {step_text});",
                        keyword
                    ),
                )?;
                for action in body {
                    self.action(*action, level + 1, false)?;
                }
                let end = self.spelling(Kind::Structural, "end")?;
                self.line(level, &format!("{end};"))?;
            }
            wir::Action::AssignMember {
                target, op, value, ..
            } => {
                let mut target_text = String::new();
                let mut value_text = String::new();
                self.value(*target, &mut target_text)?;
                self.value(*value, &mut value_text)?;
                let operator = match op {
                    None => "=".to_string(),
                    Some(op) => {
                        let token = match op {
                            wir::ModifyOp::Add => "+",
                            wir::ModifyOp::Subtract => "-",
                            wir::ModifyOp::Multiply => "*",
                            wir::ModifyOp::Divide => "/",
                            wir::ModifyOp::Modulo => "%",
                            wir::ModifyOp::Min => "min",
                            wir::ModifyOp::Max => "max",
                            _ => {
                                return Err(WorkshopError::Unsupported {
                                    message: format!(
                                        "unsupported member assignment operator {op:?}"
                                    ),
                                    span: None,
                                });
                            }
                        };
                        format!("{token}=")
                    }
                };
                self.line(level, &format!("{target_text} {operator} {value_text};"))?;
            }
            wir::Action::Call { name, args, .. } => {
                // The chase family dispatches on the first argument's
                // variable kind, mirroring the pinned reference: a global
                // variable emits the global form with the argument list
                // unchanged; a player variable emits the player form with
                // the receiver split into `player, name` leading arguments
                // (the frontend guarantees a variable first argument,
                // issue #110).
                if matches!(name.as_str(), "chaseAtRate" | "chaseOverTime") {
                    let player_var = args.first().and_then(|id| {
                        self.program
                            .values
                            .get(*id)
                            .and_then(|node| match &node.value {
                                wir::Value::PlayerVariable { player, variable } => {
                                    Some((*player, *variable))
                                }
                                _ => None,
                            })
                    });
                    let spelling = if let Some((player, variable)) = player_var {
                        let id = if name == "chaseAtRate" {
                            "chasePlayerVariableAtRate"
                        } else {
                            "chasePlayerVariableOverTime"
                        };
                        let spelling = self.spelling(Kind::Action, id)?;
                        // `Chase Player Variable At Rate(player, name, …)`:
                        // the receiver splits into `player, name` leading
                        // arguments (the pinned oracle's spelling).
                        let mut text = String::new();
                        self.value(player, &mut text)?;
                        let mut parts = vec![text, self.player_name(variable)?];
                        for arg in args.iter().skip(1) {
                            let mut part = String::new();
                            self.value(*arg, &mut part)?;
                            parts.push(part);
                        }
                        return self.line(level, &format!("{spelling}({});", parts.join(", ")));
                    } else {
                        self.spelling(Kind::Action, name)?
                    };
                    let mut args_text = String::new();
                    self.args(args, &mut args_text)?;
                    return self.line(level, &format!("{spelling}({args_text});"));
                }
                if name == "stopChasingPlayerVariable" {
                    let Some((player, variable)) = args.first().and_then(|id| {
                        self.program
                            .values
                            .get(*id)
                            .and_then(|node| match &node.value {
                                wir::Value::PlayerVariable { player, variable } => {
                                    Some((*player, *variable))
                                }
                                _ => None,
                            })
                    }) else {
                        return Err(WorkshopError::Malformed {
                            message: "Stop Chasing Player Variable requires a player variable"
                                .into(),
                            span: None,
                        });
                    };
                    let spelling = self.spelling(Kind::Action, name)?;
                    let mut player_text = String::new();
                    self.value(player, &mut player_text)?;
                    return self.line(
                        level,
                        &format!(
                            "{spelling}({player_text}, {});",
                            self.player_name(variable)?
                        ),
                    );
                }
                // Native `.opy` action names map to canonical catalog ids at
                // emission (presentation concern).
                let canonical = match name.as_str() {
                    "createBeam" => Some("createBeamEffect"),
                    _ => None,
                };
                let spelling = if let Some(canonical) = canonical {
                    self.spelling(Kind::Action, canonical)?
                } else {
                    self.spelling(Kind::Action, name)?
                };
                if args.is_empty() {
                    self.line(level, &format!("{spelling};"))?;
                } else {
                    let mut args_text = String::new();
                    for (index, arg) in args.iter().enumerate() {
                        if index > 0 {
                            args_text.push_str(", ");
                        }
                        let variable_position = match name.as_str() {
                            "setGlobalVariableAtIndex" | "modifyGlobalVariableAtIndex" => {
                                index == 0
                            }
                            "setPlayerVariableAtIndex" | "modifyPlayerVariableAtIndex" => {
                                index == 1
                            }
                            _ => false,
                        };
                        if variable_position {
                            if let Some(node) = self.program.values.get(*arg) {
                                match &node.value {
                                    wir::Value::GlobalVariable(variable) => {
                                        args_text.push_str(&self.global_name(*variable)?);
                                        continue;
                                    }
                                    wir::Value::PlayerVariable { variable, .. } => {
                                        args_text.push_str(&self.player_name(*variable)?);
                                        continue;
                                    }
                                    _ => {}
                                }
                            }
                        }
                        self.value(*arg, &mut args_text)?;
                    }
                    self.line(level, &format!("{spelling}({args_text});"))?;
                }
            }
        }
        Ok(())
    }
    pub(super) fn args(&mut self, args: &[wir::ValueId], out: &mut String) -> Result<()> {
        for (index, arg) in args.iter().enumerate() {
            if index > 0 {
                out.push_str(", ");
            }
            self.value(*arg, out)?;
        }
        Ok(())
    }
}
