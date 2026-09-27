//! The Workshop action domain.
//!
//! Action data is modeled canonically in [`crate::Program`]. Action-specific
//! layout and element-count operations live in [`crate::emitter`], the
//! Workshop-operations entry point; this domain module re-exports the model
//! types only.

pub(crate) mod emitter;
pub(crate) mod layout;
pub(crate) mod parser;
pub(crate) mod validate;

pub use crate::analysis::element_count::{
    ElementCountError, ElementCountNode, ElementCountReport, ElementNodeKind,
};
pub use crate::program::{Action, ModifyOp};
