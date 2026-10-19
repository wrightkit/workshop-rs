use crate::common::{catalog, en, zh};
use workshop_rs::catalog::{Catalog, Kind, Locale};

#[test]
fn builtin_catalog_loads_and_declares_the_pinned_workshop_locales() {
    let catalog = catalog();
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
fn builtin_ref_returns_the_same_cached_instance() {
    let first = Catalog::builtin_ref().expect("built-in catalog loads");
    let second = Catalog::builtin_ref().expect("built-in catalog loads");
    assert!(std::ptr::eq(first, second));
}

#[test]
fn localized_spelling_resolves_to_canonical_id_and_back() {
    let catalog = catalog();

    let entry = catalog
        .resolve(Kind::Action, &en(), "Disable Inspector Recording")
        .expect("spelling resolves");
    assert_eq!(entry.id, "disableInspector");
    assert_eq!(
        catalog.spelling(Kind::Action, &en(), "disableInspector"),
        Some("Disable Inspector Recording")
    );

    let entry = catalog
        .resolve(Kind::Value, &en(), "Count Of")
        .expect("count of resolves");
    assert_eq!(entry.id, "countOf");
    assert_eq!(
        catalog.spelling(Kind::Value, &en(), "countOf"),
        Some("Count Of")
    );

    let entry = catalog
        .resolve(Kind::Structural, &en(), "For Global Variable")
        .expect("structural resolves");
    assert_eq!(entry.id, "forGlobalVariable");
    assert!(catalog.entry(Kind::Value, "global").is_none());
    assert_eq!(
        catalog.spelling(Kind::Structural, &zh(), "global"),
        Some("全局")
    );
}

#[test]
fn canonical_and_localized_parameter_spellings_resolve_by_position() {
    let catalog = catalog();
    let entry = catalog.entry(Kind::Action, "wait").expect("wait action");
    let en = en();
    let zh = zh();

    assert_eq!(entry.resolve_param(&en, "Duration"), Some(0));
    assert_eq!(entry.resolve_param(&zh, "Duration"), Some(0));
    assert_eq!(entry.resolve_param(&zh, "时间"), Some(0));
    assert_eq!(entry.resolve_param(&zh, "等待行为"), Some(1));
    assert_eq!(entry.resolve_param(&zh, "missing"), None);
}

#[test]
fn localized_string_presets_resolve_and_translate_by_identity() {
    let catalog = catalog();
    let zh = zh();
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
    let catalog = catalog();
    let zh = zh();
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
    let catalog = catalog();
    let zh = zh();
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
    let catalog = catalog();
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
    let catalog = catalog();
    assert_eq!(
        catalog.localized_enum_spelling("Color", &en(), "WHITE"),
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
    let catalog = catalog();
    let zh = zh();
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
    let catalog = catalog();
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
    let catalog = catalog();
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
    let zh = zh();
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
    let catalog = catalog();

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

    let event_player = catalog
        .entry(Kind::Value, "eventPlayer")
        .expect("eventPlayer is in the catalog");
    assert!(event_player.params().is_empty());
    assert_eq!(
        catalog.spelling(Kind::Value, &en(), "eventPlayer"),
        Some("Event Player")
    );

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
fn direction_values_declare_the_vector_return_type() {
    // #495: the six unit-direction values are vectors, not arrays — the
    // pinned workshop-data export types `Vector.UP` and the other direction
    // constants as Direction (vector) returns, and the direction parameters
    // they fill are typed `Vector`.
    let catalog = catalog();
    for id in ["up", "down", "left", "right", "forward", "backward"] {
        let entry = catalog
            .entry(Kind::Value, id)
            .unwrap_or_else(|| panic!("{id} is in the catalog"));
        assert_eq!(
            entry.return_type(),
            Some("Vector"),
            "{id} must declare the vector return type"
        );
    }
}

#[test]
fn evidence_backed_signature_types_are_exposed() {
    let catalog = catalog();
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
    let catalog = catalog();
    let indexed = catalog
        .entry(Kind::Action, "setPlayerVariableAtIndex")
        .expect("indexed player-variable action");
    // The catalog declares the surface signature; the `(player, variable)`
    // pair is what the WIR argument list folds into one `PlayerVariable`.
    assert_eq!(indexed.params(), ["Player", "Variable", "Index", "Value"]);
    assert_eq!(indexed.param_type(0), Some("Player|Array"));
    assert_eq!(indexed.param_type(3), Some("Object|Array"));

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
fn builtin_entries_with_parameters_declare_reviewed_parameter_names() {
    let raw: serde_json::Value =
        serde_json::from_str(workshop_rs::catalog::CATALOG_DATA).expect("embedded catalog is JSON");
    for section in ["actions", "values"] {
        for entry in raw[section].as_array().expect("catalog section") {
            let id = entry["id"].as_str().expect("entry id");
            let params = entry["params"].as_array().map_or(0, Vec::len);
            if params == 0 {
                continue;
            }
            let names = entry["paramNames"]
                .as_array()
                .unwrap_or_else(|| panic!("{section} entry '{id}' must declare paramNames"));
            assert_eq!(
                names.len(),
                params,
                "{section} entry '{id}' must declare one paramNames entry per params entry"
            );
        }
    }
}

#[test]
fn catalog_defaults_remain_available_through_position_queries() {
    let catalog = catalog();
    let wait = catalog.entry(Kind::Action, "wait").expect("wait");

    assert!(wait.has_param_defaults());
    assert_eq!(wait.param_default(0), None);
    assert_eq!(wait.param_default(1), Some("Wait.IGNORE_CONDITION"));
}

#[test]
fn min_max_are_canonical_operator_identities() {
    let catalog = catalog();
    for (id, en_spelling, zh_spelling) in [("min", "Min", "较小"), ("max", "Max", "较大")] {
        let entry = catalog.entry(Kind::Operator, id).expect(id);
        assert_eq!(entry.spelling(&en()), Some(en_spelling));
        assert_eq!(entry.spelling(&zh()), Some(zh_spelling));
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
    let catalog = catalog();

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
    let catalog = catalog();

    assert_eq!(
        catalog.resolve_enum_member("Hero", &en(), "D.Va"),
        Some(("Hero".to_string(), "DVA".to_string()))
    );
    // Doctrine is an official-source-only member: absent from the pinned
    // export, its aliases come from the official localized hero-detail pages.
    assert_eq!(
        catalog.resolve_enum_member("Hero", &zh(), "血律"),
        Some(("Hero".to_string(), "DOCTRINE".to_string()))
    );
    assert_eq!(
        catalog.enum_spelling("Hero", &en(), "WRECKING_BALL"),
        Some("Wrecking Ball")
    );

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

    assert_eq!(
        catalog.resolve_enum_member("Map", &en(), "Watchpoint: Gibraltar"),
        Some(("Map".to_string(), "WATCHPOINT_GIBRALTAR".to_string()))
    );
    assert_eq!(
        catalog.resolve_enum_member("Map", &en(), "Grímsvötn"),
        Some(("Map".to_string(), "GRIMSVOTN".to_string()))
    );
    // The export carries only Grímsvötn's en-US spelling; the localized
    // aliases are transcribed from the official localized Season 5
    // announcement pages.
    assert_eq!(
        catalog.resolve_enum_member("Map", &zh(), "格里姆火山"),
        Some(("Map".to_string(), "GRIMSVOTN".to_string()))
    );
    assert_eq!(
        catalog.enum_spelling("Map", &Locale::new("ja-JP"), "GRIMSVOTN"),
        Some("GRÍMSVÖTN")
    );
    assert_eq!(
        catalog.enum_spelling("Map", &Locale::new("ko-KR"), "GRIMSVOTN"),
        Some("그림스뵈튼")
    );
}

/// Per-parameter reevaluation coverage (wright#562): each `*Reeval` action
/// member resolves to the parameter positions it keeps re-evaluating. The
/// data is pinned to the reviewed member-to-parameter map; positions are
/// into `params`, and members that keep nothing live resolve to an empty
/// coverage rather than `None`.
#[test]
fn reevaluation_coverage_is_per_parameter() {
    let catalog = catalog();

    // `Create Beam Effect` under `COLOR` reevaluates only its Color input;
    // `VISIBLE_TO_POSITION_AND_RADIUS` keeps Visible To + both positions
    // live (the beam has no separate radius parameter).
    let beam = catalog
        .entry(Kind::Action, "createBeamEffect")
        .expect("createBeamEffect");
    assert_eq!(beam.param_domain(5), Some("EffectReeval"));
    assert!(beam.has_reevaluation_coverage());
    assert_eq!(beam.reevaluation_coverage("NONE"), Some(&[][..]));
    assert_eq!(beam.reevaluation_coverage("COLOR"), Some(&[4][..]));
    assert_eq!(
        beam.reevaluation_coverage("VISIBLE_TO_POSITION_AND_RADIUS"),
        Some(&[0, 2, 3][..])
    );
    assert_eq!(
        beam.reevaluation_coverage("VISIBILITY_POSITION_RADIUS_AND_COLOR"),
        Some(&[0, 2, 3, 4][..])
    );

    let icon = catalog
        .entry(Kind::Action, "createIcon")
        .expect("createIcon");
    assert_eq!(icon.reevaluation_coverage("COLOR"), Some(&[4][..]));

    assert!(
        catalog
            .entry(Kind::Action, "setGlobalVariableAtIndex")
            .expect("setGlobalVariableAtIndex")
            .reevaluation_coverage("COLOR")
            .is_none()
    );
    assert!(beam.reevaluation_coverage("NOT_A_MEMBER").is_none());

    let mut reeval_actions = 0;
    for entry in catalog.entries_of(Kind::Action) {
        let Some(reeval_index) = (0..entry.param_count()).find(|index| {
            entry
                .param_domain(*index)
                .is_some_and(|domain| domain.ends_with("Reeval"))
        }) else {
            continue;
        };
        reeval_actions += 1;
        let domain = entry.param_domain(reeval_index).unwrap();
        let members: Vec<&str> = catalog
            .enum_domain(domain)
            .unwrap()
            .members
            .iter()
            .map(|m| m.member.as_str())
            .collect();
        for member in members {
            assert!(
                entry.reevaluation_coverage(member).is_some(),
                "{}'s coverage does not review {domain}::{member}",
                entry.id
            );
        }
    }
    assert_eq!(reeval_actions, 19);
}

/// workshop-rs#505: an enum-typed parameter declares its `paramDomains`
/// entry so `lookup` attaches the domain's members and validation rejects
/// enum literals from another domain at that position (ADR-0014). The
/// declared semantic type's leading `paramTypes` alternative names the
/// domain — `Hero|Array` is a Hero position, while `Vector` positions take
/// any vector value (not just the direction constants) and unions led by a
/// non-enum type such as `String|Array|EntityId|Color` are not enum
/// positions, so neither kind names a `paramDomains` entry.
#[test]
fn enum_typed_parameters_declare_their_param_domain() {
    let catalog = catalog();
    for entry in catalog
        .entries_of(Kind::Action)
        .chain(catalog.entries_of(Kind::Value))
    {
        for index in 0..entry.param_count() {
            let Some(domain) = entry
                .param_type(index)
                .and_then(|param_type| param_type.split('|').next())
                .filter(|domain| catalog.enum_domain(domain).is_some())
            else {
                continue;
            };
            if domain == "Vector" {
                continue;
            }
            assert_eq!(
                entry.param_domain(index),
                Some(domain),
                "{} '{}' parameter {} declares type '{domain}' but no paramDomains entry",
                entry.kind.as_str(),
                entry.id,
                index + 1
            );
            assert!(
                !catalog.enum_domain(domain).unwrap().members.is_empty(),
                "enum domain '{domain}' for '{}' parameter {} has no members",
                entry.id,
                index + 1
            );
        }
    }
}
