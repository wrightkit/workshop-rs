//! End-to-end raw Workshop settings coverage using the reviewed en-US and
//! zh-CN settings mappings.

use std::path::{Path, PathBuf};

use workshop_rs::catalog::{Catalog, Locale};
use workshop_rs::{convert, emitter, parser, roundtrip};

fn fixture_path(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/settings")
        .join(name)
}

fn fixture(name: &str) -> String {
    std::fs::read_to_string(fixture_path(name)).expect("settings fixture")
}

fn collapse(text: &str) -> String {
    text.chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

fn catalog() -> Catalog {
    Catalog::builtin().expect("builtin catalog")
}

#[test]
fn real_settings_fixture_parses_to_wir_and_reemits() {
    let catalog = catalog();
    let source = fixture("pixelart.settings.ws");
    let program = parser::parse(&source, &catalog, &Locale::new("en-US")).expect("parses");
    assert!(program.settings.is_some(), "settings are carried in WIR");

    let emitted = emitter::emit(&program, &catalog, &Locale::new("en-US")).expect("emits");
    assert_eq!(collapse(&emitted), collapse(&source));
    let reparsed = parser::parse(&emitted, &catalog, &Locale::new("en-US")).expect("reparses");
    assert!(roundtrip::equivalent(&program, &reparsed));
}

#[test]
fn reviewed_settings_conversion_round_trips_en_us_and_zh_cn() {
    let catalog = catalog();
    let en = Locale::new("en-US");
    let zh = Locale::new("zh-CN");
    let source = fixture("pixelart.settings.ws");
    let expected_zh = fixture("pixelart.zh-CN.settings.ws");

    let to_zh = convert::convert(&source, &catalog, &en, &zh, &Default::default())
        .expect("en-US -> zh-CN settings conversion");
    assert!(to_zh.fallback_ids.is_empty());
    assert_eq!(collapse(&to_zh.text), collapse(&expected_zh));

    let zh_program = parser::parse(&to_zh.text, &catalog, &zh).expect("zh-CN settings parse");
    let en_program = parser::parse(&source, &catalog, &en).expect("en-US settings parse");
    assert!(roundtrip::equivalent(&en_program, &zh_program));

    let back_to_en = convert::convert(&expected_zh, &catalog, &zh, &en, &Default::default())
        .expect("zh-CN -> en-US settings conversion");
    assert_eq!(collapse(&back_to_en.text), collapse(&source));
}

#[test]
fn composed_blizzard_settings_labels_convert_in_both_directions() {
    let source = "settings {
    heroes {
        General {
            Mei {
                Ultimate Generation - Passive Blizzard: 0%
                Ultimate Generation - Combat Blizzard: 0%
            }
        }
    }
}";
    let catalog = catalog();
    let en = Locale::new("en-US");
    let zh = Locale::new("zh-CN");
    let to_zh = convert::convert(source, &catalog, &en, &zh, &Default::default())
        .expect("composed settings labels convert to zh-CN");
    assert!(to_zh.fallback_ids.is_empty());
    assert!(to_zh.text.contains("终极技能自动充能速度 暴雪"));
    assert!(to_zh.text.contains("战斗时终极技能充能速度 暴雪"));

    let back_to_en = convert::convert(&to_zh.text, &catalog, &zh, &en, &Default::default())
        .expect("composed settings labels convert back to en-US");
    assert!(back_to_en.fallback_ids.is_empty());
    assert_eq!(collapse(&back_to_en.text), collapse(source));
}

#[test]
fn supported_apostrophe_map_name_parses() {
    let catalog = catalog();
    let source = "settings { modes { Deathmatch { enabled maps { King's Row Winter } } } }";
    let program = parser::parse(source, &catalog, &Locale::new("en-US")).expect("parses");
    let emitted = emitter::emit(&program, &catalog, &Locale::new("en-US")).expect("emits");
    assert!(emitted.contains("King's Row Winter"));
}

#[test]
fn generated_capture_the_flag_settings_surface_is_canonical() {
    let catalog = catalog();
    let source = "settings { modes { Capture The Flag { enabled maps { Ayutthaya } Flag Score Respawn Time: 15 Flag Return Time: 4 Flag Dropped Lock Time: 5 } } }";
    let program = parser::parse(source, &catalog, &Locale::new("en-US")).expect("parses");
    assert!(
        program
            .semantic_issues(&catalog)
            .iter()
            .all(|issue| { issue.kind != workshop_rs::semantic::IncompletenessKind::RawSetting })
    );
}

#[test]
fn pinned_ai_hero_setting_aliases_are_canonical() {
    let catalog = catalog();
    let source = "设置 { 英雄 { 综合 { 索杰恩 { 充能速度 充能射击: 200% } 路霸 { 呼吸器充能速度: 150% } 骇灾 { 尖刺护体资源恢复: 150% 尖刺护体资源消耗: 50% } } } }";
    let program = parser::parse(source, &catalog, &Locale::new("zh-CN")).expect("parses");
    assert!(
        program
            .semantic_issues(&catalog)
            .iter()
            .all(|issue| { issue.kind != workshop_rs::semantic::IncompletenessKind::RawSetting })
    );
}

