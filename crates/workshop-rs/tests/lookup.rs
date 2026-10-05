//! Lookup-surface tests: `Catalog::lookup` answers in canonical Workshop
//! terms (identity, kind, localized display name, signature, settings keys)
//! derived from the same catalog and settings table that drive parsing and
//! emission, and `unknown ... spelling` diagnostics carry nearest valid
//! candidates.

use crate::common::{catalog, en, zh};
use workshop_rs::WorkshopError;
use workshop_rs::catalog::{Kind, Locale};
use workshop_rs::lookup::LookupMatch;
use workshop_rs::settings::SettingValueDomain;
use workshop_rs::{parser, settings};

fn builtin_id<'m>(matches: &'m [LookupMatch], id: &str) -> &'m LookupMatch {
    matches
        .iter()
        .find(|lookup| matches!(lookup, LookupMatch::Builtin { id: found, .. } if found == id))
        .unwrap_or_else(|| panic!("lookup should match builtin '{id}': {matches:?}"))
}

fn parse_error(source: &str) -> WorkshopError {
    parser::parse(source, &catalog(), &en()).expect_err("source must not parse")
}

#[test]
fn display_name_and_near_spelling_lookup_return_canonical_entry() {
    let catalog = catalog();
    for query in ["Create HUD Text", "create hud txt", "createHudText"] {
        let matches = catalog.lookup(&en(), query).expect("lookup answers");
        let first = matches.first().expect("a match exists");
        let LookupMatch::Builtin {
            kind,
            id,
            display_name,
            signature,
        } = first
        else {
            panic!("'{query}' should match a builtin first: {matches:?}");
        };
        assert_eq!(*kind, Kind::Action);
        assert_eq!(id, "createHudText");
        assert_eq!(display_name.as_deref(), Some("Create HUD Text"));
        assert!(signature.is_some());
    }
}

#[test]
fn localized_display_name_lookup_returns_the_requesting_locales_names() {
    let catalog = catalog();
    let matches = catalog
        .lookup(&zh(), "创建HUD文本")
        .expect("lookup answers");
    let LookupMatch::Builtin {
        id, display_name, ..
    } = builtin_id(&matches, "createHudText")
    else {
        panic!("expected a builtin match");
    };
    assert_eq!(id, "createHudText");
    assert_eq!(display_name.as_deref(), Some("创建HUD文本"));
}

#[test]
fn signature_lists_params_defaults_and_inline_enum_members_once() {
    let catalog = catalog();
    let matches = catalog.lookup(&en(), "Create HUD Text").expect("lookup");
    let LookupMatch::Builtin {
        signature: Some(signature),
        ..
    } = builtin_id(&matches, "createHudText")
    else {
        panic!("createHudText is an action with a signature");
    };
    let text = &signature.text;
    // Required non-enum parameters show `name: Type`.
    assert!(
        text.starts_with("Create HUD Text(VisibleTo: Player|Array"),
        "{text}"
    );
    assert!(text.contains("Header: Object|String"), "{text}");
    // Declared null defaults read `name?`; the type is omitted.
    assert!(text.contains("Subheader?"), "{text}");
    assert!(text.contains("Text?"), "{text}");
    // Required enum parameters list the domain's members once per entry.
    assert!(
        text.contains("Location: HudPosition(Left|Right|Top)"),
        "{text}"
    );
    assert!(
        text.contains("HeaderColor: Color(Yellow|White|Red|Orange|Green|Purple|Blue|Aqua|Sky Blue|Turquoise|Lime Green|Gray|Violet|Rose|Black|Team 1|Team 2)"),
        "{text}"
    );
    // The same domain is not listed a second time.
    assert_eq!(text.matches("Yellow").count(), 1, "{text}");
    // Optional enum parameters show the default in emitted call form, not
    // the members: constructor-form domains render `Domain(Member)`.
    assert!(text.contains("SubheaderColor=Color(White)"), "{text}");
    assert!(text.contains("TextColor=Color(White)"), "{text}");
    // Larger required enum domains still list members when inside the limit.
    assert!(
        text.contains(
            "Spectators: SpecVisibility(Default Visibility|Visible Always|Visible Never)"
        ),
        "{text}"
    );

    assert_eq!(signature.params.len(), 11);
    assert_eq!(signature.required_params, 11);
    assert!(!signature.variadic);
    let header_color = &signature.params[6];
    assert_eq!(header_color.name, "HeaderColor");
    assert_eq!(header_color.domain.as_deref(), Some("Color"));
    assert_eq!(header_color.default, None);
    let subheader_color = &signature.params[7];
    assert_eq!(subheader_color.default.as_deref(), Some("Color.WHITE"));

    // Referenced domains are exposed for follow-up member inspection.
    let domains: Vec<&str> = signature
        .domains
        .iter()
        .map(|domain| domain.domain.as_str())
        .collect();
    for expected in ["HudPosition", "Color", "HudReeval", "SpecVisibility"] {
        assert!(domains.contains(&expected), "{domains:?}");
    }
}

