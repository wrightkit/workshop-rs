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

/// Errors returned while querying public canonical native action layout.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionLayoutError {
    /// The public program cannot be materialized as a valid Workshop program.
    InvalidProgram { message: String },
    /// Canonical emission could not expand the requested actions.
    Emission(crate::WorkshopError),
}

impl std::fmt::Display for ActionLayoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidProgram { message } => write!(f, "invalid program: {message}"),
            Self::Emission(error) => write!(f, "action layout emission failed: {error}"),
        }
    }
}

impl std::error::Error for ActionLayoutError {}

#[doc(hidden)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WIRActionLayoutError {
    InvalidWIR(crate::wir::error::IrError),
    Emission(crate::WorkshopError),
}

impl std::fmt::Display for WIRActionLayoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidWIR(error) => write!(f, "invalid WIR: {error}"),
            Self::Emission(error) => write!(f, "action layout emission failed: {error}"),
        }
    }
}

impl std::error::Error for WIRActionLayoutError {}

/// Query the native Workshop action width of a validated public action sequence.
///
/// The sequence is expanded using the same recursive action implementation as
/// [`crate::emitter::emit`]. Every action in the sequence is treated as
/// non-rule-final, which is the canonical stream contract needed for relative
/// action offsets. The returned width counts native Workshop action lines,
/// including structural headers and terminators.
pub fn action_width(
    program: &crate::Program,
    catalog: &Catalog,
    locale: &Locale,
    actions: &[crate::Action],
) -> std::result::Result<ActionLayout, ActionLayoutError> {
    let mut input = crate::Program::new();
    input.settings = program.settings.clone();
    input.global_variables = program.global_variables.clone();
    input.player_variables = program.player_variables.clone();
    input.subroutines = program.subroutines.clone();
    input
        .rules
        .push(crate::Rule::new("action layout", crate::Event::Global));
    input.rules[0].actions = actions.to_vec();
    let storage = input
        .to_wir()
        .map_err(|error| ActionLayoutError::InvalidProgram {
            message: error.to_string(),
        })?;
    let rule = storage.rules.iter().next().ok_or_else(|| {
        ActionLayoutError::Emission(crate::WorkshopError::Malformed {
            message: "action layout program has no rule".to_string(),
            span: None,
        })
    })?;
    action_width_wir(&storage, catalog, locale, &rule.actions).map_err(|error| match error {
        WIRActionLayoutError::InvalidWIR(error) => ActionLayoutError::InvalidProgram {
            message: error.to_string(),
        },
        WIRActionLayoutError::Emission(error) => ActionLayoutError::Emission(error),
    })
}

#[doc(hidden)]
pub fn action_width_wir(
    program: &wir::Program,
    catalog: &Catalog,
    locale: &Locale,
    actions: &[wir::ActionId],
) -> std::result::Result<ActionLayout, WIRActionLayoutError> {
    program
        .validate()
        .map_err(WIRActionLayoutError::InvalidWIR)?;
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
            .map_err(WIRActionLayoutError::Emission)?;
    }
    Ok(ActionLayout {
        width: emitter.line_count,
    })
}