#[test]
fn mixed_locale_primary_hero_setting_name_is_canonical() {
    let catalog = catalog();
    let source = "设置 { 英雄 { 队伍1 { D.Mon { 伤害量: 140% } } } }";
    let program = parser::parse(source, &catalog, &Locale::new("zh-CN"))
        .expect("primary-locale D.Mon spelling parses in mixed zh-CN output");
    assert!(
        program
            .semantic_issues(&catalog)
            .iter()
            .all(|issue| { issue.kind != workshop_rs::semantic::IncompletenessKind::RawSetting })
    );
}

#[test]
fn team_deathmatch_enabled_maps_is_canonical() {
    let text = r#"
        settings {
            modes {
                Team Deathmatch {
                    enabled maps { }
                }
            }
        }
    "#;
    let catalog = Catalog::builtin().unwrap();
    let program = parser::parse_with_context(text, &catalog, &Locale::new("en-US"), &catalog)
        .expect("Team Deathmatch enabled maps must parse");
    let emitted = emitter::emit(&program, &catalog, &Locale::new("en-US"))
        .expect("Team Deathmatch enabled maps must emit");
    assert!(emitted.contains("Team Deathmatch"));
    let reparsed = parser::parse_with_context(&emitted, &catalog, &Locale::new("en-US"), &catalog)
        .expect("emitted Team Deathmatch settings must reparse");
    assert!(roundtrip::equivalent(&program, &reparsed));
}

#[test]
fn canonical_percent_setting_keys_parse_from_mixed_locale_exports() {
    let text = r#"
        settings {
            heroes {
                General {
                    Roadhog {
                        secondaryFireRechargeRate%: 150
                        secondaryFireCooldown%: 480
                    }
                }
            }
        }
    "#;
    let catalog = Catalog::builtin().unwrap();
    parser::parse_with_context(text, &catalog, &Locale::new("zh-CN"), &catalog)
        .expect("canonical percent setting keys must parse");
}

#[test]
fn supported_dva_name_parses() {
    let catalog = catalog();
    let source =
        "settings {\n heroes {\n  General {\n   D.Va {\n    Primary Fire: Off\n   }\n  }\n }\n}";
    let program = parser::parse(source, &catalog, &Locale::new("en-US")).expect("parses");
    let emitted = emitter::emit(&program, &catalog, &Locale::new("en-US")).expect("emits");
    assert!(emitted.contains("D.Va"));
}

#[test]
fn hero_ability_names_resolve_through_gameplay_catalog_in_both_locales() {
    let catalog = catalog();
    let en = Locale::new("en-US");
    let zh = Locale::new("zh-CN");
    let source = "settings { heroes { General { Mei { Cryo-Freeze: Off Ice Wall: On } } } }";
    let program = parser::parse(source, &catalog, &en).expect("English ability names parse");
    let emitted = emitter::emit(&program, &catalog, &zh).expect("Chinese ability names emit");
    assert!(emitted.contains("急冻: 关"));
    assert!(emitted.contains("冰墙: 开"));
    let reparsed = parser::parse(&emitted, &catalog, &zh).expect("Chinese ability names parse");
    assert!(roundtrip::equivalent(&program, &reparsed));
    let back = emitter::emit(&reparsed, &catalog, &en).expect("English ability names re-emit");
    assert!(back.contains("Cryo-Freeze: Off"));
    assert!(back.contains("Ice Wall: On"));
}

#[test]
fn disabled_maps_is_a_known_symmetric_settings_list() {
    let catalog = catalog();
    let source = "settings { modes { disabled Skirmish { disabled maps {\nKing's Row Winter\nWorkshop Island\n} } } }";
    let program = parser::parse(source, &catalog, &Locale::new("en-US")).expect("parses");
    let issues = program.semantic_issues(&catalog);
    assert!(
        issues
            .iter()
            .all(|issue| issue.kind != workshop_rs::semantic::IncompletenessKind::RawSetting),
        "known disabled-map settings must not remain raw: {issues:?}"
    );
    let emitted = emitter::emit(&program, &catalog, &Locale::new("en-US")).expect("emits");
    assert!(emitted.contains("disabled Skirmish"));
    assert!(emitted.contains("disabled maps"));
    assert!(emitted.contains("King's Row Winter"));
    assert!(emitted.contains("Workshop Island"));
}

#[test]
fn unknown_settings_list_members_remain_semantically_incomplete() {
    let catalog = catalog();
    let source = "settings { modes { Skirmish { enabled maps { Future Map } } } }";
    let program = parser::parse(source, &catalog, &Locale::new("en-US")).expect("preserves");
    assert!(program.semantic_issues(&catalog).iter().any(|issue| {
        issue.kind == workshop_rs::semantic::IncompletenessKind::RawSetting
            && issue.name == "enabledMaps"
    }));
}
