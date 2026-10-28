use super::common::{catalog, en};
use super::internal;
use workshop_rs::parser;
use workshop_rs::settings::{SettingTarget, SettingValue, definitions};

#[test]
fn utf8_bom_is_accepted() {
    let plain = "rule(\"r\")\n{\n    event\n    {\n        Ongoing - Global;\n    }\n}";
    let source = format!("\u{FEFF}{plain}");
    let program = internal::parse(&source);
    assert_eq!(
        internal::emit(&program),
        internal::emit(&internal::parse(plain)),
        "a BOM-prefixed program emits identically to the BOM-less one"
    );
    let span = program
        .rules
        .iter()
        .next()
        .unwrap()
        .span
        .expect("rule span");
    assert_eq!(
        (span.start.line, span.start.col),
        (1, 2),
        "spans index the authored text including the BOM"
    );
}

#[test]
fn rule_name_backslash_round_trips() {
    let source = "rule(\"a\\\\nb\")\n{\n    event\n    {\n        Ongoing - Global;\n    }\n}";
    let program = internal::parse(source);
    assert_eq!(program.rules.iter().next().unwrap().name, "a\\nb");
    let emitted = internal::emit(&program);
    assert!(
        emitted.contains("\"a\\\\nb\""),
        "backslash must re-escape, got:\n{emitted}"
    );
    internal::assert_emit_round_trip(&program);
}

#[test]
fn non_integer_variable_index_is_rejected() {
    for source in [
        "variables\n{\n    global:\n        1.9: b\n}",
        "variables\n{\n    global:\n        4294967296: b\n}",
    ] {
        let error = internal::try_parse(source).unwrap_err();
        assert!(
            error.to_string().contains("integer index"),
            "the diagnostic must name the index, got: {error}"
        );
    }
    assert!(internal::try_parse("variables\n{\n    player:\n        -1: b\n}").is_err());
}

#[test]
fn unreferenceable_variable_name_is_rejected() {
    for source in [
        "variables\n{\n    global:\n        0: a.b\n}",
        "variables\n{\n    global:\n        0: a % b\n}",
    ] {
        let error = internal::try_parse(source).unwrap_err();
        assert!(
            error.to_string().contains("cannot be referenced"),
            "the diagnostic must name the unreferenceable name, got: {error}"
        );
    }
}

#[test]
fn duplicate_settings_block_is_rejected() {
    let source = "settings\n{\n    main\n    {\n        Description: \"first\"\n    }\n}\nsettings\n{\n    main\n    {\n        Description: \"second\"\n    }\n}";
    let error = internal::try_parse(source).unwrap_err();
    assert!(
        error.to_string().contains("duplicate 'settings' block"),
        "the diagnostic must name the duplicate block, got: {error}"
    );
}

#[test]
fn opaque_settings_string_reescapes_controls() {
    let source =
        "settings\n{\n    extensions\n    {\n        Custom Key: \"a\\nb\\rc\\\\d\"\n    }\n}";
    let program = internal::parse(source);
    let emitted = internal::emit(&program);
    assert!(
        emitted.contains("Custom Key: \"a\\nb\\rc\\\\d\""),
        "control characters must re-escape, got:\n{emitted}"
    );
    internal::assert_emit_round_trip(&program);
}

#[test]
fn forward_reference_binds_to_later_declaration() {
    let source = "rule(\"r\")\n{\n    event\n    {\n        Ongoing - Global;\n    }\n    actions\n    {\n        Set Global Variable(x, 1);\n    }\n}\nvariables\n{\n    global:\n        3: x\n}";
    let program = internal::parse(source);
    let emitted = internal::emit(&program);
    assert!(
        emitted.contains("3: x"),
        "declaration index must be preserved, got:\n{emitted}"
    );
    assert!(
        !emitted.contains("0: x"),
        "no phantom declaration may survive, got:\n{emitted}"
    );
    let reparsed = internal::parse(&emitted);
    let var = reparsed.global_variables.iter().next().unwrap();
    assert_eq!(var.index, 3);
    internal::assert_emit_round_trip(&program);
}

#[test]
fn duplicate_variable_name_is_rejected() {
    assert!(
        internal::try_parse("variables\n{\n    global:\n        0: x\n        1: x\n}").is_err()
    );
}

