//! Language detection and locale-override tests: representative
//! supported-language fixtures are detected with confidence, ambiguous input
//! fails explicitly, and an explicit locale override always wins.

use super::common::{self, catalog, en};
use super::internal;
use workshop_rs::detect::{self, MIN_MATCHES};

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
        let text = common::corpus_text(fixture_id);
        let detection = detect::detect(&text, &catalog());
        assert_eq!(detection.locale, en());
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
    let text = common::corpus_text("basic-rule");
    let locale = detect::resolve_locale(&text, &catalog(), None).expect("detected");
    assert_eq!(locale, en());
}

#[test]
fn explicit_locale_override_bypasses_detection() {
    // The explicit locale wins even for input that would not auto-detect
    // confidently (garbage), because override skips detection.
    let garbage = "not workshop at all";
    let error = detect::resolve_locale(garbage, &catalog(), None).expect_err("no detection");
    assert!(error.to_string().contains("language"), "{error}");
    let locale = detect::resolve_locale(garbage, &catalog(), Some(&en())).expect("override wins");
    assert_eq!(locale, en());
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
    let text = common::corpus_text("overpy-cake");
    let first = detect::detect(&text, &catalog());
    let second = detect::detect(&text, &catalog());
    assert_eq!(first, second);
}

#[test]
fn detected_locale_parses_the_input() {
    // The full loop: detect, then parse with the detected locale.
    let text = common::corpus_text("control-flow");
    let locale = detect::resolve_locale(&text, &catalog(), None).expect("detected");
    let program = internal::parse_in(&text, &locale);
    assert!(!program.rules.is_empty());
}
