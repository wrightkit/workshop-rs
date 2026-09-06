//! Shared Workshop output operations.
//!
//! Emission, locale conversion, and round-trip comparison operate on complete
//! canonical programs and therefore remain an explicit cross-domain boundary.

pub(crate) mod convert;
pub(crate) mod emitter;
pub(crate) mod roundtrip;
