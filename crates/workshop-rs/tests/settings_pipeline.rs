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
fn supported_dva_name_parses() {
    let catalog = catalog();
    let source =
        "settings {\n heroes {\n  General {\n   D.Va {\n    Primary Fire: Off\n   }\n  }\n }\n}";
    let program = parser::parse(source, &catalog, &Locale::new("en-US")).expect("parses");
    let emitted = emitter::emit(&program, &catalog, &Locale::new("en-US")).expect("emits");
    assert!(emitted.contains("D.Va"));
}
