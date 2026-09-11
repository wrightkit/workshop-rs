//! The Workshop action domain.
//!
//! Action data is modeled canonically in [`crate::Program`]. Action-specific
//! layout and element-count operations are re-exported here so contributors
//! can start from the domain rather than from an implementation phase.

pub(crate) mod emitter;
mod layout;
pub(crate) mod parser;
pub(crate) mod validate;

pub use crate::analysis::element_count::{
    ElementCountError, ElementCountNode, ElementCountReport, ElementNodeKind,
};
pub use crate::program::{Action, ModifyOp};
pub use layout::{
    ActionLayout, ActionLayoutError, WIRActionLayoutError, action_width, action_width_wir,
};
