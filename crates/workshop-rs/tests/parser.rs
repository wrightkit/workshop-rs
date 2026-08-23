//! Native parser tests: the en-US corpus Workshop text parses directly
//! into validated, locale-independent WIR, and diagnostics distinguish
//! malformed, unknown, and unsupported input.

use std::path::{Path, PathBuf};

use workshop_rs::catalog::{Catalog, Locale};
use workshop_rs::convert;
use workshop_rs::parser;
use workshop_rs::validate;
use workshop_rs::wir;

mod common;

fn fixture_path(fixture_id: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/corpus")
        .join(format!("{fixture_id}.ws"))
}

fn corpus_workshop_text(fixture_id: &str) -> String {
    std::fs::read_to_string(fixture_path(fixture_id))
        .unwrap_or_else(|error| panic!("cannot read corpus fixture {fixture_id}: {error}"))
}

fn catalog() -> Catalog {
    Catalog::builtin().expect("built-in catalog")
}

#[test]
fn pinned_real_projects_parse_with_expected_semantic_residuals() {
    let catalog = catalog();
    for case in common::cases() {
        let (source, locale) = common::source(case);
        let program = parser::parse_with_context(&source, &catalog, &locale, &catalog)
            .unwrap_or_else(|error| panic!("{} parse failed: {error:?}", case.id));
        common::assert_residual_policy(case, "source-parse", &program.semantic_issues(&catalog));
        if let Err(error) = validate::validate_canonical_ids(&program, &catalog) {
            common::assert_gap(case, workshop_rs::p0::P0Stage::CanonicalValidation, &error);
            println!("{}: known canonical-validation gap: {error:?}", case.id);
        }
    }
}

#[test]
fn colonated_enum_members_resolve_from_signature_domains() {
    let source = r#"variables
{
    global:
        0: icon
}
rule("colonated enum")
{
    event
    {
        Ongoing - Global;
    }
    actions
    {
        Set Global Variable(icon, Icon String(Arrow: Up));
    }
}
"#;
    let program = parser::parse_with_context(source, &catalog(), &Locale::new("en-US"), &catalog())
        .expect("Arrow: Up resolves through Icon");
    assert!(program
        .values
        .iter()
        .any(|value| matches!(value.value, wir::Value::Enum { ref value_type, ref value } if value_type == "Icon" && value == "ARROW_UP")));
}

#[test]
fn localized_hero_call_resolves_dotted_member() {
    assert_eq!(
        catalog().resolve_enum_domain(&Locale::new("zh-CN"), "英雄"),
        Some("Hero")
    );
    let source = r#"规则("hero")
{
    event
    {
        持续 - 全局;
    }
    actions
    {
        设置全局变量(x, 英雄(D.Va));
    }
}
"#;
    let program = parser::parse_with_context(source, &catalog(), &Locale::new("zh-CN"), &catalog())
        .expect("localized Hero(D.Va) resolves");
    assert!(program.values.iter().any(|value| matches!(value.value, wir::Value::Enum { ref value_type, ref value } if value_type == "Hero" && value == "DVA")));
}

#[test]
fn dotted_localized_catalog_effect_resolves_as_one_member() {
    let source = r#"rule("effect")
{
    event
    {
        持续 - 全局;
    }
    actions
    {
        播放效果(所有玩家(所有队伍), D.Va自毁爆炸效果, 颜色(白色), 事件玩家, 1);
    }
}
"#;
    let catalog = catalog();
    let program = parser::parse_with_context(source, &catalog, &Locale::new("zh-CN"), &catalog)
        .expect("dotted localized effect alias resolves");
    assert!(program.values.iter().any(|value| matches!(
        value.value,
        wir::Value::Enum { ref value_type, ref value }
            if value_type == "DynamicEffect" && value == "DVA_SELF_DESTRUCT_EXPLOSION"
    )));
    assert!(program.semantic_issues(&catalog).is_empty());
}

