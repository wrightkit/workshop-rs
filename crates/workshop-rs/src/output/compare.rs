use crate::ModifyOp;
use crate::core::format::format_number;
use crate::wir;

use super::roundtrip::{ambiguous_enum_equivalent, canonical_value_name, settings_equivalent};

/// Which program of a [`difference`] pair a report refers to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgramSide {
    /// `difference`'s first argument.
    First,
    /// `difference`'s second argument.
    Second,
}

/// A declaration table in a [`Program`](crate::Program).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeclarationTable {
    /// `program.global_variables`
    GlobalVariables,
    /// `program.player_variables`
    PlayerVariables,
    /// `program.subroutines`
    Subroutines,
}

/// One step from a compared element to the differing child element.
///
/// Indices count only compared elements: disabled conditions, disabled
/// actions, and inert rules are skipped before comparison.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum PathStep {
    /// The n-th enabled condition of the compared rules; a `Value(0)` step
    /// after it enters the condition's value.
    Condition(usize),
    /// The n-th enabled action of the current action sequence — the rule's
    /// actions with each control-flow group folded into its header action
    /// (`If`, `While`, `For`), or a nested list entered through `Branch`,
    /// `ElseBody`, or `Body`.
    Action(usize),
    /// The n-th value operand of an action — its value arguments in
    /// declaration order (`If` carries one condition per branch) — or the
    /// n-th child of a value: `Array` elements, `Call` arguments, `Vector`
    /// x/y/z, or a `PlayerVariable`'s player. For the `Set`/`Modify` spelling
    /// pair the steps index the `Set` value's call arguments; for a
    /// `PlayerVariable`/`memberAccess` pair `Value(1)` is the memberAccess's
    /// member operand.
    Value(usize),
    /// The body of the n-th `if`/`else if` branch of an `If` action.
    Branch(usize),
    /// The `else` body of an `If` action.
    ElseBody,
    /// The body of a `While` or `For` action.
    Body,
}

/// What differs at a [`Difference`]'s location.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum DifferenceKind {
    /// A program could not be materialized into canonical form, so the
    /// comparison is undefined for it.
    Malformed {
        program: ProgramSide,
        message: String,
    },
    /// The programs' `settings` differ.
    Settings,
    /// A declaration table declares a different index set.
    DeclarationTable {
        table: DeclarationTable,
        first: Vec<u32>,
        second: Vec<u32>,
    },
    /// Compared element counts differ; `element` names the compared list —
    /// `"rules"`, `"subroutine rules"`, `"conditions"`, `"actions"`,
    /// `"branches"`, `"arguments"`, or `"elements"`.
    Count {
        element: &'static str,
        first: usize,
        second: usize,
    },
    /// The compared rules' events differ; `first` and `second` describe the
    /// two events.
    Event { first: String, second: String },
    /// The compared nodes have different kinds, e.g. `"setGlobalVariable"`
    /// vs `"modifyGlobalVariable"` or `"number"` vs `"call"`.
    Shape {
        first: &'static str,
        second: &'static str,
    },
    /// Variable or subroutine references resolve to different declaration
    /// indices.
    Reference {
        table: DeclarationTable,
        first: u32,
        second: u32,
    },
    /// Modify operations differ (`"add"`, `"subtract"`, …).
    Operation { first: String, second: String },
    /// Scalar payloads differ — numbers, strings, enum members, call ids.
    Scalar { first: String, second: String },
}

/// The first behavior-relevant difference [`difference`] found.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct Difference {
    /// The compared rules' positions in each program's `rules`, or `None`
    /// for differences above the rule level.
    pub rules: Option<(usize, usize)>,
    /// The path to the differing element inside the compared rules; empty
    /// for rule-level differences (the event, or an element count).
    pub path: Vec<PathStep>,
    /// What differs at that location.
    pub kind: DifferenceKind,
}

/// Whether two programs are behaviorally equivalent under this module's
/// [comparison contract](self).
pub fn equivalent(first: &crate::Program, second: &crate::Program) -> bool {
    difference(first, second).is_none()
}

/// The first behavior-relevant difference between `first` and `second`
/// under this module's [comparison contract](self), or `None` when the
/// programs are equivalent.
pub fn difference(first: &crate::Program, second: &crate::Program) -> Option<Difference> {
    let a = match first.to_wir() {
        Ok(program) => program,
        Err(error) => return Some(malformed(ProgramSide::First, error)),
    };
    let b = match second.to_wir() {
        Ok(program) => program,
        Err(error) => return Some(malformed(ProgramSide::Second, error)),
    };
    compare(&a, &b)
}

fn malformed(program: ProgramSide, error: crate::WorkshopError) -> Difference {
    Difference {
        rules: None,
        path: Vec::new(),
        kind: DifferenceKind::Malformed {
            program,
            message: error.to_string(),
        },
    }
}

fn program_level(kind: DifferenceKind) -> Difference {
    Difference {
        rules: None,
        path: Vec::new(),
        kind,
    }
}

