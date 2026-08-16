//! Multi-locale mechanics tests (ADR-0001 Decisions 3, 7): canonical
//! identities are locale-independent; locale tables are mappings; missing
//! target-locale mappings fail explicitly by default; fallback is opt-in and
//! visible; settings follow the same contract.
//!
//! The committed catalog declares `zh-CN` with an empty mapping set (0/344)
//! pending a reviewed, MIT-permissible reference source. This suite pins the
//! honest behavior that follows: conversion into zh-CN fails explicitly for
//! every entry, and the full conversion machinery is proven end-to-end with
//! a synthetic declared locale carrying clearly synthetic test spellings.

use workshop_rs::catalog::{Catalog, Kind, Locale};
use workshop_rs::convert::{self, ConvertOptions};
use workshop_rs::emitter::{self, EmitOptions};
use workshop_rs::parser;

fn builtin() -> Catalog {
    Catalog::builtin().expect("built-in catalog")
}

fn en() -> Locale {
    Locale::new("en-US")
}

fn zh() -> Locale {
    Locale::new("zh-CN")
}

const BASIC_RULE: &str = "rule (\"setup\") {
    event {
        Ongoing - Global;
    }
    actions {
        Disable Inspector Recording;
    }
}
";

#[test]
fn emission_into_zh_cn_fails_explicitly_on_missing_mappings() {
    // The zh-CN locale is declared with zero mappings: emitting any builtin
    // into zh-CN is a missing-mapping diagnostic, never a silent passthrough
    // of the en-US spelling. The first missing mapping (the event line,
    // emitted before the actions) surfaces first.
    let catalog = builtin();
    let program = parser::parse(BASIC_RULE, &catalog, &en()).expect("parses");
    let error = emitter::emit(&program, &catalog, &zh()).expect_err("must fail explicitly");
    assert!(
        matches!(
            error,
            workshop_rs::WorkshopError::MissingMapping { kind: "event", .. }
        ),
        "expected a structured missing-mapping diagnostic: {error}"
    );
    assert!(
        error.to_string().contains("zh-cn") && error.to_string().contains("global"),
        "{error}"
    );
}

#[test]
fn conversion_en_to_zh_cn_fails_explicitly_without_fallback() {
    let catalog = builtin();
    let error = convert::convert(
        BASIC_RULE,
        &catalog,
        &en(),
        &zh(),
        &ConvertOptions::default(),
    )
    .expect_err("missing mappings must fail");
    assert!(error.to_string().contains("missing"), "{error}");
}

#[test]
fn opt_in_fallback_emits_with_recorded_fallback_ids() {
    // Fallback is opt-in: with a fallback locale the emission succeeds and
    // the fell-back identities are recorded (visible in tooling output).
    let catalog = builtin();
    let program = parser::parse(BASIC_RULE, &catalog, &en()).expect("parses");
    let options = EmitOptions {
        fallback_locale: Some(en()),
    };
    let output =
        emitter::emit_with_options(&program, &catalog, &zh(), &options).expect("fallback emits");
    assert_eq!(
        output.text.trim_end(),
        BASIC_RULE.trim_end(),
        "fallback text is the en-US spelling"
    );
    assert_eq!(
        output.fallback_ids,
        vec!["global".to_string(), "disableInspector".to_string()],
        "every fell-back canonical id is recorded (event then action)"
    );
}

#[test]
fn opt_in_fallback_conversion_round_trips_through_zh_cn() {
    // convert en -> zh-CN with fallback to en-US: the output is the
    // fallback-locale (en-US) spelling surface, the fallback choice is
    // recorded, and the output parses and emits identically in en-US.
    let catalog = builtin();
    let options = ConvertOptions {
        fallback_locale: Some(en()),
    };
    let out = convert::convert(BASIC_RULE, &catalog, &en(), &zh(), &options)
        .expect("fallback conversion emits");
    assert!(!out.fallback_ids.is_empty(), "fallback is recorded");
    assert_eq!(
        out.text.trim_end(),
        BASIC_RULE.trim_end(),
        "the fallback output is the en-US spelling surface"
    );
    // The output is en-US text: it parses in en-US and re-emits identically.
    let reparsed = parser::parse(&out.text, &catalog, &en()).expect("fallback output parses");
    let reemitted = emitter::emit(&reparsed, &catalog, &en()).expect("re-emits");
    assert_eq!(out.text, reemitted, "fallback output is a fixed point");
}