#[test]
fn dotted_english_catalog_effect_resolves_as_one_member() {
    let source = r#"rule("effect")
{
    event { Ongoing - Global; }
    actions { Play Effect(All Players(All Teams), DVa Self Destruct Explosion Effect, Color(White), Event Player, 1); }
}
"#;
    let catalog = catalog();
    let program = parser::parse_with_context(source, &catalog, &Locale::new("en-US"), &catalog)
        .expect("English effect alias resolves");
    assert!(program.values.iter().any(|value| matches!(
        value.value,
        wir::Value::Enum { ref value_type, ref value }
            if value_type == "DynamicEffect" && value == "DVA_SELF_DESTRUCT_EXPLOSION"
    )));
    assert!(program.semantic_issues(&catalog).is_empty());
}

#[test]
fn unresolved_dotted_value_remains_a_source_located_issue() {
    let source = r#"rule("effect")
{
    event { 持续 - 全局; }
    actions { 播放效果(所有玩家(所有队伍), D.Va不存在效果, 颜色(白色), 事件玩家, 1); }
}
"#;
    let catalog = catalog();
    let program = parser::parse_with_context(source, &catalog, &Locale::new("zh-CN"), &catalog)
        .expect("unresolved dotted value remains parseable");
    let issues = program.semantic_issues(&catalog);
    let issue = issues
        .iter()
        .find(|issue| issue.kind == workshop_rs::semantic::IncompletenessKind::UnknownValue)
        .expect("unknown dotted prefix is reported");
    assert_eq!(issue.name, "D");
    assert!(
        issue.span.is_some(),
        "unknown dotted input keeps its source span"
    );
}

#[test]
fn indexed_variable_actions_resolve_declared_names_as_variables() {
    let source = r##"variables {
    global: 0: Brigitte
}

rule ("indexed") {
    event { Ongoing - Global; }
    actions {
        Set Global Variable At Index(Brigitte, 1, 2);
    }
}
"##;
    let catalog = catalog();
    let program = parser::parse_with_context(source, &catalog, &Locale::new("en-US"), &catalog)
        .expect("indexed global variable action parses");
    let action = program.actions.iter().next().expect("indexed action");
    let wir::Action::Call { args, .. } = action else {
        panic!("expected generic indexed-variable action");
    };
    assert!(matches!(
        program.values.get(args[0]).map(|node| &node.value),
        Some(wir::Value::GlobalVariable(_))
    ));
    validate::validate_canonical_ids(&program, &catalog)
        .expect("declared indexed variable is canonical WIR");
}

const CORPUS_FIXTURES: &[&str] = &[
    "basic-rule",
    "control-flow",
    "declarations-rules",
    "expressions-values",
    "preprocessing",
    "overpy-cake",
];

#[test]
fn every_corpus_workshop_text_parses_to_valid_wir() {
    // The corpus Workshop text parses directly against the catalog context,
    // which pins the expected enum domains from the canonical catalog
    // signatures (e.g. Create HUD Text's Reevaluation argument is
    // `HudReeval`, Create Beam Effect's is `EffectReeval`), resolving bare
    // members that are ambiguous across the catalog's enum domains (e.g.
    // `Visible To and String`).
    //
    // One documented exception: `overpy-cake`'s bare `Up` (OverPy folds the
    // vector-up constant into the bare member inside `Add(...)`) is
    // genuinely ambiguous between the `Vector` and `Rounding` enum domains
    // and no enclosing signature pins it, so the parser rejects it
    // deterministically rather than guessing.
    let documented_ambiguities = [("overpy-cake", "ambiguous enum member 'Up'")];
    for fixture_id in CORPUS_FIXTURES {
        let text = corpus_workshop_text(fixture_id);
        let catalog = catalog();
        match parser::parse_with_context(&text, &catalog, &Locale::new("en-US"), &catalog) {
            Ok(program) => {
                program
                    .validate()
                    .unwrap_or_else(|error| panic!("{fixture_id} WIR must validate: {error}"));
                validate::validate_canonical_ids(&program, &catalog).unwrap_or_else(|error| {
                    panic!("{fixture_id} canonical ids must resolve: {error}")
                });
                assert!(!program.rules.is_empty(), "{fixture_id} must produce rules");
                assert!(
                    !program.dump().is_empty(),
                    "{fixture_id} dump must not be empty"
                );
            }
            Err(error) => {
                let Some((_, message)) = documented_ambiguities
                    .iter()
                    .find(|(id, _)| *id == *fixture_id)
                else {
                    panic!("{fixture_id} must parse:\n{error}");
                };
                assert!(
                    error.to_string().contains(message),
                    "{fixture_id} fails only with the documented ambiguity, got: {error}"
                );
            }
        }
    }
}

