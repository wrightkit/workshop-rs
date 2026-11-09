use crate::catalog::{Catalog, Locale};
use crate::core::error::{Result, WorkshopError};
use crate::output::emitter::{EmitOptions, EmitOutput, emit_with_options_inner};
use crate::rules::validate::validate_wir;
use crate::wir;

use super::Program;

/// A [`Program`] materialized once into its canonical form.
///
/// [`Program::validate`], the `validate` module functions, and the `emitter`
/// module functions each materialize the program on every call. A caller that
/// runs several of them on the same unchanged program calls
/// [`Program::canonical`] once and uses the methods here, which give the same
/// results without repeating the materialization.
#[derive(Debug)]
pub struct CanonicalProgram(wir::Program);

impl Program {
    /// Materialize the canonical form used by validation and emission.
    pub fn canonical(&self) -> Result<CanonicalProgram> {
        self.to_wir().map(CanonicalProgram)
    }
}

impl CanonicalProgram {
    /// Same contract as [`Program::validate`].
    pub fn validate(&self) -> Result<()> {
        self.0
            .validate()
            .map_err(|error| WorkshopError::malformed(error.to_string(), error.span()))?;
        if let Some(settings) = &self.0.settings
            && let Some(error) = crate::settings::check_emission(settings).into_iter().next()
        {
            return Err(error);
        }
        Ok(())
    }

    /// Same contract as [`crate::validate::validate_canonical_ids`].
    pub fn validate_canonical_ids(&self, catalog: &Catalog) -> Result<()> {
        validate_wir(&self.0, catalog, false)
    }

    /// Same contract as
    /// [`crate::validate::validate_canonical_ids_tolerating_residuals`].
    pub fn validate_canonical_ids_tolerating_residuals(&self, catalog: &Catalog) -> Result<()> {
        validate_wir(&self.0, catalog, true)
    }

    /// Same contract as [`crate::emitter::emit_with_options`].
    pub fn emit_with_options(
        &self,
        catalog: &Catalog,
        locale: &Locale,
        options: &EmitOptions,
    ) -> Result<EmitOutput> {
        emit_with_options_inner(&self.0, catalog, locale, options)
    }
}
