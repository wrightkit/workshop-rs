//! Catalog tests: canonical identities resolve localized spellings to
//! locale-independent ids and back, and catalog validation rejects
//! malformed or colliding data. The primary locale (en-US) is complete;
//! additional declared locales may be partially covered.

use workshop_rs::catalog::{Catalog, Kind, Locale};

fn builtin() -> Catalog {
    Catalog::builtin().expect("built-in catalog validates")
}

fn en() -> Locale {
    Locale::new("en-US")
}

#[test]
fn builtin_catalog_loads_and_declares_the_pinned_workshop_locales() {
    let catalog = builtin();
    assert!(catalog.supports(&en()));
    assert_eq!(
        catalog
            .locales()
            .iter()
            .map(Locale::as_str)
            .collect::<Vec<_>>(),
        vec![
            "en-us", "de-de", "es-es", "es-mx", "fr-fr", "it-it", "ja-jp", "ko-kr", "pl-pl",
            "pt-br", "ru-ru", "th-th", "tr-tr", "zh-cn", "zh-tw",
        ]
    );
    // The primary locale is first and complete.
    assert_eq!(catalog.locales()[0], en());
    assert_eq!(catalog.primary_locale(), &en());
    assert_eq!(
        catalog.locale_coverage(&en()).mapped,
        catalog.locale_coverage(&en()).total
    );
    for locale in catalog.locales().iter().skip(1) {
        let coverage = catalog.locale_coverage(locale);
        assert!(coverage.mapped > 0 && coverage.mapped < coverage.total);
    }
}

#[test]
fn localized_spelling_resolves_to_canonical_id_and_back() {
    let catalog = builtin();

    // Action: "Disable Inspector Recording" -> disableInspector -> spelling.
    let entry = catalog
        .resolve(Kind::Action, &en(), "Disable Inspector Recording")
        .expect("spelling resolves");
    assert_eq!(entry.id, "disableInspector");
    assert_eq!(
        catalog.spelling(Kind::Action, &en(), "disableInspector"),
        Some("Disable Inspector Recording")
    );

    // Value: multi-word "Count Of" -> countOf.
    let entry = catalog
        .resolve(Kind::Value, &en(), "Count Of")
        .expect("count of resolves");
    assert_eq!(entry.id, "countOf");
    assert_eq!(
        catalog.spelling(Kind::Value, &en(), "countOf"),
        Some("Count Of")
    );

    // Structural: "For Global Variable" -> forGlobalVariable.
    let entry = catalog
        .resolve(Kind::Structural, &en(), "For Global Variable")
        .expect("structural resolves");
    assert_eq!(entry.id, "forGlobalVariable");
    assert!(catalog.entry(Kind::Value, "global").is_none());
    assert_eq!(
        catalog.spelling(Kind::Structural, &Locale::new("zh-CN"), "global"),
        Some("全局")
    );
}

#[test]
fn canonical_and_localized_parameter_spellings_resolve_by_position() {
    let catalog = builtin();
    let entry = catalog.entry(Kind::Action, "wait").expect("wait action");
    let en = en();
    let zh = Locale::new("zh-CN");

    assert_eq!(entry.resolve_param(&en, "Duration"), Some(0));
    assert_eq!(entry.resolve_param(&zh, "Duration"), Some(0));
    assert_eq!(entry.resolve_param(&zh, "时间"), Some(0));
    assert_eq!(entry.resolve_param(&zh, "等待行为"), Some(1));
    assert_eq!(entry.resolve_param(&zh, "missing"), None);
}

#[test]
fn localized_string_presets_resolve_and_translate_by_identity() {
    let catalog = builtin();
    let zh = Locale::new("zh-CN");
    let preset = catalog
        .resolve_localized_string(&en(), "Hello")
        .expect("reviewed Hello preset resolves");
    assert_eq!(preset.id, "hello");
    assert_eq!(
        catalog.localized_string_spelling(&zh, "hello"),
        Some("问候")
    );
    assert!(
        catalog
            .resolve_localized_string(&en(), "Not A Preset")
            .is_none()
    );
}

#[test]
fn zh_cn_break_and_abort_spellings_stay_distinct() {
    let catalog = builtin();
    let zh = Locale::new("zh-CN");
    for (spelling, id) in [("中断", "break"), ("跳出循环", "break"), ("中止", "abort")] {
        let entry = catalog
            .resolve(Kind::Action, &zh, spelling)
            .expect("reviewed spelling resolves");
        assert_eq!(entry.id, id, "{spelling}");
    }
    assert_eq!(catalog.spelling(Kind::Action, &zh, "break"), Some("中断"));
    assert_eq!(catalog.spelling(Kind::Action, &zh, "abort"), Some("中止"));
}

