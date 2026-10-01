//! Shared helpers for the crate-internal test modules: they exercise the
//! `pub(crate)` WIR parse/emit surface, so this file is compiled only as
//! part of the library unit-test build (see `src/tests.rs`), never by the
//! public-API `integration` target.
#![allow(dead_code)]

use workshop_rs::catalog::Locale;
use workshop_rs::{WorkshopError, emitter, parser, roundtrip, wir};

use super::common;

/// Parse Workshop source to the internal WIR program in en-US.
pub(crate) fn parse(source: &str) -> wir::Program {
    parse_in(source, &common::en())
}

/// Parse Workshop source to the internal WIR program in `locale`.
pub(crate) fn parse_in(source: &str, locale: &Locale) -> wir::Program {
    try_parse_in(source, locale).expect("test source parses")
}

/// Fallible variant of [`parse`]: returns the raw parser result.
pub(crate) fn try_parse(source: &str) -> Result<wir::Program, WorkshopError> {
    try_parse_in(source, &common::en())
}

/// Fallible variant of [`parse_in`]: returns the raw parser result.
pub(crate) fn try_parse_in(source: &str, locale: &Locale) -> Result<wir::Program, WorkshopError> {
    parser::parse_wir_with_context(source, &common::catalog(), locale, &common::catalog())
}

/// Emit a WIR program as en-US Workshop text.
pub(crate) fn emit(program: &wir::Program) -> String {
    emit_in(program, &common::en())
}

/// Emit a WIR program as Workshop text in `locale`.
pub(crate) fn emit_in(program: &wir::Program, locale: &Locale) -> String {
    try_emit_in(program, locale).expect("test program emits")
}

/// Fallible variant of [`emit`].
pub(crate) fn try_emit(program: &wir::Program) -> Result<String, WorkshopError> {
    try_emit_in(program, &common::en())
}

/// Fallible variant of [`emit_in`]: returns the raw emitter result.
pub(crate) fn try_emit_in(
    program: &wir::Program,
    locale: &Locale,
) -> Result<String, WorkshopError> {
    emitter::emit_wir(program, &common::catalog(), locale)
}

/// Emit -> reparse -> semantic-equivalence gate shared by emitter tests:
/// the emitted text must reparse to a semantically equivalent program and
/// re-emit to byte-identical text.
pub(crate) fn assert_emit_round_trip(program: &wir::Program) -> String {
    let emitted = emit(program);
    let reparsed = parse(&emitted);
    assert!(
        roundtrip::equivalent_wir(program, &reparsed),
        "emitted text must reparse to an equivalent WIR program"
    );
    assert_eq!(emitted, emit(&reparsed), "emission must be a fixed point");
    emitted
}

/// Reparse `emitted` and assert structural equivalence with `program`.
pub(crate) fn assert_reparse_equivalent(program: &wir::Program, emitted: &str) {
    assert_reparse_equivalent_in(program, emitted, &common::en())
}

/// Locale-parameterized [`assert_reparse_equivalent`].
pub(crate) fn assert_reparse_equivalent_in(program: &wir::Program, emitted: &str, locale: &Locale) {
    let reparsed = parse_in(emitted, locale);
    assert!(
        roundtrip::equivalent_wir(program, &reparsed),
        "emitted text must reparse to an equivalent WIR program"
    );
}
