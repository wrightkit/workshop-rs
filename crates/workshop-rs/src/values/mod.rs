//! The Workshop value and expression domain.
//!
//! Values and their canonical expression forms are modeled in [`crate::Program`].
//! The shared parser and output boundaries preserve their locale-independent
//! identities.

pub(crate) mod emitter;
pub(crate) mod parser;
pub(crate) mod validate;

pub use crate::program::Value;
