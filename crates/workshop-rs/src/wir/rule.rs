use crate::core::source::Span;

use super::{ActionId, Event, ValueId};

#[derive(Debug, Clone)]
pub(crate) struct WorkshopVariable {
    pub(crate) name: String,
    pub(crate) index: u32,
    pub(crate) span: Option<Span>,
    pub(crate) name_span: Option<Span>,
}

#[derive(Debug, Clone)]
pub(crate) struct WorkshopSubroutine {
    pub(crate) name: String,
    pub(crate) index: u32,
    pub(crate) span: Option<Span>,
    pub(crate) name_span: Option<Span>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Condition {
    pub(crate) value: ValueId,
    pub(crate) disabled: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct Rule {
    pub(crate) name: String,
    pub(crate) span: Option<Span>,
    pub(crate) name_span: Option<Span>,
    pub(crate) disabled: bool,
    pub(crate) event: Event,
    pub(crate) conditions: Vec<Condition>,
    pub(crate) actions: Vec<ActionId>,
}
