//! Public raw Workshop parsing operation over the canonical [`crate::Program`].

pub use crate::frontend::parser::*;

#[cfg(test)]
pub(crate) use crate::frontend::parser::parse_wir;
