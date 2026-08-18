//! The canonical multi-locale Overwatch Workshop semantic core.
//!
//! `workshop-rs` owns the Workshop-language foundation of the WrightKit
//! ecosystem: canonical Workshop semantics, the catalog of Workshop-defined
//! content, the raw Workshop frontend, validation, deterministic emission,
//! and Workshop IR. It is standalone and MIT-licensed; it has no dependency on
//! Wright tooling crates or on any source-language provider implementation.
//!
//! * [`catalog`] — the canonical Workshop catalog: stable, locale-independent
//!   semantic identities, kinds, parameters, and locale tables binding
//!   identities to client spellings; catalog version/digest identity and
//!   per-locale coverage;
//! * `lexer`/`parser` — the native localized Workshop frontend producing
//!   validated Workshop IR;
//! * [`wir`] — the Workshop IR model (locale-independent semantic
//!   representation) with its arena/source/settings support;
//! * [`emitter`] — deterministic localized Workshop emission, failing
//!   explicitly on missing target-locale mappings (opt-in fallback);
//! * [`detect`] — Workshop client-language detection and explicit override;
//! * [`validate`] — catalog-backed validation of canonical builtin references;
//! * [`roundtrip`] — cross-locale round-trip validation;
//! * [`convert`] — raw Workshop locale conversion (parse -> canonical
//!   semantics -> emit).
//!
//! The catalog is locale-independent at the identity layer: analyzer and WIR
//! APIs never need locale-specific strings to identify a builtin. Locale
//! coverage is data, declared in the catalog dataset
//! ([`catalog::Catalog`], [`docs/adr/0001-catalog-boundaries.md`](https://github.com/wrightkit/workshop-rs/blob/main/docs/adr/0001-catalog-boundaries.md)).

pub mod arena;
pub mod catalog;
pub mod census;
pub mod conformance;
pub mod convert;
pub mod detect;
pub mod emitter;
mod error;
pub mod format;
pub mod ids;
pub mod lexer;
pub mod parser;
pub mod roundtrip;
pub mod settings;
pub mod signatures;
pub mod source;
pub mod validate;
pub mod wir;

pub use error::{CatalogError, WorkshopError};
