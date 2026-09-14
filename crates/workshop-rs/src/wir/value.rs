//! Canonical Workshop value and expression forms.

use crate::core::source::Span;

use super::{GlobalVarId, PlayerVarId, SubroutineId, ValueId};

/// A workshop value (expression) node with its source span.
#[derive(Debug, Clone)]
pub struct ValueNode {
    pub value: Value,
    pub span: Option<Span>,
}

/// A workshop value (expression).
#[derive(Debug, Clone)]
pub enum Value {
    /// A numeric literal with its source spelling (`5`, `0.0`, `-22.05`);
    /// computed values (constant folding) carry the formatted spelling.
    Number {
        value: f64,
        text: String,
    },
    String(String),
    /// A reviewed localized Workshop preset-string identity.
    LocalizedString(String),
    Bool(bool),
    Null,
    Array(Vec<ValueId>),
    Vector {
        x: ValueId,
        y: ValueId,
        z: ValueId,
    },
    /// A built-in enumerated value, e.g. `Team.ALL`.
    Enum {
        value_type: String,
        value: String,
    },
    GlobalVariable(GlobalVarId),
    PlayerVariable {
        player: ValueId,
        variable: PlayerVarId,
    },
    /// A declared Workshop subroutine referenced by a generic action such as
    /// `Start Rule`. The identity is source-owned, not a catalog builtin.
    Subroutine(SubroutineId),
    EventPlayer,
    /// A function call over workshop values.
    Call {
        name: String,
        args: Vec<ValueId>,
    },
}

impl ValueNode {
    /// Build a value node with a source span.
    pub fn new(value: Value, span: Option<Span>) -> Self {
        ValueNode { value, span }
    }
}
