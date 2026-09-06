//! The Workshop rule and declaration domain.
//!
//! Rule, variable, and subroutine data is modeled canonically in [`crate::wir`].
//! Whole-program inspection and validation are available from this domain
//! entry point as well as their compatibility modules.

pub use crate::analysis::semantic::{
    IncompletenessKind, ResidualClassification, SemanticIssue, inspect,
};
pub use crate::analysis::validate::validate_canonical_ids;
pub use crate::wir::{Program, Rule, RuleId, WorkshopSubroutine, WorkshopVariable};
