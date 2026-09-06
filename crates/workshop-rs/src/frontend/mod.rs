//! Shared raw Workshop text frontend.
//!
//! Parsing is an orchestration boundary across Workshop domains. The semantic
//! identities it produces live in [`crate::wir`] and the domain modules.

pub(crate) mod lexer;
pub(crate) mod parser;
