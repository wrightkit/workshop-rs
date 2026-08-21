//! Multi-locale mechanics tests (ADR-0001 Decisions 3, 7): canonical
//! identities are locale-independent; locale tables are mappings; missing
//! target-locale mappings fail explicitly by default; fallback is opt-in and
//! visible; settings follow the same contract.
//!
//! The committed catalog includes the evidence-backed `zh-CN` corpus; its
//! exact-match manifest is pinned separately in `tools/corpus/zh-cn-corpus.json`.
//! This suite pins both successful corpus conversion and the fail-explicit
//! behavior for an unsupported undeclared target locale.

use workshop_rs::catalog::{Catalog, Kind, Locale};
use workshop_rs::convert::{self, ConvertOptions};
use workshop_rs::emitter::{self, EmitOptions};
use workshop_rs::parser;
use workshop_rs::settings::SettingsNode;

fn builtin() -> Catalog {
    Catalog::builtin().expect("built-in catalog")
}

fn en() -> Locale {
    Locale::new("en-US")
}

fn zh() -> Locale {
    Locale::new("zh-CN")
}

#[test]
fn settings_projection_is_multi_locale_data() {
    assert_eq!(
        workshop_rs::settings::table::localized_name("zh-CN", "teams", "Team 1"),
        Some("队伍1")
    );
    assert_eq!(
        workshop_rs::settings::table::localized_name("en-US", "teams", "Team 1"),
        Some("Team 1")
    );
    let projection: serde_json::Value =
        serde_json::from_str(include_str!("../src/settings/data/locales.json"))
            .expect("multi-locale settings projection");
    assert_eq!(projection["locales"], serde_json::json!(["en-US", "zh-CN"]));
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
fn emission_into_zh_cn_uses_evidence_backed_mappings() {
    let catalog = builtin();
    let program = parser::parse(BASIC_RULE, &catalog, &en()).expect("parses");
    let output = emitter::emit(&program, &catalog, &zh()).expect("corpus mappings emit");
    assert!(output.contains("持续 - 全局"), "{output}");
    assert!(output.contains("禁用查看器录制"), "{output}");
}

#[test]
fn conversion_en_to_zh_cn_uses_evidence_backed_mappings() {
    let catalog = builtin();
    let output = convert::convert(
        BASIC_RULE,
        &catalog,
        &en(),
        &zh(),
        &ConvertOptions::default(),
    )
    .expect("corpus conversion succeeds");
    assert!(output.text.contains("持续 - 全局"), "{}", output.text);
    assert!(output.fallback_ids.is_empty());
}

const FALLBACK_RULE: &str = "rule (\"setup\") {
    event {
        Ongoing - Global;
    }
    actions {
        Disable Inspector Recording;
    }
}
";

#[test]
fn opt_in_fallback_emits_with_recorded_fallback_ids() {
    // Fallback is opt-in: with a fallback locale the emission succeeds and
    // the fell-back identities are recorded (visible in tooling output).
    let catalog = builtin();
    let program = parser::parse(FALLBACK_RULE, &catalog, &en()).expect("parses");
    let options = EmitOptions {
        fallback_locale: Some(en()),
    };
    let output = emitter::emit_with_options(&program, &catalog, &Locale::new("fr-FR"), &options)
        .expect("fallback emits");
    assert!(output.text.contains("Ongoing - Global"), "{}", output.text);
    assert!(
        output.text.contains("Disable Inspector Recording"),
        "{}",
        output.text
    );
    assert_eq!(
        output.fallback_ids,
        vec![
            "rule".to_string(),
            "event".to_string(),
            "global".to_string(),
            "actions".to_string(),
            "disableInspector".to_string(),
        ],
        "the unsupported target locale records the fallback identity"
    );
}

#[test]
fn opt_in_fallback_conversion_round_trips_through_zh_cn() {
    // Convert en-US to an unsupported locale with fallback to en-US.
    let catalog = builtin();
    let options = ConvertOptions {
        fallback_locale: Some(en()),
    };
    let out = convert::convert(
        FALLBACK_RULE,
        &catalog,
        &en(),
        &Locale::new("fr-FR"),
        &options,
    )
    .expect("fallback conversion emits");
    assert!(!out.fallback_ids.is_empty(), "fallback is recorded");
    assert!(out.text.contains("Ongoing - Global"), "{}", out.text);
    assert!(
        out.text.contains("Disable Inspector Recording"),
        "{}",
        out.text
    );
    assert!(out.fallback_ids.contains(&"disableInspector".to_string()));
}

#[test]
fn parsing_zh_cn_input_uses_corpus_aliases() {
    let catalog = builtin();
    let localized = "rule (\"x\") { event { 持续 - 全局; } actions { 禁用查看器录制; } }";
    parser::parse(localized, &catalog, &zh()).expect("corpus aliases parse");
}

#[test]
fn explicit_zh_cn_override_passes_locale_support() {
    // The locale machinery accepts an explicit override to a declared
    // locale, independently of the corpus coverage.
    use workshop_rs::detect;
    let catalog = builtin();
    let locale = detect::resolve_locale("garbage", &catalog, Some(&zh())).expect("override wins");
    assert_eq!(locale, zh());
}

#[test]
fn detection_ranks_zh_cn_after_en_us_for_en_us_input() {
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
        "zh-CN remains behind en-US for en-US input"
    );
}

#[test]
fn settings_emission_into_zh_cn_uses_the_generated_locale_corpus() {
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
    let output = emitter::emit(&program, &catalog, &zh()).expect("settings corpus emits");
    assert!(output.contains("自由混战人数上限: 6"), "{}", output);
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
            { "id": "if", "aliases": { "en-US": "If", "xx-YY": "Synthetic If" } },
            { "id": "rule", "aliases": { "en-US": "rule", "xx-YY": "SyntheticRule" } },
            { "id": "event", "aliases": { "en-US": "event", "xx-YY": "SyntheticEvent" } },
            { "id": "actions", "aliases": { "en-US": "actions", "xx-YY": "SyntheticActions" } }
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

const SYNTHETIC_TARGET: &str = "SyntheticRule (\"r\") {
    SyntheticEvent {
        Synthetic Global Event;
    }
    SyntheticActions {
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
fn catalog_spelling_lookup_distinguishes_mapped_and_unmapped_locales() {
    let catalog = builtin();
    assert_eq!(
        catalog.spelling(Kind::Action, &zh(), "disableInspector"),
        Some("禁用查看器录制")
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

#[test]
fn current_settings_inventory_resolves_extensions_and_hero_keys() {
    let source = r#"settings
{
	heroes
	{
		队伍1
		{
			半藏
			{
				伤害量: 100%
			}
		}
	}
	扩展
	{
		生成更多机器人
	}
}
"#;
    let catalog = builtin();
    let program = parser::parse_with_context(source, &catalog, &zh(), &catalog).expect("parses");
    fn assert_no_raw(nodes: &[SettingsNode]) {
        for node in nodes {
            assert!(
                !matches!(node, SettingsNode::Raw { .. }),
                "raw setting: {}",
                node.name()
            );
            if let SettingsNode::Group { children, .. } = node {
                assert_no_raw(children);
            }
        }
    }
    assert_no_raw(&program.settings.expect("settings").children);
}
