use std::collections::HashMap;

use super::{Result, StatementEditError, StatementList};
use crate::catalog::{Catalog, Locale};
use crate::core::error::{Result as WorkshopResult, WorkshopError};
use crate::output::emitter::EmitContext;
use crate::program::shared::{wir_action, wir_value};
use crate::program::{Action, Program};
use crate::rules::statement::Statement;
use crate::wir::{self, GlobalVarId, PlayerVarId, SubroutineId};

enum LoweredStatement {
    Condition(wir::Condition),
    Leaf(wir::ActionId),
    Header {
        keyword: &'static str,
        args: Vec<wir::ValueId>,
        name: String,
    },
    Disabled(Box<LoweredStatement>),
}

pub(super) fn emit_statement_rows(
    statements: &[Statement],
    program: &Program,
    catalog: &Catalog,
    locale: &Locale,
) -> Result<Vec<String>> {
    let mut storage = wir::Program::default();
    let (globals, players, subroutines) = seed_declarations(&mut storage, program);
    let lowered = statements
        .iter()
        .map(|statement| lower_statement(statement, &mut storage, &globals, &players, &subroutines))
        .collect::<WorkshopResult<Vec<_>>>()?;
    let mut ctx = EmitContext {
        program: &storage,
        catalog,
        locale: locale.clone(),
        fallback: None,
        fallback_ids: Vec::new(),
        out: String::new(),
        line_count: 0,
    };
    let mut rows = Vec::with_capacity(lowered.len());
    for statement in &lowered {
        let start = ctx.out.len();
        emit_lowered(&mut ctx, statement).map_err(StatementEditError::Emission)?;
        let row = ctx.out[start..]
            .strip_suffix('\n')
            .unwrap_or(&ctx.out[start..]);
        rows.push(row.to_string());
    }
    Ok(rows)
}

fn seed_declarations(
    storage: &mut wir::Program,
    program: &Program,
) -> (
    HashMap<String, GlobalVarId>,
    HashMap<String, PlayerVarId>,
    HashMap<String, SubroutineId>,
) {
    let mut globals = HashMap::new();
    for (position, variable) in program.global_variables.iter().enumerate() {
        let id = storage.global_variables.push(wir::WorkshopVariable {
            name: variable.name.clone(),
            index: variable.index.unwrap_or(position as u32),
            span: None,
            name_span: None,
        });
        globals.insert(variable.name.clone(), id);
    }
    let mut players = HashMap::new();
    for (position, variable) in program.player_variables.iter().enumerate() {
        let id = storage.player_variables.push(wir::WorkshopVariable {
            name: variable.name.clone(),
            index: variable.index.unwrap_or(position as u32),
            span: None,
            name_span: None,
        });
        players.insert(variable.name.clone(), id);
    }
    let mut subroutines = HashMap::new();
    for (position, subroutine) in program.subroutines.iter().enumerate() {
        let id = storage.subroutines.push(wir::WorkshopSubroutine {
            name: subroutine.name.clone(),
            index: subroutine.index.unwrap_or(position as u32),
            span: None,
            name_span: None,
        });
        subroutines.insert(subroutine.name.clone(), id);
    }
    (globals, players, subroutines)
}

fn lower_statement(
    statement: &Statement,
    storage: &mut wir::Program,
    globals: &HashMap<String, GlobalVarId>,
    players: &HashMap<String, PlayerVarId>,
    subroutines: &HashMap<String, SubroutineId>,
) -> WorkshopResult<LoweredStatement> {
    match statement {
        Statement::Condition(condition) => Ok(LoweredStatement::Condition(wir::Condition {
            value: wir_value(&condition.value, storage, globals, players, subroutines)?,
            disabled: condition.disabled,
        })),
        Statement::Action(action) => lower_action(action, storage, globals, players, subroutines),
    }
}