#[test]
fn zh_cn_is_firing_secondary_uses_the_pinned_overpy_spelling() {
    let catalog = builtin();
    let zh = Locale::new("zh-CN");
    for spelling in ["正在使用辅助武器", "正在发射辅助攻击"] {
        let entry = catalog
            .resolve(Kind::Value, &zh, spelling)
            .expect("reviewed spelling resolves");
        assert_eq!(entry.id, "isFiringSecondary", "{spelling}");
    }
    assert_eq!(
        catalog.spelling(Kind::Value, &zh, "isFiringSecondary"),
        Some("正在使用辅助武器")
    );
}

#[test]
fn enums_resolve_members_to_canonical_identity() {
    let catalog = builtin();
    assert_eq!(
        catalog.resolve_enum_member("Beam", &en(), "Grapple Beam"),
        Some(("Beam".to_string(), "GRAPPLE".to_string()))
    );
    assert_eq!(
        catalog.enum_spelling("Beam", &en(), "GRAPPLE"),
        Some("Grapple Beam")
    );
    assert_eq!(
        catalog.resolve_enum_member("Color", &en(), "Yellow"),
        Some(("Color".to_string(), "YELLOW".to_string()))
    );
    assert_eq!(
        catalog.resolve_enum_member("Wait", &en(), "Ignore Condition"),
        Some(("Wait".to_string(), "IGNORE_CONDITION".to_string()))
    );
}

#[test]
fn canonical_color_white_spellings_resolve_through_the_catalog_boundary() {
    let catalog = builtin();
    assert_eq!(
        catalog.localized_enum_spelling("Color", &Locale::new("en-US"), "WHITE"),
        Some("White")
    );
    assert_eq!(
        catalog.localized_enum_spelling("Color", &Locale::new("fr-FR"), "WHITE"),
        Some("Blanc")
    );
    assert_eq!(
        catalog.localized_enum_spelling("Color", &Locale::new("zh-TW"), "WHITE"),
        Some("白色")
    );
}

#[test]
fn localized_enum_domains_and_real_project_values_resolve_canonically() {
    let catalog = builtin();
    let zh = Locale::new("zh-CN");
    assert_eq!(catalog.resolve_enum_domain(&zh, "按钮"), Some("Button"));
    assert_eq!(
        catalog.resolve_enum_member("Button", &zh, "技能1"),
        Some(("Button".to_string(), "ABILITY_1".to_string()))
    );
    assert_eq!(
        catalog
            .resolve(Kind::Value, &zh, "射线命中位置")
            .map(|entry| entry.id.as_str()),
        Some("raycastHitPosition")
    );
    assert_eq!(
        catalog
            .resolve(Kind::Value, &zh, "空")
            .map(|entry| entry.id.as_str()),
        Some("null")
    );
}

#[test]
fn unknown_spellings_and_ids_do_not_resolve() {
    let catalog = builtin();
    assert!(
        catalog
            .resolve(Kind::Action, &en(), "Totally Unknown Thing")
            .is_none()
    );
    assert!(catalog.entry(Kind::Value, "noSuchId").is_none());
    assert!(
        catalog
            .resolve_enum_member("Beam", &en(), "Purple Beam")
            .is_none()
    );
}

#[test]
fn locale_normalization_is_case_insensitive() {
    let catalog = builtin();
    let en_upper = Locale::new("EN-US");
    assert_eq!(en_upper, en());
    assert!(catalog.supports(&en_upper));
    assert_eq!(
        catalog.spelling(Kind::Action, &en_upper, "disableInspector"),
        Some("Disable Inspector Recording")
    );
}

#[test]
fn duplicate_aliases_fail_validation() {
    let bad = r#"{
        "schemaVersion": 1,
        "locales": ["en-US"],
        "target": { "game": "g", "format": "f", "surface": "s" },
        "provenance": { "generator": "g", "generatorVersion": "0", "source": "s", "license": "l", "reviewed": true },
        "structural": [
            { "id": "if", "aliases": { "en-US": "If" } },
            { "id": "elseIf", "aliases": { "en-US": "If" } }
        ]
    }"#;
    let error = Catalog::load(bad).expect_err("colliding aliases must fail");
    assert!(error.to_string().contains("duplicate"));
}

#[test]
fn missing_primary_locale_alias_fails_validation() {
    // The primary locale's declared surface must be complete.
    let bad = r#"{
        "schemaVersion": 1,
        "locales": ["en-US"],
        "target": { "game": "g", "format": "f", "surface": "s" },
        "provenance": { "generator": "g", "generatorVersion": "0", "source": "s", "license": "l", "reviewed": true },
        "structural": [
            { "id": "if", "aliases": { "en-US": "If" } }
        ],
        "actions": [
            { "id": "wait", "aliases": {} }
        ]
    }"#;
    let error = Catalog::load(bad).expect_err("missing alias must fail");
    assert!(error.to_string().contains("missing"));
}

