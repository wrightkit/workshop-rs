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
//! * [`actions`], [`events`], [`rules`], [`values`], [`settings`], and
//!   [`gameplay`] — the discoverable Workshop domains;
//! * [`wir`] — the Workshop IR model (locale-independent semantic
//!   representation) with its arena/source/settings support;
//! * [`parser`] and [`emitter`] — compatibility paths for the shared frontend
//!   and output boundaries;
//! * [`census`], [`conformance`], [`live_capture`], and [`real_projects`] —
//!   compatibility paths for the separate evidence support tree.
//!
//! The catalog is locale-independent at the identity layer: analyzer and WIR
//! APIs never need locale-specific strings to identify a builtin. Locale
//! coverage is data, declared in the catalog dataset
//! ([`catalog::Catalog`], [`docs/adr/0001-catalog-boundaries.md`](https://github.com/wrightkit/workshop-rs/blob/main/docs/adr/0001-catalog-boundaries.md)).

pub mod actions;
mod analysis;
pub mod arena;
pub mod catalog;
pub mod census;
pub mod conformance;
pub mod convert;
mod core;
pub mod detect;
pub mod element_count;
pub mod emitter;
mod error;
pub mod events;
mod evidence;
pub mod format;
mod frontend;
pub mod gameplay;
pub mod gameplay_data;
pub mod gameplay_query;
pub mod ids;
pub mod lexer;
pub mod live_capture;
mod output;
pub mod parser;
pub mod real_projects;
pub mod roundtrip;
pub mod rules;
pub mod semantic;
pub mod settings;
pub mod signatures;
pub mod source;
pub mod validate;
pub mod values;
pub mod wir;

pub use error::{CatalogError, WorkshopError};
