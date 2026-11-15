//! Hero settings producer aliases: the spellings the pinned OverPy producer
//! writes for hero settings where they differ from the export labels, split
//! out of `settings_pipeline.rs` under the size guard (workshop-rs#540).

use super::common::{catalog, en, zh};
use super::internal;
use workshop_rs::roundtrip;

#[test]
fn pinned_ai_hero_setting_aliases_are_canonical() {
    let catalog = catalog();
    let source = "设置 { 英雄 { 综合 { 索杰恩 { 充能速度 Charged Shot: 200% } 路霸 { secondaryFireRechargeRate%: 150 } 骇灾 { 尖刺护体资源恢复倍率: 150% 尖刺护体资源消耗倍率: 50% } } } }";
    let program = internal::parse_in(source, &zh());
    assert!(
        program
            .semantic_issues(&catalog)
            .iter()
            .all(|issue| { issue.kind != workshop_rs::rules::IncompletenessKind::RawSetting })
    );
}

#[test]
fn pinned_hero_setting_names_emit_the_oracle_spellings() {
    let catalog = catalog();
    let en = en();
    let source = "settings { heroes { General { \
        Ana { No Automatic Fire: Off No Scope: Off } \
        Genji { Secondary Fire: Off } \
        Roadhog { secondaryFireCooldown%: 50 secondaryFireMaximumTime%: 60 secondaryFireRechargeRate%: 70 Secondary Fire: Off Take a Breather: Off Take a Breather Cooldown Time: 80% } \
        Vendetta { Warding Stance Regen Scalar: 90% } \
        } } }";
    let program = internal::parse_in(source, &en);
    let emitted = internal::emit_in(&program, &en);
    for expected in [
        "No Automatic Fire: Off",
        "No Scope: Off",
        "Secondary Fire: Off",
        "secondaryFireCooldown%: 50",
        "secondaryFireMaximumTime%: 60",
        "secondaryFireRechargeRate%: 70",
        "Take a Breather: Off",
        "Take a Breather Cooldown Time: 80%",
        "Warding Stance Regen Scalar: 90%",
    ] {
        assert!(emitted.contains(expected), "{emitted} missing {expected}");
    }
    let reparsed = internal::parse_in(&emitted, &en);
    assert!(roundtrip::equivalent_wir(&program, &reparsed));
    assert!(
        program
            .semantic_issues(&catalog)
            .iter()
            .all(|issue| issue.kind != workshop_rs::rules::IncompletenessKind::RawSetting)
    );
}

/// workshop-rs#618: the pinned OverPy 9.7.17 table declares the tiebreaker and
/// reveal-heroes keys for `general` and `elimination` only.
#[test]
fn general_tiebreaker_and_reveal_keys_emit_localized_names() {
    let catalog = catalog();
    let (en, zh) = (en(), zh());
    let source = "settings { modes { General { \
        Draw After Match Time Elapsed With No Tiebreaker: 300 \
        Reveal Heroes: On \
        Reveal Heroes After Match Time Elapsed: 25 \
        Capture Objective Tiebreaker: Off \
        Tiebreaker After Match Time Elapsed: 300 } } }";
    let program = internal::parse_in(source, &en);
    let emitted = internal::emit_in(&program, &zh);
    for expected in [
        "无目标点决胜机制时平局时限: 300",
        "显示英雄位置: 开启",
        "比赛开始后显示英雄位置时间: 25",
        "目标点决胜机制: 关闭",
        "目标点决胜机制激活时间: 300",
    ] {
        assert!(emitted.contains(expected), "{emitted} missing {expected}");
    }
    assert!(
        program
            .semantic_issues(&catalog)
            .iter()
            .all(|issue| issue.kind != workshop_rs::rules::IncompletenessKind::RawSetting)
    );

    let other_mode = "settings { modes { Team Deathmatch { Reveal Heroes: On } } }";
    let program = internal::parse_in(other_mode, &en);
    assert!(
        program
            .semantic_issues(&catalog)
            .iter()
            .any(|issue| issue.kind == workshop_rs::rules::IncompletenessKind::RawSetting),
        "a mode other than general and elimination must not inherit the key"
    );
}

#[test]
fn spawn_health_packs_label_and_values_convert_to_zh_cn_and_back() {
    let catalog = catalog();
    let en = en();
    let zh = zh();
    let squash = |text: &str| text.split_whitespace().collect::<String>();
    // The enabled value must not use the `Enabled` spelling shared with the
    // lobby voice-chat setting.
    for (value, zh_value) in [
        ("Enabled", "开启"),
        ("Disabled", "禁用"),
        ("Determined By Mode", "依据比赛模式而定"),
    ] {
        let source =
            format!("settings {{ modes {{ General {{ Spawn Health Packs: {value} }} }} }}");
        let to_zh = workshop_rs::convert::convert(&source, &catalog, &en, &zh, &Default::default())
            .expect("spawn health packs converts to zh-CN");
        assert!(to_zh.text.contains("急救包刷新"), "{}", to_zh.text);
        assert!(!to_zh.text.contains("生成治疗包"));
        assert!(
            squash(&to_zh.text).contains(&format!("急救包刷新:{zh_value}")),
            "{}",
            to_zh.text
        );

        let back =
            workshop_rs::convert::convert(&to_zh.text, &catalog, &zh, &en, &Default::default())
                .expect("zh-CN spawn health packs converts back to en-US");
        assert_eq!(squash(&back.text), squash(&source));
    }
}

#[test]
fn shared_enabled_spelling_keeps_its_other_domains() {
    let catalog = catalog();
    let en = en();
    let zh = zh();
    let source = "settings { lobby { Match Voice Chat: Enabled } }";
    let to_zh = workshop_rs::convert::convert(source, &catalog, &en, &zh, &Default::default())
        .expect("match voice chat converts to zh-CN");
    assert!(to_zh.text.contains("启用"), "{}", to_zh.text);
}

#[test]
fn spawn_health_packs_source_edit_writes_the_same_zh_cn_value_as_emit() {
    use workshop_rs::settings::{SettingTarget, SettingValue, definitions};
    let source = "设置 { 模式 { 综合 { 急救包刷新: 禁用 } } }";
    let settings = internal::parse_in(source, &zh())
        .settings
        .expect("a settings block");
    let definition = definitions()
        .find(|definition| definition.path() == "gamemodes.general.spawnHealthPacks")
        .expect("spawnHealthPacks definition");
    let edit = definition
        .source_edit(
            source,
            &settings,
            "zh-CN",
            &SettingTarget::Mode("general".to_string()),
            SettingValue::Enum("enabled".to_string()),
        )
        .expect("zh-CN source edit");
    let edited = edit.apply(source).expect("edit applies");
    assert_eq!(edited, "设置 { 模式 { 综合 { 急救包刷新: 开启 } } }");
}