#[test]
fn large_enum_domain_reports_member_count_and_follow_up_lists_members() {
    let catalog = catalog();
    // The member count is catalog data: derive it so a catalog update does
    // not break the lookup-behavior assertion.
    let hero_count = catalog
        .enum_domain("Hero")
        .expect("the Hero domain exists")
        .members
        .len();
    assert!(
        hero_count > 32,
        "the Hero domain must exceed the inline limit"
    );
    let matches = catalog
        .lookup(&en(), "Start Forcing Player To Be Hero")
        .expect("lookup");
    let LookupMatch::Builtin {
        signature: Some(signature),
        ..
    } = builtin_id(&matches, "startForcingHero")
    else {
        panic!("startForcingHero is an action with a signature");
    };
    assert!(
        signature
            .text
            .contains(&format!("hero: Hero({hero_count} members)")),
        "{}",
        signature.text
    );
    let hero_domain = signature
        .domains
        .iter()
        .find(|domain| domain.domain == "Hero")
        .expect("the Hero domain is exposed for follow-up");
    assert_eq!(hero_domain.members.len(), hero_count);

    // A follow-up query on the domain lists every member.
    let domain_matches = catalog.lookup(&en(), "Hero").expect("lookup");
    let domain = domain_matches.iter().find_map(|lookup| match lookup {
        LookupMatch::EnumDomain {
            domain, members, ..
        } if domain == "Hero" => Some(members),
        _ => None,
    });
    let members = domain.expect("the Hero domain matches a domain query");
    assert_eq!(members.len(), hero_count);
    assert!(members.iter().any(|member| member.id == "SOLDIER_76"));
}

#[test]
fn enum_member_lookup_reports_domain_member_and_display_name() {
    let catalog = catalog();
    for query in ["Soldier: 76", "SOLDIER_76", "Hero.SOLDIER_76"] {
        let matches = catalog.lookup(&en(), query).expect("lookup");
        let member = matches.iter().find_map(|lookup| match lookup {
            LookupMatch::EnumMember {
                domain,
                member,
                display_name,
            } if domain == "Hero" => Some((member.as_str(), display_name.as_deref())),
            _ => None,
        });
        assert_eq!(
            member,
            Some(("SOLDIER_76", Some("Soldier: 76"))),
            "query '{query}' should resolve to Hero.SOLDIER_76"
        );
    }
}

#[test]
fn settings_lookup_by_display_name_returns_keys_kinds_and_value_forms() {
    let catalog = catalog();
    let matches = catalog.lookup(&en(), "Score To Win").expect("lookup");
    let settings: Vec<&workshop_rs::settings::SettingDefinition> = matches
        .iter()
        .filter_map(|lookup| match lookup {
            LookupMatch::Setting {
                definition,
                display_name,
            } if display_name == "Score To Win" => Some(definition),
            _ => None,
        })
        .collect();
    assert!(
        settings.iter().any(|definition| {
            definition.path() == "gamemodes.control.scoreToWin"
                && matches!(definition.domain(), SettingValueDomain::Number(_))
        }),
        "a Score To Win key with its kind and value form must match: {matches:?}"
    );
}