#[test]
fn player_variable_at_index_keeps_qualified_index() {
    let source = "variables\n{\n    global:\n        0: idx\n    player:\n        0: A\n}\nrule(\"r\")\n{\n    event\n    {\n        Ongoing - Global;\n    }\n    actions\n    {\n        Set Player Variable At Index(Event Player, A, Global.idx, 5);\n        Modify Player Variable At Index(Event Player, A, Global.idx, Add, 2);\n    }\n}";
    let program = internal::parse(source);
    let emitted = internal::emit(&program);
    assert!(
        emitted.contains("At Index(Event Player, A, Global.idx, 5)"),
        "index argument must keep its Global. qualification, got:\n{emitted}"
    );
    assert!(
        emitted.contains("At Index(Event Player, A, Global.idx, Add, 2)"),
        "index argument must keep its Global. qualification, got:\n{emitted}"
    );
    internal::assert_emit_round_trip(&program);
}

#[test]
fn mode_inherited_setting_resolves_typed() {
    let source = "settings\n{\n    modes\n    {\n        Skirmish\n        {\n            Kill Cam: Off\n        }\n    }\n}";
    let program = parser::parse(source, &catalog(), &en()).expect("source parses");
    let diagnostics = program.settings_diagnostics();
    assert!(
        diagnostics.is_empty(),
        "inherited key must not warn, got {diagnostics:?}"
    );
    let emitted = internal::emit(&internal::parse(source));
    assert!(
        emitted.contains("Kill Cam: Off"),
        "inherited key must emit its declared spelling, got:\n{emitted}"
    );
}

#[test]
fn general_setting_definition_applies_to_inheriting_modes() {
    let source = "settings\n{\n    modes\n    {\n        Skirmish\n        {\n            Kill Cam: Off\n        }\n    }\n}";
    let program = internal::parse(source);
    let kill_cam = definitions()
        .find(|definition| definition.path() == "gamemodes.general.enableKillCam")
        .expect("enableKillCam definition");
    let skirmish = SettingTarget::Mode("skirmish".to_string());
    let elimination = SettingTarget::Mode("elimination".to_string());
    assert_eq!(
        kill_cam
            .applicability(&skirmish)
            .expect("applicability resolves"),
        workshop_rs::settings::Applicability::Applicable
    );
    assert_eq!(
        kill_cam
            .applicability(&elimination)
            .expect("applicability resolves"),
        workshop_rs::settings::Applicability::Applicable,
        "enableKillCam is in the reviewed elimination copy set"
    );
    let respawn = definitions()
        .find(|definition| definition.path() == "gamemodes.general.respawnTime%")
        .expect("respawnTime% definition");
    assert_eq!(
        respawn
            .applicability(&elimination)
            .expect("applicability resolves"),
        workshop_rs::settings::Applicability::NotApplicable,
        "respawnTime% is not in the reviewed elimination copy set"
    );
    let occurrence = kill_cam
        .read(program.settings.as_ref().expect("settings"), &skirmish)
        .expect("the skirmish occurrence reads");
    assert_eq!(occurrence.authored, SettingValue::Boolean(false));
}

#[test]
fn wrecking_ball_hero_settings_emit() {
    let source = "settings\n{\n    heroes\n    {\n        Team 1\n        {\n            Wrecking Ball\n            {\n                ultGen%: 50%\n            }\n        }\n    }\n}";
    let program = internal::parse(source);
    let emitted = internal::emit(&program);
    assert!(
        emitted.contains("Minefield"),
        "the row's reviewed label must emit, got:\n{emitted}"
    );
    internal::assert_emit_round_trip(&program);
}

#[test]
fn game_mode_start_all_slots_filled_resolves() {
    let source = "settings\n{\n    modes\n    {\n        General\n        {\n            Game Mode Start: All Slots Filled\n        }\n    }\n}";
    let program = parser::parse(source, &catalog(), &en()).expect("source parses");
    assert!(
        program.settings_diagnostics().is_empty(),
        "declared enum member must not warn: {:?}",
        program.settings_diagnostics()
    );
    let emitted = internal::emit(&internal::parse(source));
    assert!(
        emitted.contains("Game Mode Start: All Slots Filled"),
        "member must keep its declared spelling, got:\n{emitted}"
    );
}

