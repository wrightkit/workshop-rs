//! The Workshop value and expression domain.
//!
//! Value nodes and their canonical expression forms are modeled in [`crate::wir`].
//! The shared parser and output boundaries preserve their locale-independent
//! identities.

pub use crate::wir::{Value, ValueId, ValueNode};