fn compare(a: &wir::Program, b: &wir::Program) -> Option<Difference> {
    if !settings_equivalent(a.settings.as_ref(), b.settings.as_ref()) {
        return Some(program_level(DifferenceKind::Settings));
    }
    for (table, first, second) in [
        (
            DeclarationTable::GlobalVariables,
            index_set(a.global_variables.iter().map(|variable| variable.index)),
            index_set(b.global_variables.iter().map(|variable| variable.index)),
        ),
        (
            DeclarationTable::PlayerVariables,
            index_set(a.player_variables.iter().map(|variable| variable.index)),
            index_set(b.player_variables.iter().map(|variable| variable.index)),
        ),
        (
            DeclarationTable::Subroutines,
            index_set(a.subroutines.iter().map(|subroutine| subroutine.index)),
            index_set(b.subroutines.iter().map(|subroutine| subroutine.index)),
        ),
    ] {
        if first != second {
            return Some(program_level(DifferenceKind::DeclarationTable {
                table,
                first,
                second,
            }));
        }
    }
    let first_rules = effective_rules(a);
    let second_rules = effective_rules(b);
    if first_rules.ordered.len() != second_rules.ordered.len() {
        return Some(program_level(DifferenceKind::Count {
            element: "rules",
            first: first_rules.ordered.len(),
            second: second_rules.ordered.len(),
        }));
    }
    for (&(first, left), &(second, right)) in first_rules.ordered.iter().zip(&second_rules.ordered)
    {
        if let Some((path, kind)) = rule_difference(a, b, left, right) {
            return Some(Difference {
                rules: Some((first, second)),
                path,
                kind,
            });
        }
    }
    if first_rules.subroutines.len() != second_rules.subroutines.len() {
        return Some(program_level(DifferenceKind::Count {
            element: "subroutine rules",
            first: first_rules.subroutines.len(),
            second: second_rules.subroutines.len(),
        }));
    }
    for (&(_, first, left), &(_, second, right)) in first_rules
        .subroutines
        .iter()
        .zip(&second_rules.subroutines)
    {
        if let Some((path, kind)) = rule_difference(a, b, left, right) {
            return Some(Difference {
                rules: Some((first, second)),
                path,
                kind,
            });
        }
    }
    None
}

fn index_set(indices: impl Iterator<Item = u32>) -> Vec<u32> {
    let mut indices: Vec<u32> = indices.collect();
    indices.sort_unstable();
    indices
}

struct EffectiveRules<'p> {
    ordered: Vec<(usize, &'p wir::Rule)>,
    subroutines: Vec<(u32, usize, &'p wir::Rule)>,
}

fn effective_rules(program: &wir::Program) -> EffectiveRules<'_> {
    let mut ordered = Vec::new();
    let mut subroutines = Vec::new();
    for (position, rule) in program.rules.iter().enumerate() {
        if rule.disabled
            || !rule
                .actions
                .iter()
                .any(|action| !is_disabled(program, *action))
        {
            continue;
        }
        if let wir::Event::Subroutine { subroutine, .. } = &rule.event {
            subroutines.push((subroutine_index(program, *subroutine), position, rule));
        } else {
            ordered.push((position, rule));
        }
    }
    subroutines.sort_by_key(|(index, ..)| *index);
    EffectiveRules {
        ordered,
        subroutines,
    }
}

fn is_disabled(program: &wir::Program, action: wir::ActionId) -> bool {
    matches!(
        program.actions.get(action),
        Some(wir::Action::Disabled { .. })
    )
}

fn enabled_actions(program: &wir::Program, actions: &[wir::ActionId]) -> Vec<wir::ActionId> {
    actions
        .iter()
        .copied()
        .filter(|action| !is_disabled(program, *action))
        .collect()
}

fn global_index(program: &wir::Program, variable: wir::GlobalVarId) -> u32 {
    program
        .global_variables
        .get(variable)
        .map_or(u32::MAX, |variable| variable.index)
}

fn player_index(program: &wir::Program, variable: wir::PlayerVarId) -> u32 {
    program
        .player_variables
        .get(variable)
        .map_or(u32::MAX, |variable| variable.index)
}

fn subroutine_index(program: &wir::Program, subroutine: wir::SubroutineId) -> u32 {
    program
        .subroutines
        .get(subroutine)
        .map_or(u32::MAX, |subroutine| subroutine.index)
}

fn rule_difference(
    a: &wir::Program,
    b: &wir::Program,
    left: &wir::Rule,
    right: &wir::Rule,
) -> Option<(Vec<PathStep>, DifferenceKind)> {
    let mut path = Vec::new();
    let kind = rule_kind(a, b, left, right, &mut path)?;
    Some((path, kind))
}

fn rule_kind(
    a: &wir::Program,
    b: &wir::Program,
    left: &wir::Rule,
    right: &wir::Rule,
    path: &mut Vec<PathStep>,
) -> Option<DifferenceKind> {
    if !event_equivalent(a, &left.event, b, &right.event) {
        return Some(DifferenceKind::Event {
            first: describe_event(a, &left.event),
            second: describe_event(b, &right.event),
        });
    }
    let left_conditions: Vec<wir::Condition> = left
        .conditions
        .iter()
        .copied()
        .filter(|condition| !condition.disabled)
        .collect();
    let right_conditions: Vec<wir::Condition> = right
        .conditions
        .iter()
        .copied()
        .filter(|condition| !condition.disabled)
        .collect();
    if left_conditions.len() != right_conditions.len() {
        return Some(DifferenceKind::Count {
            element: "conditions",
            first: left_conditions.len(),
            second: right_conditions.len(),
        });
    }
    for (index, (left, right)) in left_conditions.iter().zip(&right_conditions).enumerate() {
        path.push(PathStep::Condition(index));
        if let Some(kind) = value_difference(a, b, left.value, right.value, path) {
            return Some(kind);
        }
        path.pop();
    }
    action_list_difference(a, b, &left.actions, &right.actions, path)
}