#[test]
fn rowless_hero_composes_templated_labels() {
    let source = "settings\n{\n    heroes\n    {\n        Team 1\n        {\n            Doctrine\n            {\n                enableSecondaryFire: Off\n                ability1Cooldown%: 50%\n            }\n        }\n    }\n}";
    let program = internal::parse(source);
    let emitted = internal::emit(&program);
    assert!(
        !emitted.contains('$'),
        "no raw %1$s placeholder may reach emission, got:\n{emitted}"
    );
    assert!(
        emitted.contains("Infuse"),
        "the secondary-fire ability name must fill the template, got:\n{emitted}"
    );
    assert!(
        emitted.contains("Invigorating Drones Cooldown Time"),
        "the template context must be emitted, got:\n{emitted}"
    );
    internal::assert_emit_round_trip(&program);
}

#[test]
fn wrecking_ball_zh_labels_bind_their_canonical_keys() {
    let zh = workshop_rs::catalog::Locale::new("zh-CN");
    let source = "settings\n{\n    heroes\n    {\n        Team 1\n        {\n            Wrecking Ball\n            {\n                工程抓钩冷却时间: 50%\n                感应护盾冷却时间: 80%\n                重力坠击冷却时间: 75%\n            }\n        }\n    }\n}";
    let program = parser::parse(source, &catalog(), &zh).expect("zh source parses");
    let settings = program.settings.as_ref().expect("settings");
    fn group(node: &workshop_rs::settings::SettingsNode) -> &[workshop_rs::settings::SettingsNode] {
        match node {
            workshop_rs::settings::SettingsNode::Group { children, .. }
            | workshop_rs::settings::SettingsNode::RawGroup { children, .. } => children,
            other => panic!("expected a group, got {other:?}"),
        }
    }
    let leaves = group(&group(&group(&settings.children[0])[0])[0]);
    for (leaf, expected) in leaves.iter().zip([
        "secondaryFireCooldown%",
        "ability2Cooldown%",
        "ability3Cooldown%",
    ]) {
        assert!(
            matches!(leaf, workshop_rs::settings::SettingsNode::Number { name, .. } if name == expected),
            "the zh spelling must bind {expected}, got {leaf:?}"
        );
    }
}

#[test]
fn general_hero_scope_stays_addressable() {
    assert!(
        definitions().any(|definition| definition.path() == "heroes.general.disabledHeroes"),
        "the literal general group's leaf must project a definition"
    );
    assert!(
        definitions().any(|definition| definition.path() == "heroes.<team>.disabledHeroes"),
        "the reachable wildcard entry still projects"
    );
}

#[test]
fn duplicate_generated_spellings_accepted() {
    let zh = workshop_rs::catalog::Locale::new("zh-CN");
    let source = "settings\n{\n    heroes\n    {\n        Team 1\n        {\n            Hanzo\n            {\n                跃冷却时间: 50%\n            }\n        }\n    }\n}";
    let program = parser::parse(source, &catalog(), &zh).expect("zh source parses");
    assert!(
        program.settings_diagnostics().is_empty(),
        "the second reviewed spelling must bind typed: {:?}",
        program.settings_diagnostics()
    );
}

#[test]
fn localized_control_flow_reparses() {
    let source = "variables\n{\n    global:\n        0: x\n}\nrule(\"r\")\n{\n    event\n    {\n        Ongoing - Global;\n    }\n    actions\n    {\n        If(Global.x == 1);\n            Set Global Variable(x, 2);\n        Else If(Global.x == 2);\n            Set Global Variable(x, 3);\n        Else;\n            Set Global Variable(x, 4);\n        End;\n        While(Global.x < 5);\n            Set Global Variable(x, 0);\n        End;\n    }\n}";
    let program = internal::parse(source);
    for tag in [
        "fr-fr", "tr-tr", "es-mx", "ja-jp", "de-de", "pl-pl", "zh-cn",
    ] {
        let locale = super::common::locale(tag);
        let emitted = internal::emit_in(&program, &locale);
        internal::assert_reparse_equivalent_in(&program, &emitted, &locale);
    }
}

