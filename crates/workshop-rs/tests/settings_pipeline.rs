//! End-to-end raw Workshop settings coverage using the reviewed en-US and
//! zh-CN settings mappings.

use super::common::{self, catalog, en, zh};
use super::internal;
use workshop_rs::catalog::Catalog;
use workshop_rs::gameplay::{AbilityVariant, HeroId, LogicalSlot, hero_ids, slots};
use workshop_rs::settings::{
    Applicability, NumericBounds, SettingId, SettingIdentity, SettingOperationError, SettingScope,
    SettingSourceKind, SettingTarget, SettingTargetKind, SettingValue, SettingValueDomain, TeamId,
    definitions, definitions_by_id,
};
use workshop_rs::{analysis::semantic, convert, emitter, parser, roundtrip};

fn fixture(name: &str) -> String {
    common::fixture_text("settings", name)
}

fn collapse(text: &str) -> String {
    text.chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

#[test]
fn real_settings_fixture_parses_to_wir_and_reemits() {
    let _catalog = catalog();
    let source = fixture("pixelart.settings.ws");
    let program = internal::parse(&source);
    assert!(program.settings.is_some(), "settings are carried in WIR");

    let emitted = internal::emit(&program);
    assert_eq!(collapse(&emitted), collapse(&source));
    internal::assert_reparse_equivalent(&program, &emitted);
}

#[test]
fn reviewed_settings_conversion_round_trips_en_us_and_zh_cn() {
    let catalog = catalog();
    let en = en();
    let zh = zh();
    let source = fixture("pixelart.settings.ws");
    let expected_zh = fixture("pixelart.zh-CN.settings.ws");

    let to_zh = convert::convert(&source, &catalog, &en, &zh, &Default::default())
        .expect("en-US -> zh-CN settings conversion");
    assert!(to_zh.fallback_ids.is_empty());
    assert_eq!(collapse(&to_zh.text), collapse(&expected_zh));

    let zh_program = internal::parse_in(&to_zh.text, &zh);
    let en_program = internal::parse_in(&source, &en);
    assert!(roundtrip::equivalent_wir(&en_program, &zh_program));

    let back_to_en = convert::convert(&expected_zh, &catalog, &zh, &en, &Default::default())
        .expect("zh-CN -> en-US settings conversion");
    assert_eq!(collapse(&back_to_en.text), collapse(&source));
}

#[test]
fn match_voice_chat_uses_reviewed_enabled_tokens_in_both_locales() {
    let _catalog = catalog();
    let en = en();
    let zh = zh();
    let source = "settings { lobby { Match Voice Chat: Enabled } }";
    let program = internal::parse_in(source, &en);

    let emitted_en = internal::emit_in(&program, &en);
    assert!(emitted_en.contains("Match Voice Chat: Enabled"));
    assert!(!emitted_en.contains("Match Voice Chat: On"));
    let reparsed_en = internal::parse_in(&emitted_en, &en);
    assert!(roundtrip::equivalent_wir(&program, &reparsed_en));

    let emitted_zh = internal::emit_in(&program, &zh);
    assert!(emitted_zh.contains("比赛语音聊天: 启用"));
    let reparsed_zh = internal::parse_in(&emitted_zh, &zh);
    assert!(roundtrip::equivalent_wir(&program, &reparsed_zh));
}

#[test]
fn match_voice_chat_rejects_unreviewed_disabled_token() {
    let _catalog = catalog();
    let en = en();
    let source = "settings { lobby { Match Voice Chat: Disabled } }";
    let error = internal::try_parse_in(source, &en).expect_err("disabled state is source-unbacked");
    assert!(format!("{error:?}").contains("settings enum"));
}

#[test]
fn capture_the_flag_settings_emit_and_reparse_in_zh_cn() {
    let _catalog = catalog();
    let source = "settings { modes { Capture The Flag {} } }";
    let en = en();
    let zh = zh();
    let program = internal::parse_in(source, &en);

    let emitted = internal::emit_in(&program, &zh);
    assert!(emitted.contains("勇夺锦旗"), "{emitted}");

    internal::assert_reparse_equivalent_in(&program, &emitted, &zh);
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
    let en = en();
    let zh = zh();
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

// Minimized from OWBastion/Bastion 7debbcb, src/composition/bootstrap.opy.
#[test]
fn map_lists_preserve_adjacent_number_and_localized_text() {
    let catalog = catalog();
    let en = en();
    let zh = zh();
    for list in ["enabled maps", "disabled maps"] {
        let source = format!("settings {{ modes {{ Skirmish {{ {list} {{\nRoute 66\n}} }} }} }}");
        let program = parser::parse_wir(&source, &catalog, &en).expect("English map list");
        let localized = emitter::emit_wir(&program, &catalog, &zh).expect("Chinese map list");
        let reparsed =
            parser::parse_wir(&localized, &catalog, &zh).expect("numeric Chinese map name");
        assert!(roundtrip::equivalent_wir(&program, &reparsed));
    }
}

#[test]
fn map_lists_preserve_colons_in_map_names() {
    let catalog = catalog();
    let en = en();
    let zh = zh();
    for list in ["enabled maps", "disabled maps"] {
        let source = format!(
            "settings {{ modes {{ Skirmish {{ {list} {{\nWatchpoint: Gibraltar\nEcopoint: Antarctica\n}} }} }} }}"
        );
        let program =
            parser::parse_wir(&source, &catalog, &en).expect("colon-containing map names");
        let emitted = emitter::emit_wir(&program, &catalog, &en).expect("English map list");
        let reparsed =
            parser::parse_wir(&emitted, &catalog, &en).expect("English map list reparses");
        assert!(roundtrip::equivalent_wir(&program, &reparsed));
        let localized = emitter::emit_wir(&program, &catalog, &zh).expect("Chinese map list");
        let reparsed =
            parser::parse_wir(&localized, &catalog, &zh).expect("Chinese map list reparses");
        assert!(roundtrip::equivalent_wir(&program, &reparsed));
    }
}

#[test]
fn map_lists_preserve_spaced_colons_in_map_names() {
    let catalog = catalog();
    let en = en();
    let fr = common::locale("fr-FR");
    for list in ["enabled maps", "disabled maps"] {
        let source = format!(
            "settings {{ modes {{ Skirmish {{ {list} {{\nWatchpoint: Gibraltar\nEcopoint: Antarctica\n}} }} }} }}"
        );
        let program =
            parser::parse_wir(&source, &catalog, &en).expect("colon-containing map names");
        let localized = emitter::emit_wir(&program, &catalog, &fr).expect("French map list");
        let reparsed =
            parser::parse_wir(&localized, &catalog, &fr).expect("French map list reparses");
        assert!(roundtrip::equivalent_wir(&program, &reparsed));
    }
}

#[test]
fn supported_apostrophe_map_name_parses() {
    let _catalog = catalog();
    let source = "settings { modes { Deathmatch { enabled maps { King's Row Winter } } } }";
    let program = internal::parse(source);
    let emitted = internal::emit(&program);
    assert!(emitted.contains("King's Row Winter"));
}

#[test]
fn generated_capture_the_flag_settings_surface_is_canonical() {
    let catalog = catalog();
    let source = "settings { modes { Capture The Flag { enabled maps { Ayutthaya } Flag Score Respawn Time: 15 Flag Return Time: 4 Flag Dropped Lock Time: 5 } } }";
    let program = internal::parse(source);
    assert!(
        program
            .semantic_issues(&catalog)
            .iter()
            .all(|issue| { issue.kind != workshop_rs::rules::IncompletenessKind::RawSetting })
    );
}

#[test]
fn pinned_ai_hero_setting_aliases_are_canonical() {
    let catalog = catalog();
    let source = "设置 { 英雄 { 综合 { 索杰恩 { 充能速度 充能射击: 200% } 路霸 { 呼吸器充能速度: 150% } 骇灾 { 尖刺护体资源恢复: 150% 尖刺护体资源消耗: 50% } } } }";
    let program = internal::parse_in(source, &zh());
    assert!(
        program
            .semantic_issues(&catalog)
            .iter()
            .all(|issue| { issue.kind != workshop_rs::rules::IncompletenessKind::RawSetting })
    );
}

#[test]
fn mixed_locale_primary_hero_setting_name_is_canonical() {
    let catalog = catalog();
    let source = "设置 { 英雄 { 队伍1 { D.Mon { 伤害量: 140% } } } }";
    let program = internal::parse_in(source, &zh());
    assert!(
        program
            .semantic_issues(&catalog)
            .iter()
            .all(|issue| { issue.kind != workshop_rs::rules::IncompletenessKind::RawSetting })
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
    let _catalog = Catalog::builtin().unwrap();
    let program = internal::parse(text);
    let emitted = internal::emit(&program);
    assert!(emitted.contains("Team Deathmatch"));
    internal::assert_reparse_equivalent(&program, &emitted);
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
    let _catalog = Catalog::builtin().unwrap();
    internal::parse_in(text, &zh());
}

#[test]
fn supported_dva_name_parses() {
    let _catalog = catalog();
    let source =
        "settings {\n heroes {\n  General {\n   D.Va {\n    Primary Fire: Off\n   }\n  }\n }\n}";
    let program = internal::parse(source);
    let emitted = internal::emit(&program);
    assert!(emitted.contains("D.Va"));
}

#[test]
fn hero_ability_names_resolve_through_gameplay_catalog_in_both_locales() {
    let _catalog = catalog();
    let en = en();
    let zh = zh();
    let source = "settings { heroes { General { Mei { Cryo-Freeze: Off Ice Wall: On } } } }";
    let program = internal::parse_in(source, &en);
    let emitted = internal::emit_in(&program, &zh);
    assert!(emitted.contains("急冻: 关"));
    assert!(emitted.contains("冰墙: 开"));
    let reparsed = internal::parse_in(&emitted, &zh);
    internal::assert_reparse_equivalent_in(&program, &emitted, &zh);
    let back = internal::emit_in(&reparsed, &en);
    assert!(back.contains("Cryo-Freeze: Off"));
    assert!(back.contains("Ice Wall: On"));
}

#[test]
fn disabled_maps_is_a_known_symmetric_settings_list() {
    let catalog = catalog();
    let source = "settings { modes { disabled Skirmish { disabled maps {\nKing's Row Winter\nWorkshop Island\n} } } }";
    let program = internal::parse(source);
    let issues = program.semantic_issues(&catalog);
    assert!(
        issues
            .iter()
            .all(|issue| issue.kind != workshop_rs::rules::IncompletenessKind::RawSetting),
        "known disabled-map settings must not remain raw: {issues:?}"
    );
    let emitted = internal::emit(&program);
    assert!(emitted.contains("disabled Skirmish"));
    assert!(emitted.contains("disabled maps"));
    assert!(emitted.contains("King's Row Winter"));
    assert!(emitted.contains("Workshop Island"));
}

#[test]
fn unknown_settings_list_members_remain_semantically_incomplete() {
    let catalog = catalog();
    let source = "settings { modes { Skirmish { enabled maps { Future Map } } } }";
    let program = internal::parse(source);
    assert!(program.semantic_issues(&catalog).iter().any(|issue| {
        issue.kind == workshop_rs::rules::IncompletenessKind::RawSetting
            && issue.name == "Future Map"
    }));
}

#[test]
fn workshop_namespace_preserves_custom_settings_without_residuals() {
    let source = r#"settings {
 workshop {
  AI-PVE {
   Custom Label: "Keep this"
   Custom Number: 42
  }
 }
}"#;
    let catalog = Catalog::builtin().expect("catalog");
    let locale = en();
    let program = internal::parse_in(source, &locale);
    assert!(
        semantic::inspect_wir(&program, &catalog).is_empty(),
        "issues: {:?}, settings: {:?}",
        semantic::inspect_wir(&program, &catalog),
        program.settings
    );
    let emitted = internal::emit_in(&program, &locale);
    internal::assert_reparse_equivalent_in(&program, &emitted, &locale);
    assert!(emitted.contains("Custom Label: \"Keep this\""));
    assert!(emitted.contains("Custom Number: 42"));
}

#[test]
fn localized_workshop_namespace_is_known() {
    let source = "settings { 地图工坊 { 自定义: 1 } }";
    let catalog = Catalog::builtin().expect("catalog");
    let program = internal::parse_in(source, &zh());
    assert!(
        semantic::inspect_wir(&program, &catalog).is_empty(),
        "issues: {:?}, settings: {:?}",
        semantic::inspect_wir(&program, &catalog),
        program.settings
    );
}

#[test]
fn localized_wrecking_ball_settings_aliases_are_known() {
    let source = "settings { heroes { 综合 { 破坏球 {\n工程抓钩冷却时间: 80%\n感应护盾冷却时间: 80%\n重力坠击冷却时间: 75%\n} } } }";
    let catalog = Catalog::builtin().expect("catalog");
    let program = internal::parse_in(source, &zh());
    assert!(
        semantic::inspect_wir(&program, &catalog).is_empty(),
        "issues: {:?}, settings: {:?}",
        semantic::inspect_wir(&program, &catalog),
        program.settings
    );
}

#[test]
fn settings_schema_projects_workshop_facts_without_display_names_in_ids() {
    let definitions: Vec<_> = definitions().collect();
    let hero_ability = definitions
        .iter()
        .find(|definition| {
            definition.path().ends_with("enablePrimaryFire")
                && definition.presentation().english_name == "Primary Fire"
        })
        .expect("hero ability definition");
    assert_eq!(hero_ability.presentation().english_name, "Primary Fire");
    assert_eq!(
        hero_ability.localized_name(
            "zh-CN",
            &SettingTarget::HeroAbility {
                team: None,
                hero: HeroId::from(hero_ids::MAUGA),
                slot: LogicalSlot::from(slots::PRIMARY_FIRE),
                variant: None,
            },
        ),
        Ok(Some("燃火链式机枪"))
    );
    assert!(hero_ability.source().reviewed);
    assert_eq!(
        hero_ability.id().map(SettingId::as_str),
        Some("setting.hero.ability.enabled")
    );
    assert!(matches!(hero_ability.identity(), SettingIdentity::Known(_)));
    assert_eq!(
        hero_ability.target_kind(),
        SettingTargetKind::HeroAbility {
            slot: LogicalSlot::from(slots::PRIMARY_FIRE),
            variant: None,
        }
    );
}

#[test]
fn settings_schema_keeps_distinct_hero_modifier_identities() {
    let damage_dealt = definitions_by_id(&SettingId::from("setting.hero.damageDealt"))
        .find(|definition| definition.target_kind() == SettingTargetKind::Hero)
        .expect("hero damage-dealt definition");
    let damage_received = definitions_by_id(&SettingId::from("setting.hero.damageReceived"))
        .find(|definition| definition.target_kind() == SettingTargetKind::Hero)
        .expect("hero damage-received definition");

    assert_ne!(damage_dealt.id(), damage_received.id());
    assert_ne!(damage_dealt.path(), damage_received.path());
}

#[test]
fn settings_schema_exposes_normal_enum_and_list_domains() {
    let definitions: Vec<_> = definitions().collect();
    let description = definitions
        .iter()
        .find(|definition| definition.path() == "main.description")
        .expect("main description definition");
    assert_eq!(description.scope(), SettingScope::Main);
    assert_eq!(description.target_kind(), SettingTargetKind::Global);
    assert!(matches!(description.domain(), SettingValueDomain::String));

    let role_limit = definitions
        .iter()
        .find(|definition| definition.path().ends_with("roleLimit"))
        .expect("role limit definition");
    assert!(matches!(
        role_limit.domain(),
        SettingValueDomain::Enum { domain } if domain == "roleLimit"
    ));

    let enabled_maps = definitions
        .iter()
        .find(|definition| definition.path().ends_with("enabledMaps"))
        .expect("enabled maps definition");
    assert!(matches!(enabled_maps.domain(), SettingValueDomain::MapList));
}

#[test]
fn settings_schema_distinguishes_applicability_and_unknown_hero_source_status() {
    let definitions: Vec<_> = definitions().collect();
    let ability3 = definitions
        .iter()
        .find(|definition| definition.path().ends_with("enableAbility3"))
        .expect("ability 3 definition");
    let ana = SettingTarget::HeroAbility {
        team: Some(TeamId::new("allTeams")),
        hero: HeroId::from(hero_ids::ANA),
        slot: LogicalSlot::from(slots::ABILITY_3),
        variant: Some(AbilityVariant::new("missing")),
    };
    assert_eq!(
        ability3.applicability(&ana).expect("applicability"),
        Applicability::NotApplicable
    );
    let ana_missing_slot_without_variant = SettingTarget::HeroAbility {
        team: Some(TeamId::new("allTeams")),
        hero: HeroId::from(hero_ids::ANA),
        slot: LogicalSlot::from(slots::ABILITY_3),
        variant: None,
    };
    assert_eq!(
        ability3
            .applicability(&ana_missing_slot_without_variant)
            .expect("applicability"),
        Applicability::NotApplicable
    );
    assert_eq!(
        ability3
            .localized_name("en-US", &ana)
            .expect("presentation"),
        None
    );

    let unknown = SettingTarget::HeroAbility {
        team: None,
        hero: HeroId::new("futureHero"),
        slot: LogicalSlot::from(slots::ABILITY_3),
        variant: None,
    };
    assert_eq!(
        ability3.applicability(&unknown).expect("applicability"),
        Applicability::Unknown
    );

    let ability1_enemy_kb = definitions
        .iter()
        .find(|definition| definition.path().ends_with("ability1EnemyKb%"))
        .expect("ability 1 enemy knockback setting");
    let ana_ability1 = SettingTarget::HeroAbility {
        team: None,
        hero: HeroId::from(hero_ids::ANA),
        slot: LogicalSlot::from(slots::ABILITY_1),
        variant: None,
    };
    assert_eq!(
        ability1_enemy_kb
            .applicability(&ana_ability1)
            .expect("applicability"),
        Applicability::Unknown
    );

    let ability1 = definitions
        .iter()
        .find(|definition| {
            definition.path().ends_with("enableAbility1")
                && matches!(
                    definition.target_kind(),
                    SettingTargetKind::HeroAbility { .. }
                )
        })
        .expect("ability 1 setting");
    assert_eq!(
        ability1
            .applicability(&SettingTarget::HeroAbility {
                team: None,
                hero: HeroId::from(hero_ids::MAUGA),
                slot: LogicalSlot::from(slots::ABILITY_1),
                variant: Some(AbilityVariant::new("missing")),
            })
            .expect("applicability"),
        Applicability::NotApplicable
    );

    let primary = definitions
        .iter()
        .find(|definition| {
            definition.path().ends_with("enablePrimaryFire")
                && matches!(
                    definition.target_kind(),
                    SettingTargetKind::TeamAbility { .. }
                )
        })
        .expect("generic primary-fire setting");
    assert_eq!(
        primary
            .applicability(&SettingTarget::HeroAbility {
                team: None,
                hero: HeroId::from(hero_ids::MAUGA),
                slot: LogicalSlot::from(slots::PRIMARY_FIRE),
                variant: None,
            })
            .expect("applicability"),
        Applicability::Unknown
    );

    let health = definitions
        .iter()
        .find(|definition| definition.path().ends_with("health%"))
        .expect("hero health setting");
    assert_eq!(
        health
            .applicability(&SettingTarget::Hero {
                team: None,
                hero: HeroId::new("futureHero"),
            })
            .expect("applicability"),
        Applicability::Unknown
    );
}

#[test]
fn settings_schema_preserves_authored_value_when_effective_value_is_clamped() {
    let domain = SettingValueDomain::Percent(
        NumericBounds::new(Some(0.0), Some(500.0)).expect("valid bounds"),
    );
    let effective = domain.effective_number(650.0).expect("finite number");
    assert_eq!(effective.authored, 650.0);
    assert_eq!(effective.effective, 500.0);

    let lower_bounded = NumericBounds::new(Some(0.0), None).expect("valid lower bound");
    assert!(lower_bounded.effective(1000.0).is_none());
    assert_eq!(lower_bounded.effective(-1.0).unwrap().effective, 0.0);
    let upper_bounded = NumericBounds::new(None, Some(500.0)).expect("valid upper bound");
    assert!(upper_bounded.effective(-1.0).is_none());
    assert_eq!(upper_bounded.effective(1000.0).unwrap().effective, 500.0);
}

#[test]
fn settings_schema_rejects_unknown_or_invalid_numeric_bounds() {
    let definitions: Vec<_> = definitions().collect();
    let percent = definitions
        .iter()
        .find(|definition| matches!(definition.domain(), SettingValueDomain::Percent(_)))
        .expect("percent definition");
    assert!(percent.effective_number(650.0).is_none());
    assert!(NumericBounds::new(Some(f64::NAN), Some(1.0)).is_err());
    assert!(NumericBounds::new(Some(2.0), Some(1.0)).is_err());
}

#[test]
fn settings_schema_normalizes_concept_ids_and_group_targets() {
    let definitions: Vec<_> = definitions().collect();
    for suffix in [
        "ability1EnemyKb%",
        "ability2FuseTime%",
        "secondaryFireRechargeRate%",
        "enableGenericSecondaryFire",
        "enableAutomaticFire",
        "enableScoping",
        "enablePassiveUnlimitedFuel",
        "enablePrimaryFireFreezeStack",
        "passiveUltGen%",
        "combatUltGen%",
        "ultGen%",
    ] {
        let definition = definitions
            .iter()
            .find(|definition| definition.path().ends_with(suffix))
            .expect("projected ability setting");
        assert!(definition.id().is_some());
        assert!(definition.source().reviewed);
    }
    let automatic_fire = definitions
        .iter()
        .find(|definition| definition.path().ends_with("enableAutomaticFire"))
        .expect("automatic-fire setting");
    let scoping = definitions
        .iter()
        .find(|definition| definition.path().ends_with("enableScoping"))
        .expect("scoping setting");
    assert_eq!(
        automatic_fire.id().map(SettingId::as_str),
        Some("setting.hero.primaryFire.automaticFireEnabled")
    );
    assert_eq!(
        scoping.id().map(SettingId::as_str),
        Some("setting.hero.primaryFire.scopingEnabled")
    );
    assert_ne!(automatic_fire.path(), scoping.path());

    let general = definitions
        .iter()
        .find(|definition| definition.path() == "gamemodes.general.heroLimit")
        .expect("general mode-group setting");
    assert_eq!(general.target_kind(), SettingTargetKind::Global);
    assert_eq!(
        general
            .applicability(&SettingTarget::Global)
            .expect("applicability"),
        Applicability::Applicable
    );

    let team_primary = definitions
        .iter()
        .find(|definition| {
            definition.path().ends_with("enablePrimaryFire")
                && matches!(
                    definition.target_kind(),
                    SettingTargetKind::TeamAbility { .. }
                )
        })
        .expect("team primary-fire setting");
    assert_eq!(
        team_primary.target_kind(),
        SettingTargetKind::TeamAbility {
            slot: LogicalSlot::from(slots::PRIMARY_FIRE),
            variant: None,
        }
    );
    assert_eq!(
        team_primary
            .applicability(&SettingTarget::TeamAbility {
                team: Some(TeamId::new("allTeams")),
                slot: LogicalSlot::from(slots::PRIMARY_FIRE),
                variant: None,
            })
            .expect("applicability"),
        Applicability::Applicable
    );
    assert_eq!(
        team_primary
            .applicability(&SettingTarget::HeroAbility {
                team: Some(TeamId::new("team1")),
                hero: HeroId::from(hero_ids::DVA),
                slot: LogicalSlot::from(slots::PRIMARY_FIRE),
                variant: Some(AbilityVariant::new("mech")),
            })
            .expect("applicability"),
        Applicability::Unknown
    );
    assert_eq!(
        team_primary
            .applicability(&SettingTarget::HeroAbility {
                team: Some(TeamId::new("team1")),
                hero: HeroId::from(hero_ids::DVA),
                slot: LogicalSlot::from(slots::PRIMARY_FIRE),
                variant: Some(AbilityVariant::new("pilot")),
            })
            .expect("applicability"),
        Applicability::Unknown
    );
    assert_eq!(
        team_primary
            .applicability(&SettingTarget::HeroAbility {
                team: Some(TeamId::new("team1")),
                hero: HeroId::from(hero_ids::DVA),
                slot: LogicalSlot::from(slots::ABILITY_1),
                variant: Some(AbilityVariant::new("mech")),
            })
            .expect("applicability"),
        Applicability::NotApplicable
    );
    let team_health = definitions
        .iter()
        .find(|definition| {
            definition.path().ends_with("health%")
                && definition.target_kind() == SettingTargetKind::Team
        })
        .expect("common team health setting");
    assert_eq!(
        team_health
            .applicability(&SettingTarget::Hero {
                team: Some(TeamId::new("team1")),
                hero: HeroId::from(hero_ids::ANA),
            })
            .expect("applicability"),
        Applicability::Unknown
    );
}

#[test]
fn settings_schema_preserves_locale_and_source_metadata() {
    let definitions: Vec<_> = definitions().collect();
    let main = definitions
        .iter()
        .find(|definition| definition.path() == "main.description")
        .expect("main definition");
    assert_eq!(
        main.presentation().localized_name("en-US"),
        Some("Description")
    );
    assert_eq!(main.source().kind, SettingSourceKind::RawWorkshopFixture);

    let generated = definitions
        .iter()
        .find(|definition| definition.path() == "extensions.beamEffects")
        .expect("generated definition");
    assert_eq!(
        generated.source().kind,
        SettingSourceKind::WorkshopDataExport
    );
}

#[test]
fn settings_schema_catalog_is_complete_and_conflict_checked() {
    workshop_rs::settings::schema::validate_catalog().expect("reviewed settings catalog");
}

#[test]
fn reconciled_export_enum_members_remain_writable() {
    let _catalog = catalog();
    for (source, id, path, member, expected) in [
        (
            "settings { modes { General { Hero Limit: Off } } }",
            "setting.gameMode.heroLimit",
            "gamemodes.general.heroLimit",
            "1PerTeam",
            "Hero Limit: 1 Per Team",
        ),
        (
            "settings { lobby { Map Rotation: After A Game } }",
            "setting.lobby.mapRotation",
            "lobby.mapRotation",
            "afterMirrorMatch",
            "Map Rotation: After A Mirror Match",
        ),
    ] {
        let mut program = internal::parse(source);
        let definition = definitions_by_id(&SettingId::from(id))
            .find(|definition| definition.path() == path)
            .expect("reconciled enum definition");
        definition
            .write(
                program.settings.as_mut().expect("settings"),
                &SettingTarget::Global,
                SettingValue::Enum(member.to_string()),
            )
            .expect("export-backed enum member remains writable");

        let emitted = internal::emit(&program);
        assert!(emitted.contains(expected), "{emitted}");
    }
}

#[test]
fn typed_settings_read_and_write_preserve_unrelated_structure() {
    let _catalog = Catalog::builtin().expect("catalog");
    let source = r#"settings {
        main {
            Description: "keep this"
        }
        lobby {
            Max Spectators: 2
        }
        heroes {
            General {
                D.Va {
                    Primary Fire: On
                }
            }
        }
    }"#;
    let mut program = internal::parse(source);
    let lobby = definitions_by_id(&SettingId::from("setting.lobby.spectatorSlots"))
        .next()
        .expect("lobby definition");
    let read = lobby
        .read(
            program.settings.as_ref().expect("settings"),
            &SettingTarget::Global,
        )
        .expect("typed read");
    assert_eq!(read.authored, SettingValue::Number(2.0));
    assert_eq!(read.effective, None);
    lobby
        .write(
            program.settings.as_mut().expect("settings"),
            &SettingTarget::Global,
            SettingValue::Number(4.0),
        )
        .expect("typed write");
    let emitted = internal::emit(&program);
    assert!(emitted.contains("Description: \"keep this\""));
    assert!(emitted.contains("Max Spectators: 4"));

    let primary = definitions()
        .find(|definition| {
            definition.path().ends_with("enablePrimaryFire")
                && matches!(
                    definition.target_kind(),
                    SettingTargetKind::HeroAbility { .. }
                )
        })
        .expect("primary-fire definition");
    let target = SettingTarget::HeroAbility {
        team: Some(TeamId::new("allTeams")),
        hero: HeroId::from(hero_ids::DVA),
        slot: LogicalSlot::from(slots::PRIMARY_FIRE),
        variant: None,
    };
    assert_eq!(
        primary.applicability(&target).expect("applicability"),
        Applicability::Unknown
    );
    assert!(matches!(
        primary
            .read(program.settings.as_ref().expect("settings"), &target)
            .expect("typed reads preserve unresolved occurrences")
            .authored,
        SettingValue::Boolean(true)
    ));

    let error = primary
        .write(
            program.settings.as_mut().expect("settings"),
            &target,
            SettingValue::Boolean(false),
        )
        .expect_err("unknown applicability must reject writes");
    assert!(matches!(
        error,
        SettingOperationError::ApplicabilityUnknown { .. }
    ));
}

#[test]
fn typed_settings_source_edit_replaces_only_the_existing_value_bytes() {
    let _catalog = Catalog::builtin().expect("catalog");
    let source = "// 保留这条注释\nsettings {\n    lobby {\n        Max Spectators: 2 // and this one\n    }\n}";
    let program = internal::parse(source);
    let settings = program.settings.as_ref().expect("settings");
    let definition = definitions_by_id(&SettingId::from("setting.lobby.spectatorSlots"))
        .next()
        .expect("spectator-slot definition");

    let edit = definition
        .source_edit(
            source,
            settings,
            "en-US",
            &SettingTarget::Global,
            SettingValue::Number(4.0),
        )
        .expect("editable source occurrence");
    assert_eq!(&source[edit.range()], "2");

    let edited = edit.apply(source).expect("apply exact-source edit");
    assert_eq!(
        edited,
        "// 保留这条注释\nsettings {\n    lobby {\n        Max Spectators: 4 // and this one\n    }\n}"
    );
    let reparsed = internal::parse(&edited);
    assert_eq!(
        definition
            .read(
                reparsed.settings.as_ref().expect("settings"),
                &SettingTarget::Global,
            )
            .expect("read edited value")
            .authored,
        SettingValue::Number(4.0)
    );
    assert!(matches!(
        edit.apply(&source.replacen("2", "3", 1)),
        Err(SettingOperationError::SourceMismatch)
    ));
}

#[test]
fn typed_settings_source_edit_rejects_missing_source_location() {
    let definition = definitions_by_id(&SettingId::from("setting.lobby.spectatorSlots"))
        .next()
        .expect("spectator-slot definition");
    let settings = workshop_rs::settings::Settings {
        span: None,
        children: vec![workshop_rs::settings::SettingsNode::Group {
            name: "lobby".to_string(),
            children: vec![workshop_rs::settings::SettingsNode::Number {
                name: "spectatorSlots".to_string(),
                value: 2.0,
                span: None,
            }],
            span: None,
        }],
    };

    let error = definition
        .source_edit(
            "settings { lobby { Max Spectators: 2 } }",
            &settings,
            "en-US",
            &SettingTarget::Global,
            SettingValue::Number(4.0),
        )
        .expect_err("source edits require a source location");
    assert!(matches!(
        error,
        SettingOperationError::SourceUnavailable { .. }
    ));
}

#[test]
fn typed_settings_source_edit_uses_settings_string_escaping() {
    let catalog = Catalog::builtin().expect("catalog");
    let source = "settings { main { Description: \"old\" } }";
    let program = internal::parse(source);
    let definition = definitions_by_id(&SettingId::from("setting.main.description"))
        .next()
        .expect("description definition");
    let value = "line one\nline\\two\t\"quoted\"\r".to_string();
    let edit = definition
        .source_edit(
            source,
            program.settings.as_ref().expect("settings"),
            "en-US",
            &SettingTarget::Global,
            SettingValue::String(value.clone()),
        )
        .expect("editable string occurrence");
    assert_eq!(
        edit.replacement(),
        "\"line one\\nline\\\\two\\t\\\"quoted\\\"\\r\""
    );

    let reparsed = parser::parse_wir(
        &edit.apply(source).expect("apply source edit"),
        &catalog,
        &en(),
    )
    .expect("reparse escaped string");
    assert_eq!(
        definition
            .read(
                reparsed.settings.as_ref().expect("settings"),
                &SettingTarget::Global,
            )
            .expect("read escaped string")
            .authored,
        SettingValue::String(value)
    );
}

#[test]
fn typed_settings_source_edit_rejects_false_boolean_enum_values() {
    let _catalog = Catalog::builtin().expect("catalog");
    let source = "settings { lobby { Match Voice Chat: Enabled } }";
    let program = internal::parse(source);
    let definition = definitions_by_id(&SettingId::from("setting.lobby.enableMatchVoiceChat"))
        .next()
        .expect("match voice chat definition");

    let edit = definition
        .source_edit(
            source,
            program.settings.as_ref().expect("settings"),
            "en-US",
            &SettingTarget::Global,
            SettingValue::Boolean(true),
        )
        .expect("enabled enum member is writable");
    assert_eq!(edit.replacement(), "Enabled");
    assert!(matches!(
        definition.source_edit(
            source,
            program.settings.as_ref().expect("settings"),
            "en-US",
            &SettingTarget::Global,
            SettingValue::Boolean(false),
        ),
        Err(SettingOperationError::InvalidValue { .. })
    ));
}

#[test]
fn typed_settings_errors_reject_invalid_members_and_non_applicable_targets() {
    let catalog = Catalog::builtin().expect("catalog");
    let mut program = parser::parse_wir(
        "settings { modes { Assault { Limit Roles: 2 Of Each Role Per Team } } }",
        &catalog,
        &en(),
    )
    .expect("parse");
    let role_limit = definitions_by_id(&SettingId::from("setting.gameMode.roleLimit"))
        .find(|definition| definition.path().ends_with("assault.roleLimit"))
        .expect("role-limit definition");
    let error = role_limit
        .write(
            program.settings.as_mut().expect("settings"),
            &SettingTarget::Mode("assault".to_string()),
            SettingValue::Enum("unknownRoleLimit".to_string()),
        )
        .expect_err("unknown enum member must be rejected");
    assert!(matches!(
        error,
        SettingOperationError::InvalidValue { span: Some(_), .. }
    ));

    let ability1_enemy_kb = definitions()
        .find(|definition| definition.path().ends_with("ability1EnemyKb%"))
        .expect("ability 1 enemy knockback setting");
    let error = ability1_enemy_kb
        .write(
            program.settings.as_mut().expect("settings"),
            &SettingTarget::HeroAbility {
                team: None,
                hero: HeroId::from(hero_ids::ANA),
                slot: LogicalSlot::from(slots::ABILITY_1),
                variant: None,
            },
            SettingValue::Percent(10.0),
        )
        .expect_err("uncertain hero setting must be rejected for writes");
    assert!(matches!(
        error,
        SettingOperationError::ApplicabilityUnknown { .. }
    ));
}

#[test]
fn typed_settings_writes_fail_closed_for_unknown_applicability() {
    let health = definitions()
        .find(|definition| {
            definition.path().ends_with("health%")
                && definition.target_kind() == SettingTargetKind::Hero
        })
        .expect("hero health definition");
    let mut settings = workshop_rs::settings::Settings {
        span: None,
        children: Vec::new(),
    };
    let unknown_error = health
        .write(
            &mut settings,
            &SettingTarget::Hero {
                team: None,
                hero: HeroId::new("futureHero"),
            },
            SettingValue::Percent(100.0),
        )
        .expect_err("unknown applicability must refuse writes");
    assert!(matches!(
        unknown_error,
        SettingOperationError::ApplicabilityUnknown { .. }
    ));

    let team_definition = definitions()
        .find(|definition| definition.target_kind() == SettingTargetKind::Team)
        .expect("team definition");
    let widening_error = team_definition
        .write(
            &mut settings,
            &SettingTarget::Hero {
                team: None,
                hero: HeroId::from(hero_ids::ANA),
            },
            SettingValue::Boolean(false),
        )
        .expect_err("team-to-hero applicability without sources must refuse writes");
    assert!(matches!(
        widening_error,
        SettingOperationError::ApplicabilityUnknown { .. }
    ));
}

#[test]
fn declared_modes_inherit_general_settings_entries() {
    use workshop_rs::settings::{Settings, SettingsListElement, SettingsNode, check_emission};

    let bool_node = |name: &str, value: bool| SettingsNode::Bool {
        name: name.to_string(),
        value,
        span: None,
    };
    let list_node = |name: &str, elements: &[&str]| SettingsNode::List {
        name: name.to_string(),
        elements: elements
            .iter()
            .map(|value| SettingsListElement {
                value: value.to_string(),
                span: None,
            })
            .collect(),
        span: None,
    };
    let group = |name: &str, children: Vec<SettingsNode>| SettingsNode::Group {
        name: name.to_string(),
        children,
        span: None,
    };

    // The opy-rs #411 reproducer shape: `enableKillCam` is a `general` leaf
    // that every declared mode inherits.
    let settings = Settings {
        span: None,
        children: vec![
            group(
                "main",
                vec![SettingsNode::String {
                    name: "description".to_string(),
                    value: "t".to_string(),
                    span: None,
                }],
            ),
            group(
                "gamemodes",
                vec![group(
                    "ffa",
                    vec![
                        list_node("enabledMaps", &["workshopIsland"]),
                        bool_node("enableKillCam", false),
                    ],
                )],
            ),
        ],
    };
    assert!(check_emission(&settings).is_empty());
    // The inherited path resolves to the same canonical definition as the
    // `general` leaf.
    let inherited = workshop_rs::settings::definition(&[
        workshop_rs::settings::PathPart::Part("gamemodes"),
        workshop_rs::settings::PathPart::Part("ffa"),
        workshop_rs::settings::PathPart::Part("enableKillCam"),
    ])
    .expect("inherited general setting resolves");
    assert_eq!(
        inherited.id(),
        Some(&SettingId::new("setting.gameMode.enableKillCam"))
    );
    let mut program = workshop_rs::Program::new();
    program.settings = Some(settings);
    let emitted = emitter::emit(&program, &catalog(), &en()).expect("emits settings");
    let deathmatch = emitted.find("Deathmatch").expect("Deathmatch mode block");
    let kill_cam = emitted.find("Kill Cam: Off").expect("Kill Cam: Off");
    assert!(deathmatch < kill_cam, "{emitted}");

    // `elimination` inherits only the reviewed key subset: `enableKillCam`
    // resolves, a non-inherited `general` leaf does not.
    let settings = Settings {
        span: None,
        children: vec![group(
            "gamemodes",
            vec![
                group("elimination", vec![bool_node("enableKillCam", true)]),
                group(
                    "elimination",
                    vec![SettingsNode::String {
                        name: "heroLimit".to_string(),
                        value: "off".to_string(),
                        span: None,
                    }],
                ),
            ],
        )],
    };
    let errors = check_emission(&settings);
    assert_eq!(errors.len(), 1);
    assert!(
        errors[0]
            .to_string()
            .contains("gamemodes.elimination.heroLimit"),
        "{errors:?}"
    );

    // An undeclared mode slot keeps every member outside the emission table.
    let settings = Settings {
        span: None,
        children: vec![group(
            "gamemodes",
            vec![group("notAMode", vec![bool_node("enableKillCam", false)])],
        )],
    };
    let errors = check_emission(&settings);
    assert_eq!(errors.len(), 1);
    assert!(
        errors[0]
            .to_string()
            .contains("gamemodes.notAMode.enableKillCam"),
        "{errors:?}"
    );
}

#[test]
fn check_emission_mirrors_emitter_acceptance() {
    use workshop_rs::settings::{Settings, SettingsNode, check_emission};

    let group = |name: &str, children: Vec<SettingsNode>| SettingsNode::Group {
        name: name.to_string(),
        children,
        span: None,
    };
    let settings = Settings {
        span: None,
        children: vec![
            group(
                "gamemodes",
                vec![group(
                    "ffa",
                    vec![
                        SettingsNode::Bool {
                            name: "enabled".to_string(),
                            value: false,
                            span: None,
                        },
                        SettingsNode::Bool {
                            name: "bogusKey".to_string(),
                            value: true,
                            span: None,
                        },
                    ],
                )],
            ),
            group(
                "extensions",
                vec![SettingsNode::Number {
                    name: "beamEffects".to_string(),
                    value: 3.0,
                    span: None,
                }],
            ),
        ],
    };
    let errors = check_emission(&settings);
    let mut program = workshop_rs::Program::new();
    program.settings = Some(settings);
    let emit_error = emitter::emit(&program, &catalog(), &en())
        .expect_err("the first offending member fails emission");
    assert_eq!(errors.len(), 2, "{errors:?}");
    assert_eq!(errors[0].to_string(), emit_error.to_string());
    assert!(
        errors[1]
            .to_string()
            .contains("does not match its table kind")
    );
}

/// workshop-rs#360 reproducer: the raw Workshop settings block where only the
/// marked key and map spelling differ between the three reported cases.
fn issue_360_source(key: &str, map: &str) -> String {
    format!(
        r#"settings
{{
    main
    {{
        Description: "map spelling"
    }}

    modes
    {{
        Deathmatch
        {{
            {key}
            {{
                {map}
            }}
        }}
    }}
}}

rule("r")
{{
    event
    {{
        Ongoing - Global;
    }}

    actions
    {{
        Wait(1, Ignore Condition);
    }}
}}
"#
    )
}

/// `check` (parse + `Program::validate`) and `compile` (`emitter::emit`) must
/// agree on settings blocks: same failure, same message, same span.
fn assert_check_compile_parity(source: &str) {
    let program = parser::parse(source, &catalog(), &en()).expect("source parses");
    let check = program.validate();
    let compile = emitter::emit(&program, &catalog(), &en());
    match (check, compile) {
        (Ok(()), Ok(_)) => {}
        (Err(check_error), Err(compile_error)) => {
            assert_eq!(check_error.to_string(), compile_error.to_string());
            assert_eq!(check_error.span(), compile_error.span());
        }
        (check, compile) => panic!("check/compile disagree: check={check:?} compile={compile:?}"),
    }
}

#[test]
fn issue_360_exact_spellings_pass_check_and_compile() {
    assert_check_compile_parity(&issue_360_source("enabled maps", "Château Guillard"));
}

#[test]
fn issue_360_settings_names_match_case_insensitively() {
    // The client accepts case variants (Deltinteger/OverPy decompilers match
    // case-insensitively): `Enabled Maps` must not be rejected or warned.
    for (key, map) in [
        ("Enabled Maps", "Château Guillard"),
        ("ENABLED MAPS", "CHÂTEAU GUILLARD"),
        ("enabled maps", "château guillard"),
    ] {
        assert_check_compile_parity(&issue_360_source(key, map));
    }
}

#[test]
fn issue_360_accent_stripped_map_is_rejected_with_suggestion() {
    let source = issue_360_source("enabled maps", "Chateau Guillard");
    let program = parser::parse(&source, &catalog(), &en()).expect("source parses");
    let error = program.validate().expect_err("check rejects the map");
    assert!(
        error
            .to_string()
            .contains("unknown map 'Chateau Guillard' in settings list 'enabledMaps'"),
        "{error}"
    );
    assert!(
        error
            .to_string()
            .contains("did you mean 'Château Guillard'?"),
        "{error}"
    );
    assert!(error.span().is_some());
    // The structured diagnostic carries the suggestion so a caller can
    // apply it without parsing the message.
    let diagnostics = program.settings_diagnostics();
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].suggestion.as_deref(),
        Some("Château Guillard")
    );
    assert_check_compile_parity(&source);
}