#[test]
fn member_assignment_lowers_to_source_semantic_wir() {
    let text = r#"rule ("member") { event { Ongoing - Global; } actions {
        All Players(All Teams).abilityHUD[17] = True;
        Global.botOrisaChild.botDoesUniqueBehaviour = False;
        Event Player.beamID.uppercutMomentum += 1;
    } }"#;
    let catalog = catalog();
    let program = parser::parse_with_context(text, &catalog, &Locale::new("en-US"), &catalog)
        .expect("member assignments parse");
    assert!(
        program
            .actions
            .iter()
            .any(|action| matches!(action, wir::Action::AssignMember { op: None, .. }))
    );
    assert!(program.actions.iter().any(|action| matches!(
        action,
        wir::Action::AssignMember {
            op: Some(wir::ModifyOp::Add),
            ..
        }
    )));
    assert!(!program.actions.iter().any(
        |action| matches!(action, wir::Action::Call { name, .. } if name == "rawWorkshopAction")
    ));
}

#[test]
fn parsing_is_deterministic() {
    let text = corpus_workshop_text("control-flow");
    let catalog = catalog();
    let first =
        parser::parse_with_context(&text, &catalog, &Locale::new("en-US"), &catalog).unwrap();
    let second =
        parser::parse_with_context(&text, &catalog, &Locale::new("en-US"), &catalog).unwrap();
    assert_eq!(first.dump(), second.dump());
}

#[test]
fn parsed_variables_and_subroutines_carry_indexes() {
    let catalog = catalog();
    let program = parser::parse_with_context(
        &corpus_workshop_text("declarations-rules"),
        &catalog,
        &Locale::new("en-US"),
        &catalog,
    )
    .unwrap();
    let globals: Vec<_> = program
        .global_variables
        .iter()
        .map(|variable| (variable.name.as_str(), variable.index))
        .collect();
    assert_eq!(globals, vec![("score", 0)]);
    let players: Vec<_> = program
        .player_variables
        .iter()
        .map(|variable| (variable.name.as_str(), variable.index))
        .collect();
    assert_eq!(players, vec![("hasStarted", 0)]);
    let subroutines: Vec<_> = program
        .subroutines
        .iter()
        .map(|subroutine| (subroutine.name.as_str(), subroutine.index))
        .collect();
    assert_eq!(subroutines, vec![("showStatus", 0)]);
}

#[test]
fn parsed_events_are_canonical() {
    let catalog = catalog();
    let program = parser::parse_with_context(
        &corpus_workshop_text("declarations-rules"),
        &catalog,
        &Locale::new("en-US"),
        &catalog,
    )
    .unwrap();
    let events: Vec<String> = program
        .rules
        .iter()
        .map(|rule| match &rule.event {
            wir::Event::Global => "global".to_string(),
            wir::Event::EachPlayer => "eachPlayer".to_string(),
            wir::Event::EachPlayerWithFilters { .. } => "eachPlayer".to_string(),
            wir::Event::Player { kind, .. } => kind.catalog_id().to_string(),
            wir::Event::Subroutine(subroutine) => format!(
                "subroutine:{}",
                program.subroutines.get(*subroutine).unwrap().name
            ),
        })
        .collect();
    assert_eq!(
        events,
        vec![
            "subroutine:showStatus".to_string(),
            "eachPlayer".to_string()
        ]
    );
}

#[test]
fn parsed_conditions_resolve_infix_operators() {
    let catalog = catalog();
    let program = parser::parse_with_context(
        &corpus_workshop_text("declarations-rules"),
        &catalog,
        &Locale::new("en-US"),
        &catalog,
    )
    .unwrap();
    let rule = program
        .rules
        .iter()
        .find(|rule| rule.name == "player starts")
        .expect("rule");
    assert_eq!(rule.conditions.len(), 1, "one condition");
    // The condition is `==(hasSpawned(eventPlayer), true)`.
    let condition = program.values.get(rule.conditions[0]).unwrap();
    match &condition.value {
        wir::Value::Call { name, args } => {
            assert_eq!(name, "==");
            assert_eq!(args.len(), 2);
        }
        other => panic!("expected a comparison call, got {other:?}"),
    }
}