#[test]
fn localized_disabled_modifier_reparses() {
    let source = "variables\n{\n    global:\n        0: x\n}\nrule(\"r\")\n{\n    event\n    {\n        Ongoing - Global;\n    }\n    conditions\n    {\n        disabled Global.x == 0;\n    }\n    actions\n    {\n        disabled Small Message(Event Player, Custom String(\"hi\"));\n    }\n}";
    let program = internal::parse(source);
    for tag in ["tr-tr", "pl-pl", "fr-fr", "zh-cn"] {
        let locale = super::common::locale(tag);
        let emitted = internal::emit_in(&program, &locale);
        internal::assert_reparse_equivalent_in(&program, &emitted, &locale);
    }
}

#[test]
fn localized_section_headers_reparse() {
    let source = "variables\n{\n    player:\n        0: score\n}\nsubroutines\n{\n    0: init\n}\nrule(\"r\")\n{\n    event\n    {\n        Ongoing - Global;\n    }\n}";
    let program = internal::parse(source);
    for tag in ["fr-fr", "tr-tr"] {
        let locale = super::common::locale(tag);
        let emitted = internal::emit_in(&program, &locale);
        internal::assert_reparse_equivalent_in(&program, &emitted, &locale);
    }
}

#[test]
fn localized_event_player_value_reparses() {
    let source = "rule(\"r\")\n{\n    event\n    {\n        Ongoing - Global;\n    }\n    actions\n    {\n        Small Message(Event Player, Custom String(\"hi\"));\n    }\n}";
    let program = internal::parse(source);
    for tag in ["fr-fr", "tr-tr", "ja-jp", "es-mx", "pt-br", "zh-cn"] {
        let locale = super::common::locale(tag);
        let emitted = internal::emit_in(&program, &locale);
        internal::assert_reparse_equivalent_in(&program, &emitted, &locale);
    }
}

#[test]
fn localized_assignment_receivers_reparse() {
    let source = "variables\n{\n    global:\n        0: x\n    player:\n        0: score\n}\nrule(\"r\")\n{\n    event\n    {\n        Ongoing - Global;\n    }\n    actions\n    {\n        Global.x = 1;\n        Event Player.score = 2;\n    }\n}";
    let program = internal::parse(source);
    for tag in ["fr-fr", "tr-tr", "ja-jp", "zh-cn"] {
        let locale = super::common::locale(tag);
        let emitted = internal::emit_in(&program, &locale);
        internal::assert_reparse_equivalent_in(&program, &emitted, &locale);
    }
}

#[test]
fn localized_action_spelling_and_primary_fallback_resolve() {
    let zh = super::common::locale("zh-cn");
    let zh_source = "变量\n{\n    全局:\n        0: x\n}\n规则(\"r\")\n{\n    事件\n    {\n        持续 - 全局;\n    }\n    动作\n    {\n        设置全局变量(x, 1);\n    }\n}";
    let en_source = "variables\n{\n    global:\n        0: x\n}\nrule(\"r\")\n{\n    event\n    {\n        Ongoing - Global;\n    }\n    actions\n    {\n        Set Global Variable(x, 1);\n    }\n}";
    let zh_program = internal::parse_in(zh_source, &zh);
    let en_program = internal::parse(en_source);
    assert!(
        workshop_rs::roundtrip::equivalent_wir(&zh_program, &en_program),
        "the zh-authored action must bind the same canonical identity"
    );
    internal::parse_in(en_source, &zh);
}

#[test]
fn non_finite_setting_number_is_rejected() {
    use workshop_rs::settings::{Settings, SettingsNode, check_emission};
    let settings = Settings {
        span: None,
        children: vec![SettingsNode::Group {
            name: "modes".to_string(),
            children: vec![SettingsNode::Group {
                name: "General".to_string(),
                children: vec![SettingsNode::Number {
                    name: "respawnTime%".to_string(),
                    value: f64::NAN,
                    span: None,
                }],
                span: None,
            }],
            span: None,
        }],
    };
    let errors = check_emission(&settings);
    assert!(
        !errors.is_empty(),
        "NaN must be rejected before emission spells it"
    );
}