#[test]
fn issue_360_distant_and_ambiguous_spellings_suggest_nothing() {
    // Distant spelling: no candidate is close.
    let source = issue_360_source("enabled maps", "Zzyzx Wonderland");
    let program = parser::parse(&source, &catalog(), &en()).expect("source parses");
    let error = program.validate().expect_err("check rejects the map");
    assert!(!error.to_string().contains("did you mean"), "{error}");
    assert!(
        program
            .settings_diagnostics()
            .iter()
            .all(|diagnostic| diagnostic.suggestion.is_none())
    );

    // Ambiguous spelling: `Team 1` and `Team 2` tie on `Team X` in the
    // heroes namespace, so no suggestion is emitted.
    let ambiguous = r#"settings
{
    heroes
    {
        Team X
        {
            Ana
            {
            }
        }
    }
}
rule("r")
{
    event
    {
        Ongoing - Global;
    }
    actions
    {
        Wait(1, Ignore Condition);
    }
}
"#;
    match parser::parse(ambiguous, &catalog(), &en()) {
        Err(error) => {
            assert!(!error.to_string().contains("did you mean"), "{error}");
        }
        Ok(program) => {
            let diagnostics = program.settings_diagnostics();
            assert!(!diagnostics.is_empty(), "an unknown team is rejected");
            assert!(
                diagnostics
                    .iter()
                    .all(|diagnostic| diagnostic.suggestion.is_none()),
                "{diagnostics:?}"
            );
        }
    }
}

