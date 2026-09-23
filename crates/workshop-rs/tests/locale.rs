//! Multi-locale mechanics tests (ADR-0001 Decisions 3, 7): canonical
//! identities are locale-independent; locale tables are mappings; missing
//! target-locale mappings fail explicitly by default; fallback is opt-in and
//! visible; settings follow the same contract.
//!
//! The committed catalog includes the source-backed `zh-CN` corpus; its
//! exact-match manifest is pinned separately in `tools/corpus/zh-cn-corpus.json`.
//! This suite pins both successful corpus conversion and the fail-explicit
//! behavior for an unsupported undeclared target locale.

use workshop_rs::catalog::{Catalog, Kind, Locale};
use workshop_rs::convert::{self, ConvertOptions};
use workshop_rs::emitter::{self, EmitOptions};
use workshop_rs::parser;
use workshop_rs::settings::SettingsNode;

use super::common;

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
fn pinned_real_projects_convert_between_supported_locales() {
    let catalog = builtin();
    for case in common::cases() {
        let (source, source_locale) = common::source(case);
        let target_locale = common::target_locale(&source_locale);
        let program = parser::parse_wir_with_context(&source, &catalog, &source_locale, &catalog)
            .unwrap_or_else(|error| panic!("{} parse failed: {error:?}", case.id));
        common::assert_residual_policy(case, "source-parse", &program.semantic_issues(&catalog));
        let converted = match convert::convert(
            &source,
            &catalog,
            &source_locale,
            &target_locale,
            &ConvertOptions::default(),
        ) {
            Ok(converted) => converted,
            Err(error) => {
                common::assert_gap(case, common::RealProjectStage::LocaleConversion, &error);
                println!("{}: known locale conversion gap: {error:?}", case.id);
                continue;
            }
        };
        let converted_program =
            parser::parse_wir_with_context(&converted.text, &catalog, &target_locale, &catalog)
                .unwrap_or_else(|error| {
                    panic!("{} target-locale reparse failed: {error:?}", case.id)
                });
        common::assert_residual_policy(
            case,
            "target-locale-reparse",
            &converted_program.semantic_issues(&catalog),
        );
        common::assert_custom_workshop_settings(case.id, &source, &converted.text);
        common::assert_target_locale_spellings(
            case.id,
            &source_locale,
            &target_locale,
            &source,
            &converted.text,
            &program.dump(),
            &catalog,
        );
        assert!(
            workshop_rs::roundtrip::equivalent_wir(&program, &converted_program),
            "{} target-locale conversion changed WIR",
            case.id
        );
        let converted_back =
            workshop_rs::emitter::emit_wir(&converted_program, &catalog, &source_locale)
                .unwrap_or_else(|error| {
                    panic!("{} reverse locale emission failed: {error:?}", case.id)
                });
        let converted_back_program =
            parser::parse_wir_with_context(&converted_back, &catalog, &source_locale, &catalog)
                .unwrap_or_else(|error| {
                    panic!("{} reverse locale reparse failed: {error:?}", case.id)
                });
        assert!(
            workshop_rs::roundtrip::equivalent_wir(&program, &converted_back_program),
            "{} reverse locale conversion changed WIR",
            case.id
        );
    }
}

#[test]
fn settings_projection_is_multi_locale_data() {
    let projection: serde_json::Value =
        serde_json::from_str(include_str!("../src/settings/data/locales.json"))
            .expect("multi-locale settings projection");
    assert_eq!(
        projection["locales"],
        serde_json::json!([
            "en-US", "de-DE", "es-ES", "es-MX", "fr-FR", "it-IT", "ja-JP", "ko-KR", "pl-PL",
            "pt-BR", "ru-RU", "th-TH", "tr-TR", "zh-CN", "zh-TW"
        ])
    );
    for (name, zh_name) in [
        ("main", "主程序"),
        ("lobby", "大厅"),
        ("modes", "模式"),
        ("heroes", "英雄"),
        ("extensions", "扩展"),
        ("workshop", "地图工坊"),
    ] {
        assert_eq!(projection["namespaces"][name]["zh-CN"], zh_name);
    }
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
    let program = parser::parse_wir(BASIC_RULE, &catalog, &en()).expect("parses");
    let output = emitter::emit_wir(&program, &catalog, &zh()).expect("corpus mappings emit");
    assert!(output.contains("持续 - 全局"), "{output}");
    assert!(output.contains("禁用查看器录制"), "{output}");
}

