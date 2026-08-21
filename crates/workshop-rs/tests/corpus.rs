//! Corpus-backed zh-CN conversion evidence.

use workshop_rs::catalog::{Catalog, Locale};
use workshop_rs::convert::{self, ConvertOptions};

fn catalog() -> Catalog {
    Catalog::builtin().expect("catalog validates")
}

fn en() -> Locale {
    Locale::new("en-US")
}

fn zh() -> Locale {
    Locale::new("zh-CN")
}

const REPRESENTATIVE: &str = "variables {
    global:
        0: result
}

rule (\"corpus\") {
    event {
        Ongoing - Global;
    }
    actions {
        Set Global Variable(result, Add(1, 2));
        Wait(1, Ignore Condition);
    }
}
";

#[test]
fn manifest_pins_the_export_and_exact_match_coverage() {
    let manifest: serde_json::Value =
        serde_json::from_str(include_str!("../../../tools/corpus/zh-cn-corpus.json"))
            .expect("generated corpus manifest is valid JSON");
    assert_eq!(manifest["locale"], "zh-CN");
    assert_eq!(
        manifest["source"]["commit"],
        "d854bf01fc7bbf3b2169f67408c07a8da8989ad6"
    );
    assert_eq!(manifest["coverage"]["total"]["matched"], 366);
    assert_eq!(manifest["coverage"]["total"]["total"], 366);
    assert_eq!(manifest["matches"].as_array().unwrap().len(), 366);
    assert_eq!(manifest["excluded"].as_array().unwrap().len(), 0);
    for (kind, id, source, zh_cn) in [
        (
            "action",
            "stopChasingVariable",
            "actions.__stopChasingGlobalVariable__",
            "停止追踪全局变量",
        ),
        (
            "action",
            "forcePlayerHero",
            "actions..startForcingHero",
            "开始强制玩家选择英雄",
        ),
        (
            "action",
            "stopForcingHero",
            "actions..stopForcingCurrentHero",
            "停止强制玩家选择英雄",
        ),
        (
            "action",
            "forceThrottle",
            "actions..startForcingThrottle",
            "开始限制阈值",
        ),
        ("operator", "==", "localizedStrings.{0} == {1}", "=="),
        ("operator", "!=", "localizedStrings.{0} != {1}", "!="),
        ("operator", "<=", "localizedStrings.{0} <= {1}", "<="),
        ("operator", ">=", "localizedStrings.{0} >= {1}", ">="),
        ("operator", "<", "localizedStrings.{0} < {1}", "<"),
        ("operator", ">", "localizedStrings.{0} > {1}", ">"),
        (
            "enum member",
            "Map.LIJIANG_TOWER_LUNAR",
            "maps.lijiangTowerLny",
            "春节漓江塔",
        ),
        (
            "enum member",
            "ProgressBarWorldReeval.VISIBLE_TO_AND_VALUES",
            "constants.ProgressHudReeval.VISIBILITY_AND_VALUES",
            "可见和值",
        ),
        (
            "enum member",
            "Rounding.NEAREST",
            "constants.__Rounding__.__roundToNearest__",
            "至最近",
        ),
    ] {
        let entry = manifest["matches"]
            .as_array()
            .unwrap()
            .iter()
            .find(|entry| entry["kind"] == kind && entry["id"] == id)
            .unwrap_or_else(|| panic!("confirmed mapping is recorded: {kind} {id}"));
        assert_eq!(entry["sources"], serde_json::json!([source]));
        assert_eq!(entry["zh-CN"], zh_cn);
    }
    let set_allowed = manifest["matches"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["id"] == "setAllowedHeroes")
        .expect("confirmed setAllowedHeroes identity mapping is recorded");
    assert_eq!(set_allowed["zh-CN"], "设置玩家可选的英雄");
    assert_eq!(
        set_allowed["sources"],
        serde_json::json!(["actions..setAllowedHeroes"])
    );
}

