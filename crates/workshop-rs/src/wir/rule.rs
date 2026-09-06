//! Canonical Workshop declarations and rules.

use crate::core::source::Span;

use super::{ActionId, Event, ValueId};

/// A workshop variable (global or player) with its assigned index.
#[derive(Debug, Clone)]
pub struct WorkshopVariable {
    pub name: String,
    /// The workshop variable index assigned during lowering.
    pub index: u32,
    pub span: Option<Span>,
    /// The exact span of the declared identifier token.
    pub name_span: Option<Span>,
}

/// A workshop subroutine with its assigned index.
#[derive(Debug, Clone)]
pub struct WorkshopSubroutine {
    pub name: String,
    pub index: u32,
    pub span: Option<Span>,
    /// The exact span of the declared identifier token.
    pub name_span: Option<Span>,
}

/// A workshop rule.
#[derive(Debug, Clone)]
pub struct Rule {
    pub name: String,
    pub span: Option<Span>,
    /// The exact span of the rule name inside its string literal.
    pub name_span: Option<Span>,
    pub disabled: bool,
    pub event: Event,
    pub conditions: Vec<ValueId>,
    pub actions: Vec<ActionId>,
}