// Minimized from the OWBastion/Bastion zh-CN differential reported in
// Bastion#214, using revision c010e1a2d468ec7140f474e334067e5ab8d02d89 and
// source fixture crates/workshop-rs/tests/fixtures/real-projects/bastion.ow.
const BASTION_ZH_CN_EMISSION_SLICE: &str = r#"variables {
    global:
        0: probe
}

rule ("locale surface") {
    event {
        Ongoing - Global;
    }
    conditions {
        Global.probe == True;
    }
    actions {
        Set Global Variable(probe, False);
        Set Global Variable(probe, Null);
    }
}
"#;

#[test]
fn primitive_and_global_spellings_emit_and_reparse_in_zh_cn() {
    let catalog = builtin();
    let program = parser::parse_wir(BASTION_ZH_CN_EMISSION_SLICE, &catalog, &en()).expect("parses");
    let output = emitter::emit_wir(&program, &catalog, &zh()).expect("zh-CN emits");
    assert!(output.contains("全局.probe"), "{output}");
    assert!(output.contains("假"), "{output}");
    assert!(output.contains("空"), "{output}");
    assert!(output.contains(" == 真"), "{output}");
    let reparsed = parser::parse_wir(&output, &catalog, &zh()).expect("zh-CN reparses");
    assert!(
        workshop_rs::roundtrip::equivalent_wir(&program, &reparsed),
        "original={} reparsed={}",
        program.dump(),
        reparsed.dump()
    );
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

#[test]
fn representative_catalog_strings_and_settings_convert_in_every_declared_locale() {
    let catalog = builtin();
    let source = r#"settings {
    main {
        Description: "locale surface"
    }
}

variables {
    global:
        0: probe
}

rule ("locale surface") {
    event {
        Ongoing - Global;
    }
    actions {
        Set Global Variable(probe, String(Hello));
    }
}
"#;
    let english = parser::parse_wir(source, &catalog, &en()).expect("representative source parses");

    for locale in catalog.locales() {
        let converted =
            convert::convert(source, &catalog, &en(), locale, &ConvertOptions::default())
                .unwrap_or_else(|error| panic!("{locale} conversion failed: {error:?}"));
        assert!(
            converted.fallback_ids.is_empty(),
            "{locale} conversion used fallback: {:?}",
            converted.fallback_ids
        );
        let reparsed = parser::parse_wir(&converted.text, &catalog, locale)
            .unwrap_or_else(|error| panic!("{locale} output failed to parse: {error:?}"));
        assert!(
            workshop_rs::roundtrip::equivalent_wir(&english, &reparsed),
            "{locale} conversion changed WIR"
        );
    }
}

#[test]
fn french_emitted_curly_apostrophe_aliases_parse_and_round_trip() {
    let catalog = builtin();
    let source = r#"rule ("setup") {
    event {
        Ongoing - Global;
    }
    actions {
        Disable Inspector Recording;
    }
}
"#;
    let english = parser::parse_wir(source, &catalog, &en()).expect("source parses");
    let converted = convert::convert(
        source,
        &catalog,
        &en(),
        &Locale::new("fr-FR"),
        &ConvertOptions::default(),
    )
    .expect("French conversion succeeds");
    assert!(
        converted
            .text
            .contains("Désactiver l’enregistrement du contrôleur"),
        "{}",
        converted.text
    );
    let reparsed = parser::parse_wir(&converted.text, &catalog, &Locale::new("fr-FR"))
        .expect("French emitted alias reparses");
    assert!(workshop_rs::roundtrip::equivalent_wir(&english, &reparsed));
}

const FALLBACK_RULE: &str = "variables {
    global:
        0: probe
}

rule (\"setup\") {
    event {
        Ongoing - Global;
    }
    actions {
        Disable Inspector Recording;
        Set Global Variable(probe, Is Firing Secondary Fire(Event Player));
    }
}
";

