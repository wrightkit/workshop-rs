use workshop_rs::catalog::{Catalog, Locale};
use workshop_rs::settings::{PathPart, schema};
use workshop_rs::{
    Action, Event, MappedText, Program, Rule, SourceMap, Value, emitter, parser, roundtrip, rules,
};

#[test]
fn canonical_program_operations_cover_parse_validate_inspect_emit_and_roundtrip() {
    let catalog = Catalog::builtin().expect("built-in catalog");
    let locale = Locale::new("en-US");
    let source = r#"rule ("public api") {
        event { Ongoing - Global; }
        actions { Wait(1, Ignore Condition); }
    }"#;

    let program = parser::parse_with_context(source, &catalog, &locale, &catalog)
        .expect("Workshop parses into the canonical program");
    program
        .validate()
        .expect("canonical program is structurally valid");
    rules::validate_canonical_ids(&program, &catalog)
        .expect("canonical ids resolve through the public rule API");
    assert!(program.semantic_issues(&catalog).is_empty());

    let emitted = emitter::emit(&program, &catalog, &locale).expect("canonical emission");
    let reparsed = parser::parse(&emitted, &catalog, &locale).expect("emitted Workshop reparses");
    assert!(roundtrip::equivalent(&program, &reparsed));
}

#[test]
fn settings_schema_exposes_enum_values_without_the_internal_table() {
    let definition = schema::definition(&[
        PathPart::Part("lobby"),
        PathPart::Part("enableMatchVoiceChat"),
    ])
    .expect("match voice setting is in the canonical schema");

    let enabled = definition
        .enum_members()
        .next()
        .expect("enum-backed boolean token");
    assert_eq!(enabled.domain(), "matchVoiceChat");
    assert_eq!(enabled.id(), "enabled");
    assert_eq!(enabled.english_name(), "Enabled");
}

#[test]
fn catalog_actions_and_values_are_built_by_canonical_id() {
    let catalog = Catalog::builtin().expect("built-in catalog");
    let locale = Locale::new("en-US");
    let numbers = |values: &[i32]| values.iter().copied().map(Value::from).collect::<Vec<_>>();

    let mut program = Program::new();
    program.rule(
        Rule::new("catalog calls", Event::Global)
            .action(Action::call(
                "damage",
                [Value::EventPlayer, Value::Null, Value::from(10)],
            ))
            .action(Action::call(
                "teleport",
                [
                    Value::EventPlayer,
                    Value::call("vector", numbers(&[1, 2, 3])),
                ],
            ))
            .action(Action::call(
                "setSlowMotion",
                [Value::call(
                    "countOf",
                    [Value::call("array", numbers(&[4, 5, 6]))],
                )],
            )),
    );
    program.validate().expect("catalog calls validate");
    rules::validate_canonical_ids(&program, &catalog).expect("catalog ids resolve");

    let emitted = emitter::emit(&program, &catalog, &locale).expect("catalog calls emit");
    assert!(
        emitted.contains("Damage(Event Player, Null, 10);"),
        "{emitted}"
    );
    assert!(
        emitted.contains("Teleport(Event Player, Vector(1, 2, 3));"),
        "{emitted}"
    );
    assert!(
        emitted.contains("Set Slow Motion(Count Of(Array(4, 5, 6)));"),
        "{emitted}"
    );

    let reparsed = parser::parse(&emitted, &catalog, &locale).expect("emitted Workshop reparses");
    assert!(roundtrip::equivalent(&program, &reparsed), "{emitted}");

    let actions = &reparsed.rules[0].actions;
    assert!(matches!(
        &actions[0],
        Action::Call { name, args } if name == "damage"
            && matches!(&args[..], [Value::EventPlayer, Value::Null, amount] if is_number(amount, 10.0))
    ));
    assert!(matches!(
        &actions[1],
        Action::Call { name, args } if name == "teleport"
            && matches!(&args[..], [Value::EventPlayer, position]
                if is_call(position, "vector", &[1.0, 2.0, 3.0]))
    ));
    assert!(matches!(
        &actions[2],
        Action::Call { name, args } if name == "setSlowMotion"
            && matches!(&args[..], [Value::Call { name, args }] if name == "countOf"
                && matches!(&args[..], [array] if is_call(array, "array", &[4.0, 5.0, 6.0])))
    ));
}

fn is_call(value: &Value, id: &str, numbers: &[f64]) -> bool {
    matches!(
        value,
        Value::Call { name, args } if name == id
            && args.len() == numbers.len()
            && args.iter().zip(numbers).all(|(arg, expected)| is_number(arg, *expected))
    )
}

fn is_number(value: &Value, expected: f64) -> bool {
    matches!(value, Value::Number(actual) if *actual == expected)
}

#[test]
fn mapped_text_is_constructible_outside_the_crate() {
    let catalog = Catalog::builtin().expect("built-in catalog");
    let locale = Locale::new("en-US");
    let program = parser::parse(
        r#"rule ("mapped") {
        event { Ongoing - Global; }
        actions { Wait(1, Ignore Condition); }
    }"#,
        &catalog,
        &locale,
    )
    .expect("Workshop parses");
    let text = emitter::emit(&program, &catalog, &locale).expect("canonical emission");

    let artifact = MappedText::new(text.clone(), SourceMap::extract(&program));
    assert_eq!(artifact.text, text);
    let decoded = MappedText::from_json(&artifact.to_json()).expect("artifact decodes");
    assert_eq!(decoded, artifact);
}
