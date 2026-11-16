use workshop_rs::{compare, parser, validate};

use crate::common::{catalog, en, fixture_text, zh};

/// Each value form a client probe pasted into a typed slot, with the client's
/// verdict (`true` = accepted). The rounds and captures are in
/// `fixtures/slot-acceptance/README.md`. Validation must agree with the client
/// on every row (ADR-0014).
const CLIENT_RESULTS: &[(&str, bool)] = &[
    // Round 1.
    (r#"String Slice(Custom String("abcdef"), 1, 2)"#, true),
    ("String Slice(100, 1, 2)", false),
    ("String Slice(Add(Modulo(Global.t, 60), 100), 1, 2)", true),
    ("String Slice(Vector(1, 2, 3), 1, 2)", false),
    ("String Slice(Host Player, 1, 2)", true),
    ("Square Root(Host Player)", true),
    // Round 2.
    ("String Slice(Absolute Value(Global.t), 1, 2)", false),
    ("String Slice(Vector(Global.t, 0, 0), 1, 2)", false),
    ("String Slice(Add(100, 0), 1, 2)", true),
    ("String Slice(Global.t, 1, 2)", true),
    ("Square Root(Vector(1, 2, 3))", false),
    (r#"Square Root(Custom String("a"))"#, false),
];

const PROBES: &[&str] = &["round-1", "round-2"];

const CLIENT_EXPORTS: &[&str] = &[
    "client-export-round-1.zh-CN.ow",
    "client-export-round-2.zh-CN.ow",
];

#[test]
fn validation_agrees_with_the_client_on_every_probed_form() {
    for &(value, accepted) in CLIENT_RESULTS {
        let source = format!(
            "variables {{\n    global:\n        0: probe\n        1: t\n}}\n\
             rule (\"r\") {{\n    event {{ Ongoing - Global; }}\n    actions {{\n        \
             Set Global Variable(probe, {value});\n    }}\n}}\n"
        );
        let program = parser::parse(&source, &catalog(), &en())
            .unwrap_or_else(|error| panic!("{value} must parse: {error}"));
        let result = validate::validate_canonical_ids(&program, &catalog());
        assert_eq!(result.is_ok(), accepted, "{value}: {result:?}");
    }
}

#[test]
fn client_exported_programs_validate() {
    for name in CLIENT_EXPORTS {
        let program = parse(name, &zh());
        validate::validate_canonical_ids(&program, &catalog())
            .unwrap_or_else(|error| panic!("{name} must validate: {error}"));
    }
}

/// The zh-CN files are the texts pasted into a zh-CN client; each must be
/// the same program as its en-US source.
#[test]
fn zh_cn_probes_match_their_en_us_sources() {
    for name in PROBES {
        let en_program = parse(&format!("{name}.ws"), &en());
        let zh_program = parse(&format!("{name}.zh-CN.ow"), &zh());
        assert!(
            compare::equivalent(&en_program, &zh_program),
            "{name}: {:?}",
            compare::difference(&en_program, &zh_program)
        );
    }
}

fn parse(name: &str, locale: &workshop_rs::catalog::Locale) -> workshop_rs::Program {
    parser::parse(&fixture_text("slot-acceptance", name), &catalog(), locale)
        .unwrap_or_else(|error| panic!("{name} must parse: {error}"))
}

#[test]
fn accepted_values_must_name_declared_values() {
    let catalog = |accepted: &str| {
        format!(
            r#"{{
        "schemaVersion": 1,
        "locales": ["en-US"],
        "target": {{ "game": "g", "format": "f", "surface": "s" }},
        "provenance": {{ "generator": "g", "generatorVersion": "0", "source": "s", "license": "l", "reviewed": true }},
        "values": [
            {{ "id": "hostPlayer", "aliases": {{ "en-US": "Host Player" }}, "params": [], "returnType": "Player" }},
            {{ "id": "squareRoot", "aliases": {{ "en-US": "Square Root" }}, "params": ["Value"],
               "paramTypes": ["Number"], "paramAcceptedValues": [["{accepted}"]], "returnType": "Number" }}
        ]
    }}"#
        )
    };
    workshop_rs::catalog::Catalog::load(&catalog("hostPlayer")).expect("a declared value loads");
    let error = workshop_rs::catalog::Catalog::load(&catalog("eventPlayer"))
        .expect_err("an undeclared value must fail");
    assert!(
        error
            .to_string()
            .contains("accepts undeclared value 'eventPlayer'"),
        "{error}"
    );
}