fn event_equivalent(
    a: &wir::Program,
    left: &wir::Event,
    b: &wir::Program,
    right: &wir::Event,
) -> bool {
    match (left, right) {
        (wir::Event::Global, wir::Event::Global) => true,
        (wir::Event::EachPlayer, wir::Event::EachPlayer) => true,
        (wir::Event::EachPlayer, wir::Event::EachPlayerWithFilters { team, target })
        | (wir::Event::EachPlayerWithFilters { team, target }, wir::Event::EachPlayer) => {
            *team == wir::EventTeam::All && *target == wir::EventTarget::All
        }
        (
            wir::Event::EachPlayerWithFilters {
                team: first_team,
                target: first_target,
            },
            wir::Event::EachPlayerWithFilters {
                team: second_team,
                target: second_target,
            },
        ) => first_team == second_team && first_target == second_target,
        (
            wir::Event::Player {
                kind: first_kind,
                team: first_team,
                target: first_target,
            },
            wir::Event::Player {
                kind: second_kind,
                team: second_team,
                target: second_target,
            },
        ) => {
            first_kind == second_kind && first_team == second_team && first_target == second_target
        }
        (
            wir::Event::Subroutine {
                subroutine: first, ..
            },
            wir::Event::Subroutine {
                subroutine: second, ..
            },
        ) => subroutine_index(a, *first) == subroutine_index(b, *second),
        _ => false,
    }
}

fn describe_event(program: &wir::Program, event: &wir::Event) -> String {
    match event {
        wir::Event::Global => "global".to_string(),
        wir::Event::EachPlayer => "eachPlayer".to_string(),
        wir::Event::EachPlayerWithFilters { team, target } => {
            format!("eachPlayer(team={team:?}, target={target:?})")
        }
        wir::Event::Player { kind, team, target } => {
            format!("{}(team={team:?}, target={target:?})", kind.catalog_id())
        }
        wir::Event::Subroutine { subroutine, .. } => {
            format!(
                "subroutine(index={})",
                subroutine_index(program, *subroutine)
            )
        }
    }
}

fn action_list_difference(
    a: &wir::Program,
    b: &wir::Program,
    left: &[wir::ActionId],
    right: &[wir::ActionId],
    path: &mut Vec<PathStep>,
) -> Option<DifferenceKind> {
    let left = enabled_actions(a, left);
    let right = enabled_actions(b, right);
    if left.len() != right.len() {
        return Some(DifferenceKind::Count {
            element: "actions",
            first: left.len(),
            second: right.len(),
        });
    }
    for (index, (&left, &right)) in left.iter().zip(&right).enumerate() {
        path.push(PathStep::Action(index));
        if let Some(kind) = action_difference(a, b, left, right, path) {
            return Some(kind);
        }
        path.pop();
    }
    None
}