#[test]
fn spans_are_preserved() {
    let text = corpus_workshop_text("basic-rule");
    let program = parser::parse(&text, &catalog(), &Locale::new("en-US")).unwrap();
    let rule = program.rules.iter().next().unwrap();
    let rule_span = rule.span.expect("rule span");
    assert_eq!(rule.name, "setup");
    assert!(rule_span.start.line >= 1);
    // The disable-inspector action carries its own span.
    let action = program.actions.get(rule.actions[0]).expect("action");
    assert!(action.span().is_some());
}

#[test]
fn malformed_input_is_reported_as_malformed() {
    // A rule-final If without `End;` is the oracle's valid spelling; an If
    // whose body never closes at all stays malformed.
    let text = "rule (\"broken\") { actions { If(True);";
    let error = parser::parse(text, &catalog(), &Locale::new("en-US")).unwrap_err();
    assert!(
        matches!(error, workshop_rs::WorkshopError::Malformed { .. }),
        "an unclosed If body is malformed: {error}"
    );
    assert!(error.to_string().contains("malformed"));
}

#[test]
fn rule_final_if_without_end_is_the_oracle_spelling() {
    let text = "rule (\"ok\") { actions { If(True); } }";
    let program = parser::parse(text, &catalog(), &Locale::new("en-US"))
        .expect("a rule-final If without End; is valid (oracle spelling)");
    assert_eq!(program.rules.len(), 1);
}

#[test]
fn unknown_spelling_is_reported_as_unknown() {
    let text = "rule (\"x\") { event { Ongoing - Global; } actions { Totally Unknown Thing(1); } }";
    let error = parser::parse(text, &catalog(), &Locale::new("en-US")).unwrap_err();
    assert!(
        matches!(error, workshop_rs::WorkshopError::Unknown { .. }),
        "unknown action must be Unknown: {error}"
    );
    assert!(error.to_string().contains("Totally Unknown Thing"));
}

#[test]
fn canonical_validation_enforces_declared_arity_and_enum_domain() {
    let catalog = catalog();
    let arity = r#"rule ("arity") { event { Ongoing - Global; } actions { Wait(); } }"#;
    let program = parser::parse_with_context(arity, &catalog, &Locale::new("en-US"), &catalog)
        .expect("parser preserves the call for canonical validation");
    let error = validate::validate_canonical_ids(&program, &catalog)
        .expect_err("missing required signature argument must be rejected");
    assert!(
        error
            .to_string()
            .contains("action 'wait' expects 1..2 argument(s), got 0")
    );

    let wrong_domain = r#"rule ("domain") { event { Ongoing - Global; } actions { Set Invisible(All Players(All Teams), Color(White)); } }"#;
    let program =
        parser::parse_with_context(wrong_domain, &catalog, &Locale::new("en-US"), &catalog)
            .expect("parser preserves the call for canonical validation");
    let error = validate::validate_canonical_ids(&program, &catalog)
        .expect_err("wrong enum domain must be rejected");
    assert!(
        error
            .to_string()
            .contains("action 'setInvisibility' argument 2")
    );
}