#[test]
fn issue_360_unknown_braced_key_suggests_canonical_spelling() {
    // `Enabled Mpas` does not resolve, so the braced member is an opaque
    // group that emission rejects; the diagnostic names `enabled maps`.
    let source = issue_360_source("Enabled Mpas", "Château Guillard");
    let program = parser::parse(&source, &catalog(), &en()).expect("source parses");
    let error = program.validate().expect_err("check rejects the key");
    assert!(
        error.to_string().contains("outside the emission table"),
        "{error}"
    );
    assert!(
        error.to_string().contains("did you mean 'enabled maps'?"),
        "{error}"
    );
    let diagnostics = program.settings_diagnostics();
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].suggestion.as_deref(), Some("enabled maps"));
    assert_check_compile_parity(&source);
}

#[test]
fn issue_360_opaque_settings_remain_verbatim() {
    // Unrecognized leaf members and `settings.workshop` payloads stay
    // project-defined: both check and compile accept them unchanged.
    let source = r#"settings
{
    main
    {
        Project Custom Field: 3
    }

    workshop
    {
        Custom Payload: anything goes
    }
}
rule("r")
{
    event
    {
        Ongoing - Global;
    }
    actions
    {
        Wait(1, Ignore Condition);
    }
}
"#;
    assert_check_compile_parity(source);
}