fn action_difference(
    a: &wir::Program,
    b: &wir::Program,
    left: wir::ActionId,
    right: wir::ActionId,
    path: &mut Vec<PathStep>,
) -> Option<DifferenceKind> {
    let (Some(first), Some(second)) = (a.actions.get(left), b.actions.get(right)) else {
        return Some(DifferenceKind::Shape {
            first: "action",
            second: "missing",
        });
    };
    match (first, second) {
        (
            wir::Action::SetGlobalVariable {
                variable: first_variable,
                value: first_value,
                ..
            },
            wir::Action::SetGlobalVariable {
                variable: second_variable,
                value: second_value,
                ..
            },
        ) => {
            let difference = reference_difference(
                global_index(a, *first_variable),
                global_index(b, *second_variable),
                DeclarationTable::GlobalVariables,
            );
            if difference.is_some() {
                return difference;
            }
            argument_difference(a, b, path, 0, *first_value, *second_value)
        }
        (
            wir::Action::ModifyGlobalVariable {
                variable: first_variable,
                op: first_op,
                value: first_value,
                ..
            },
            wir::Action::ModifyGlobalVariable {
                variable: second_variable,
                op: second_op,
                value: second_value,
                ..
            },
        ) => {
            let difference = reference_difference(
                global_index(a, *first_variable),
                global_index(b, *second_variable),
                DeclarationTable::GlobalVariables,
            );
            if difference.is_some() {
                return difference;
            }
            if first_op != second_op {
                return Some(DifferenceKind::Operation {
                    first: first_op.catalog_id().to_string(),
                    second: second_op.catalog_id().to_string(),
                });
            }
            argument_difference(a, b, path, 0, *first_value, *second_value)
        }
        (
            wir::Action::SetGlobalVariable {
                variable: first_variable,
                value: first_value,
                ..
            },
            wir::Action::ModifyGlobalVariable {
                variable: second_variable,
                op: second_op,
                value: second_value,
                ..
            },
        ) => {
            let first_index = global_index(a, *first_variable);
            let second_index = global_index(b, *second_variable);
            let difference =
                reference_difference(first_index, second_index, DeclarationTable::GlobalVariables);
            if difference.is_some() {
                return difference;
            }
            self_modify_difference(
                SelfModify {
                    set_program: a,
                    modify_program: b,
                    set_is_first: true,
                    target: ModifyTarget::Global { index: first_index },
                    set_value: *first_value,
                    op: *second_op,
                    modify_value: *second_value,
                },
                path,
            )
        }
        (
            wir::Action::ModifyGlobalVariable {
                variable: first_variable,
                op: first_op,
                value: first_value,
                ..
            },
            wir::Action::SetGlobalVariable {
                variable: second_variable,
                value: second_value,
                ..
            },
        ) => {
            let first_index = global_index(a, *first_variable);
            let second_index = global_index(b, *second_variable);
            let difference =
                reference_difference(first_index, second_index, DeclarationTable::GlobalVariables);
            if difference.is_some() {
                return difference;
            }
            self_modify_difference(
                SelfModify {
                    set_program: b,
                    modify_program: a,
                    set_is_first: false,
                    target: ModifyTarget::Global { index: first_index },
                    set_value: *second_value,
                    op: *first_op,
                    modify_value: *first_value,
                },
                path,
            )
        }
        (
            wir::Action::SetPlayerVariable {
                player: first_player,
                variable: first_variable,
                value: first_value,
                ..
            },
            wir::Action::SetPlayerVariable {
                player: second_player,
                variable: second_variable,
                value: second_value,
                ..
            },
        ) => {
            if let Some(kind) = argument_difference(a, b, path, 0, *first_player, *second_player) {
                return Some(kind);
            }
            let difference = reference_difference(
                player_index(a, *first_variable),
                player_index(b, *second_variable),
                DeclarationTable::PlayerVariables,
            );
            if difference.is_some() {
                return difference;
            }
            argument_difference(a, b, path, 1, *first_value, *second_value)
        }
        (
            wir::Action::ModifyPlayerVariable {
                player: first_player,
                variable: first_variable,
                op: first_op,
                value: first_value,
                ..
            },
            wir::Action::ModifyPlayerVariable {
                player: second_player,
                variable: second_variable,
                op: second_op,
                value: second_value,
                ..
            },
        ) => {
            if let Some(kind) = argument_difference(a, b, path, 0, *first_player, *second_player) {
                return Some(kind);
            }
            let difference = reference_difference(
                player_index(a, *first_variable),
                player_index(b, *second_variable),
                DeclarationTable::PlayerVariables,
            );
            if difference.is_some() {
                return difference;
            }
            if first_op != second_op {
                return Some(DifferenceKind::Operation {
                    first: first_op.catalog_id().to_string(),
                    second: second_op.catalog_id().to_string(),
                });
            }
            argument_difference(a, b, path, 1, *first_value, *second_value)
        }
        (
            wir::Action::SetPlayerVariable {
                player: first_player,
                variable: first_variable,
                value: first_value,
                ..
            },
            wir::Action::ModifyPlayerVariable {
                player: second_player,
                variable: second_variable,
                op: second_op,
                value: second_value,
                ..
            },
        ) => {
            if let Some(kind) = argument_difference(a, b, path, 0, *first_player, *second_player) {
                return Some(kind);
            }
            let first_index = player_index(a, *first_variable);
            let second_index = player_index(b, *second_variable);
            let difference =
                reference_difference(first_index, second_index, DeclarationTable::PlayerVariables);
            if difference.is_some() {
                return difference;
            }
            self_modify_difference(
                SelfModify {
                    set_program: a,
                    modify_program: b,
                    set_is_first: true,
                    target: ModifyTarget::Player {
                        index: first_index,
                        player: *first_player,
                    },
                    set_value: *first_value,
                    op: *second_op,
                    modify_value: *second_value,
                },
                path,
            )
        }
        (
            wir::Action::ModifyPlayerVariable {
                player: first_player,
                variable: first_variable,
                op: first_op,
                value: first_value,
                ..
            },
            wir::Action::SetPlayerVariable {
                player: second_player,
                variable: second_variable,
                value: second_value,
                ..
            },
        ) => {
            if let Some(kind) = argument_difference(a, b, path, 0, *first_player, *second_player) {
                return Some(kind);
            }
            let first_index = player_index(a, *first_variable);
            let second_index = player_index(b, *second_variable);
            let difference =
                reference_difference(first_index, second_index, DeclarationTable::PlayerVariables);
            if difference.is_some() {
                return difference;
            }
            self_modify_difference(
                SelfModify {
                    set_program: b,
                    modify_program: a,
                    set_is_first: false,
                    target: ModifyTarget::Player {
                        index: first_index,
                        player: *second_player,
                    },
                    set_value: *second_value,
                    op: *first_op,
                    modify_value: *first_value,
                },
                path,
            )
        }
        (
            wir::Action::AssignMember {
                target: first_target,
                op: first_op,
                value: first_value,
                ..
            },
            wir::Action::AssignMember {
                target: second_target,
                op: second_op,
                value: second_value,
                ..
            },
        ) => {
            if first_op != second_op {
                return Some(DifferenceKind::Operation {
                    first: first_op.map_or("assign", ModifyOp::catalog_id).to_string(),
                    second: second_op.map_or("assign", ModifyOp::catalog_id).to_string(),
                });
            }
            if let Some(kind) = argument_difference(a, b, path, 0, *first_target, *second_target) {
                return Some(kind);
            }
            argument_difference(a, b, path, 1, *first_value, *second_value)
        }
        (
            wir::Action::CallSubroutine {
                subroutine: first, ..
            },
            wir::Action::CallSubroutine {
                subroutine: second, ..
            },
        ) => reference_difference(
            subroutine_index(a, *first),
            subroutine_index(b, *second),
            DeclarationTable::Subroutines,
        ),
        (
            wir::Action::If {
                branches: first_branches,
                else_body: first_else,
                ..
            },
            wir::Action::If {
                branches: second_branches,
                else_body: second_else,
                ..
            },
        ) => {
            if first_branches.len() != second_branches.len() {
                return Some(DifferenceKind::Count {
                    element: "branches",
                    first: first_branches.len(),
                    second: second_branches.len(),
                });
            }
            for (index, (first, second)) in first_branches.iter().zip(second_branches).enumerate() {
                path.push(PathStep::Value(index));
                if let Some(kind) = value_difference(a, b, first.condition, second.condition, path)
                {
                    return Some(kind);
                }
                path.pop();
                path.push(PathStep::Branch(index));
                if let Some(kind) = action_list_difference(a, b, &first.body, &second.body, path) {
                    return Some(kind);
                }
                path.pop();
            }
            match (first_else, second_else) {
                (None, None) => None,
                (Some(first), Some(second)) => {
                    path.push(PathStep::ElseBody);
                    let difference = action_list_difference(a, b, first, second, path);
                    if difference.is_none() {
                        path.pop();
                    }
                    difference
                }
                (first, second) => Some(DifferenceKind::Shape {
                    first: if first.is_some() { "if/else" } else { "if" },
                    second: if second.is_some() { "if/else" } else { "if" },
                }),
            }
        }
        (
            wir::Action::While {
                condition: first_condition,
                body: first_body,
                ..
            },
            wir::Action::While {
                condition: second_condition,
                body: second_body,
                ..
            },
        ) => {
            if let Some(kind) =
                argument_difference(a, b, path, 0, *first_condition, *second_condition)
            {
                return Some(kind);
            }
            path.push(PathStep::Body);
            let difference = action_list_difference(a, b, first_body, second_body, path);
            if difference.is_none() {
                path.pop();
            }
            difference
        }
        (
            wir::Action::ForGlobalVariable {
                variable: first_variable,
                start: first_start,
                stop: first_stop,
                step: first_step,
                body: first_body,
                ..
            },
            wir::Action::ForGlobalVariable {
                variable: second_variable,
                start: second_start,
                stop: second_stop,
                step: second_step,
                body: second_body,
                ..
            },
        ) => {
            let difference = reference_difference(
                global_index(a, *first_variable),
                global_index(b, *second_variable),
                DeclarationTable::GlobalVariables,
            );
            if difference.is_some() {
                return difference;
            }
            for (index, (&first, &second)) in [
                (first_start, second_start),
                (first_stop, second_stop),
                (first_step, second_step),
            ]
            .into_iter()
            .enumerate()
            {
                if let Some(kind) = argument_difference(a, b, path, index, first, second) {
                    return Some(kind);
                }
            }
            path.push(PathStep::Body);
            let difference = action_list_difference(a, b, first_body, second_body, path);
            if difference.is_none() {
                path.pop();
            }
            difference
        }
        (
            wir::Action::ForPlayerVariable {
                player: first_player,
                variable: first_variable,
                start: first_start,
                stop: first_stop,
                step: first_step,
                body: first_body,
                ..
            },
            wir::Action::ForPlayerVariable {
                player: second_player,
                variable: second_variable,
                start: second_start,
                stop: second_stop,
                step: second_step,
                body: second_body,
                ..
            },
        ) => {
            if let Some(kind) = argument_difference(a, b, path, 0, *first_player, *second_player) {
                return Some(kind);
            }
            let difference = reference_difference(
                player_index(a, *first_variable),
                player_index(b, *second_variable),
                DeclarationTable::PlayerVariables,
            );
            if difference.is_some() {
                return difference;
            }
            for (index, (&first, &second)) in [
                (first_start, second_start),
                (first_stop, second_stop),
                (first_step, second_step),
            ]
            .into_iter()
            .enumerate()
            {
                if let Some(kind) = argument_difference(a, b, path, index + 1, first, second) {
                    return Some(kind);
                }
            }
            path.push(PathStep::Body);
            let difference = action_list_difference(a, b, first_body, second_body, path);
            if difference.is_none() {
                path.pop();
            }
            difference
        }
        (
            wir::Action::Call {
                name: first_name,
                args: first_args,
                ..
            },
            wir::Action::Call {
                name: second_name,
                args: second_args,
                ..
            },
        ) => {
            if first_name != second_name {
                return Some(DifferenceKind::Scalar {
                    first: first_name.clone(),
                    second: second_name.clone(),
                });
            }
            if first_args.len() != second_args.len() {
                return Some(DifferenceKind::Count {
                    element: "arguments",
                    first: first_args.len(),
                    second: second_args.len(),
                });
            }
            for (index, (&first, &second)) in first_args.iter().zip(second_args).enumerate() {
                if let Some(kind) = argument_difference(a, b, path, index, first, second) {
                    return Some(kind);
                }
            }
            None
        }
        _ => Some(DifferenceKind::Shape {
            first: action_shape(first),
            second: action_shape(second),
        }),
    }
}

