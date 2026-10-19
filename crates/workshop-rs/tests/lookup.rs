use crate::common::{catalog, en, zh};
use workshop_rs::WorkshopError;
use workshop_rs::catalog::{Catalog, Kind, Locale};
use workshop_rs::lookup::{LookupMatch, Signature};
use workshop_rs::settings::SettingValueDomain;
use workshop_rs::{parser, settings};

fn builtin_id<'m>(matches: &'m [LookupMatch], id: &str) -> &'m LookupMatch {
    matches
        .iter()
        .find(|lookup| matches!(lookup, LookupMatch::Builtin { id: found, .. } if found == id))
        .unwrap_or_else(|| panic!("lookup should match builtin '{id}': {matches:?}"))
}

/// The signature lookup reports for the structural entry `id`.
fn structural_signature(catalog: &Catalog, locale: &Locale, id: &str) -> Signature {
    let matches = catalog.lookup(locale, id).expect("lookup answers");
    matches
        .iter()
        .find_map(|lookup| match lookup {
            LookupMatch::Builtin {
                kind,
                id: found,
                signature,
                ..
            } if *kind == Kind::Structural && found == id => signature.clone(),
            _ => None,
        })
        .unwrap_or_else(|| panic!("'{id}' must report a structural signature: {matches:?}"))
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
    assert!(
        text.starts_with("Create HUD Text(VisibleTo: Player|Array"),
        "{text}"
    );
    assert!(text.contains("Header: Object|String"), "{text}");
    assert!(text.contains("Subheader?"), "{text}");
    assert!(text.contains("Text?"), "{text}");
    assert!(
        text.contains("Location: HudPosition(Left|Right|Top)"),
        "{text}"
    );
    assert!(
        text.contains("HeaderColor: Color(Yellow|White|Red|Orange|Green|Purple|Blue|Aqua|Sky Blue|Turquoise|Lime Green|Gray|Violet|Rose|Black|Team 1|Team 2)"),
        "{text}"
    );
    assert_eq!(text.matches("Yellow").count(), 1, "{text}");
    assert!(text.contains("SubheaderColor=Color(White)"), "{text}");
    assert!(text.contains("TextColor=Color(White)"), "{text}");
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
fn indexed_player_variable_signature_lists_the_surface_parameters() {
    // workshop-rs#442: the player-variable index actions declare their
    // surface signature — including the `player` argument the source syntax
    // requires — even though WIR folds `(player, variable)` into one value.
    let catalog = catalog();
    let matches = catalog
        .lookup(&en(), "Set Player Variable At Index")
        .expect("lookup");
    let LookupMatch::Builtin {
        signature: Some(signature),
        ..
    } = builtin_id(&matches, "setPlayerVariableAtIndex")
    else {
        panic!("setPlayerVariableAtIndex is an action with a signature");
    };
    assert_eq!(
        signature.text,
        "Set Player Variable At Index(player: Player|Array, variable: Variable, index: Number|Boolean, value: Object|Array)"
    );
    assert_eq!(signature.params.len(), 4);
    assert_eq!(signature.required_params, 4);
}

#[test]
fn structural_constructs_report_their_call_signatures() {
    // workshop-rs#496: the structural constructs that take arguments
    // declare the signature of their `rule-content-v1` call, in argument
    // order — `setGlobalVariable(variable, value)`,
    // `forGlobalVariable(controlVariable, rangeStart, rangeStop, step)`,
    // `if(condition)`.
    let catalog = catalog();
    let cases: &[(&str, &[(&str, &str)])] = &[
        (
            "setGlobalVariable",
            &[("variable", "Variable"), ("value", "Object|Array")],
        ),
        (
            "forGlobalVariable",
            &[
                ("controlVariable", "Variable"),
                ("rangeStart", "Number"),
                ("rangeStop", "Number"),
                ("step", "Number"),
            ],
        ),
        ("if", &[("condition", "Boolean")]),
    ];
    for &(id, expected) in cases {
        let signature = structural_signature(&catalog, &zh(), id);
        let params: Vec<(&str, Option<&str>)> = signature
            .params
            .iter()
            .map(|param| (param.name.as_str(), param.param_type.as_deref()))
            .collect();
        let expected: Vec<(&str, Option<&str>)> = expected
            .iter()
            .map(|(name, param_type)| (*name, Some(*param_type)))
            .collect();
        assert_eq!(params, expected, "'{id}' signature params");
        assert_eq!(signature.required_params, expected.len());
        assert!(!signature.variadic);
    }
    // The localized display name renders the signature's call head.
    let signature = structural_signature(&catalog, &zh(), "setGlobalVariable");
    assert_eq!(
        signature.text,
        "设置全局变量(variable: Variable, value: Object|Array)"
    );
}

#[test]
fn zero_argument_constructs_report_empty_signatures_and_keywords_none() {
    let catalog = catalog();
    for id in ["else", "end"] {
        let signature = structural_signature(&catalog, &en(), id);
        assert!(signature.params.is_empty(), "'{id}' has no arguments");
        assert_eq!(signature.required_params, 0);
    }
    for id in [
        "rule",
        "event",
        "actions",
        "conditions",
        "settings",
        "variables",
        "subroutines",
        "workshop",
        "global",
        "player",
        "disabled",
    ] {
        let matches = catalog.lookup(&en(), id).expect("lookup answers");
        let LookupMatch::Builtin { signature, .. } = builtin_id(&matches, id) else {
            unreachable!("builtin_id only matches builtins");
        };
        assert!(
            signature.is_none(),
            "section keyword '{id}' stays without a signature"
        );
        let error = catalog
            .lookup_within(&en(), id, None)
            .expect_err("a signature-less entry is not a scope");
        assert!(
            matches!(
                error,
                WorkshopError::Unknown {
                    kind: "lookup scope",
                    ..
                } | WorkshopError::UnknownWithCandidates {
                    kind: "lookup scope",
                    ..
                }
            ),
            "'{id}': {error:?}"
        );
    }
}

#[test]
fn scoped_lookup_lists_a_structural_constructs_parameters() {
    // workshop-rs#496: `lookup_within` scopes to a structural call construct
    // the same way it scopes to an action.
    let catalog = catalog();
    let params = catalog
        .lookup_within(&zh(), "setGlobalVariable", None)
        .expect("a structural construct is a scope");
    let mut positions = Vec::with_capacity(params.len());
    let names: Vec<&str> = params
        .iter()
        .map(|param| match param {
            LookupMatch::Parameter {
                callable,
                position,
                required,
                param,
                ..
            } => {
                assert_eq!(callable, "setGlobalVariable");
                positions.push(*position);
                assert!(required, "every parameter is required: {param:?}");
                param.name.as_str()
            }
            other => panic!("a structural scope lists parameters: {other:?}"),
        })
        .collect();
    assert_eq!(names, ["variable", "value"]);
    assert_eq!(positions, [0, 1]);

    // `modifyGlobalVariable` is catalogued both as an action and as the
    // `rule-content-v1` structural call; the established action scope wins,
    // observable by its `value` parameter type (`Any`, where the structural
    // entry declares `Object|Array`).
    let params = catalog
        .lookup_within(&en(), "modifyGlobalVariable", None)
        .expect("a scope");
    let types: Vec<Option<&str>> = params
        .iter()
        .map(|param| match param {
            LookupMatch::Parameter { param, .. } => param.param_type.as_deref(),
            other => panic!("a scope lists parameters: {other:?}"),
        })
        .collect();
    assert_eq!(
        types,
        [Some("Variable"), Some("Operation"), Some("Any")],
        "the action entry keeps the scope"
    );
}

#[test]
fn large_enum_domain_reports_member_count_and_follow_up_lists_members() {
    let catalog = catalog();
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
    let error = catalog
        .lookup(&Locale::new("es-CL"), "Create HUD Text")
        .expect_err("an undeclared locale is unsupported");
    assert!(
        matches!(error, WorkshopError::Unsupported { .. }),
        "{error:?}"
    );
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

#[test]
fn scoped_lookup_lists_a_domains_members_in_domain_order() {
    let catalog = catalog();
    let members = catalog
        .lookup_within(&en(), "Team", None)
        .expect("a domain is a scope");
    let members: Vec<&str> = members
        .iter()
        .map(|member| match member {
            LookupMatch::EnumMember { domain, member, .. } => {
                assert_eq!(domain, "Team");
                member.as_str()
            }
            other => panic!("a domain scope lists members: {other:?}"),
        })
        .collect();
    assert_eq!(members, ["ALL", "TEAM_1", "TEAM_2"]);
}

#[test]
fn scoped_lookup_lists_settings_enum_domain_members() {
    let catalog = catalog();
    let members = catalog
        .lookup_within(&en(), "mapRotation", None)
        .expect("a settings enum domain is a scope");
    let members: Vec<&str> = members
        .iter()
        .map(|member| match member {
            LookupMatch::EnumMember { domain, member, .. } => {
                assert_eq!(domain, "mapRotation");
                member.as_str()
            }
            other => panic!("a settings enum scope lists members: {other:?}"),
        })
        .collect();
    assert!(members.contains(&"afterAGame"), "members: {members:?}");
    assert!(members.contains(&"paused"), "members: {members:?}");
}

#[test]
fn scoped_lookup_scores_settings_enum_members_in_the_requested_locale() {
    let catalog = catalog();
    // The member spellings the locale table maps must match and surface:
    // zh-CN "完成比赛后" is `mapRotation.afterAGame`'s localized spelling.
    let children = catalog
        .lookup_within(&zh(), "mapRotation", Some("完成比赛后"))
        .expect("the domain resolves under zh-CN");
    assert_eq!(
        children.len(),
        1,
        "the localized spelling matches: {children:?}"
    );
    match &children[0] {
        LookupMatch::EnumMember {
            member,
            display_name,
            ..
        } => {
            assert_eq!(member, "afterAGame");
            assert_eq!(display_name.as_deref(), Some("完成比赛后"));
        }
        other => panic!("a settings enum member: {other:?}"),
    }
}

#[test]
fn scoped_lookup_lists_callable_parameters_in_call_order() {
    let catalog = catalog();
    let params = catalog
        .lookup_within(&en(), "createHudText", None)
        .expect("a callable is a scope");
    let mut positions = Vec::with_capacity(params.len());
    let names: Vec<&str> = params
        .iter()
        .map(|param| match param {
            LookupMatch::Parameter {
                callable,
                position,
                param,
                ..
            } => {
                assert_eq!(callable, "createHudText");
                positions.push(*position);
                param.name.as_str()
            }
            other => panic!("a callable scope lists parameters: {other:?}"),
        })
        .collect();
    assert_eq!(
        positions,
        (0..params.len()).collect::<Vec<_>>(),
        "positions run in call order: {params:?}"
    );
    assert_eq!(names.first(), Some(&"VisibleTo"), "call order: {names:?}");
    assert!(names.len() > 1);
}

#[test]
fn scoped_query_filters_and_ranks_children_with_the_shared_matcher() {
    let catalog = catalog();
    let members = catalog
        .lookup_within(&en(), "Team", Some("team 1"))
        .expect("scope resolves");
    assert!(
        !members.is_empty() && members.len() < 3,
        "the query narrows the scope: {members:?}"
    );
    let LookupMatch::EnumMember { member, .. } = &members[0] else {
        panic!("the closest member ranks first: {members:?}");
    };
    assert_eq!(member, "TEAM_1");

    let params = catalog
        .lookup_within(&en(), "createHudText", Some("visibleTo"))
        .expect("scope resolves");
    let LookupMatch::Parameter { param, .. } = params.first().expect("a param matched") else {
        panic!("a param query lists parameters: {params:?}");
    };
    assert_eq!(param.name, "VisibleTo");
}

#[test]
fn scoped_lookup_lists_settings_children_segment_by_segment() {
    let catalog = catalog();
    let children = catalog
        .lookup_within(&en(), "heroes", None)
        .expect("a settings prefix is a scope");
    let paths: Vec<&str> = children
        .iter()
        .map(|child| match child {
            LookupMatch::SettingPath { path, .. } => path.as_str(),
            LookupMatch::Setting { definition, .. } => definition.path(),
            other => panic!("settings children are paths or leaves: {other:?}"),
        })
        .collect();
    assert_eq!(paths, ["heroes.<team>", "heroes.general"]);

    let children = catalog
        .lookup_within(&en(), "heroes.<team>", Some("health"))
        .expect("scope resolves");
    let leaves: Vec<&str> = children
        .iter()
        .filter_map(|child| match child {
            LookupMatch::Setting { definition, .. } => Some(definition.path()),
            _ => None,
        })
        .collect();
    assert!(
        leaves.contains(&"heroes.<team>.health%"),
        "the health leaf matches under the template scope: {children:?}"
    );
    assert!(
        leaves.iter().all(|path| path.starts_with("heroes.<team>.")),
        "every matched leaf belongs to the scope: {children:?}"
    );
}

#[test]
fn scoped_lookup_template_segments_accept_only_recognized_values() {
    let catalog = catalog();
    // `heroes.general` is a literal path, not a `<team>` value: it lists
    // only the general group's own leaf, uncontaminated by the template
    // children `heroes.<team>` declares.
    let children = catalog
        .lookup_within(&en(), "heroes.general", None)
        .expect("a literal prefix resolves");
    let paths: Vec<&str> = children
        .iter()
        .map(|child| match child {
            LookupMatch::Setting { definition, .. } => definition.path(),
            LookupMatch::SettingPath { path, .. } => path.as_str(),
            other => panic!("settings children are paths or leaves: {other:?}"),
        })
        .collect();
    assert_eq!(paths, ["heroes.general.disabledHeroes"]);

    let children = catalog
        .lookup_within(&en(), "heroes.team1", None)
        .expect("a recognized team key resolves");
    assert!(
        children.iter().any(|child| matches!(
            child,
            LookupMatch::Setting { definition, .. } if definition.path() == "heroes.<team>.enabledHeroes"
        )),
        "a recognized team key reaches the templated leaves: {children:?}"
    );
    assert!(
        children.iter().any(|child| matches!(
            child,
            LookupMatch::SettingPath { path, .. } if path == "heroes.team1.<hero>"
        )),
        "the hero slot echoes the asked concrete prefix: {children:?}"
    );
}

#[test]
fn scoped_lookup_rejects_unrecognized_template_values() {
    let catalog = catalog();
    for within in [
        "heroes.not-a-team",
        "heroes.team1.not-a-hero",
        "heroes.not-a-team.not-a-hero",
    ] {
        let error = catalog
            .lookup_within(&en(), within, None)
            .expect_err("an unrecognized template value is an unknown scope");
        assert!(
            matches!(
                error,
                WorkshopError::Unknown {
                    kind: "lookup scope",
                    ..
                } | WorkshopError::UnknownWithCandidates {
                    kind: "lookup scope",
                    ..
                }
            ),
            "{within}: {error:?}"
        );
    }
}

#[test]
fn scoped_lookup_reports_an_unknown_scope() {
    let catalog = catalog();
    let error = catalog
        .lookup_within(&en(), "not-a-scope", None)
        .expect_err("an unmatched scope errors");
    assert!(
        matches!(
            error,
            WorkshopError::Unknown {
                kind: "lookup scope",
                ..
            } | WorkshopError::UnknownWithCandidates {
                kind: "lookup scope",
                ..
            }
        ),
        "unknown scope is an Unknown diagnostic: {error:?}"
    );
}
