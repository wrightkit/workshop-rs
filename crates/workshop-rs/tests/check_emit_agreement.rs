mod common;

use common::{catalog, en};
use workshop_rs::settings::{DiagnosticSeverity, Settings, SettingsNode, check_emission};
use workshop_rs::{Program, emitter, parser};

#[derive(Debug, PartialEq, Eq)]
enum Verdict {
    ParseRejected,
    Accepted,
    Rejected,
}

#[derive(Debug)]
struct Outcome {
    verdict: Verdict,
    check_errors: Vec<String>,
    warnings: Vec<String>,
    emit: Option<Result<String, String>>,
    validate_ok: bool,
}

fn run_source(source: &str) -> Outcome {
    let catalog = catalog();
    let en = en();
    let program = match parser::parse(source, &catalog, &en) {
        Err(_) => {
            return Outcome {
                verdict: Verdict::ParseRejected,
                check_errors: Vec::new(),
                warnings: Vec::new(),
                emit: None,
                validate_ok: false,
            };
        }
        Ok(program) => program,
    };
    let diagnostics = program.settings_diagnostics();
    let check_errors = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error)
        .map(|diagnostic| diagnostic.error.to_string())
        .collect::<Vec<_>>();
    let warnings = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == DiagnosticSeverity::Warning)
        .map(|diagnostic| diagnostic.error.to_string())
        .collect::<Vec<_>>();
    let validate_ok = program.validate().is_ok();
    let emit = Some(emitter::emit(&program, &catalog, &en).map_err(|error| error.to_string()));
    Outcome {
        verdict: if check_errors.is_empty() {
            Verdict::Accepted
        } else {
            Verdict::Rejected
        },
        check_errors,
        warnings,
        emit,
        validate_ok,
    }
}

fn run_tree(children: Vec<SettingsNode>) -> Outcome {
    let catalog = catalog();
    let en = en();
    let mut program = Program::new();
    program.settings = Some(Settings {
        span: None,
        children,
    });
    let check_errors: Vec<String> = program
        .settings
        .as_ref()
        .map(|settings| {
            check_emission(settings)
                .iter()
                .map(|e| e.to_string())
                .collect()
        })
        .unwrap_or_default();
    let validate_ok = program.validate().is_ok();
    let emit = Some(emitter::emit(&program, &catalog, &en).map_err(|error| error.to_string()));
    Outcome {
        verdict: if check_errors.is_empty() {
            Verdict::Accepted
        } else {
            Verdict::Rejected
        },
        check_errors,
        warnings: Vec::new(),
        emit,
        validate_ok,
    }
}

fn assert_agree(case: &str, outcome: &Outcome) {
    let Some(emit) = &outcome.emit else {
        return;
    };
    let emit_ok = emit.is_ok();
    let check_ok = outcome.check_errors.is_empty();
    assert_eq!(
        check_ok, emit_ok,
        "{case}: check and emit disagree — check errors: {:?}, emit: {:?}",
        outcome.check_errors, emit
    );
    assert_eq!(
        outcome.validate_ok, check_ok,
        "{case}: validate and settings diagnostics disagree — validate_ok: \
         {}, check errors: {:?}",
        outcome.validate_ok, outcome.check_errors
    );
}

fn assert_source(case: &str, source: &str, expected: Verdict) -> Outcome {
    let outcome = run_source(source);
    assert_eq!(
        outcome.verdict, expected,
        "{case}: expected {expected:?}, got {outcome:?}"
    );
    assert_agree(case, &outcome);
    outcome
}

fn assert_tree(case: &str, children: Vec<SettingsNode>, expected: Verdict) -> Outcome {
    let outcome = run_tree(children);
    assert_eq!(
        outcome.verdict, expected,
        "{case}: expected {expected:?}, got {outcome:?}"
    );
    assert_agree(case, &outcome);
    outcome
}