fn lower_action(
    action: &Action,
    storage: &mut wir::Program,
    globals: &HashMap<String, GlobalVarId>,
    players: &HashMap<String, PlayerVarId>,
    subroutines: &HashMap<String, SubroutineId>,
) -> WorkshopResult<LoweredStatement> {
    Ok(match action {
        Action::Disabled { action: inner } => {
            if matches!(
                inner.as_ref(),
                Action::ElseIf { .. } | Action::Else | Action::End | Action::Disabled { .. }
            ) {
                return Err(WorkshopError::unsupported(
                    "the disabled modifier applies to a single executable action".to_string(),
                    None,
                ));
            }
            LoweredStatement::Disabled(Box::new(lower_action(
                inner,
                storage,
                globals,
                players,
                subroutines,
            )?))
        }
        Action::If { condition } => LoweredStatement::Header {
            keyword: "if",
            args: vec![wir_value(
                condition,
                storage,
                globals,
                players,
                subroutines,
            )?],
            name: String::new(),
        },
        Action::ElseIf { condition } => LoweredStatement::Header {
            keyword: "elseIf",
            args: vec![wir_value(
                condition,
                storage,
                globals,
                players,
                subroutines,
            )?],
            name: String::new(),
        },
        Action::Else => LoweredStatement::Header {
            keyword: "else",
            args: Vec::new(),
            name: String::new(),
        },
        Action::End => LoweredStatement::Header {
            keyword: "end",
            args: Vec::new(),
            name: String::new(),
        },
        Action::While { condition } => LoweredStatement::Header {
            keyword: "while",
            args: vec![wir_value(
                condition,
                storage,
                globals,
                players,
                subroutines,
            )?],
            name: String::new(),
        },
        Action::ForGlobalVariable {
            variable,
            start,
            stop,
            step,
        } => LoweredStatement::Header {
            keyword: "forGlobalVariable",
            args: vec![
                wir_value(start, storage, globals, players, subroutines)?,
                wir_value(stop, storage, globals, players, subroutines)?,
                wir_value(step, storage, globals, players, subroutines)?,
            ],
            name: variable.clone(),
        },
        Action::ForPlayerVariable {
            player,
            variable,
            start,
            stop,
            step,
        } => LoweredStatement::Header {
            keyword: "forPlayerVariable",
            args: vec![
                wir_value(player, storage, globals, players, subroutines)?,
                wir_value(start, storage, globals, players, subroutines)?,
                wir_value(stop, storage, globals, players, subroutines)?,
                wir_value(step, storage, globals, players, subroutines)?,
            ],
            name: variable.clone(),
        },
        action => {
            LoweredStatement::Leaf(wir_action(action, storage, globals, players, subroutines)?)
        }
    })
}

fn emit_lowered(ctx: &mut EmitContext<'_>, lowered: &LoweredStatement) -> WorkshopResult<()> {
    match lowered {
        LoweredStatement::Condition(condition) => ctx.condition(condition, 0),
        LoweredStatement::Leaf(id) => ctx.action(*id, 0, false),
        LoweredStatement::Header {
            keyword,
            args,
            name,
        } => {
            let spelling = ctx.structural(keyword)?;
            let text = match *keyword {
                "else" | "end" => format!("{spelling};"),
                "if" | "elseIf" | "while" => {
                    format!("{spelling}({});", ctx.value_text(args[0])?)
                }
                "forGlobalVariable" => format!(
                    "{spelling}({name}, {}, {}, {});",
                    ctx.value_text(args[0])?,
                    ctx.value_text(args[1])?,
                    ctx.value_text(args[2])?,
                ),
                "forPlayerVariable" => format!(
                    "{spelling}({}, {name}, {}, {}, {});",
                    ctx.value_text(args[0])?,
                    ctx.value_text(args[1])?,
                    ctx.value_text(args[2])?,
                    ctx.value_text(args[3])?,
                ),
                _ => unreachable!("only control-flow headers emit this path"),
            };
            ctx.line(0, &text)
        }
        LoweredStatement::Disabled(inner) => {
            let start = ctx.out.len();
            emit_lowered(ctx, inner)?;
            let modifier = format!("{} ", ctx.structural("disabled")?);
            ctx.out.insert_str(start, &modifier);
            Ok(())
        }
    }
}

pub(super) fn emit_section_keyword(
    list: StatementList,
    catalog: &Catalog,
    locale: &Locale,
) -> Result<String> {
    let id = list.section_id();
    catalog
        .spelling(crate::catalog::Kind::Structural, locale, id)
        .map(str::to_string)
        .ok_or_else(|| {
            StatementEditError::Emission(WorkshopError::MissingMapping {
                kind: "structural keyword",
                id: id.to_string(),
                locale: locale.clone(),
            })
        })
}
