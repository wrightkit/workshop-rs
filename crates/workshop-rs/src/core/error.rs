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
        /// The nearest accepted spellings for `spelling`, when the rejecting
        /// site computed them. Diagnostics constructed without candidate
        /// context leave it empty.
        candidates: Vec<String>,
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
            candidates: Vec::new(),
            span,
        }
    }

    /// Attach the nearest accepted spellings to an `Unknown` diagnostic.
    /// Other variants are returned unchanged.
    pub fn with_candidates(mut self, candidates: Vec<String>) -> Self {
        if let WorkshopError::Unknown {
            candidates: slot, ..
        } = &mut self
        {
            *slot = candidates;
        }
        self
    }

    /// The nearest accepted spellings for an `Unknown` spelling, when the
    /// rejecting site computed them. Other variants report an empty slice.
    pub fn candidates(&self) -> &[String] {
        match self {
            WorkshopError::Unknown { candidates, .. } => candidates,
            _ => &[],
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
            | WorkshopError::Malformed { span, .. }
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
            WorkshopError::MissingMapping { kind, id, locale } => {
                write!(f, "missing {kind} mapping for locale '{locale}': '{id}'")
            }
            WorkshopError::Malformed { message, .. } => write!(f, "malformed: {message}"),
            WorkshopError::Unsupported { message, .. } => write!(f, "unsupported: {message}"),
        }
    }
}
