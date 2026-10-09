//! Structured errors for the Workshop language model.

use crate::catalog::Locale;
use crate::core::source::Span;

/// A structured Workshop-language error.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum WorkshopError {
    /// Catalog data is malformed or fails validation.
    Catalog(CatalogError),
    /// A localized spelling is unknown or ambiguous.
    Unknown {
        kind: &'static str,
        spelling: String,
        locale: Locale,
        span: Option<Span>,
    },
    /// A localized spelling is unknown or ambiguous, and the rejecting site
    /// attached its nearest accepted spellings.
    ///
    /// This is a separate variant from [`WorkshopError::Unknown`] so the
    /// candidate list does not change that variant's shape; consumers
    /// matching `Unknown` should also match this variant to handle every
    /// unknown-spelling diagnostic. The rendered message names the
    /// candidates, so they reach consumers that only see the message text.
    UnknownWithCandidates {
        kind: &'static str,
        spelling: String,
        locale: Locale,
        /// The nearest accepted spellings for `spelling` in the rejecting
        /// site's accepted space; empty when nothing was close.
        candidates: Vec<String>,
        span: Option<Span>,
    },
    /// Detection found no usable evidence in the input, so nothing was
    /// resolved and no spelling was rejected.
    ///
    /// This is a separate variant from [`WorkshopError::Unknown`]: `Unknown`
    /// reports a rejected spelling under a resolved locale, while a
    /// zero-evidence detection has no rejected spelling to name and no
    /// selected locale — naming the detection ranking's first candidate
    /// would report an arbitrary ordering tiebreak as if it were evidence.
    /// Consumers matching `Unknown` do not see this variant, which is how
    /// they tell a nameless detection failure from a genuine rejection.
    NotDetected {
        kind: &'static str,
        /// What evidence was missing and how to supply it explicitly.
        message: String,
        span: Option<Span>,
    },
    /// A canonical builtin has no spelling mapped for the target locale.
    ///
    /// Missing target-locale mappings fail explicitly (never a guess, never a
    /// silent passthrough of another locale's spelling); fallback is opt-in
    /// ([`crate::emitter::EmitOptions`], [`crate::convert::ConvertOptions`]).
    MissingMapping {
        kind: &'static str,
        id: String,
        locale: Locale,
    },
    /// The input is syntactically malformed.
    Malformed { message: String, span: Option<Span> },
    /// An advisory diagnostic about accepted input: the construct is
    /// carried and emitted as written, but a caller should surface the
    /// message (for example a misspelling of a declared settings member).
    /// Warnings never appear where an error decides acceptance; they are
    /// returned only through diagnostic inventories that carry a severity.
    Warning { message: String, span: Option<Span> },
    /// A construct is recognized but outside the supported surface.
    Unsupported { message: String, span: Option<Span> },
}

impl WorkshopError {
    /// A `Malformed` error.
    pub fn malformed(message: impl Into<String>, span: Option<Span>) -> Self {
        WorkshopError::Malformed {
            message: message.into(),
            span,
        }
    }

    /// An `Unknown` spelling error.
    pub fn unknown(
        kind: &'static str,
        spelling: impl Into<String>,
        locale: Locale,
        span: Option<Span>,
    ) -> Self {
        WorkshopError::Unknown {
            kind,
            spelling: spelling.into(),
            locale,
            span,
        }
    }

    /// A `NotDetected` detection-failure error.
    pub fn not_detected(
        kind: &'static str,
        message: impl Into<String>,
        span: Option<Span>,
    ) -> Self {
        WorkshopError::NotDetected {
            kind,
            message: message.into(),
            span,
        }
    }

    /// Attach the nearest accepted spellings to an `Unknown` diagnostic,
    /// producing [`WorkshopError::UnknownWithCandidates`]. Calling this on
    /// a candidate-carrying error replaces the list; other variants are
    /// returned unchanged.
    pub fn with_candidates(self, candidates: Vec<String>) -> Self {
        match self {
            WorkshopError::Unknown {
                kind,
                spelling,
                locale,
                span,
            } => WorkshopError::UnknownWithCandidates {
                kind,
                spelling,
                locale,
                candidates,
                span,
            },
            mut error => {
                if let WorkshopError::UnknownWithCandidates {
                    candidates: slot, ..
                } = &mut error
                {
                    *slot = candidates;
                }
                error
            }
        }
    }

    /// The nearest accepted spellings for an `Unknown` spelling, when the
    /// rejecting site computed them. Other variants report an empty slice.
    pub fn candidates(&self) -> &[String] {
        match self {
            WorkshopError::UnknownWithCandidates { candidates, .. } => candidates,
            _ => &[],
        }
    }

    /// A `Warning` advisory diagnostic.
    pub fn warning(message: impl Into<String>, span: Option<Span>) -> Self {
        WorkshopError::Warning {
            message: message.into(),
            span,
        }
    }

    /// An `Unsupported` construct error.
    pub fn unsupported(message: impl Into<String>, span: Option<Span>) -> Self {
        WorkshopError::Unsupported {
            message: message.into(),
            span,
        }
    }

    /// The source span attached to this error, when recorded.
    pub fn span(&self) -> Option<Span> {
        match self {
            WorkshopError::Unknown { span, .. }
            | WorkshopError::UnknownWithCandidates { span, .. }
            | WorkshopError::NotDetected { span, .. }
            | WorkshopError::Malformed { span, .. }
            | WorkshopError::Warning { span, .. }
            | WorkshopError::Unsupported { span, .. } => *span,
            WorkshopError::Catalog(_) | WorkshopError::MissingMapping { .. } => None,
        }
    }
}

/// Catalog-specific error.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct CatalogError {
    pub code: &'static str,
    pub message: String,
}

impl CatalogError {
    pub(crate) fn malformed(message: String) -> WorkshopError {
        WorkshopError::Catalog(CatalogError {
            code: "malformed-catalog",
            message,
        })
    }

    pub(crate) fn validation(message: String) -> WorkshopError {
        WorkshopError::Catalog(CatalogError {
            code: "invalid-catalog",
            message,
        })
    }
}

/// A crate-wide result alias.
pub(crate) type Result<T> = std::result::Result<T, WorkshopError>;

impl std::fmt::Display for WorkshopError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkshopError::Catalog(error) => write!(f, "{}: {}", error.code, error.message),
            WorkshopError::Unknown {
                kind,
                spelling,
                locale,
                ..
            } => {
                write!(
                    f,
                    "unknown {kind} spelling '{spelling}' for locale '{locale}'"
                )
            }
            WorkshopError::UnknownWithCandidates {
                kind,
                spelling,
                locale,
                candidates,
                ..
            } => {
                let message = format!("unknown {kind} spelling '{spelling}' for locale '{locale}'");
                write!(
                    f,
                    "{}",
                    crate::core::suggest::with_candidates_text(message, candidates)
                )
            }
            WorkshopError::NotDetected { kind, message, .. } => {
                write!(f, "{kind} not detected: {message}")
            }
            WorkshopError::MissingMapping { kind, id, locale } => {
                write!(f, "missing {kind} mapping for locale '{locale}': '{id}'")
            }
            WorkshopError::Malformed { message, .. } => write!(f, "malformed: {message}"),
            WorkshopError::Warning { message, .. } => write!(f, "{message}"),
            WorkshopError::Unsupported { message, .. } => write!(f, "unsupported: {message}"),
        }
    }
}
