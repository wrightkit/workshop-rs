//! Regression tests for defects found by the adversarial audit. Each test
//! cites its finding id from the audit report.

use super::common::{catalog, en};
use super::internal;
use workshop_rs::parser;
use workshop_rs::settings::{
    SettingId, SettingOperationError, SettingTarget, SettingValue, definitions, definitions_by_id,
};

/// W1: a leading UTF-8 BOM must not be a lex error. The BOM stays inside
/// the source coordinate space — it occupies 1:1, so the `rule` keyword
/// starts at 1:2 rather than 1:1 — and emission is identical to the
/// BOM-less input.
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

/// W2: a backslash in a rule name must re-escape on emission so the name
/// reparses identically.
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
}

/// W3: fractional and out-of-range declaration indexes must be rejected
/// rather than truncated/saturated by `as u32`.
#[test]
fn non_integer_variable_index_is_rejected() {
    assert!(internal::try_parse("variables\n{\n    global:\n        1.9: b\n}").is_err());
    assert!(internal::try_parse("variables\n{\n    global:\n        4294967296: b\n}").is_err());
    assert!(internal::try_parse("variables\n{\n    player:\n        -1: b\n}").is_err());
}

/// W4: a declaration name a reference can never express must be rejected.
#[test]
fn unreferenceable_variable_name_is_rejected() {
    assert!(internal::try_parse("variables\n{\n    global:\n        0: a.b\n}").is_err());
    assert!(internal::try_parse("variables\n{\n    global:\n        0: a % b\n}").is_err());
}

/// W7: a second `settings {}` block must be rejected, not silently
/// overwrite the first.
#[test]
fn duplicate_settings_block_is_rejected() {
    let source = "settings\n{\n    main\n    {\n        Description: \"first\"\n    }\n}\nsettings\n{\n    main\n    {\n        Description: \"second\"\n    }\n}";
    assert!(internal::try_parse(source).is_err());
}

/// W9: opaque settings string values must re-emit decoded control
/// characters in their escaped spellings.
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
}

/// W10: a forward variable reference must reuse the real declaration's
/// index when the declaration arrives later, not keep a phantom slot.
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
}

/// W10 companion: two declarations of the same name must be rejected.
#[test]
fn duplicate_variable_name_is_rejected() {
    assert!(
        internal::try_parse("variables\n{\n    global:\n        0: x\n        1: x\n}").is_err()
    );
}

/// W5: the merged `(player, name)` pair of the indexed player-variable
/// actions splits back into two textual arguments, so the index argument is
/// not a variable-name slot and must keep its `Global.`/`Event Player.`
/// qualification.
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

/// S1: `source_edit` must reject a non-leaf occurrence instead of carving a
/// corrupting range out of the block's interior.
#[test]
fn source_edit_rejects_non_leaf_occurrence() {
    let source = "settings\n{\n    main\n    {\n        Description\n        {\n            Custom: X\n        }\n    }\n}";
    let program = internal::parse(source);
    let definition = definitions_by_id(&SettingId::from("setting.main.description"))
        .next()
        .expect("main.description definition");
    let error = definition
        .source_edit(
            source,
            program.settings.as_ref().expect("settings"),
            "en-US",
            &SettingTarget::Global,
            SettingValue::String("new".to_string()),
        )
        .expect_err("a group occurrence is not a scalar source edit");
    assert!(
        matches!(error, SettingOperationError::InvalidValue { .. }),
        "expected the same leaf gate read/write apply, got {error:?}"
    );
}

/// S3: a mode-inherited `gamemodes.general` key must parse typed; the
/// carried-member fallback must never suggest the authored spelling back.
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

/// S7: `gamemodes.general` definitions apply to inheriting modes — their
/// occurrences live under the mode group, and `elimination` inherits only
/// its reviewed key subset.
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

/// S2: `Wrecking Ball` must canonicalize to `wreckingBall` — the id the
/// generated hero rows and the gameplay catalog key — not the stale
/// `hammond` key that left every hero-scoped member unemittable.
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

/// S4: `gamemodes.general.gamemodeStartTrigger` is one canonical id; the
/// generated-only member `allSlotsFilled` must resolve under the authored
/// `Game Mode Start` label.
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

/// S5/S6: a hero with gameplay data but no generated settings row composes
/// `%1$s` templates with the ability name — never emitting the raw
/// placeholder (which is unlexable) or dropping the template context.
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

/// S8: under `wreckingBall` the zh spelling `工程抓钩冷却时间` is the
/// generated `secondaryFireCooldown%` label — the stale alias must not bind
/// it to `ability1Cooldown%` (whose zh label is `动力铁球冷却时间`).
#[test]
fn wrecking_ball_zh_label_binds_secondary_fire() {
    let zh = workshop_rs::catalog::Locale::new("zh-CN");
    let source = "settings\n{\n    heroes\n    {\n        Team 1\n        {\n            Wrecking Ball\n            {\n                工程抓钩冷却时间: 50%\n            }\n        }\n    }\n}";
    let program = parser::parse(source, &catalog(), &zh).expect("zh source parses");
    let settings = program.settings.as_ref().expect("settings");
    fn group(node: &workshop_rs::settings::SettingsNode) -> &[workshop_rs::settings::SettingsNode] {
        match node {
            workshop_rs::settings::SettingsNode::Group { children, .. }
            | workshop_rs::settings::SettingsNode::RawGroup { children, .. } => children,
            other => panic!("expected a group, got {other:?}"),
        }
    }
    let leaf = &group(&group(&group(&settings.children[0])[0])[0])[0];
    assert!(
        matches!(leaf, workshop_rs::settings::SettingsNode::Number { name, .. } if name == "secondaryFireCooldown%"),
        "the zh spelling must bind secondaryFireCooldown%, got {leaf:?}"
    );
}

/// The `heroes.general` literal group is a reviewed settings scope — it must
/// project its own leaf definition alongside the `<team>` wildcard.
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

/// S11: duplicate generated rows for a `(hero, key)` pair carry distinct
/// reviewed spellings — the second row's zh spelling must still parse
/// typed rather than demoting to `Raw`.
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

/// F1: localized control-flow keywords (`Tant que`, `Başka Eğer`, `Si no`,
/// `Son`, `Fin`...) must reparse — emission wrote them but only literal
/// English `End`/`Else If`/`Else` was consumed before.
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

/// F2: the `disabled` modifier carries catalog spellings like tr
/// `devre dışı` and pl `wył.` — multi-word and dotted forms the word-at-a-
/// time parser dropped.
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

/// F3: localized multi-word section spellings — fr `de joueur`, tr
/// `alt yordam` — must resolve to their canonical sections.
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

/// F4: a localized `Event Player` value spelling (fr `Joueur exécutant`,
/// tr `Etkinlik Oyuncusu`) resolves to the `eventPlayer` identity rather
/// than an `eventPlayer()` call.
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

/// F5: localized `Global`/`Event Player` assignment receivers parse through
/// the dedicated variable paths — fr `globale.x = 1` must yield
/// `setGlobalVariable`, not a rejected `AssignMember`.
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

/// F6: structural action spellings resolve with the primary-locale
/// fallback like every other kind — the authored zh `设置全局变量` and an
/// English `Set Global Variable` under a zh parse both bind
/// `setGlobalVariable`.
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

/// S9: a non-finite `Number`/`Percent` member has no emittable spelling —
/// `check_emission` must reject it rather than emit `inf`/`NaN`.
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
