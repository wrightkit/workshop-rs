//! Public cross-domain canonical validation over [`crate::Program`].

pub use crate::rules::validate::*;

#[cfg(test)]
pub(crate) use crate::rules::validate::validate_canonical_ids_wir;