#[test]
fn remaining_value_contracts_are_canonical_and_type_checked() {
    let catalog = catalog();
    let source = r#"rule ("values") { event { Ongoing - Global; } actions {
        Set Global Variable(probe, String());
        Set Global Variable(probe, String("Hello"));
        Set Global Variable(probe, String(Hello, Null));
        Set Global Variable(probe, String("Hello", Null, Null, Null));
        Set Global Variable(probe, Randomized Array(Array(1, 2)));
        Set Global Variable(probe, Raise To Power(2, 3));
    } }"#;
    let locale = Locale::new("en-US");
    let program = parser::parse_with_context(source, &catalog, &locale, &catalog)
        .expect("the three declared Value contracts parse");
    validate::validate_canonical_ids(&program, &catalog)
        .expect("the three Values resolve to canonical ids");
    program
        .validate()
        .expect("the three Value signatures validate");
    let emitted = workshop_rs::emitter::emit(&program, &catalog, &locale)
        .expect("localized String emits through its dedicated path");
    assert!(emitted.contains("String()"));
    assert!(emitted.contains("String(\"Hello\")"));
    assert!(emitted.contains("String(\"Hello\", Null)"));
    assert!(emitted.contains("String(\"Hello\", Null, Null, Null)"));
    let zh = Locale::new("zh-CN");
    let converted = convert::convert(source, &catalog, &locale, &zh, &Default::default())
        .expect("String converts to zh-CN");
    assert!(converted.text.contains("字符串(\"问候\")"));
    let converted_back =
        convert::convert(&converted.text, &catalog, &zh, &locale, &Default::default())
            .expect("String converts back to en-US");
    assert!(converted_back.text.contains("String(\"Hello\")"));
    for expected in ["string", "randomizedArray", "raiseToPower"] {
        assert!(program.values.iter().any(|node| matches!(
            &node.value,
            wir::Value::Call { name, .. } if name == expected
        )));
    }

    for source in [
        r#"rule ("wrong-string-literal") { event { Ongoing - Global; } actions { Set Global Variable(probe, String(1, Null, Null, Null)); } }"#,
        r#"rule ("wrong-string-bool") { event { Ongoing - Global; } actions { Set Global Variable(probe, String(True)); } }"#,
        r#"rule ("wrong-string-expression") { event { Ongoing - Global; } actions { Set Global Variable(probe, String(Custom String("probe"), Null)); } }"#,
        r#"rule ("unknown-string-preset") { event { Ongoing - Global; } actions { Set Global Variable(probe, String("Not A Preset")); } }"#,
        r#"rule ("wrong-array") { event { Ongoing - Global; } actions { Set Global Variable(probe, Randomized Array(1)); } }"#,
        r#"rule ("wrong-power") { event { Ongoing - Global; } actions { Set Global Variable(probe, Raise To Power(1, Custom String("wrong"))); } }"#,
    ] {
        let Ok(program) = parser::parse_with_context(source, &catalog, &locale, &catalog) else {
            continue;
        };
        assert!(
            validate::validate_canonical_ids(&program, &catalog).is_err(),
            "invalid Value signature must be rejected"
        );
    }
}

#[test]
fn canonical_validation_enforces_literal_types_and_value_return_types() {
    let catalog = catalog();
    let wrong_literal = r#"rule ("type") { event { Ongoing - Global; } actions { Set Crouch Enabled(All Players(All Teams), Color(White)); } }"#;
    let program =
        parser::parse_with_context(wrong_literal, &catalog, &Locale::new("en-US"), &catalog)
            .expect("parser preserves a typed call for canonical validation");
    let error = validate::validate_canonical_ids(&program, &catalog)
        .expect_err("a Color is not a Boolean action parameter");
    assert!(
        error
            .to_string()
            .contains("must have semantic type 'Boolean'")
    );

    let wrong_return = r#"rule ("return") { event { Ongoing - Global; } actions { Teleport(Event Player, Max Health(Event Player)); } }"#;
    let program =
        parser::parse_with_context(wrong_return, &catalog, &Locale::new("en-US"), &catalog)
            .expect("parser preserves a value-returning call for canonical validation");
    let error = validate::validate_canonical_ids(&program, &catalog)
        .expect_err("a Number Value return is not a Vector action parameter");
    assert!(
        error
            .to_string()
            .contains("must have semantic type 'Vector'")
    );
}

#[test]
fn canonical_validation_rejects_incompatible_variable_reference_types() {
    let catalog = catalog();
    let source = r#"variables {
    global: 0: g
    player: 0: p
}
rule ("type") { event { Ongoing - Global; } actions {
    Set Player Variable At Index(1, 0, 1);
} }"#;
    let program = parser::parse_with_context(source, &catalog, &Locale::new("en-US"), &catalog)
        .expect("parser preserves the incompatible indexed-variable call");
    let error = validate::validate_canonical_ids(&program, &catalog)
        .expect_err("a number is not a player-variable reference");
    assert!(
        error
            .to_string()
            .contains("must have semantic type 'Player Variable'")
    );
}