fn reference_difference(
    first: u32,
    second: u32,
    table: DeclarationTable,
) -> Option<DifferenceKind> {
    (first != second).then_some(DifferenceKind::Reference {
        table,
        first,
        second,
    })
}

fn argument_difference(
    a: &wir::Program,
    b: &wir::Program,
    path: &mut Vec<PathStep>,
    index: usize,
    first: wir::ValueId,
    second: wir::ValueId,
) -> Option<DifferenceKind> {
    path.push(PathStep::Value(index));
    let difference = value_difference(a, b, first, second, path);
    if difference.is_none() {
        path.pop();
    }
    difference
}

struct SelfModify<'p> {
    set_program: &'p wir::Program,
    modify_program: &'p wir::Program,
    set_is_first: bool,
    target: ModifyTarget,
    set_value: wir::ValueId,
    op: ModifyOp,
    modify_value: wir::ValueId,
}

#[derive(Clone, Copy)]
enum ModifyTarget {
    Global { index: u32 },
    Player { index: u32, player: wir::ValueId },
}

impl ModifyTarget {
    fn table(&self) -> DeclarationTable {
        match self {
            Self::Global { .. } => DeclarationTable::GlobalVariables,
            Self::Player { .. } => DeclarationTable::PlayerVariables,
        }
    }

