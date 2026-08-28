//! End-to-end raw Workshop settings coverage using the reviewed en-US and
//! zh-CN settings mappings.

use std::path::{Path, PathBuf};

use workshop_rs::catalog::{Catalog, Locale};
use workshop_rs::gameplay::{AbilityVariant, HeroId, LogicalSlot, hero_ids, slots};
use workshop_rs::settings::{
    Applicability, NumericBounds, SettingEvidenceKind, SettingId, SettingIdentity,
    SettingOperationError, SettingScope, SettingTarget, SettingTargetKind, SettingValue,
    SettingValueDomain, TeamId, definitions, definitions_by_id,
};
use workshop_rs::{convert, emitter, parser, roundtrip, semantic};

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
    let locale = Locale::new("en-US");
    let program = parser::parse(source, &catalog, &locale).expect("custom settings parse");
    assert!(
        semantic::inspect(&program, &catalog).is_empty(),
        "issues: {:?}, settings: {:?}",
        semantic::inspect(&program, &catalog),
        program.settings
    );
    let emitted = emitter::emit(&program, &catalog, &locale).expect("custom settings emit");
    let reparsed = parser::parse(&emitted, &catalog, &locale).expect("custom settings reparse");
    assert!(roundtrip::equivalent(&program, &reparsed));
    assert!(emitted.contains("Custom Label: \"Keep this\""));
    assert!(emitted.contains("Custom Number: 42"));
}

#[test]
fn localized_workshop_namespace_is_known() {
    assert_eq!(
        workshop_rs::settings::table::localized_name("zh-CN", "namespaces", "workshop"),
        Some("地图工坊")
    );
    let source = "settings { 地图工坊 { 自定义: 1 } }";
    let catalog = Catalog::builtin().expect("catalog");
    let program = parser::parse(source, &catalog, &Locale::new("zh-CN")).expect("parse");
    assert!(
        semantic::inspect(&program, &catalog).is_empty(),
        "issues: {:?}, settings: {:?}",
        semantic::inspect(&program, &catalog),
        program.settings
    );
}

#[test]
fn localized_wrecking_ball_settings_aliases_are_known() {
    let source = "settings { heroes { 综合 { 破坏球 {\n工程抓钩冷却时间: 80%\n感应护盾冷却时间: 80%\n重力坠击冷却时间: 75%\n} } } }";
    let catalog = Catalog::builtin().expect("catalog");
    let program = parser::parse(source, &catalog, &Locale::new("zh-CN")).expect("parse");
    assert!(
        semantic::inspect(&program, &catalog).is_empty(),
        "issues: {:?}, settings: {:?}",
        semantic::inspect(&program, &catalog),
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
    assert!(hero_ability.provenance().reviewed);
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
fn settings_schema_distinguishes_applicability_and_unknown_hero_evidence() {
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

    let ashe_only = definitions
        .iter()
        .find(|definition| definition.path().ends_with("ability1EnemyKb%"))
        .expect("Ashe-only ability setting");
    let ana_ability1 = SettingTarget::HeroAbility {
        team: None,
        hero: HeroId::from(hero_ids::ANA),
        slot: LogicalSlot::from(slots::ABILITY_1),
        variant: None,
    };
    assert_eq!(
        ashe_only
            .applicability(&ana_ability1)
            .expect("applicability"),
        Applicability::NotApplicable
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
        Applicability::Applicable
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
        assert!(definition.provenance().reviewed);
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
        Applicability::Applicable
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
        Applicability::Applicable
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
        Applicability::Applicable
    );
}

#[test]
fn settings_schema_preserves_locale_and_evidence_provenance() {
    let definitions: Vec<_> = definitions().collect();
    let main = definitions
        .iter()
        .find(|definition| definition.path() == "main.description")
        .expect("main definition");
    assert_eq!(
        main.presentation().localized_name("en-US"),
        Some("Description")
    );
    assert_eq!(
        main.provenance().kind,
        SettingEvidenceKind::RawWorkshopFixture
    );

    let generated = definitions
        .iter()
        .find(|definition| definition.path() == "extensions.beamEffects")
        .expect("generated definition");
    assert_eq!(
        generated.provenance().kind,
        SettingEvidenceKind::WorkshopDataExport
    );
}

#[test]
fn settings_schema_catalog_is_complete_and_conflict_checked() {
    workshop_rs::settings::schema::validate_catalog().expect("reviewed settings catalog");
}

#[test]
fn typed_settings_read_and_write_preserve_unrelated_structure() {
    let catalog = Catalog::builtin().expect("catalog");
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
    let mut program = parser::parse(source, &catalog, &Locale::new("en-US")).expect("parse");
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
    let emitted = emitter::emit(&program, &catalog, &Locale::new("en-US")).expect("emit");
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
    primary
        .write(
            program.settings.as_mut().expect("settings"),
            &target,
            SettingValue::Boolean(false),
        )
        .expect("hero typed write");
    assert!(matches!(
        primary
            .read(program.settings.as_ref().expect("settings"), &target)
            .expect("hero typed read")
            .authored,
        SettingValue::Boolean(false)
    ));

    let error = primary
        .write(
            program.settings.as_mut().expect("settings"),
            &target,
            SettingValue::Number(1.0),
        )
        .expect_err("wrong kind must be rejected");
    assert!(matches!(
        error,
        SettingOperationError::WrongValueKind { span: Some(_), .. }
    ));
}

#[test]
fn typed_settings_errors_reject_invalid_members_and_non_applicable_targets() {
    let catalog = Catalog::builtin().expect("catalog");
    let mut program = parser::parse(
        "settings { modes { Assault { Limit Roles: 2 Of Each Role Per Team } } }",
        &catalog,
        &Locale::new("en-US"),
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

    let ashe_only = definitions()
        .find(|definition| definition.path().ends_with("ability1EnemyKb%"))
        .expect("Ashe-only setting");
    let error = ashe_only
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
        .expect_err("non-applicable hero setting must be rejected");
    assert!(matches!(error, SettingOperationError::NotApplicable { .. }));
}
