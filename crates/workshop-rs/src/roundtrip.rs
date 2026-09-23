//! Public complete-program round-trip comparison over [`crate::Program`].

pub use crate::output::roundtrip::*;

#[cfg(test)]
pub(crate) use crate::output::roundtrip::equivalent_wir;