    fn index(&self) -> u32 {
        match self {
            Self::Global { index } | Self::Player { index, .. } => *index,
        }
    }

    fn shapes(&self) -> (&'static str, &'static str) {
        match self {
            Self::Global { .. } => ("setGlobalVariable", "modifyGlobalVariable"),
            Self::Player { .. } => ("setPlayerVariable", "modifyPlayerVariable"),
        }
    }

    fn value_step(&self) -> usize {
        match self {
            Self::Global { .. } => 0,
            Self::Player { .. } => 1,
        }
    }
}

fn variable_read(
    program: &wir::Program,
    value: wir::ValueId,
    target: ModifyTarget,
) -> Option<(u32, Option<wir::ValueId>)> {
    match (target, program.values.get(value).map(|node| &node.value)) {
        (ModifyTarget::Global { .. }, Some(wir::Value::GlobalVariable(variable))) => {
            Some((global_index(program, *variable), None))
        }
        (
            ModifyTarget::Player { .. },
            Some(wir::Value::PlayerVariable {
                player, variable, ..
            }),
        ) => Some((player_index(program, *variable), Some(*player))),
        (ModifyTarget::Player { .. }, Some(wir::Value::Call { name, args }))
            if name == "memberAccess" && args.len() == 2 =>
        {
            let Some(wir::Value::String(member)) =
                program.values.get(args[1]).map(|node| &node.value)
            else {
                return None;
            };
            Some((member_binding(program, member), Some(args[0])))
        }
        _ => None,
    }
}

fn self_modify_difference(
    pair: SelfModify<'_>,
    path: &mut Vec<PathStep>,
) -> Option<DifferenceKind> {
    let base = path.len();
    let difference = self_modify_inner(pair, path);
    if difference.is_none() {
        path.truncate(base);
    }
    difference
}

fn self_modify_inner(pair: SelfModify<'_>, path: &mut Vec<PathStep>) -> Option<DifferenceKind> {
    let (set_shape, modify_shape) = pair.target.shapes();
    let (first_shape, second_shape) = if pair.set_is_first {
        (set_shape, modify_shape)
    } else {
        (modify_shape, set_shape)
    };
    let Some(wir::Value::Call { name, args }) = pair
        .set_program
        .values
        .get(pair.set_value)
        .map(|node| &node.value)
    else {
        return Some(DifferenceKind::Shape {
            first: first_shape,
            second: second_shape,
        });
    };
    if args.len() != 2 {
        return Some(DifferenceKind::Shape {
            first: first_shape,
            second: second_shape,
        });
    }
    path.push(PathStep::Value(pair.target.value_step()));
    let Some((found, read_player)) = variable_read(pair.set_program, args[0], pair.target) else {
        path.pop();
        return Some(DifferenceKind::Shape {
            first: first_shape,
            second: second_shape,
        });
    };
    if found != pair.target.index() {
        path.push(PathStep::Value(0));
        let (first, second) = if pair.set_is_first {
            (found, pair.target.index())
        } else {
            (pair.target.index(), found)
        };
        return Some(DifferenceKind::Reference {
            table: pair.target.table(),
            first,
            second,
        });
    }
    if let ModifyTarget::Player { player, .. } = pair.target
        && let Some(read_player) = read_player
    {
        path.push(PathStep::Value(0));
        let difference = argument_difference(
            pair.set_program,
            pair.set_program,
            path,
            0,
            read_player,
            player,
        );
        match difference {
            Some(difference) => return Some(difference),
            None => {
                path.pop();
            }
        }
    }
    if canonical_value_name(name) != pair.op.catalog_id() {
        let (first, second) = if pair.set_is_first {
            (
                canonical_value_name(name).to_string(),
                pair.op.catalog_id().to_string(),
            )
        } else {
            (
                pair.op.catalog_id().to_string(),
                canonical_value_name(name).to_string(),
            )
        };
        return Some(DifferenceKind::Operation { first, second });
    }
    path.push(PathStep::Value(1));
    let (a, b, left, right) = if pair.set_is_first {
        (
            pair.set_program,
            pair.modify_program,
            args[1],
            pair.modify_value,
        )
    } else {
        (
            pair.modify_program,
            pair.set_program,
            pair.modify_value,
            args[1],
        )
    };
    value_difference(a, b, left, right, path)
}

