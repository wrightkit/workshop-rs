//! The Workshop event domain.
//!
//! Event identities and filters are canonical public data. Parsing, validation,
//! and emission use the shared frontend, analysis, and output boundaries
//! documented in `docs/architecture/source-layout.md`.

pub(crate) mod emitter;
pub(crate) mod parser;
pub(crate) mod validate;

pub use crate::program::{Event, EventTarget, EventTeam, PlayerEventKind};
