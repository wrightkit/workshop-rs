use crate::core::source::Span;

use super::{GlobalVarId, PlayerVarId, SubroutineId, ValueId};

#[derive(Debug, Clone)]
pub(crate) struct ValueNode {
    pub(crate) value: Value,
    pub(crate) span: Option<Span>,
    /// The authored extent of the whole sub-expression this node heads: from
    /// its first token through its last token, including infix operands and
    /// closing delimiters. Recorded by raw parsing only.
    pub(crate) extent: Option<Span>,
    pub(crate) identifier: Option<Span>,
}

#[derive(Debug, Clone)]
pub(crate) enum Value {
    Number {
        value: f64,
        text: String,
    },
    String(String),
    LocalizedString(String),
    Bool(bool),
    Null,
    Array(Vec<ValueId>),
    Vector {
        x: ValueId,
        y: ValueId,
        z: ValueId,
    },
    Enum {
        value_type: String,
        value: String,
    },
    GlobalVariable(GlobalVarId),
    PlayerVariable {
        player: ValueId,
        variable: PlayerVarId,
    },
    Subroutine(SubroutineId),
    EventPlayer,
    Call {
        name: String,
        args: Vec<ValueId>,
    },
}

impl ValueNode {
    pub(crate) fn new(value: Value, span: Option<Span>) -> Self {
        ValueNode {
            value,
            span,
            extent: None,
            identifier: None,
        }
    }

    pub(crate) fn with_identifier(mut self, identifier: Option<Span>) -> Self {
        self.identifier = identifier;
        self
    }

    /// Record the authored extent of this node's sub-expression.
    pub(crate) fn with_extent(mut self, extent: Option<Span>) -> Self {
        self.extent = extent;
        self
    }
}
