//! The Workshop event domain.
//!
//! Event identities and filters are canonical WIR data. Parsing, validation,
//! and emission use the shared frontend, analysis, and output boundaries
//! documented in `docs/architecture/source-layout.md`.

pub use crate::wir::{Event, EventTarget, EventTeam, PlayerEventKind};