#[test]
fn settings_corpus_includes_general_mode_and_team_labels() {
    let settings: serde_json::Value =
        serde_json::from_str(include_str!("../src/settings/data/locales.json"))
            .expect("generated settings corpus is valid JSON");
    assert_eq!(settings["locales"], serde_json::json!(["en-US", "zh-CN"]));
    assert_eq!(settings["modes"]["General"]["zh-CN"], "综合");
    assert_eq!(settings["teams"]["General"]["zh-CN"], "综合");
    assert_eq!(
        settings["labels"]["Ultimate Generation - Passive Blizzard"]["zh-CN"],
        "终极技能自动充能速度 暴雪"
    );
    assert_eq!(
        settings["labels"]["Ultimate Generation - Combat Blizzard"]["zh-CN"],
        "战斗时终极技能充能速度 暴雪"
    );
}

#[test]
fn representative_corpus_converts_in_both_directions() {
    let catalog = catalog();
    let to_zh = convert::convert(
        REPRESENTATIVE,
        &catalog,
        &en(),
        &zh(),
        &ConvertOptions::default(),
    )
    .expect("en-US corpus converts to zh-CN");
    assert!(to_zh.fallback_ids.is_empty());
    assert!(to_zh.text.contains("持续 - 全局"), "{}", to_zh.text);
    assert!(to_zh.text.contains("设置全局变量"), "{}", to_zh.text);
    assert!(to_zh.text.contains("加(1, 2)"), "{}", to_zh.text);
    assert!(to_zh.text.contains("等待(1, 无视条件)"), "{}", to_zh.text);

    let back_to_en = convert::convert(
        &to_zh.text,
        &catalog,
        &zh(),
        &en(),
        &ConvertOptions::default(),
    )
    .expect("zh-CN corpus converts back to en-US");
    assert_eq!(back_to_en.fallback_ids, Vec::<String>::new());
    assert_eq!(back_to_en.text.trim_end(), REPRESENTATIVE.trim_end());
}

#[test]
fn confirmed_set_allowed_heroes_mapping_converts_in_both_directions() {
    let source = "rule (\"set-allowed\") {
    event {
        Ongoing - Global;
    }
    actions {
        Set Allowed Heroes(All Players(Team(All Teams)), Ana);
    }
}
";
    let catalog = catalog();
    let to_zh = convert::convert(source, &catalog, &en(), &zh(), &ConvertOptions::default())
        .expect("setAllowedHeroes converts to zh-CN");
    assert_eq!(to_zh.fallback_ids, Vec::<String>::new());
    assert!(to_zh.text.contains("设置玩家可选的英雄"), "{}", to_zh.text);

    let back_to_en = convert::convert(
        &to_zh.text,
        &catalog,
        &zh(),
        &en(),
        &ConvertOptions::default(),
    )
    .expect("setAllowedHeroes converts back to en-US");
    assert_eq!(back_to_en.fallback_ids, Vec::<String>::new());
    assert_eq!(back_to_en.text.trim_end(), source.trim_end());
}

#[test]
fn confirmed_legacy_aliases_convert_in_both_directions() {
    let source = "variables {
    global:
        0: value
}

rule (\"legacy-aliases\") {
    event {
        Ongoing - Global;
    }
    actions {
        Stop Chasing Variable(Global.value);
        Force Player Hero(Event Player, Ana);
        Stop Forcing Hero(Event Player);
        Force Throttle(Event Player, 100, 100, 100, 100, 100, 100);
    }
}
";
    let catalog = catalog();
    let to_zh = convert::convert(source, &catalog, &en(), &zh(), &ConvertOptions::default())
        .expect("confirmed legacy aliases convert to zh-CN");
    assert_eq!(to_zh.fallback_ids, Vec::<String>::new());
    for spelling in [
        "停止追踪全局变量",
        "开始强制玩家选择英雄",
        "停止强制玩家选择英雄",
        "开始限制阈值",
    ] {
        assert!(
            to_zh.text.contains(spelling),
            "missing {spelling}: {}",
            to_zh.text
        );
    }

    let back_to_en = convert::convert(
        &to_zh.text,
        &catalog,
        &zh(),
        &en(),
        &ConvertOptions::default(),
    )
    .expect("confirmed legacy aliases convert back to en-US");
    assert_eq!(back_to_en.fallback_ids, Vec::<String>::new());
    assert_eq!(back_to_en.text.trim_end(), source.trim_end());
}