#[test]
fn settings_lookup_by_path_prefix_includes_templated_paths() {
    let catalog = catalog();
    let matches = catalog.lookup(&en(), "gamemodes.control").expect("lookup");
    assert!(
        matches.iter().any(|lookup| matches!(
            lookup,
            LookupMatch::Setting { definition, .. }
                if definition.path() == "gamemodes.control.scoreToWin"
        )),
        "a path prefix returns the keys below it: {matches:?}"
    );

    // Template segments accept concrete team and hero names.
    let hero_matches = catalog.lookup(&en(), "heroes.team1.ana").expect("lookup");
    let paths: Vec<&str> = hero_matches
        .iter()
        .filter_map(|lookup| match lookup {
            LookupMatch::Setting { definition, .. } => Some(definition.path()),
            _ => None,
        })
        .collect();
    assert!(
        paths
            .iter()
            .any(|path| path.starts_with("heroes.<team>.<hero>.")),
        "templated hero paths answer concrete hero queries: {paths:?}"
    );
    assert!(
        !paths.iter().any(|path| path.starts_with("gamemodes.")),
        "unrelated paths do not match: {paths:?}"
    );
}

#[test]
fn every_parser_accepted_construct_is_reachable_through_lookup() {
    // Divergence protection: lookup draws from the same catalog and settings
    // table as the parser and emitter, so every accepted identity and every
    // declared settings path must resolve back to itself.
    let catalog = catalog();
    for kind in [Kind::Structural, Kind::Event, Kind::Action, Kind::Value] {
        for entry in catalog.entries_of(kind) {
            let matches = catalog.lookup(&en(), &entry.id).expect("lookup");
            let first = matches.first().unwrap_or_else(|| {
                panic!("catalog {kind:?} '{}' must be lookup-reachable", entry.id)
            });
            assert!(
                matches!(first, LookupMatch::Builtin { id, .. } if id == &entry.id),
                "the exact catalog id '{}' must rank first: {matches:?}",
                entry.id
            );
        }
    }
    for definition in settings::definitions() {
        let matches = catalog.lookup(&en(), definition.path()).expect("lookup");
        assert!(
            matches.iter().any(|lookup| matches!(
                lookup,
                LookupMatch::Setting { definition: found, .. }
                    if found.path() == definition.path()
            )),
            "settings path '{}' must be lookup-reachable",
            definition.path()
        );
    }
}

#[test]
fn a_query_the_catalog_cannot_answer_is_explicitly_unsupported() {
    let catalog = catalog();
    // An undeclared locale cannot answer localized names at all.
    let error = catalog
        .lookup(&Locale::new("es-CL"), "Create HUD Text")
        .expect_err("an undeclared locale is unsupported");
    assert!(
        matches!(error, WorkshopError::Unsupported { .. }),
        "{error:?}"
    );
    // A query with no near construct answers nothing rather than guessing.
    let matches = catalog.lookup(&en(), "zzqqxxjjj").expect("lookup answers");
    assert!(matches.is_empty(), "{matches:?}");
}

#[test]
fn unknown_action_spelling_carries_nearest_valid_candidates() {
    let error = parse_error(
        "rule (\"x\") {\n    event {\n        Ongoing - Global;\n    }\n    actions {\n        Create HUD Txt(Null, Null, Null, Null, Left, 0, White, White, White, Visible To, Default Visibility);\n    }\n}\n",
    );
    // The message itself names the candidates: consumers that only see the
    // diagnostic text, such as a closed-schema wire diagnostic, still get
    // them (wrightkit/workshop-rs#379).
    assert_eq!(
        error.to_string(),
        "unknown action spelling 'Create HUD Txt' for locale 'en-us' \
         (did you mean 'Create HUD Text'?)"
    );
    let WorkshopError::UnknownWithCandidates {
        kind, candidates, ..
    } = error
    else {
        panic!("an unknown action spelling is an Unknown diagnostic: {error:?}");
    };
    assert_eq!(kind, "action");
    assert_eq!(
        candidates.first().map(String::as_str),
        Some("Create HUD Text")
    );
}