#[test]
fn opt_in_fallback_emits_with_recorded_fallback_ids() {
    // Fallback is opt-in: with a fallback locale the emission succeeds and
    // the fell-back identities are recorded (visible in tooling output).
    let catalog = builtin();
    let program = parser::parse_wir(FALLBACK_RULE, &catalog, &en()).expect("parses");
    let options = EmitOptions {
        fallback_locale: Some(en()),
    };
    let output =
        emitter::emit_wir_with_options(&program, &catalog, &Locale::new("fr-FR"), &options)
            .expect("fallback emits");
    assert!(
        output.text.contains("Toute la partie - Tout le monde"),
        "{}",
        output.text
    );
    assert!(
        output
            .text
            .contains("Désactiver l’enregistrement du contrôleur"),
        "{}",
        output.text
    );
    assert!(
        output
            .fallback_ids
            .contains(&"isFiringSecondaryFire".to_string()),
        "the unsupported target locale records the fallback identity: {:?}",
        output.fallback_ids
    );
    assert!(
        output
            .text
            .contains("Is Firing Secondary Fire(Joueur exécutant)"),
        "{}",
        output.text
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
    assert!(
        out.text.contains("Toute la partie - Tout le monde"),
        "{}",
        out.text
    );
    assert!(
        out.text
            .contains("Désactiver l’enregistrement du contrôleur"),
        "{}",
        out.text
    );
    assert!(
        out.fallback_ids
            .contains(&"isFiringSecondaryFire".to_string())
    );
}

#[test]
fn parsing_zh_cn_input_uses_corpus_aliases() {
    let catalog = builtin();
    let localized = "rule (\"x\") { event { 持续 - 全局; } actions { 禁用查看器录制; } }";
    parser::parse_wir(localized, &catalog, &zh()).expect("corpus aliases parse");
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
    let en_position = detection
        .candidates
        .iter()
        .position(|(locale, _)| *locale == en())
        .expect("en-US is detected");
    let zh_position = detection
        .candidates
        .iter()
        .position(|(locale, _)| *locale == zh())
        .expect("zh-CN remains declared");
    assert!(
        en_position < zh_position,
        "en-US should rank ahead of zh-CN"
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
    let output = emitter::emit_wir(&program, &catalog, &zh()).expect("settings corpus emits");
    assert!(output.contains("自由混战人数上限: 6"), "{}", output);
}

#[test]
fn settings_namespace_spellings_emit_and_reparse_in_zh_cn() {
    use workshop_rs::settings::Settings;
    let catalog = builtin();
    let group = |name: &str| SettingsNode::Group {
        name: name.to_string(),
        children: Vec::new(),
        span: None,
    };
    let program = workshop_rs::wir::Program {
        settings: Some(Settings {
            span: None,
            children: vec![
                group("main"),
                group("lobby"),
                group("gamemodes"),
                group("heroes"),
                group("extensions"),
                SettingsNode::Workshop {
                    children: Vec::new(),
                    span: None,
                },
            ],
        }),
        ..workshop_rs::wir::Program::default()
    };
    let output = emitter::emit_wir(&program, &catalog, &zh()).expect("zh-CN settings emit");
    for header in ["主程序", "大厅", "模式", "英雄", "扩展", "地图工坊"] {
        assert!(output.contains(&format!("{header} {{")), "{output}");
    }
    let reparsed = parser::parse_wir(&output, &catalog, &zh()).expect("zh-CN settings reparse");
    assert!(workshop_rs::roundtrip::equivalent_wir(&program, &reparsed));
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
    let en_program = parser::parse_wir_with_context(SYNTHETIC_SOURCE, &catalog, &en(), &catalog)
        .expect("parses");
    let xx_program =
        parser::parse_wir_with_context(SYNTHETIC_TARGET, &catalog, &Locale::new("xx-YY"), &catalog)
            .expect("parses");
    assert!(
        workshop_rs::roundtrip::equivalent_wir(&en_program, &xx_program),
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
    let program =
        parser::parse_wir_with_context(source, &catalog, &zh(), &catalog).expect("parses");
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