#[test]
fn parsing_zh_cn_input_fails_explicitly_without_data() {
    // With zero zh-CN aliases, zh-CN Workshop text cannot resolve any
    // builtin: the parse fails with a structured Unknown diagnostic at the
    // first spelling. No guessing, no fallback.
    let catalog = builtin();
    let synthetic_zh = "rule (\"x\") { event { Ongoing - Global; } actions { Synthetic Action; } }";
    let error = parser::parse(synthetic_zh, &catalog, &zh()).expect_err("no zh-CN data yet");
    assert!(
        matches!(error, workshop_rs::WorkshopError::Unknown { .. }),
        "expected an Unknown diagnostic: {error}"
    );
}

#[test]
fn explicit_zh_cn_override_passes_locale_support() {
    // The locale machinery accepts an explicit override to a declared
    // locale; the parse then fails on data, not on locale support.
    use workshop_rs::detect;
    let catalog = builtin();
    let locale = detect::resolve_locale("garbage", &catalog, Some(&zh())).expect("override wins");
    assert_eq!(locale, zh());
}

#[test]
fn detection_is_unaffected_by_the_empty_zh_cn_locale() {
    use workshop_rs::detect;
    let catalog = builtin();
    let detection = detect::detect(BASIC_RULE, &catalog);
    assert_eq!(detection.locale, en());
    assert_eq!(
        detection
            .candidates
            .last()
            .map(|(locale, _)| locale.clone()),
        Some(zh()),
        "zh-CN ranks last with zero matches"
    );
}

#[test]
fn settings_emission_into_zh_cn_fails_without_fallback_and_works_with_it() {
    use workshop_rs::settings::{Settings, SettingsNode};
    let catalog = builtin();
    let program = workshop_rs::wir::Program {
        settings: Some(Settings {
            span: None,
            children: vec![SettingsNode::Group {
                name: "lobby".to_string(),
                children: vec![SettingsNode::Number {
                    name: "ffaSlots".to_string(),
                    value: 6.0,
                    span: None,
                }],
                span: None,
            }],
        }),
        ..workshop_rs::wir::Program::default()
    };
    let error = emitter::emit(&program, &catalog, &zh()).expect_err("settings must fail");
    assert!(
        matches!(
            error,
            workshop_rs::WorkshopError::MissingMapping {
                kind: "setting",
                ..
            }
        ),
        "settings emission into zh-CN fails explicitly: {error}"
    );
    let options = EmitOptions {
        fallback_locale: Some(en()),
    };
    let output = emitter::emit_with_options(&program, &catalog, &zh(), &options)
        .expect("settings fallback emits");
    assert!(
        output.text.contains("Max FFA Players: 6"),
        "{}",
        output.text
    );
    assert!(output.fallback_ids.contains(&"settings".to_string()));
}

/// A test-only catalog with a second declared locale carrying clearly
/// synthetic spellings, to prove the full conversion machinery end-to-end
/// without fabricating real zh-CN data.
fn synthetic_catalog() -> Catalog {
    let json = r#"{
        "schemaVersion": 1,
        "version": "test",
        "locales": ["en-US", "xx-YY"],
        "target": { "game": "test", "format": "test", "surface": "test" },
        "provenance": { "generator": "test", "generatorVersion": "0", "source": "synthetic test data", "license": "MIT", "reviewed": true },
        "structural": [
            { "id": "if", "aliases": { "en-US": "If", "xx-YY": "Synthetic If" } }
        ],
        "actions": [
            { "id": "disableInspector", "aliases": { "en-US": "Disable Inspector Recording", "xx-YY": "Synthetic Disable" } },
            { "id": "wait", "aliases": { "en-US": "Wait", "xx-YY": "Synthetic Wait" }, "paramDomains": [null, "Wait"], "params": ["Duration", "WaitBehavior"] },
            { "id": "abort", "aliases": { "en-US": "Abort" } }
        ],
        "events": [
            { "id": "global", "aliases": { "en-US": "Ongoing - Global", "xx-YY": "Synthetic Global Event" } }
        ],
        "enums": [
            { "domain": "Wait", "members": [
                { "id": "IGNORE_CONDITION", "aliases": { "en-US": "Ignore Condition", "xx-YY": "Synthetic Ignore" } }
            ] }
        ]
    }"#;
    Catalog::load(json).expect("synthetic catalog validates")
}