fn hero_source(hero: &str, member: &str) -> String {
    format!(
        "settings {{\n    heroes {{\n        General {{\n            {hero} {{\n                {member}\n            }}\n        }}\n    }}\n}}"
    )
}

fn group(name: &str, children: Vec<SettingsNode>) -> SettingsNode {
    SettingsNode::Group {
        name: name.into(),
        children,
        span: None,
    }
}

fn raw_group(name: &str, children: Vec<SettingsNode>) -> SettingsNode {
    SettingsNode::RawGroup {
        name: name.into(),
        children,
        span: None,
    }
}

fn raw(name: &str, value: &str) -> SettingsNode {
    SettingsNode::Raw {
        name: name.into(),
        value: value.into(),
        span: None,
    }
}

fn number(name: &str, value: f64) -> SettingsNode {
    SettingsNode::Number {
        name: name.into(),
        value,
        span: None,
    }
}

fn boolean(name: &str, value: bool) -> SettingsNode {
    SettingsNode::Bool {
        name: name.into(),
        value,
        span: None,
    }
}

fn heroes(children: Vec<SettingsNode>) -> Vec<SettingsNode> {
    vec![group("heroes", children)]
}

fn team(team: &str, children: Vec<SettingsNode>) -> SettingsNode {
    group(team, children)
}

#[test]
fn hero_members_without_a_spellable_slot_are_rejected_identically() {
    for member in [
        "enableAbility3: Off",
        "ability3Cooldown%: 50%",
        "enablePassive: Off",
    ] {
        let outcome = assert_source(
            &format!("doctrine/{member}"),
            &hero_source("doctrine", member),
            Verdict::Rejected,
        );
        assert!(
            outcome
                .check_errors
                .iter()
                .any(|error| error.contains("has no")),
            "check must name the missing slot: {:?}",
            outcome.check_errors
        );
        assert!(
            matches!(
                outcome.emit.as_ref().expect("emit ran"),
                Err(error) if error.contains("missing setting mapping")
            ),
            "emit must fail on the missing mapping: {:?}",
            outcome.emit
        );
    }

    assert_source(
        "doctrine/enableAbility3 block",
        &hero_source("doctrine", "enableAbility3 { leaf: 1 }"),
        Verdict::Rejected,
    );
}

#[test]
fn hero_members_with_a_spellable_slot_are_accepted_identically() {
    for (member, emitted_fragment) in [
        ("enableAbility1: Off", "Invigorating Drones: Off"),
        (
            "ability1Cooldown%: 50%",
            "Invigorating Drones Cooldown Time: 50%",
        ),
        ("Secondary Fire: Off", "Infuse: Off"),
        ("Health: 150%", "Health: 150%"),
        ("enableAbility1 { leaf: 1 }", "Invigorating Drones {"),
    ] {
        let outcome = assert_source(
            &format!("doctrine/{member}"),
            &hero_source("doctrine", member),
            Verdict::Accepted,
        );
        let emitted = outcome
            .emit
            .as_ref()
            .expect("emit ran")
            .as_ref()
            .expect("accepted");
        assert!(
            emitted.contains(emitted_fragment),
            "{member}: emitted text must carry {emitted_fragment:?}\n{emitted}"
        );
    }

    assert_source(
        "ana/enableAbility3",
        &hero_source("ana", "enableAbility3: Off"),
        Verdict::Accepted,
    );
    assert_source(
        "ana/enableAbility3 block",
        &hero_source("ana", "enableAbility3 { leaf: 1 }"),
        Verdict::Accepted,
    );
    let outcome = assert_source(
        "ana/Secondary Fire",
        &hero_source("ana", "Secondary Fire: Off"),
        Verdict::Accepted,
    );
    let emitted = outcome
        .emit
        .as_ref()
        .expect("emit ran")
        .as_ref()
        .expect("accepted");
    assert!(emitted.contains("enableSecondaryFire: Off"), "{emitted}");

    let outcome = assert_source(
        "brigitte/Shield Bash",
        &hero_source("brigitte", "Shield Bash: Off"),
        Verdict::Accepted,
    );
    let emitted = outcome
        .emit
        .as_ref()
        .expect("emit ran")
        .as_ref()
        .expect("accepted");
    assert!(emitted.contains("Shield Bash: Off"), "{emitted}");
}

