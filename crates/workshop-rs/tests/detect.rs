//! Language detection and locale-override tests: representative
//! supported-language fixtures are detected with confidence, ambiguous input
//! fails explicitly, and an explicit locale override always wins.

use super::common::{self, catalog, en};
use super::internal;
use workshop_rs::catalog::{Catalog, Kind, Locale};
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
fn zero_marker_input_reports_a_detection_failure() {
    // With no language markers nothing was resolved: the error must not
    // name an arbitrary detection-ranking locale or a `'<none>'` spelling,
    // and stays distinguishable from a rejected-name Unknown diagnostic.
    let error = detect::resolve_locale("0.11.0\n", &catalog(), None)
        .expect_err("no markers means detection failure");
    let workshop_rs::WorkshopError::NotDetected { kind, message, .. } = &error else {
        panic!("zero-marker input is a NotDetected diagnostic: {error:?}");
    };
    assert_eq!(*kind, "language");
    assert!(!message.contains("'<none>'"), "{error}");
    assert!(!message.contains("for locale"), "{error}");
    assert!(error.span().is_none());
    assert!(error.candidates().is_empty());
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

// Pre-index detector retained as a differential oracle for the detection contract.
/// Count distinct catalog aliases of `locale` that appear in the input.
fn reference_alias_matches(input: &str, catalog: &Catalog, locale: &Locale) -> usize {
    let mut matches = 0usize;
    for kind in [
        crate::catalog::Kind::Structural,
        crate::catalog::Kind::Action,
        crate::catalog::Kind::Value,
        crate::catalog::Kind::Event,
        crate::catalog::Kind::Operator,
    ] {
        for entry in catalog.entries_of(kind) {
            if let Some(spelling) = entry.spelling(locale) {
                if locale != catalog.primary_locale()
                    && entry.spelling(catalog.primary_locale()) == Some(spelling)
                {
                    continue;
                }
                if contains_word(input, spelling) {
                    matches += 1;
                }
            }
        }
    }
    // Enum member spellings (e.g. "Grapple Beam", "Ignore Condition").
    for domain in catalog.enum_domains() {
        for member in &domain.members {
            if let Some(spelling) = member.spelling(locale) {
                if locale != catalog.primary_locale()
                    && member.spelling(catalog.primary_locale()) == Some(spelling)
                {
                    continue;
                }
                if contains_word(input, spelling) {
                    matches += 1;
                }
            }
        }
    }
    matches
}

/// Whether `needle` appears in `haystack` bounded by non-word characters.
fn contains_word(haystack: &str, needle: &str) -> bool {
    if needle.is_empty() {
        return false;
    }
    for (start, _) in haystack.match_indices(needle) {
        let end = start + needle.len();
        let before_ok = start == 0
            || !haystack[..start]
                .chars()
                .next_back()
                .is_some_and(is_word_char);
        let after_ok =
            end >= haystack.len() || !haystack[end..].chars().next().is_some_and(is_word_char);
        if before_ok && after_ok {
            return true;
        }
    }
    false
}

fn is_word_char(ch: char) -> bool {
    unicode_ident::is_xid_continue(ch) || ch == '_' || ch == '-'
}

fn reference_candidates(input: &str, catalog: &Catalog) -> Vec<(Locale, usize)> {
    let mut candidates: Vec<_> = catalog
        .locales()
        .iter()
        .map(|locale| {
            (
                locale.clone(),
                reference_alias_matches(input, catalog, locale),
            )
        })
        .collect();
    candidates.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    candidates
}

fn assert_reference(input: &str, catalog: &Catalog) {
    let expected = reference_candidates(input, catalog);
    let actual = detect::detect(input, catalog);
    assert_eq!(actual.candidates, expected);
    assert_eq!(actual.locale, expected[0].0);
    assert_eq!(actual.matches, expected[0].1);
    assert_eq!(
        actual.confidence,
        actual.matches as f64 / (actual.matches as f64 + 1.0)
    );
}

#[test]
fn indexed_detection_matches_reference_corpus_and_locale_spellings() {
    let catalog = catalog();
    for fixture in [
        "basic-rule",
        "control-flow",
        "declarations-rules",
        "expressions-values",
        "preprocessing",
        "overpy-cake",
        "receiver-calls",
    ] {
        assert_reference(&common::corpus_text(fixture), &catalog);
    }
    for case in common::cases() {
        assert_reference(&common::source(case).0, &catalog);
    }
    for locale in catalog.locales() {
        let mut input = String::new();
        for kind in [
            Kind::Structural,
            Kind::Action,
            Kind::Value,
            Kind::Event,
            Kind::Operator,
        ] {
            for entry in catalog.entries_of(kind) {
                if let Some(spelling) = entry.spelling(locale) {
                    input.push_str(&format!("({spelling}) x{spelling} {spelling}- "));
                }
            }
        }
        for domain in catalog.enum_domains() {
            for member in &domain.members {
                if let Some(spelling) = member.spelling(locale) {
                    input.push_str(&format!("({spelling}) "));
                }
            }
        }
        assert_reference(&input, &catalog);
    }
}

#[test]
fn indexed_detection_preserves_boundaries_multiplicity_and_resolution_errors() {
    let catalog = Catalog::load(r#"{
        "schemaVersion": 1,
        "locales": ["en-US", "xx-YY"],
        "target": {"game":"test", "format":"test", "surface":"test"},
        "provenance": {"generator":"test", "generatorVersion":"0", "source":"synthetic", "license":"MIT", "reviewed":true},
        "structural": [
            {"id":"rule", "aliases":{"en-US":["Alpha", "Secondary"], "xx-YY":"Uno"}},
            {"id":"event", "aliases":{"en-US":"Beta", "xx-YY":"Dos"}},
            {"id":"actions", "aliases":{"en-US":"Shared", "xx-YY":"Shared"}},
            {"id":"if", "aliases":{"en-US":"Alpha Beta"}},
            {"id":"else", "aliases":{"en-US":"!a!"}}
        ],
        "enums": [{"domain":"Synthetic", "members":[
            {"id":"DUPLICATE", "aliases":{"en-US":"Alpha", "xx-YY":"Uno"}}
        ]}]
    }"#).expect("synthetic catalog");
    for input in [
        "",
        "Alpha Alpha",
        "Alpha Beta",
        "Shared",
        "Secondary",
        "Uno Dos",
        "Alpha Uno",
        "x!a!a!",
        "x!a!a! !a!",
        "xAlpha Alpha_ Alpha-",
        "éAlpha Alpha中 Alpha\u{301}",
        "(Alpha)😀Beta!",
    ] {
        assert_reference(input, &catalog);
    }
    assert_eq!(detect::detect("Alpha Alpha", &catalog).matches, 2);
    assert_eq!(detect::detect("Alpha Beta", &catalog).matches, 4);
    assert_eq!(detect::detect("x!a!a!", &catalog).matches, 0);
    assert_eq!(detect::detect("Secondary", &catalog).matches, 0);
    assert_eq!(
        detect::detect("Shared", &catalog).candidates,
        vec![(en(), 1), (Locale::new("xx-YY"), 0)]
    );
    let error = detect::resolve_locale("x!a!a!", &catalog, None).expect_err("no markers");
    assert!(
        matches!(error, workshop_rs::WorkshopError::NotDetected { .. }),
        "{error:?}"
    );
    let error = detect::resolve_locale("Shared", &catalog, None).expect_err("insufficient");
    assert!(error.to_string().contains("insufficient evidence"));
    let error = detect::resolve_locale("Alpha Uno", &catalog, None).expect_err("tie");
    assert!(error.to_string().contains("multiple locales tie"));
    assert_eq!(
        detect::resolve_locale("Alpha Uno", &catalog, Some(&en())).unwrap(),
        en()
    );
    assert!(detect::resolve_locale("Alpha Beta", &catalog, Some(&Locale::new("zz-ZZ"))).is_err());
    let other = super::common::catalog();
    assert_reference("Alpha Beta", &other);
    assert_reference("Alpha Beta", &catalog.clone());
}

#[test]
#[ignore = "manual locale detection timing comparison"]
fn locale_detection_benchmark() {
    use std::hint::black_box;
    use std::time::Instant;
    let start = Instant::now();
    let catalog = catalog();
    println!("catalog load including index: {:?}", start.elapsed());
    let fixture = common::corpus_text("overpy-cake");
    for (label, input, iterations) in [
        ("small", fixture.clone(), 100),
        ("large", fixture.repeat(100), 10),
    ] {
        assert_reference(&input, &catalog);
        let start = Instant::now();
        for _ in 0..iterations {
            black_box(reference_candidates(black_box(&input), black_box(&catalog)));
        }
        let reference = start.elapsed() / iterations;
        let start = Instant::now();
        for _ in 0..iterations {
            black_box(detect::detect(black_box(&input), black_box(&catalog)));
        }
        let indexed = start.elapsed() / iterations;
        println!(
            "{label}: {} bytes, {iterations} iterations, reference={reference:?}/op indexed={indexed:?}/op speedup={:.1}x",
            input.len(),
            reference.as_secs_f64() / indexed.as_secs_f64()
        );
    }
}