const SYNTHETIC_SOURCE: &str = "rule (\"r\") {
    event {
        Ongoing - Global;
    }
    actions {
        Wait(1, Ignore Condition);
        Disable Inspector Recording;
    }
}
";

const SYNTHETIC_TARGET: &str = "rule (\"r\") {
    event {
        Synthetic Global Event;
    }
    actions {
        Synthetic Wait(1, Synthetic Ignore);
        Synthetic Disable;
    }
}
";

#[test]
fn conversion_round_trips_through_a_declared_non_primary_locale() {
    let catalog = synthetic_catalog();
    let out = convert::convert(
        SYNTHETIC_SOURCE,
        &catalog,
        &en(),
        &Locale::new("xx-YY"),
        &ConvertOptions::default(),
    )
    .expect("converts");
    assert_eq!(
        out.text.trim_end(),
        SYNTHETIC_TARGET.trim_end(),
        "canonical semantics emit in the target locale:\n{}",
        out.text
    );
    assert!(out.fallback_ids.is_empty());

    let back = convert::convert(
        &out.text,
        &catalog,
        &Locale::new("xx-YY"),
        &en(),
        &ConvertOptions::default(),
    )
    .expect("converts back");
    assert_eq!(
        back.text.trim_end(),
        SYNTHETIC_SOURCE.trim_end(),
        "xx-YY -> en-US preserves the text"
    );
}

#[test]
fn partial_coverage_fails_explicitly_only_for_unmapped_identities() {
    let catalog = synthetic_catalog();
    // wait is mapped in xx-YY, but createHudText is not declared there: a
    // program using only mapped ids converts; one using an unmapped id fails.
    let mapped =
        "rule (\"r\") { event { Ongoing - Global; } actions { Wait(1, Ignore Condition); } }";
    let out = convert::convert(
        mapped,
        &catalog,
        &en(),
        &Locale::new("xx-YY"),
        &ConvertOptions::default(),
    )
    .expect("mapped ids convert");
    assert!(out.text.contains("Synthetic Wait"));

    let unmapped = "rule (\"r\") { event { Ongoing - Global; } actions { Abort; } }";
    let error = convert::convert(
        unmapped,
        &catalog,
        &en(),
        &Locale::new("xx-YY"),
        &ConvertOptions::default(),
    )
    .expect_err("unmapped ids must fail explicitly");
    assert!(error.to_string().contains("missing"), "{error}");
    assert!(error.to_string().contains("abort"), "{error}");
}

#[test]
fn canonical_ids_are_locale_independent_in_wir() {
    // Parsing the same program in en-US and xx-YY yields the same canonical
    // WIR (ids, not spellings).
    let catalog = synthetic_catalog();
    let en_program =
        parser::parse_with_context(SYNTHETIC_SOURCE, &catalog, &en(), &catalog).expect("parses");
    let xx_program =
        parser::parse_with_context(SYNTHETIC_TARGET, &catalog, &Locale::new("xx-YY"), &catalog)
            .expect("parses");
    assert!(
        workshop_rs::roundtrip::equivalent(&en_program, &xx_program),
        "the WIR of both locales is equivalent"
    );
}

#[test]
fn catalog_spelling_lookup_answers_none_for_unmapped_locales() {
    let catalog = builtin();
    assert_eq!(
        catalog.spelling(Kind::Action, &zh(), "disableInspector"),
        None,
        "no zh-CN mapping exists"
    );
    assert_eq!(
        catalog.spelling(Kind::Action, &en(), "disableInspector"),
        Some("Disable Inspector Recording")
    );
    assert!(
        catalog
            .resolve(Kind::Action, &zh(), "Disable Inspector Recording")
            .is_none(),
        "en-US spellings never resolve in zh-CN"
    );
}
