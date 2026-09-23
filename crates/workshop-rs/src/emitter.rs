//! Public complete-program Workshop emission over [`crate::Program`].

pub use crate::actions::{ActionLayout, ActionLayoutError, action_width};
pub use crate::output::emitter::*;

#[cfg(test)]
pub(crate) use crate::output::emitter::{emit_wir, emit_wir_with_options};