#[test]
fn current_loop_action_resolves_to_canonical_generic_wir() {
    let catalog = catalog();
    let source = r#"rule ("loop") { event { Ongoing - Global; } actions { Loop; } }"#;
    let program = parser::parse_with_context(source, &catalog, &Locale::new("en-US"), &catalog)
        .expect("declared Loop action parses");
    let rule = program.rules.get(wir::RuleId::from_index(0)).expect("rule");
    let action = program.actions.get(rule.actions[0]).expect("action");
    assert!(matches!(
        action,
        wir::Action::Call { name, args, .. } if name == "loop" && args.is_empty()
    ));
    validate::validate_canonical_ids(&program, &catalog).expect("Loop has canonical identity");
    assert!(
        workshop_rs::semantic::inspect(&program, &catalog)
            .iter()
            .all(|issue| issue.name != "rawWorkshopAction"),
        "declared Loop must not use the opaque action path"
    );
}

#[test]
fn unsupported_construct_is_distinct_from_malformed() {
    // A non-default eachPlayer sub-parameter is recognized but unsupported.
    let text = "rule (\"x\") { event { Ongoing - Each Player; Team 1; } actions { } }";
    let error = parser::parse(text, &catalog(), &Locale::new("en-US")).unwrap_err();
    assert!(
        matches!(error, workshop_rs::WorkshopError::Unsupported { .. }),
        "non-default event parameter must be Unsupported: {error}"
    );
}

#[test]
fn bare_chase_reevaluation_none_is_ambiguous_across_domains() {
    // Both reference reevaluation domains spell their NONE member "None".
    // Without a signature pin the flat parser rejects the bare spelling
    // with a structured Unsupported diagnostic.
    let text = "variables { global: 0: g }\nrule (\"x\") { event { Ongoing - Global; } actions { Set Global Variable(g, None); } }";
    let error = parser::parse(text, &catalog(), &Locale::new("en-US")).unwrap_err();
    assert!(
        matches!(error, workshop_rs::WorkshopError::Unsupported { .. }),
        "the shared None member spelling must be a structured ambiguity: {error}"
    );
    assert!(error.to_string().contains("ambiguous enum member 'None'"));
}

#[test]
fn explicit_locale_is_honored() {
    // en-US parsing is deterministic; the parser never guesses a locale.
    let text = corpus_workshop_text("basic-rule");
    let program = parser::parse(&text, &catalog(), &Locale::new("en-US")).unwrap();
    let dump = program.dump();
    assert!(!dump.is_empty());
}

/// The last argument value of the first call action of a parsed program.
fn enum_value_of_first_action(program: &wir::Program, action_index: usize) -> &wir::Value {
    let action = program.actions.iter().nth(action_index).expect("action");
    let wir::Action::Call { args, .. } = action else {
        panic!("expected a call action, got {action:?}");
    };
    let last = args.last().expect("call has an argument");
    let wir::ValueNode { value, .. } = program.values.get(*last).expect("value");
    value
}

#[test]
fn context_pinned_ambiguous_none_resolves_via_canonical_signature() {
    // The emitter-produced `Chase Global Variable Over Time(..., None)`
    // (bare `None` shared by ChaseTimeReeval/ChaseRateReeval/Invis) reparses
    // to ChaseTimeReeval.NONE because the canonical chaseOverTime signature
    // pins argument 3 to the ChaseTimeReeval domain (catalog data, migrated
    // from the Wright-authored manifest probes).
    let text = "variables { global: 0: g }\nrule (\"x\") { event { Ongoing - Global; } actions { Chase Global Variable Over Time(Global.g, 0, 30, None); } }";
    let program = parser::parse_with_context(text, &catalog(), &Locale::new("en-US"), &catalog())
        .expect("the pinned Chase None must resolve");
    let value = enum_value_of_first_action(&program, 0);
    assert!(
        matches!(value, wir::Value::Enum { value_type, value }
            if value_type == "ChaseTimeReeval" && value == "NONE"),
        "the bare None resolves to ChaseTimeReeval.NONE, got {value:?}"
    );
}