#[test]
fn unknown_spelling_candidates_cover_observed_near_spellings() {
    // Observed misspelling shapes from the wright#482 agent benchmark: a
    // dropped word, a camelCase guess written where a display name belongs,
    // and a mistyped enum member.
    let cases = [
        (
            "rule (\"x\") {\n    event {\n        Ongoing - Global;\n    }\n    actions {\n        Create HUD Txt(Null, Null, Null, Null, Left, 0, White, White, White, Visible To, Default Visibility);\n    }\n}\n",
            "Create HUD Text",
        ),
        (
            "rule (\"x\") {\n    event {\n        Ongoing - Global;\n    }\n    actions {\n        createHudText(Null, Null, Null, Null, Left, 0, White, White, White, Visible To, Default Visibility);\n    }\n}\n",
            "Create HUD Text",
        ),
        (
            "rule (\"x\") {\n    event {\n        Ongoing - Each Player;\n        All;\n        Reinhardtt;\n    }\n}\n",
            "Reinhardt",
        ),
    ];
    for (source, expected) in cases {
        let error = parse_error(source);
        assert!(
            error.candidates().iter().any(|c| c == expected),
            "'{expected}' should be a candidate for {source:?}: {error:?}"
        );
        assert!(
            error.to_string().contains(expected),
            "the message names '{expected}': {error}"
        );
    }
}

#[test]
fn unknown_spelling_candidates_follow_the_rejecting_locale() {
    // The candidate space is the rejecting locale's accepted spellings, so a
    // zh-CN near spelling names zh-CN candidates.
    let source = "规则 (\"x\") {\n    事件 {\n        持续 - 全局;\n    }\n    动作 {\n        创建HUD文(0);\n    }\n}\n";
    let error = parser::parse(source, &catalog(), &zh())
        .expect_err("a near zh-CN action spelling must not parse");
    assert!(
        error
            .candidates()
            .iter()
            .any(|candidate| candidate == "创建HUD文本"),
        "the zh-CN display name reports as a candidate: {error:?}"
    );
    assert!(error.to_string().contains("创建HUD文本"), "{error}");
}

#[test]
fn unknown_spelling_candidates_are_deterministic_across_runs() {
    // The candidate ranking is a stable sort over the accepted-spelling
    // order: `Team 3` ties `Team 1`/`Team 2`, and repeated parses must report
    // the same list and the same message.
    let source = "settings\n{\n\theroes\n\t{\n\t\tTeam 3\n\t\t{\n\t\t}\n\t}\n}\n";
    let baseline = parse_error(source);
    assert!(
        baseline.candidates().len() > 1,
        "the tied candidates exercise ordering: {baseline:?}"
    );
    for _ in 0..5 {
        let error = parse_error(source);
        assert_eq!(error.candidates(), baseline.candidates());
        assert_eq!(error.to_string(), baseline.to_string());
    }
}

#[test]
fn unknown_enum_member_spelling_carries_domain_candidates() {
    let error = parse_error(
        "rule (\"x\") {\n    event {\n        Ongoing - Each Player;\n        All;\n        Reinhardtt;\n    }\n}\n",
    );
    let WorkshopError::UnknownWithCandidates { candidates, .. } = error else {
        panic!("an unknown hero is an Unknown diagnostic: {error:?}");
    };
    assert!(
        candidates.iter().any(|candidate| candidate == "Reinhardt"),
        "the nearest hero reports as a candidate: {candidates:?}"
    );
}

#[test]
fn unknown_settings_team_carries_valid_candidates() {
    let error = parse_error("settings\n{\n\theroes\n\t{\n\t\tTeam 3\n\t\t{\n\t\t}\n\t}\n}\n");
    let WorkshopError::UnknownWithCandidates {
        kind, candidates, ..
    } = error
    else {
        panic!("an unknown settings team is an Unknown diagnostic: {error:?}");
    };
    assert_eq!(kind, "setting");
    assert!(
        candidates
            .iter()
            .any(|candidate| candidate == "Team 1" || candidate == "Team 2"),
        "the nearest team names report as candidates: {candidates:?}"
    );
}