#[test]
fn uncatalogued_hero_members_are_carried_identically() {
    for member in [
        "myCustomKey: 42",
        "Ability 3: Off",
        "customBlock { leaf: 1 }",
    ] {
        let outcome = assert_source(
            &format!("doctrine/{member}"),
            &hero_source("doctrine", member),
            Verdict::Accepted,
        );
        let emitted = outcome
            .emit
            .as_ref()
            .expect("emit ran")
            .as_ref()
            .expect("accepted");
        let fragment = member.split('{').next().unwrap_or(member).trim();
        assert!(
            emitted.contains(fragment),
            "{member}: carried member must emit as written\n{emitted}"
        );
    }
}

#[test]
fn carried_blocks_treat_catalogued_and_uncatalogued_keys_consistently() {
    let pairs: &[(&str, &str, &str)] = &[
        (
            "lobby",
            "settings { lobby { Map Rotation { a b } } }",
            "settings { lobby { customRotation { a b } } }",
        ),
        (
            "main",
            "settings { main { Description { a: 1 } } }",
            "settings { main { customMain { a: 1 } } }",
        ),
        (
            "extensions",
            "settings { extensions { Beam Effects { a } } }",
            "settings { extensions { customExt { a } } }",
        ),
        (
            "catalogued mode",
            "settings { modes { Skirmish { Score To Win { a: 1 } } } }",
            "settings { modes { Skirmish { customKey { a: 1 } } } }",
        ),
        (
            "uncatalogued mode",
            "settings { modes { Custom Mode { Score To Win { a } } } }",
            "settings { modes { Custom Mode { customKey { a } } } }",
        ),
        (
            "hero",
            "settings { heroes { General { Ana { Damage Dealt { a: 1 } } } } }",
            "settings { heroes { General { Ana { customBlock { a: 1 } } } } }",
        ),
        (
            "team",
            "settings { heroes { Team 1 { Damage Dealt { a: 1 } } } }",
            "settings { heroes { Team 1 { customKey { a: 1 } } } }",
        ),
        (
            "top level",
            "settings { lobby { a: 1 } }",
            "settings { customNs { a: 1 } }",
        ),
        (
            "workshop namespace",
            "settings { workshop { Custom { a: 1 } } }",
            "settings { workshop { customBlock { a: 1 } } }",
        ),
    ];
    for (position, catalogued, uncatalogued) in pairs {
        let catalogued_outcome = run_source(catalogued);
        let uncatalogued_outcome = run_source(uncatalogued);
        assert_agree(&format!("{position}/{catalogued}"), &catalogued_outcome);
        assert_agree(&format!("{position}/{uncatalogued}"), &uncatalogued_outcome);
        assert_eq!(
            catalogued_outcome.verdict, uncatalogued_outcome.verdict,
            "{position}: catalogued and uncatalogued keyed blocks must take \
             the same verdict\ncatalogued: {catalogued_outcome:?}\n\
             uncatalogued: {uncatalogued_outcome:?}"
        );
    }

    for (source, block_header) in [
        (
            "settings {\n    lobby {\n        Map Rotation {\n            a\n            b\n        }\n    }\n}",
            "Map Rotation {",
        ),
        (
            "settings {\n    lobby {\n        customRotation {\n            a\n            b\n        }\n    }\n}",
            "customRotation {",
        ),
    ] {
        let outcome = assert_source(source, source, Verdict::Accepted);
        let emitted = outcome
            .emit
            .as_ref()
            .expect("emit ran")
            .as_ref()
            .expect("accepted");
        assert!(emitted.contains(block_header), "{emitted}");
        for leaf in ["a", "b"] {
            assert!(
                emitted.lines().any(|line| line.trim() == leaf),
                "{emitted} missing carried leaf {leaf:?}"
            );
        }
    }
}