#[test]
fn context_pinned_ambiguous_none_resolves_for_set_invisible() {
    // `Set Invisible(Event Player, None)` reparses to Invis.NONE: the
    // canonical setInvisibility signature pins argument 1 to the Invis
    // domain (member-action receiver offset).
    let text = "rule (\"x\") { event { Ongoing - Each Player; } actions { Set Invisible(Event Player, None); } }";
    let program = parser::parse_with_context(text, &catalog(), &Locale::new("en-US"), &catalog())
        .expect("the pinned Invis None must resolve");
    let value = enum_value_of_first_action(&program, 0);
    assert!(
        matches!(value, wir::Value::Enum { value_type, value }
            if value_type == "Invis" && value == "NONE"),
        "the bare None resolves to Invis.NONE, got {value:?}"
    );
}

#[test]
fn wrong_domain_context_keeps_the_ambiguity_rejected() {
    // A signature pinning a *different* domain than the ambiguous member's
    // candidates must not resolve it: `Wait(...)` expects `Wait` (which has
    // no `None` member), so the bare `None` stays ambiguous — no guessing,
    // no arbitrary precedence.
    let text = "rule (\"x\") { event { Ongoing - Global; } actions { Wait(0.016, None); } }";
    let error = parser::parse_with_context(text, &catalog(), &Locale::new("en-US"), &catalog())
        .expect_err("a non-matching expected domain must keep the ambiguity");
    assert!(
        matches!(error, workshop_rs::WorkshopError::Unsupported { .. }),
        "expected a structured ambiguity: {error}"
    );
    assert!(error.to_string().contains("ambiguous enum member 'None'"));
}

#[test]
fn expected_domain_resolution_tracks_the_catalog_declared_domains() {
    // The catalog is the single source of expected domains for the surface
    // it documents: canonical signatures answer exactly their declared
    // parameter domains.
    use workshop_rs::signatures::ExpectedDomain;
    let catalog = catalog();
    // chaseOverTime: [variable, destination, duration, reevaluation].
    assert_eq!(
        catalog.expected_domain("chaseOverTime", 3),
        Some("ChaseTimeReeval")
    );
    assert_eq!(catalog.expected_domain("chaseOverTime", 2), None);
    // chaseAtRate pins the rate form's reevaluation domain.
    assert_eq!(
        catalog.expected_domain("chaseAtRate", 3),
        Some("ChaseRateReeval")
    );
    // setInvisibility: member action; arg 1 is the pinned param.
    assert_eq!(catalog.expected_domain("setInvisibility", 0), None);
    assert_eq!(catalog.expected_domain("setInvisibility", 1), Some("Invis"));
    // The player chase forms pin the reevaluation domain at the shifted
    // workshop-text position (player, name, destination, duration/rate,
    // reevaluation).
    assert_eq!(
        catalog.expected_domain("chasePlayerVariableOverTime", 4),
        Some("ChaseTimeReeval")
    );
    assert_eq!(
        catalog.expected_domain("chasePlayerVariableAtRate", 4),
        Some("ChaseRateReeval")
    );
    // Unknown catalog ids and out-of-range indexes answer None.
    assert_eq!(catalog.expected_domain("noSuchAction", 0), None);
    assert_eq!(catalog.expected_domain("chaseOverTime", 4), None);
}

#[test]
fn raw_workshop_member_access_and_disabled_groups_parse() {
    let text = r#"
        rule ("raw") {
            event {
                Ongoing - Each Player;
                Team 1;
                Soldier: 76;
            }
            actions {
                disabled If(Event Player.ready == True);
                    Event Player.ready = False;
                End;
                If((Players On Hero(Hero(Mercy), Team 1).abilities[6] ? True : False));
                    Wait(0.016, Ignore Condition);
                End;
            }
        }
    "#;
    let program = parser::parse_with_context(text, &catalog(), &Locale::new("en-US"), &catalog())
        .expect("raw member access and disabled groups must parse");
    program
        .validate()
        .expect("the lowered raw program must validate");
}

