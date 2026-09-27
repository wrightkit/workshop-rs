//! The Workshop rule and declaration domain.
//!
//! Rule, variable, and subroutine data is modeled canonically in [`crate::Program`].
//! Whole-program inspection is available from this domain entry point;
//! canonical validation is at [`crate::validate::validate_canonical_ids`].

pub(crate) mod emitter;
pub(crate) mod parser;
pub(crate) mod validate;

pub use crate::analysis::semantic::{
    IncompletenessKind, ResidualClassification, SemanticIssue, inspect,
};
pub use crate::program::{Condition, Program, Rule, Subroutine, Variable};
