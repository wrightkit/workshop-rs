#![allow(dead_code)]

use workshop_rs::catalog::Locale;
use workshop_rs::{WorkshopError, emitter, parser, roundtrip, wir};

use super::common;

pub(crate) fn parse(source: &str) -> wir::Program {
    parse_in(source, &common::en())
}

pub(crate) fn parse_in(source: &str, locale: &Locale) -> wir::Program {
    try_parse_in(source, locale).expect("test source parses")
}

pub(crate) fn try_parse(source: &str) -> Result<wir::Program, WorkshopError> {
    try_parse_in(source, &common::en())
}

pub(crate) fn try_parse_in(source: &str, locale: &Locale) -> Result<wir::Program, WorkshopError> {
    parser::parse_wir_with_context(source, &common::catalog(), locale, &common::catalog())
}

pub(crate) fn emit(program: &wir::Program) -> String {
    emit_in(program, &common::en())
}

pub(crate) fn emit_in(program: &wir::Program, locale: &Locale) -> String {
    try_emit_in(program, locale).expect("test program emits")
}

pub(crate) fn try_emit(program: &wir::Program) -> Result<String, WorkshopError> {
    try_emit_in(program, &common::en())
}

pub(crate) fn try_emit_in(
    program: &wir::Program,
    locale: &Locale,
) -> Result<String, WorkshopError> {
    emitter::emit_wir(program, &common::catalog(), locale)
}

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

pub(crate) fn assert_reparse_equivalent(program: &wir::Program, emitted: &str) {
    assert_reparse_equivalent_in(program, emitted, &common::en())
}

pub(crate) fn assert_reparse_equivalent_in(program: &wir::Program, emitted: &str, locale: &Locale) {
    let reparsed = parse_in(emitted, locale);
    assert!(
        roundtrip::equivalent_wir(program, &reparsed),
        "emitted text must reparse to an equivalent WIR program"
    );
}
