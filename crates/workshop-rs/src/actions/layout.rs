//! Native Workshop layout queries for canonical actions.

use crate::catalog::{Catalog, Locale};
use crate::output::emitter::EmitContext;
use crate::wir;

/// The number of native Workshop actions emitted by a canonical WIR action
/// sequence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActionLayout {
    /// The action count in the canonical native action stream.
    pub width: usize,
}

/// Errors returned while querying canonical native action layout.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionLayoutError {
    /// The WIR does not satisfy its structural invariants.
    InvalidWIR(crate::wir::error::IrError),
    /// Canonical emission could not expand the requested actions.
    Emission(crate::WorkshopError),
}

impl std::fmt::Display for ActionLayoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidWIR(error) => write!(f, "invalid WIR: {error}"),
            Self::Emission(error) => write!(f, "action layout emission failed: {error}"),
        }
    }
}

impl std::error::Error for ActionLayoutError {}

/// Query the native Workshop action width of a validated WIR action sequence.
///
/// The sequence is expanded using the same recursive action implementation as
/// [`crate::emitter::emit`]. Every action in the sequence is treated as
/// non-rule-final, which is the canonical stream contract needed for relative
/// action offsets. The returned width counts native Workshop action lines,
/// including structural headers and terminators.
pub fn action_width(
    program: &wir::Program,
    catalog: &Catalog,
    locale: &Locale,
    actions: &[wir::ActionId],
) -> std::result::Result<ActionLayout, ActionLayoutError> {
    program.validate().map_err(ActionLayoutError::InvalidWIR)?;
    let mut emitter = EmitContext {
        program,
        catalog,
        locale: locale.clone(),
        fallback: None,
        fallback_ids: Vec::new(),
        force_hero_constructors: false,
        out: String::new(),
        line_count: 0,
    };
    for action in actions {
        emitter
            .action(*action, 0, false)
            .map_err(ActionLayoutError::Emission)?;
    }
    Ok(ActionLayout {
        width: emitter.line_count,
    })
}
