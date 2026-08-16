//! Raw Workshop locale conversion: parse -> canonical semantics -> emit.
//!
//! [`convert`] parses raw Workshop text in a source locale into
//! locale-independent WIR, then emits it in a target locale. Canonical
//! identities are locale-independent; only the spellings change. Missing
//! target-locale mappings fail explicitly by default (an error, never a
//! guess and never a silent passthrough of another locale's spelling);
//! fallback is opt-in via [`ConvertOptions`] and recorded in
//! [`Conversion::fallback_ids`].

use crate::catalog::{Catalog, Locale};
use crate::emitter::{self, EmitOptions};
use crate::error::Result;
use crate::parser;
use crate::signatures::ExpectedDomain;

/// Conversion options: opt-in fallback for missing target-locale mappings.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ConvertOptions {
    /// When a canonical identity has no spelling for the target locale, its
    /// spelling in this declared locale is used instead. `None` (the
    /// default) keeps missing mappings failing explicitly. The fallback
    /// choice is visible in [`Conversion::fallback_ids`].
    pub fallback_locale: Option<Locale>,
}

/// The result of a raw Workshop locale conversion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Conversion {
    /// The converted localized Workshop text.
    pub text: String,
    /// Canonical identities (and the `settings` marker) whose spelling came
    /// from the opt-in fallback locale instead of the target locale. Empty
    /// when no fallback occurred.
    pub fallback_ids: Vec<String>,
}

/// Convert raw Workshop text from `from` to `to`, parsing with the catalog as
/// the canonical signature context (expected enum domains resolve ambiguous
/// bare members that the catalog documents).
pub fn convert(
    input: &str,
    catalog: &Catalog,
    from: &Locale,
    to: &Locale,
    options: &ConvertOptions,
) -> Result<Conversion> {
    convert_with_context(input, catalog, from, to, options, catalog)
}

/// The context-aware form of [`convert`]: `context` supplies the expected
/// enum domains for argument positions the catalog does not document (e.g. a
/// provider manifest chained with the catalog via
/// [`crate::signatures::ChainedExpectedDomain`]).
pub fn convert_with_context(
    input: &str,
    catalog: &Catalog,
    from: &Locale,
    to: &Locale,
    options: &ConvertOptions,
    context: &dyn ExpectedDomain,
) -> Result<Conversion> {
    let program = parser::parse_with_context(input, catalog, from, context)?;
    let emit_options = EmitOptions {
        fallback_locale: options.fallback_locale.clone(),
    };
    let output = emitter::emit_with_options(&program, catalog, to, &emit_options)?;
    Ok(Conversion {
        text: output.text,
        fallback_ids: output.fallback_ids,
    })
}
