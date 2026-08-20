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
fn builtin_catalog_loads_and_declares_en_us_and_zh_cn() {
    let catalog = builtin();
    assert!(catalog.supports(&en()));
    assert_eq!(catalog.locales().len(), 2);
    // The primary locale is first and complete.
    assert_eq!(catalog.locales()[0], en());
    assert_eq!(catalog.primary_locale(), &en());
    assert_eq!(
        catalog.locale_coverage(&en()).mapped,
        catalog.locale_coverage(&en()).total
    );
    // zh-CN is the evidence-backed corpus locale and is complete for the
    // canonical entries consumed by the raw-project corpus.
    assert!(catalog.supports(&Locale::new("zh-CN")));
    assert_eq!(
        catalog.locale_coverage(&Locale::new("zh-CN")).mapped,
        catalog.locale_coverage(&Locale::new("zh-CN")).total
    );
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
}

#[test]
fn reviewed_locale_alias_conflicts_resolve_to_one_canonical_identity() {
    let catalog = builtin();
    for spelling in ["中止", "中断"] {
        let entry = catalog
            .resolve(Kind::Action, &Locale::new("zh-CN"), spelling)
            .expect("reviewed alias resolves");
        assert_eq!(entry.id, "abort");
    }
    assert_eq!(
        catalog.spelling(Kind::Action, &Locale::new("zh-CN"), "abort"),
        Some("中止")
    );
    assert_eq!(
        catalog
            .entry(Kind::Action, "abort")
            .expect("abort entry")
            .spellings(&Locale::new("zh-CN")),
        &["中止".to_string(), "中断".to_string()]
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
        effect.params,
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
    assert!(event_player.params.is_empty());
    assert_eq!(
        catalog.spelling(Kind::Value, &en(), "eventPlayer"),
        Some("Event Player")
    );

    // A shared canonical identity: `Wait`/`MinWait` both bind to `wait`.
    assert_eq!(
        catalog
            .entry(Kind::Action, "wait")
            .map(|e| e.params.clone()),
        Some(vec!["Duration".to_string(), "WaitBehavior".to_string()])
    );

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
