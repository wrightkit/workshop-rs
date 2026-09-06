//! The Workshop action domain.
//!
//! Action data is modeled canonically in [`crate::wir`]. Action-specific
//! layout and element-count operations are re-exported here so contributors
//! can start from the domain rather than from an implementation phase.

pub(crate) mod emitter;
mod layout;
pub(crate) mod parser;
pub(crate) mod validate;

pub use crate::analysis::element_count::{
    ElementCountError, ElementCountNode, ElementCountReport, ElementNodeKind,
};
pub use crate::wir::{Action, ActionId, IfBranch, ModifyOp};
pub use layout::{ActionLayout, ActionLayoutError, action_width};