#[test]
fn catalogued_values_and_lists_agree_across_stages() {
    assert_source(
        "lobby/Map Rotation scalar",
        "settings { lobby { Map Rotation: After Every Game } }",
        Verdict::Accepted,
    );
    assert_source(
        "lobby/Map Rotation undeclared value",
        "settings { lobby { Map Rotation: Bogus Value } }",
        Verdict::Accepted,
    );
    assert_source(
        "skirmish/enabled maps declared element",
        "settings { modes { Skirmish { enabled maps { Route 66 } } } }",
        Verdict::Accepted,
    );
    assert_source(
        "skirmish/enabled maps variant-qualified element",
        "settings { modes { Skirmish { enabled maps { Workshop Chamber 972777519512064577 } } } }",
        Verdict::Accepted,
    );
    assert_source(
        "skirmish/enabled maps undeclared element",
        "settings { modes { Skirmish { enabled maps { Future Map } } } }",
        Verdict::Rejected,
    );
    assert_source(
        "skirmish/enabled maps undeclared qualified element",
        "settings { modes { Skirmish { enabled maps { Future Map 123 } } } }",
        Verdict::Rejected,
    );
    assert_source(
        "team/enabled heroes declared element",
        "settings { heroes { Team 1 { enabled heroes { Ana } } } }",
        Verdict::Accepted,
    );
    assert_source(
        "team/enabled heroes undeclared element",
        "settings { heroes { Team 1 { enabled heroes { NotAHero } } } }",
        Verdict::Rejected,
    );
}

#[test]
fn rejected_members_fail_check_and_emit_alike() {
    for source in [
        "settings { heroes { Team 1 { NotAHero { a } } } }",
        "settings { heroes { Team 1 { Damage Dealt { a } } } }",
    ] {
        let outcome = assert_source(source, source, Verdict::Rejected);
        assert!(
            outcome
                .check_errors
                .iter()
                .any(|error| error.contains("unknown hero")),
            "{source}: check must reject as an unknown hero: {:?}",
            outcome.check_errors
        );
        assert!(
            matches!(
                outcome.emit.as_ref().expect("emit ran"),
                Err(error) if error.contains("unknown hero")
            ),
            "{source}: emit must fail the same way: {:?}",
            outcome.emit
        );
    }

    assert_source(
        "unknown team",
        "settings { heroes { Bogus Team { Ana { Ability 1: On } } } }",
        Verdict::ParseRejected,
    );
}

#[test]
fn producer_trees_agree_on_pass_through_and_resolved_shapes() {
    assert_tree(
        "rawgroup under team",
        heroes(vec![team(
            "allTeams",
            vec![raw_group("customData", vec![raw("leaf", "1")])],
        )]),
        Verdict::Accepted,
    );
    assert_tree(
        "group under team",
        heroes(vec![team(
            "allTeams",
            vec![group("customData", vec![raw("leaf", "1")])],
        )]),
        Verdict::Rejected,
    );
    assert_tree(
        "rawgroup slot key under doctrine",
        heroes(vec![team(
            "allTeams",
            vec![group(
                "doctrine",
                vec![raw_group("enableAbility3", vec![raw("leaf", "1")])],
            )],
        )]),
        Verdict::Accepted,
    );
    assert_tree(
        "group slot key under doctrine",
        heroes(vec![team(
            "allTeams",
            vec![group(
                "doctrine",
                vec![group("enableAbility3", vec![raw("leaf", "1")])],
            )],
        )]),
        Verdict::Rejected,
    );
    for (case, children) in [
        (
            "rawgroup under modes",
            vec![group(
                "gamemodes",
                vec![raw_group("customData", vec![raw("leaf", "1")])],
            )],
        ),
        (
            "group uncatalogued mode",
            vec![group(
                "gamemodes",
                vec![group("Custom Mode", vec![raw("a", "1")])],
            )],
        ),
        (
            "rawgroup top level",
            vec![raw_group("customNs", vec![raw("a", "1")])],
        ),
        (
            "group named workshop",
            vec![group("workshop", vec![raw("a", "1")])],
        ),
    ] {
        assert_tree(case, children, Verdict::Accepted);
    }

    assert_tree(
        "raw slot key under doctrine",
        heroes(vec![team(
            "allTeams",
            vec![group("doctrine", vec![raw("enableAbility3", "Off")])],
        )]),
        Verdict::Accepted,
    );
    assert_tree(
        "typed slot key under doctrine",
        heroes(vec![team(
            "allTeams",
            vec![group("doctrine", vec![boolean("enableAbility3", false)])],
        )]),
        Verdict::Rejected,
    );
    assert_tree(
        "typed slot key under ana",
        heroes(vec![team(
            "allTeams",
            vec![group("ana", vec![boolean("enableAbility3", false)])],
        )]),
        Verdict::Accepted,
    );
}