fn value_difference(
    a: &wir::Program,
    b: &wir::Program,
    left: wir::ValueId,
    right: wir::ValueId,
    path: &mut Vec<PathStep>,
) -> Option<DifferenceKind> {
    let (Some(first), Some(second)) = (
        a.values.get(left).map(|node| &node.value),
        b.values.get(right).map(|node| &node.value),
    ) else {
        return Some(DifferenceKind::Shape {
            first: "value",
            second: "missing",
        });
    };
    match (first, second) {
        (wir::Value::Number { value: first, .. }, wir::Value::Number { value: second, .. }) => {
            if *first == *second || (first.is_nan() && second.is_nan()) {
                None
            } else {
                Some(DifferenceKind::Scalar {
                    first: format_number(*first),
                    second: format_number(*second),
                })
            }
        }
        (wir::Value::String(first), wir::Value::String(second)) => {
            if first == second {
                None
            } else {
                Some(DifferenceKind::Scalar {
                    first: first.clone(),
                    second: second.clone(),
                })
            }
        }
        (wir::Value::LocalizedString(first), wir::Value::LocalizedString(second)) => {
            if first == second {
                None
            } else {
                Some(DifferenceKind::Scalar {
                    first: first.clone(),
                    second: second.clone(),
                })
            }
        }
        (wir::Value::Bool(first), wir::Value::Bool(second)) => {
            if first == second {
                None
            } else {
                Some(DifferenceKind::Scalar {
                    first: first.to_string(),
                    second: second.to_string(),
                })
            }
        }
        (wir::Value::Null, wir::Value::Null) => None,
        (wir::Value::Array(first), wir::Value::Array(second)) => {
            if first.len() != second.len() {
                return Some(DifferenceKind::Count {
                    element: "elements",
                    first: first.len(),
                    second: second.len(),
                });
            }
            for (index, (&first, &second)) in first.iter().zip(second).enumerate() {
                if let Some(kind) = argument_difference(a, b, path, index, first, second) {
                    return Some(kind);
                }
            }
            None
        }
        (
            wir::Value::Vector {
                x: first_x,
                y: first_y,
                z: first_z,
            },
            wir::Value::Vector {
                x: second_x,
                y: second_y,
                z: second_z,
            },
        ) => {
            for (index, (&first, &second)) in [
                (first_x, second_x),
                (first_y, second_y),
                (first_z, second_z),
            ]
            .into_iter()
            .enumerate()
            {
                if let Some(kind) = argument_difference(a, b, path, index, first, second) {
                    return Some(kind);
                }
            }
            None
        }
        (
            wir::Value::Enum {
                value_type: first_type,
                value: first_value,
            },
            wir::Value::Enum {
                value_type: second_type,
                value: second_value,
            },
        ) => {
            if first_type == second_type && first_value == second_value {
                None
            } else {
                Some(DifferenceKind::Scalar {
                    first: format!("{first_type}.{first_value}"),
                    second: format!("{second_type}.{second_value}"),
                })
            }
        }
        (wir::Value::GlobalVariable(first), wir::Value::GlobalVariable(second)) => {
            reference_difference(
                global_index(a, *first),
                global_index(b, *second),
                DeclarationTable::GlobalVariables,
            )
        }
        (
            wir::Value::PlayerVariable {
                player: first_player,
                variable: first_variable,
            },
            wir::Value::PlayerVariable {
                player: second_player,
                variable: second_variable,
            },
        ) => {
            if let Some(kind) = argument_difference(a, b, path, 0, *first_player, *second_player) {
                return Some(kind);
            }
            reference_difference(
                player_index(a, *first_variable),
                player_index(b, *second_variable),
                DeclarationTable::PlayerVariables,
            )
        }
        (wir::Value::Subroutine(first), wir::Value::Subroutine(second)) => reference_difference(
            subroutine_index(a, *first),
            subroutine_index(b, *second),
            DeclarationTable::Subroutines,
        ),
        (wir::Value::EventPlayer, wir::Value::EventPlayer) => None,
        (wir::Value::PlayerVariable { player, variable }, wir::Value::Call { name, args })
        | (wir::Value::Call { name, args }, wir::Value::PlayerVariable { player, variable })
            if name == "memberAccess" && args.len() == 2 =>
        {
            member_access_difference(
                a,
                b,
                matches!(first, wir::Value::PlayerVariable { .. }),
                *player,
                *variable,
                args,
                path,
            )
        }
        (
            wir::Value::Call {
                name: first_name,
                args: first_args,
            },
            wir::Value::Call {
                name: second_name,
                args: second_args,
            },
        ) => {
            if first_name == wir::AMBIGUOUS_ENUM_CALL && second_name == wir::AMBIGUOUS_ENUM_CALL {
                return (!ambiguous_enum_equivalent(a, b, first_args, second_args)).then_some(
                    DifferenceKind::Scalar {
                        first: describe_ambiguous_enum(a, first_args),
                        second: describe_ambiguous_enum(b, second_args),
                    },
                );
            }
            if first_name == "memberAccess"
                && second_name == "memberAccess"
                && first_args.len() == 2
                && second_args.len() == 2
            {
                return member_access_pair_difference(a, b, first_args, second_args, path);
            }
            if canonical_value_name(first_name) != canonical_value_name(second_name) {
                return Some(DifferenceKind::Scalar {
                    first: first_name.clone(),
                    second: second_name.clone(),
                });
            }
            if first_args.len() != second_args.len() {
                return Some(DifferenceKind::Count {
                    element: "arguments",
                    first: first_args.len(),
                    second: second_args.len(),
                });
            }
            for (index, (&first, &second)) in first_args.iter().zip(second_args).enumerate() {
                if let Some(kind) = argument_difference(a, b, path, index, first, second) {
                    return Some(kind);
                }
            }
            None
        }
        _ => Some(DifferenceKind::Shape {
            first: value_shape(first),
            second: value_shape(second),
        }),
    }
}