#[test]
fn partial_non_primary_locale_coverage_is_allowed() {
    // ADR-0001 Decision 7: additional declared locales may be partially
    // covered; missing mappings fail explicitly at conversion time, not at
    // catalog validation.
    let partial = r#"{
        "schemaVersion": 1,
        "locales": ["en-US", "zh-CN"],
        "target": { "game": "g", "format": "f", "surface": "s" },
        "provenance": { "generator": "g", "generatorVersion": "0", "source": "s", "license": "l", "reviewed": true },
        "actions": [
            { "id": "wait", "aliases": { "en-US": "Wait", "zh-CN": "Synthetic" } },
            { "id": "disableInspector", "aliases": { "en-US": "Disable Inspector Recording" } }
        ]
    }"#;
    let catalog = Catalog::load(partial).expect("partial coverage loads");
    let zh = Locale::new("zh-CN");
    assert_eq!(catalog.locale_coverage(&zh).mapped, 1);
    assert_eq!(catalog.locale_coverage(&zh).total, 2);
    assert_eq!(
        catalog
            .resolve(Kind::Action, &zh, "Synthetic")
            .map(|e| e.id.as_str()),
        Some("wait")
    );
    assert_eq!(
        catalog.spelling(Kind::Action, &zh, "disableInspector"),
        None
    );
}

#[test]
fn undeclared_locale_fails_validation() {
    let bad = r#"{
        "schemaVersion": 1,
        "locales": ["en-US"],
        "target": { "game": "g", "format": "f", "surface": "s" },
        "provenance": { "generator": "g", "generatorVersion": "0", "source": "s", "license": "l", "reviewed": true },
        "structural": [
            { "id": "if", "aliases": { "en-US": "If", "zh-CN": "Synthetic" } }
        ]
    }"#;
    let error = Catalog::load(bad).expect_err("undeclared locale must fail");
    assert!(error.to_string().contains("undeclared locale"));
}

#[test]
fn exercised_builtin_surface_resolves_with_canonical_params_and_spellings() {
    // The canonical param order (named-arg binding, probes P6/P6b) and
    // en-US spellings are catalog-owned.
    let catalog = builtin();

    // Action with a full canonical param list.
    let effect = catalog
        .entry(Kind::Action, "createEffect")
        .expect("createEffect is in the catalog");
    assert_eq!(
        effect.params(),
        vec![
            "VisibleTo",
            "Type",
            "Color",
            "Position",
            "Radius",
            "Reevaluation"
        ]
    );
    assert_eq!(
        catalog.spelling(Kind::Action, &en(), "createEffect"),
        Some("Create Effect")
    );

    // Value with no params.
    let event_player = catalog
        .entry(Kind::Value, "eventPlayer")
        .expect("eventPlayer is in the catalog");
    assert!(event_player.params().is_empty());
    assert_eq!(
        catalog.spelling(Kind::Value, &en(), "eventPlayer"),
        Some("Event Player")
    );

    // A shared canonical identity: `Wait`/`MinWait` both bind to `wait`.
    assert_eq!(
        catalog
            .entry(Kind::Action, "wait")
            .map(|e| e.params().to_vec()),
        Some(vec!["Duration".to_string(), "WaitBehavior".to_string()])
    );
    let crouch = catalog
        .entry(Kind::Action, "setCrouchEnabled")
        .expect("setCrouchEnabled is in the catalog");
    assert_eq!(crouch.param_name(0), Some("player"));
    assert_eq!(crouch.param_name(1), Some("enabled"));

    // The exercised param surface resolves by en-US spelling too.
    assert!(
        catalog
            .resolve(
                Kind::Action,
                &en(),
                "Disable Movement Collision With Environment"
            )
            .is_some()
    );
    assert!(
        catalog
            .resolve(Kind::Value, &en(), "Workshop Setting Combo")
            .is_some()
    );
}

#[test]
fn evidence_backed_signature_types_are_exposed() {
    let catalog = builtin();
    let max_health = catalog
        .entry(Kind::Value, "getMaxHealth")
        .expect("getMaxHealth");
    assert_eq!(max_health.param_type(0), Some("Player"));
    assert_eq!(max_health.return_type(), Some("Number"));
    assert_eq!(
        catalog
            .entry(Kind::Action, "setCrouchEnabled")
            .expect("setCrouchEnabled")
            .param_type(1),
        Some("Boolean")
    );
}

