//! Language detection and locale-override tests: representative
//! supported-language fixtures are detected with confidence, ambiguous input
//! fails explicitly, and an explicit locale override always wins.

use std::path::{Path, PathBuf};

use workshop_rs::catalog::{Catalog, Locale};
use workshop_rs::detect::{self, MIN_MATCHES};
use workshop_rs::parser;

fn corpus_path(fixture_id: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/corpus")
        .join(format!("{fixture_id}.ws"))
}

fn corpus_text(fixture_id: &str) -> String {
    std::fs::read_to_string(corpus_path(fixture_id)).unwrap()
}

fn catalog() -> Catalog {
    Catalog::builtin().unwrap()
}

#[test]
fn supported_language_fixtures_are_detected_confidently() {
    for fixture_id in [
        "basic-rule",
        "control-flow",
        "declarations-rules",
        "expressions-values",
        "preprocessing",
        "overpy-cake",
    ] {
        let text = corpus_text(fixture_id);
        let detection = detect::detect(&text, &catalog());
        assert_eq!(detection.locale, Locale::new("en-US"));
        assert!(
            detection.matches >= MIN_MATCHES,
            "{fixture_id} must have enough evidence: {detection:?}"
        );
        assert!(
            detection.confidence > 0.5,
            "{fixture_id} confidence: {detection:?}"
        );
    }
}

#[test]
fn resolve_locale_auto_detects_supported_input() {
    let text = corpus_text("basic-rule");
    let locale = detect::resolve_locale(&text, &catalog(), None).expect("detected");
    assert_eq!(locale, Locale::new("en-US"));
}

#[test]
fn explicit_locale_override_bypasses_detection() {
    // The explicit locale wins even for input that would not auto-detect
    // confidently (garbage), because override skips detection.
    let garbage = "not workshop at all";
    let error = detect::resolve_locale(garbage, &catalog(), None).expect_err("no detection");
    assert!(error.to_string().contains("language"), "{error}");
    let locale = detect::resolve_locale(garbage, &catalog(), Some(&Locale::new("en-US")))
        .expect("override wins");
    assert_eq!(locale, Locale::new("en-US"));
}

#[test]
fn insufficient_evidence_fails_explicitly() {
    let garbage = "hello world this is not workshop syntax at all";
    let error = detect::resolve_locale(garbage, &catalog(), None).expect_err("ambiguous");
    assert!(
        error.to_string().contains("language") || error.to_string().contains("insufficient"),
        "{error}"
    );
}

#[test]
fn detection_is_deterministic() {
    let text = corpus_text("overpy-cake");
    let first = detect::detect(&text, &catalog());
    let second = detect::detect(&text, &catalog());
    assert_eq!(first, second);
}

#[test]
fn detected_locale_parses_the_input() {
    // The full loop: detect, then parse with the detected locale.
    let text = corpus_text("control-flow");
    let locale = detect::resolve_locale(&text, &catalog(), None).expect("detected");
    let program = parser::parse(&text, &catalog(), &locale).expect("parses with detected locale");
    assert!(!program.rules.is_empty());
}
