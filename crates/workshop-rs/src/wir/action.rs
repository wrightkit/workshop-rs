//! Canonical Workshop action forms.

use crate::core::source::Span;
use crate::program::shared::ModifyOp;

use super::{ActionId, GlobalVarId, PlayerVarId, SubroutineId, ValueId};

/// A workshop action.
#[derive(Debug, Clone)]
pub(crate) enum Action {
    SetGlobalVariable {
        variable: GlobalVarId,
        value: ValueId,
        span: Option<Span>,
        target_span: Option<Span>,
    },
    ModifyGlobalVariable {
        variable: GlobalVarId,
        op: ModifyOp,
        value: ValueId,
        span: Option<Span>,
        target_span: Option<Span>,
    },
    SetPlayerVariable {
        player: ValueId,
        variable: PlayerVarId,
        value: ValueId,
        span: Option<Span>,
        target_span: Option<Span>,
    },
    ModifyPlayerVariable {
        player: ValueId,
        variable: PlayerVarId,
        op: ModifyOp,
        value: ValueId,
        span: Option<Span>,
        target_span: Option<Span>,
    },
    /// Assignment to a canonical Workshop member-access target, optionally
    /// indexed. This is not a builtin catalog action; the emitter preserves
    /// the native member-assignment syntax.
    AssignMember {
        target: ValueId,
        op: Option<ModifyOp>,
        value: ValueId,
        span: Option<Span>,
    },
    CallSubroutine {
        subroutine: SubroutineId,
        span: Option<Span>,
        callee_span: Option<Span>,
    },
    If {
        branches: Vec<IfBranch>,
        else_body: Option<Vec<ActionId>>,
        span: Option<Span>,
    },
    While {
        condition: ValueId,
        body: Vec<ActionId>,
        span: Option<Span>,
    },
    ForGlobalVariable {
        variable: GlobalVarId,
        start: ValueId,
        stop: ValueId,
        step: ValueId,
        body: Vec<ActionId>,
        span: Option<Span>,
        target_span: Option<Span>,
    },
    /// `For Player Variable(player, name, start, stop, step)`: the
    /// per-player loop form (frontend-neutral; parsed from reference
    /// evidence, not emitted by Wright's own lowering, which models
    /// foreach counters as globals under the declared #119 contract).
    ForPlayerVariable {
        player: ValueId,
        variable: PlayerVarId,
        start: ValueId,
        stop: ValueId,
        step: ValueId,
        body: Vec<ActionId>,
        span: Option<Span>,
        target_span: Option<Span>,
    },
    /// An action carrying the `disabled` modifier: the wrapped action stays
    /// in the program but does not execute. For a control-flow group the
    /// modifier applies to the group header; the body is unchanged.
    Disabled {
        action: ActionId,
        span: Option<Span>,
    },
    /// Any other action call with side effects.
    Call {
        name: String,
        args: Vec<ValueId>,
        span: Option<Span>,
    },
}

impl Action {
    /// The source span of this action, if any.
    pub(crate) fn span(&self) -> Option<Span> {
        match self {
            Action::SetGlobalVariable { span, .. }
            | Action::ModifyGlobalVariable { span, .. }
            | Action::SetPlayerVariable { span, .. }
            | Action::ModifyPlayerVariable { span, .. }
            | Action::AssignMember { span, .. }
            | Action::CallSubroutine { span, .. }
            | Action::If { span, .. }
            | Action::While { span, .. }
            | Action::ForGlobalVariable { span, .. }
            | Action::ForPlayerVariable { span, .. }
            | Action::Disabled { span, .. }
            | Action::Call { span, .. } => *span,
        }
    }

    /// The source span of the variable or subroutine this action names: the
    /// target of set/modify and for-variable actions, or the callee of a
    /// `Call Subroutine` action. `None` for variants without an identifier
    /// field.
    pub(crate) fn identifier_span(&self) -> Option<Span> {
        match self {
            Action::SetGlobalVariable { target_span, .. }
            | Action::ModifyGlobalVariable { target_span, .. }
            | Action::SetPlayerVariable { target_span, .. }
            | Action::ModifyPlayerVariable { target_span, .. }
            | Action::ForGlobalVariable { target_span, .. }
            | Action::ForPlayerVariable { target_span, .. } => *target_span,
            Action::CallSubroutine { callee_span, .. } => *callee_span,
            _ => None,
        }
    }

    /// Mutable access to this action's source span field.
    pub(crate) fn span_mut(&mut self) -> &mut Option<Span> {
        match self {
            Action::SetGlobalVariable { span, .. }
            | Action::ModifyGlobalVariable { span, .. }
            | Action::SetPlayerVariable { span, .. }
            | Action::ModifyPlayerVariable { span, .. }
            | Action::AssignMember { span, .. }
            | Action::CallSubroutine { span, .. }
            | Action::If { span, .. }
            | Action::While { span, .. }
            | Action::ForGlobalVariable { span, .. }
            | Action::ForPlayerVariable { span, .. }
            | Action::Disabled { span, .. }
            | Action::Call { span, .. } => span,
        }
    }

    /// Mutable access to this action's identifier span field, when the
    /// variant carries one (see [`identifier_span`](Self::identifier_span)).
    pub(crate) fn identifier_span_mut(&mut self) -> Option<&mut Option<Span>> {
        match self {
            Action::SetGlobalVariable { target_span, .. }
            | Action::ModifyGlobalVariable { target_span, .. }
            | Action::SetPlayerVariable { target_span, .. }
            | Action::ModifyPlayerVariable { target_span, .. }
            | Action::ForGlobalVariable { target_span, .. }
            | Action::ForPlayerVariable { target_span, .. } => Some(target_span),
            Action::CallSubroutine { callee_span, .. } => Some(callee_span),
            _ => None,
        }
    }

    /// The direct value arguments of this action, in canonical order. Block
    /// bodies are separate actions and are not included. Provenance rows pair
    /// these positionally with `program::action_argument_values`, so the two
    /// tables must stay consistent.
    pub(crate) fn value_args(&self) -> Vec<ValueId> {
        match self {
            Action::SetGlobalVariable { value, .. }
            | Action::ModifyGlobalVariable { value, .. } => {
                vec![*value]
            }
            Action::SetPlayerVariable { player, value, .. }
            | Action::ModifyPlayerVariable { player, value, .. } => vec![*player, *value],
            Action::AssignMember { target, value, .. } => vec![*target, *value],
            Action::If { branches, .. } => branches.iter().map(|branch| branch.condition).collect(),
            Action::While { condition, .. } => vec![*condition],
            Action::ForGlobalVariable {
                start, stop, step, ..
            } => vec![*start, *stop, *step],
            Action::ForPlayerVariable {
                player,
                start,
                stop,
                step,
                ..
            } => vec![*player, *start, *stop, *step],
            Action::Call { args, .. } => args.clone(),
            Action::CallSubroutine { .. } | Action::Disabled { .. } => Vec::new(),
        }
    }

    /// The modify operation for `Modify*`/`AssignMember` actions; `None` for
    /// plain sets and non-assignment actions.
    pub(crate) fn modify_op(&self) -> Option<ModifyOp> {
        match self {
            Action::ModifyGlobalVariable { op, .. } | Action::ModifyPlayerVariable { op, .. } => {
                Some(*op)
            }
            Action::AssignMember { op, .. } => *op,
            _ => None,
        }
    }
}

/// One condition/body pair of an `If` action.
#[derive(Debug, Clone)]
pub(crate) struct IfBranch {
    pub(crate) condition: ValueId,
    pub(crate) body: Vec<ActionId>,
}