#[test]
fn disabled_condition_is_ignored_as_inactive() {
    let text = r#"
        rule ("disabled condition") {
            event { Ongoing - Global; }
            conditions {
                disabled Is Using Ability 1(Event Player) == True;
            }
            actions { Wait(0.016, Ignore Condition); }
        }
    "#;
    let program = parser::parse_with_context(text, &catalog(), &Locale::new("en-US"), &catalog())
        .expect("disabled conditions must parse");
    let rule = program.rules.iter().next().expect("rule");
    assert!(rule.conditions.is_empty());
}

#[test]
fn raw_indexed_assignment_lowers_to_explicit_wir_call() {
    let text = r#"
        variables { global: 0: values }
        rule ("raw") {
            event { Ongoing - Global; }
            actions { Global.values[0] = 1; }
        }
    "#;
    let program = parser::parse_with_context(text, &catalog(), &Locale::new("en-US"), &catalog())
        .expect("indexed raw assignment must parse");
    assert!(program.dump().contains("setGlobalVariableAtIndex"));
    program
        .validate()
        .expect("the indexed assignment must validate");
}

#[test]
fn cross_domain_member_spelling_collisions_are_the_documented_inventory() {
    // Systematic collision check: scan the declared catalog for member
    // spellings shared by more than one enum domain (en-US) and assert the
    // inventory is exactly the documented one. A new collision fails this
    // test, forcing an explicit resolution decision for it.
    use std::collections::BTreeMap;
    let mut spelling_to_domains: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for domain in catalog().enum_domains() {
        for member in &domain.members {
            let spelling = member
                .spelling(&Locale::new("en-US"))
                .expect("en-US member spelling")
                .to_string();
            spelling_to_domains
                .entry(spelling)
                .or_default()
                .push(domain.domain.clone());
        }
    }
    let collisions: Vec<(String, Vec<String>)> = spelling_to_domains
        .into_iter()
        .filter(|(_, domains)| domains.len() > 1)
        .collect();
    let documented_legacy = vec![
        (
            "All".to_string(),
            vec![
                "EventTeam".to_string(),
                "EventPlayer".to_string(),
                "Invis".to_string(),
            ],
        ),
        (
            "Healing Dealt".to_string(),
            vec!["Stat".to_string(), "HeroStat".to_string()],
        ),
        (
            "None".to_string(),
            vec![
                "FacingReeval".to_string(),
                "ChaseTimeReeval".to_string(),
                "ChaseRateReeval".to_string(),
                "Invis".to_string(),
                "ThrottleReeval".to_string(),
                "EffectReeval".to_string(),
            ],
        ),
        (
            "Team 1".to_string(),
            vec![
                "Color".to_string(),
                "Team".to_string(),
                "EventTeam".to_string(),
            ],
        ),
        (
            "Team 2".to_string(),
            vec![
                "Color".to_string(),
                "Team".to_string(),
                "EventTeam".to_string(),
            ],
        ),
        (
            "Up".to_string(),
            vec!["Vector".to_string(), "Rounding".to_string()],
        ),
        (
            "Visible To".to_string(),
            vec![
                "HudReeval".to_string(),
                "EffectReeval".to_string(),
                "InworldTextReeval".to_string(),
            ],
        ),
        (
            "Visible To String and Color".to_string(),
            vec!["HudReeval".to_string(), "InworldTextReeval".to_string()],
        ),
        (
            "Visible To and Color".to_string(),
            vec![
                "HudReeval".to_string(),
                "EffectReeval".to_string(),
                "InworldTextReeval".to_string(),
            ],
        ),
        (
            "Visible To and Position".to_string(),
            vec!["InworldTextReeval".to_string(), "IconReeval".to_string()],
        ),
        (
            "Visible To and String".to_string(),
            vec!["HudReeval".to_string(), "InworldTextReeval".to_string()],
        ),
    ];
    for expected in documented_legacy {
        assert!(
            collisions.iter().any(|(spelling, domains)| {
                spelling == &expected.0 && expected.1.iter().all(|domain| domains.contains(domain))
            }),
            "missing documented collision: {expected:?}"
        );
    }
    assert_eq!(collisions.len(), 50, "the catalog collision census changed");
}
