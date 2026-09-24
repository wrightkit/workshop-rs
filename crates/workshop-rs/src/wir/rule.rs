//! Canonical Workshop declarations and rules.

use crate::core::source::Span;

use super::{ActionId, Event, ValueId};

/// A workshop variable (global or player) with its assigned index.
#[derive(Debug, Clone)]
pub(crate) struct WorkshopVariable {
    pub(crate) name: String,
    /// The workshop variable index assigned during lowering.
    pub(crate) index: u32,
    pub(crate) span: Option<Span>,
    /// The exact span of the declared identifier token.
    pub(crate) name_span: Option<Span>,
}

/// A workshop subroutine with its assigned index.
#[derive(Debug, Clone)]
pub(crate) struct WorkshopSubroutine {
    pub(crate) name: String,
    pub(crate) index: u32,
    pub(crate) span: Option<Span>,
    /// The exact span of the declared identifier token.
    pub(crate) name_span: Option<Span>,
}

/// A rule condition; `disabled` conditions stay in the rule but are not
/// evaluated.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Condition {
    pub(crate) value: ValueId,
    pub(crate) disabled: bool,
}

/// A workshop rule.
#[derive(Debug, Clone)]
pub(crate) struct Rule {
    pub(crate) name: String,
    pub(crate) span: Option<Span>,
    #[allow(dead_code)]
    pub(crate) name_span: Option<Span>,
    pub(crate) disabled: bool,
    pub(crate) event: Event,
    pub(crate) conditions: Vec<Condition>,
    pub(crate) actions: Vec<ActionId>,
}
