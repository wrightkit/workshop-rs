//! Shared, locale-independent infrastructure used by Workshop domains.
//!
//! These modules provide storage, identity, source locations, formatting, and
//! shared errors. They do not own Workshop features.

pub(crate) mod arena;
pub(crate) mod error;
pub(crate) mod format;
pub(crate) mod ids;
pub(crate) mod signatures;
pub(crate) mod source;
