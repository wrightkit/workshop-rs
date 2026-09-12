//! The canonical multi-locale Overwatch Workshop semantic core.
//!
//! `workshop-rs` owns the Workshop-language foundation of the WrightKit
//! ecosystem: canonical Workshop semantics, the catalog of Workshop-defined
//! content, the raw Workshop frontend, validation, deterministic emission,
//! and the canonical public `Program` model. It is standalone and MIT-licensed; it has no dependency on
//! Wright tooling crates or on any source-language provider implementation.
//!
//! * [`catalog`] — the canonical Workshop catalog: stable, locale-independent
//!   semantic identities, kinds, parameters, and locale tables binding
//!   identities to client spellings; catalog version/digest identity and
//!   per-locale coverage;
//! * [`actions`], [`events`], [`rules`], [`values`], [`settings`], and
//!   [`gameplay`] — the discoverable Workshop domains;
//! * [`program`] — the canonical public Workshop program model;
//! * [`parser`], [`validate`], [`convert`], [`roundtrip`], and [`emitter`] —
//!   the public Workshop operations over that model;
//!
//! The catalog is locale-independent at the identity layer: analyzer and
//! APIs never need locale-specific strings to identify a builtin. Locale
//! coverage is data, declared in the catalog dataset
//! ([`catalog::Catalog`], [`docs/adr/0001-catalog-boundaries.md`](https://github.com/wrightkit/workshop-rs/blob/main/docs/adr/0001-catalog-boundaries.md)).

pub mod actions;
mod analysis;
#[doc(hidden)]
pub mod arena;
pub mod catalog;
pub mod convert;
mod core;
pub mod detect;
#[doc(hidden)]
pub mod element_count;
pub mod emitter;
mod error;
pub mod events;
#[doc(hidden)]
pub mod format;
mod frontend;
pub mod gameplay;
#[doc(hidden)]
pub mod gameplay_data;
#[doc(hidden)]
pub mod gameplay_query;
#[doc(hidden)]
pub mod ids;
#[doc(hidden)]
pub mod lexer;
mod output;
pub mod parser;
pub mod program;
pub mod roundtrip;
pub mod rules;
#[doc(hidden)]
pub mod semantic;
pub mod settings;
pub mod signatures;
pub mod source;
pub mod validate;
pub mod values;
#[doc(hidden)]
pub mod wir;

pub use error::{CatalogError, WorkshopError};
pub use program::{
    Action, Condition, Event, EventTarget, EventTeam, ModifyOp, PlayerEventKind, Program,
    ProvenanceError, Rule, Subroutine, Value, Variable,
};