#[test]
fn typed_member_diagnostics_preserve_messages_spans_suggestions_and_order() {
    use workshop_rs::WorkshopError;
    use workshop_rs::settings::{Settings, SettingsListElement, SettingsNode, check_emission};
    use workshop_rs::source::{Position, SourceFile, Span};

    let mut program = workshop_rs::Program::new();
    let file = program.add_file(SourceFile::new("settings.ws"));
    let span = |line| {
        Some(Span::new(
            file,
            Position::new(line, 1),
            Position::new(line, 10),
        ))
    };
    let group = |name: &str, children| SettingsNode::Group {
        name: name.into(),
        children,
        span: None,
    };
    let list = |name: &str, elements: &[(&str, u32)]| SettingsNode::List {
        name: name.into(),
        span: span(10),
        elements: elements
            .iter()
            .map(|(value, line)| SettingsListElement {
                value: (*value).into(),
                span: span(*line),
            })
            .collect(),
    };
    let groups = vec![
        group(
            "lobby",
            vec![SettingsNode::String {
                name: "mapRotation".into(),
                value: "After A Gmae".into(),
                span: span(1),
            }],
        ),
        group(
            "lobby",
            vec![SettingsNode::Bool {
                name: "enableMatchVoiceChat".into(),
                value: false,
                span: span(2),
            }],
        ),
        group(
            "gamemodes",
            vec![group(
                "ffa",
                vec![list(
                    "enabledMaps",
                    &[("Chateau Guillard", 3), ("Zzyzx Wonderland", 4)],
                )],
            )],
        ),
        group(
            "heroes",
            vec![group(
                "allTeams",
                vec![list("enabledHeroes", &[("Mercyy", 5), ("Zzyzx Hero", 6)])],
            )],
        ),
        group(
            "extensions",
            vec![SettingsNode::Number {
                name: "beamEffects".into(),
                value: 3.0,
                span: span(7),
            }],
        ),
        group(
            "main",
            vec![SettingsNode::Bool {
                name: "descriptino".into(),
                value: true,
                span: span(8),
            }],
        ),
    ];
    let expected = [
        (
            "unknown value 'After A Gmae' for settings key 'mapRotation' (did you mean 'After A Game'?)",
            Some("After A Game"),
        ),
        (
            "unsupported false value for settings key 'enableMatchVoiceChat'",
            None,
        ),
        (
            "unknown map 'Chateau Guillard' in settings list 'enabledMaps' (did you mean 'Château Guillard'?)",
            Some("Château Guillard"),
        ),
        (
            "unknown map 'Zzyzx Wonderland' in settings list 'enabledMaps'",
            None,
        ),
        (
            "unknown hero 'Mercyy' in settings list 'enabledHeroes' (did you mean 'Mercy'?)",
            Some("Mercy"),
        ),
        (
            "unknown hero 'Zzyzx Hero' in settings list 'enabledHeroes'",
            None,
        ),
        (
            "settings key 'beamEffects' does not match its table kind",
            None,
        ),
        (
            "settings key 'main.descriptino' is outside the emission table (did you mean 'Description'?)",
            Some("Description"),
        ),
    ];
    program.settings = Some(Settings {
        children: groups.clone(),
        span: None,
    });
    let diagnostics = program.settings_diagnostics();
    assert_eq!(diagnostics.len(), expected.len());
    for (index, (diagnostic, (message, suggestion))) in diagnostics.iter().zip(expected).enumerate()
    {
        assert_eq!(
            diagnostic.error,
            WorkshopError::malformed(message, span(index as u32 + 1))
        );
        assert_eq!(diagnostic.suggestion.as_deref(), suggestion);
    }
    assert_eq!(
        check_emission(program.settings.as_ref().unwrap()),
        diagnostics
            .iter()
            .map(|d| d.error.clone())
            .collect::<Vec<_>>()
    );
    assert_eq!(program.validate().unwrap_err(), diagnostics[0].error);
    assert_eq!(
        emitter::emit(&program, &catalog(), &en()).unwrap_err(),
        diagnostics[0].error
    );

    for group in groups {
        program.settings = Some(Settings {
            children: vec![group],
            span: None,
        });
        let first = program.settings_diagnostics().remove(0).error;
        assert_eq!(program.validate().unwrap_err(), first);
        assert_eq!(
            emitter::emit(&program, &catalog(), &en()).unwrap_err(),
            first
        );
    }
}