#[test]
fn producer_trees_agree_on_unwritable_shapes() {
    for marker in [group, raw_group] {
        assert_tree(
            "typed leaf in carried block",
            vec![group(
                "lobby",
                vec![marker("customBlock", vec![number("x", 5.0)])],
            )],
            Verdict::Rejected,
        );
    }
    assert_tree(
        "list leaf in carried block",
        vec![group(
            "lobby",
            vec![group(
                "customBlock",
                vec![SettingsNode::List {
                    name: "enabledHeroes".into(),
                    elements: Vec::new(),
                    span: None,
                }],
            )],
        )],
        Verdict::Rejected,
    );
    assert_tree(
        "workshop with typed member",
        vec![SettingsNode::Workshop {
            children: vec![number("x", 5.0)],
            span: None,
        }],
        Verdict::Rejected,
    );
    assert_tree(
        "workshop node in lobby",
        vec![group(
            "lobby",
            vec![SettingsNode::Workshop {
                children: Vec::new(),
                span: None,
            }],
        )],
        Verdict::Rejected,
    );
    for (case, node) in [
        ("raw at top level", raw("stray", "1")),
        ("number at top level", number("x", 5.0)),
    ] {
        assert_tree(case, vec![node], Verdict::Rejected);
    }
    assert_tree(
        "kind mismatch at catalogued key",
        vec![group("lobby", vec![number("description", 5.0)])],
        Verdict::Rejected,
    );
}

#[test]
fn near_miss_carried_members_warn_but_agree() {
    let outcome = run_source("settings { lobby { Map Rotatio { a: 1 } } }");
    assert_agree("near-miss block", &outcome);
    assert_eq!(outcome.verdict, Verdict::Accepted);
    assert!(
        outcome
            .warnings
            .iter()
            .any(|warning| warning.contains("Map Rotation")),
        "the near-miss must warn toward the declared spelling: {:?}",
        outcome.warnings
    );
}

#[test]
fn accepted_sources_emit_reparseable_text() {
    for source in [
        "settings { lobby { Map Rotation { a b } } }",
        "settings { lobby { customRotation { a b } } }",
        "settings { heroes { General { Doctrine { ability1Cooldown%: 50% } } } }",
        "settings { modes { Skirmish { enabled maps { Route 66 } } } }",
        "settings { modes { Skirmish { enabled maps { Workshop Chamber 972777519512064577 } } } }",
        "settings { customNs { a: 1 } }",
        "settings { workshop { Custom { a: 1 } } }",
    ] {
        let outcome = assert_source(source, source, Verdict::Accepted);
        let emitted = outcome
            .emit
            .as_ref()
            .expect("emit ran")
            .as_ref()
            .expect("accepted");
        let reparsed = run_source(emitted);
        assert_agree(&format!("reparse/{source}"), &reparsed);
        assert_eq!(
            reparsed.verdict,
            Verdict::Accepted,
            "{source}: emitted text must reparse and pass check\n{emitted}"
        );
    }
}
