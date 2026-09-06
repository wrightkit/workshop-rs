//! Canonical Workshop action forms.

use crate::core::source::Span;

use super::{ActionId, GlobalVarId, PlayerVarId, SubroutineId, ValueId};

/// A workshop action.
#[derive(Debug, Clone)]
pub enum Action {
    SetGlobalVariable {
        variable: GlobalVarId,
        value: ValueId,
        span: Option<Span>,
        /// The exact span of the assigned variable identifier.
        target_span: Option<Span>,
    },
    ModifyGlobalVariable {
        variable: GlobalVarId,
        op: ModifyOp,
        value: ValueId,
        span: Option<Span>,
        /// The exact span of the modified variable identifier.
        target_span: Option<Span>,
    },
    SetPlayerVariable {
        player: ValueId,
        variable: PlayerVarId,
        value: ValueId,
        span: Option<Span>,
        /// The exact span of the assigned variable identifier.
        target_span: Option<Span>,
    },
    ModifyPlayerVariable {
        player: ValueId,
        variable: PlayerVarId,
        op: ModifyOp,
        value: ValueId,
        span: Option<Span>,
        /// The exact span of the modified variable identifier.
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
        /// The exact span of the callee identifier occurrence.
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
        /// The exact span of the loop variable identifier.
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
    pub fn span(&self) -> Option<Span> {
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
            | Action::Call { span, .. } => *span,
        }
    }
}

/// One condition/body pair of an `If` action.
#[derive(Debug, Clone)]
pub struct IfBranch {
    pub condition: ValueId,
    pub body: Vec<ActionId>,
}

/// The modify operators of the v0.1 surface.
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

impl ModifyOp {
    /// A short canonical name for dumps and diagnostics.
    pub fn as_str(self) -> &'static str {
        match self {
            ModifyOp::Add => "Add",
            ModifyOp::Subtract => "Subtract",
            ModifyOp::Multiply => "Multiply",
            ModifyOp::Divide => "Divide",
            ModifyOp::Modulo => "Modulo",
            ModifyOp::Min => "Min",
            ModifyOp::Max => "Max",
            ModifyOp::RaiseToPower => "RaiseToPower",
            ModifyOp::AppendToArray => "AppendToArray",
            ModifyOp::RemoveFromArray => "RemoveFromArray",
            ModifyOp::RemoveFromArrayByIndex => "RemoveFromArrayByIndex",
        }
    }

    /// The canonical catalog identity for this modification operation.
    pub fn catalog_id(self) -> &'static str {
        match self {
            ModifyOp::Add => "add",
            ModifyOp::Subtract => "subtract",
            ModifyOp::Multiply => "multiply",
            ModifyOp::Divide => "divide",
            ModifyOp::Modulo => "modulo",
            ModifyOp::Min => "min",
            ModifyOp::Max => "max",
            ModifyOp::RaiseToPower => "raiseToPower",
            ModifyOp::AppendToArray => "appendToArray",
            ModifyOp::RemoveFromArray => "removeFromArray",
            ModifyOp::RemoveFromArrayByIndex => "removeFromArrayByIndex",
        }
    }
}