#[test]
fn typed_member_emission_keeps_raw_extensions_and_mode_headers() {
    use workshop_rs::settings::{Settings, SettingsNode};
    let raw = |name: &str, value: &str| SettingsNode::Raw {
        name: name.into(),
        value: value.into(),
        span: None,
    };
    let group = |name: &str, children| SettingsNode::Group {
        name: name.into(),
        children,
        span: None,
    };
    let mut program = workshop_rs::Program::new();
    program.settings = Some(Settings {
        span: None,
        children: vec![
            group(
                "custom",
                vec![group("nested", vec![raw("Payload", "anything goes")])],
            ),
            SettingsNode::Workshop {
                children: vec![raw("Custom Field", "3")],
                span: None,
            },
            group(
                "extensions",
                vec![SettingsNode::Flag {
                    name: "beamEffects".into(),
                    span: None,
                }],
            ),
            group(
                "gamemodes",
                vec![group(
                    "ffa",
                    vec![SettingsNode::Bool {
                        name: "enabled".into(),
                        value: false,
                        span: None,
                    }],
                )],
            ),
        ],
    });
    program.validate().unwrap();
    assert!(program.settings_diagnostics().is_empty());
    let text = emitter::emit(&program, &catalog(), &en()).unwrap();
    for preserved in [
        "custom {",
        "nested {",
        "Payload: anything goes",
        "Custom Field: 3",
        "Beam Effects",
        "disabled Deathmatch {",
    ] {
        assert!(text.contains(preserved), "{text}");
    }
    assert!(!text.contains("enabled:"), "{text}");
}

