//! The Workshop action domain.
//!
//! Action data is modeled canonically in [`crate::wir`]. Action-specific
//! layout and element-count operations are re-exported here so contributors
//! can start from the domain rather than from an implementation phase.

pub use crate::analysis::element_count::{
    ElementCountError, ElementCountNode, ElementCountReport, ElementNodeKind,
};
pub use crate::output::emitter::{ActionLayout, ActionLayoutError, action_width};
pub use crate::wir::{Action, ActionId, IfBranch, ModifyOp};