#[test]
fn documented_action_and_value_signatures_are_inventory_entries() {
    let catalog = builtin();
    let indexed = catalog
        .entry(Kind::Action, "setPlayerVariableAtIndex")
        .expect("indexed player-variable action");
    assert_eq!(indexed.params(), ["Variable", "Index", "Value"]);
    assert_eq!(indexed.param_type(0), Some("Player Variable"));
    assert_eq!(indexed.param_type(2), Some("Object|Array"));

    let custom_string = catalog
        .entry(Kind::Value, "customString")
        .expect("custom string");
    assert_eq!(custom_string.param_count(), 4);
    assert_eq!(custom_string.required_param_count(), 1);
    assert_eq!(custom_string.return_type(), Some("String"));

    let array = catalog.entry(Kind::Value, "array").expect("array");
    assert!(array.is_variadic());
    assert_eq!(array.return_type(), Some("Array"));
    assert_eq!(array.param_type(3), Some("Object|Array"));
}

#[test]
fn catalog_defaults_remain_available_through_position_queries() {
    let catalog = builtin();
    let wait = catalog.entry(Kind::Action, "wait").expect("wait");

    assert!(wait.has_param_defaults());
    assert_eq!(wait.param_default(0), None);
    assert_eq!(wait.param_default(1), Some("Wait.IGNORE_CONDITION"));
}

#[test]
fn min_max_are_canonical_operator_identities() {
    let catalog = builtin();
    for (id, en_spelling, zh_spelling) in [("min", "Min", "较小"), ("max", "Max", "较大")] {
        let entry = catalog.entry(Kind::Operator, id).expect(id);
        assert_eq!(entry.spelling(&en()), Some(en_spelling));
        assert_eq!(entry.spelling(&Locale::new("zh-CN")), Some(zh_spelling));
        assert_eq!(
            catalog
                .resolve(Kind::Operator, &en(), en_spelling)
                .map(|entry| entry.id.as_str()),
            Some(id)
        );
    }
}

#[test]
fn array_removal_value_and_modification_identities_are_distinct() {
    let catalog = builtin();

    assert_eq!(
        catalog
            .resolve(Kind::Value, &en(), "Remove From Array")
            .map(|entry| entry.id.as_str()),
        Some("removeFromArray")
    );
    assert!(catalog.entry(Kind::Operator, "removeFromArray").is_none());
    assert!(
        catalog
            .resolve(Kind::Operator, &en(), "Remove From Array")
            .is_none()
    );

    for (id, spelling) in [
        ("removeFromArrayByValue", "Remove From Array By Value"),
        ("removeFromArrayByIndex", "Remove From Array By Index"),
    ] {
        assert_eq!(
            catalog
                .resolve(Kind::Operator, &en(), spelling)
                .map(|entry| entry.id.as_str()),
            Some(id)
        );
    }
}

#[test]
fn exercised_enum_domains_resolve_members_to_canonical_identity() {
    let catalog = builtin();

    // Hero members resolve with their canonical ids and en-US spellings.
    assert_eq!(
        catalog.resolve_enum_member("Hero", &en(), "D.Va"),
        Some(("Hero".to_string(), "DVA".to_string()))
    );
    assert_eq!(
        catalog.enum_spelling("Hero", &en(), "WRECKING_BALL"),
        Some("Wrecking Ball")
    );

    // Button, Team, Color, and the reevaluation domains exercised by the
    // protect-ban closure.
    assert_eq!(
        catalog.resolve_enum_member("Button", &en(), "Ability 2"),
        Some(("Button".to_string(), "ABILITY_2".to_string()))
    );
    assert_eq!(
        catalog.resolve_enum_member("Team", &en(), "Team 1"),
        Some(("Team".to_string(), "TEAM_1".to_string()))
    );
    assert_eq!(
        catalog.resolve_enum_member("Color", &en(), "Sky Blue"),
        Some(("Color".to_string(), "SKY_BLUE".to_string()))
    );
    assert_eq!(
        catalog.resolve_enum_member("EffectReeval", &en(), "Visible To Position and Radius"),
        Some((
            "EffectReeval".to_string(),
            "VISIBLE_TO_POSITION_AND_RADIUS".to_string()
        ))
    );
    assert_eq!(
        catalog.resolve_enum_member(
            "InworldTextReeval",
            &en(),
            "Visible To Position String and Color"
        ),
        Some((
            "InworldTextReeval".to_string(),
            "VISIBLE_TO_POSITION_STRING_AND_COLOR".to_string()
        ))
    );

    // Map members resolve (exercised by the protect-ban MapData surface).
    assert_eq!(
        catalog.resolve_enum_member("Map", &en(), "Watchpoint: Gibraltar"),
        Some(("Map".to_string(), "WATCHPOINT_GIBRALTAR".to_string()))
    );
}