#[test]
fn typed_member_emission_resolves_hero_display_before_value_acceptance() {
    use workshop_rs::WorkshopError;
    use workshop_rs::settings::{Settings, SettingsNode};
    let group = |name: &str, children| SettingsNode::Group {
        name: name.into(),
        children,
        span: None,
    };
    let mut program = workshop_rs::Program::new();
    program.settings = Some(Settings {
        span: None,
        children: vec![group(
            "heroes",
            vec![group(
                "allTeams",
                vec![group(
                    "ana",
                    vec![SettingsNode::String {
                        name: "ability3Cooldown%".into(),
                        value: "invalid".into(),
                        span: None,
                    }],
                )],
            )],
        )],
    });
    let rejection = program.validate().unwrap_err();
    assert_eq!(
        rejection,
        WorkshopError::malformed(
            "settings key 'ability3Cooldown%' does not match its table kind",
            None
        )
    );
    assert_eq!(
        emitter::emit(&program, &catalog(), &en()).unwrap_err(),
        rejection
    );
    assert!(
        matches!(emitter::emit(&program, &catalog(), &zh()).unwrap_err(), WorkshopError::MissingMapping { kind: "setting", id, locale } if id == "heroes.<team>.<hero>.ability3Cooldown%" && locale == zh())
    );
}

#[test]
fn typed_member_emission_keeps_missing_locale_mapping_and_fallback_reporting() {
    use workshop_rs::WorkshopError;
    use workshop_rs::catalog::Locale;
    use workshop_rs::settings::{Settings, SettingsNode};
    let mut program = workshop_rs::Program::new();
    program.settings = Some(Settings {
        span: None,
        children: vec![SettingsNode::Group {
            name: "lobby".into(),
            span: None,
            children: vec![SettingsNode::Number {
                name: "team1Slots".into(),
                value: 6.0,
                span: None,
            }],
        }],
    });
    program.validate().unwrap();
    let locale = Locale::new("fr-FR");
    assert!(
        matches!(emitter::emit(&program, &catalog(), &locale).unwrap_err(), WorkshopError::MissingMapping { kind: "setting", id, locale: missing } if id == "lobby.team1Slots" && missing == locale)
    );
    let options = emitter::EmitOptions {
        fallback_locale: Some(en()),
    };
    let output = emitter::emit_with_options(&program, &catalog(), &locale, &options).unwrap();
    assert!(
        output.text.contains("Max Team 1 Players: 6"),
        "{}",
        output.text
    );
    assert_eq!(output.fallback_ids, ["settings"]);
}