fn member_access_difference(
    a: &wir::Program,
    b: &wir::Program,
    variable_is_first: bool,
    player: wir::ValueId,
    variable: wir::PlayerVarId,
    args: &[wir::ValueId],
    path: &mut Vec<PathStep>,
) -> Option<DifferenceKind> {
    let base = path.len();
    let difference = member_access_inner(a, b, variable_is_first, player, variable, args, path);
    if difference.is_none() {
        path.truncate(base);
    }
    difference
}

fn member_access_inner(
    a: &wir::Program,
    b: &wir::Program,
    variable_is_first: bool,
    player: wir::ValueId,
    variable: wir::PlayerVarId,
    args: &[wir::ValueId],
    path: &mut Vec<PathStep>,
) -> Option<DifferenceKind> {
    let (variable_program, member_program) = if variable_is_first { (a, b) } else { (b, a) };
    let difference = if variable_is_first {
        argument_difference(a, b, path, 0, player, args[0])
    } else {
        argument_difference(a, b, path, 0, args[0], player)
    };
    if difference.is_some() {
        return difference;
    }
    let member_node = member_program.values.get(args[1]).map(|node| &node.value);
    let Some(wir::Value::String(member)) = member_node else {
        let (first, second) = if variable_is_first {
            ("playerVariable", "call")
        } else {
            ("call", "playerVariable")
        };
        return Some(DifferenceKind::Shape { first, second });
    };
    path.push(PathStep::Value(1));
    let member_index = member_binding(member_program, member);
    let variable_index = player_index(variable_program, variable);
    let (first, second) = if variable_is_first {
        (variable_index, member_index)
    } else {
        (member_index, variable_index)
    };
    reference_difference(first, second, DeclarationTable::PlayerVariables)
}

fn member_binding(program: &wir::Program, member: &str) -> u32 {
    program
        .player_variables
        .iter()
        .find(|variable| variable.name == member)
        .map_or(u32::MAX, |variable| variable.index)
}

fn member_access_pair_difference(
    a: &wir::Program,
    b: &wir::Program,
    first_args: &[wir::ValueId],
    second_args: &[wir::ValueId],
    path: &mut Vec<PathStep>,
) -> Option<DifferenceKind> {
    let base = path.len();
    let difference = member_access_pair_inner(a, b, first_args, second_args, path);
    if difference.is_none() {
        path.truncate(base);
    }
    difference
}

fn member_access_pair_inner(
    a: &wir::Program,
    b: &wir::Program,
    first_args: &[wir::ValueId],
    second_args: &[wir::ValueId],
    path: &mut Vec<PathStep>,
) -> Option<DifferenceKind> {
    if let Some(kind) = argument_difference(a, b, path, 0, first_args[0], second_args[0]) {
        return Some(kind);
    }
    let members = (
        a.values.get(first_args[1]).map(|node| &node.value),
        b.values.get(second_args[1]).map(|node| &node.value),
    );
    let (Some(wir::Value::String(first_member)), Some(wir::Value::String(second_member))) = members
    else {
        return argument_difference(a, b, path, 1, first_args[1], second_args[1]);
    };
    path.push(PathStep::Value(1));
    let first_index = member_binding(a, first_member);
    let second_index = member_binding(b, second_member);
    if first_index != second_index {
        return Some(DifferenceKind::Reference {
            table: DeclarationTable::PlayerVariables,
            first: first_index,
            second: second_index,
        });
    }
    if first_index == u32::MAX && first_member != second_member {
        return Some(DifferenceKind::Scalar {
            first: first_member.clone(),
            second: second_member.clone(),
        });
    }
    None
}

fn describe_ambiguous_enum(program: &wir::Program, args: &[wir::ValueId]) -> String {
    match wir::ambiguous_enum_parts_by_args(program, args) {
        Some((spelling, candidates)) => format!("{spelling} over {candidates:?}"),
        None => "ambiguous enum".to_string(),
    }
}

fn action_shape(action: &wir::Action) -> &'static str {
    match action {
        wir::Action::SetGlobalVariable { .. } => "setGlobalVariable",
        wir::Action::ModifyGlobalVariable { .. } => "modifyGlobalVariable",
        wir::Action::SetPlayerVariable { .. } => "setPlayerVariable",
        wir::Action::ModifyPlayerVariable { .. } => "modifyPlayerVariable",
        wir::Action::AssignMember { op, .. } => {
            if op.is_some() {
                "modifyMember"
            } else {
                "assignMember"
            }
        }
        wir::Action::CallSubroutine { .. } => "callSubroutine",
        wir::Action::If { .. } => "if",
        wir::Action::While { .. } => "while",
        wir::Action::ForGlobalVariable { .. } => "forGlobalVariable",
        wir::Action::ForPlayerVariable { .. } => "forPlayerVariable",
        wir::Action::Disabled { .. } => "disabled",
        wir::Action::Call { .. } => "call",
    }
}

fn value_shape(value: &wir::Value) -> &'static str {
    match value {
        wir::Value::Number { .. } => "number",
        wir::Value::String(_) => "string",
        wir::Value::LocalizedString(_) => "localizedString",
        wir::Value::Bool(_) => "bool",
        wir::Value::Null => "null",
        wir::Value::Array(_) => "array",
        wir::Value::Vector { .. } => "vector",
        wir::Value::Enum { .. } => "enum",
        wir::Value::GlobalVariable(_) => "globalVariable",
        wir::Value::PlayerVariable { .. } => "playerVariable",
        wir::Value::Subroutine(_) => "subroutine",
        wir::Value::EventPlayer => "eventPlayer",
        wir::Value::Call { .. } => "call",
    }
}
