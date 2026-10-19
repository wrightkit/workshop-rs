//! Behavioral comparison of canonical Workshop programs.
//!
//! [`equivalent`] reports whether two [`Program`](crate::Program)s behave the
//! same and [`difference`] returns the first behavior-relevant difference.
//! This is the comparison behind source-language reconstruction contracts
//! such as Workshop→OPY: reconstruction is judged by behavior, not by
//! structural identity. [`crate::roundtrip::equivalent`] remains the strict
//! structural check.

pub use crate::output::compare::*;
