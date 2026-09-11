use workshop_rs::{Action, Condition, Event, Program, Rule, Value, Variable};

fn catalog() -> workshop_rs::catalog::Catalog {
    workshop_rs::catalog::Catalog::builtin().expect("builtin catalog")
}

#[test]
fn program_is_constructible_without_storage_ids() {
    let predicate = Value::call("isAlive", [Value::global_variable("Target")]);
    let condition = Condition::new(predicate.clone());
    let rule = Rule::new("Linear control flow", Event::Global)
        .condition(condition)
        .action(Action::If {
            condition: predicate.clone(),
        })
        .action(Action::call("Wait", [Value::number(1.0)]))
        .action(Action::ElseIf {
            condition: Value::Bool(false),
        })
        .action(Action::Else)
        .action(Action::While {
            condition: predicate,
        })
        .action(Action::disabled(Action::call("Abort", std::iter::empty())))
        .action(Action::End);

    let mut program = Program::new();
    program.global_variable(Variable::new("Target")).rule(rule);

    assert_eq!(program.global_variables[0].name, "Target");
    assert!(matches!(program.rules[0].actions[0], Action::If { .. }));
    assert!(matches!(program.rules[0].actions[2], Action::ElseIf { .. }));
    assert!(matches!(program.rules[0].actions[3], Action::Else));
    assert!(matches!(
        program.rules[0].actions[5],
        Action::Disabled { .. }
    ));
    assert!(matches!(program.rules[0].actions[6], Action::End));
}

#[test]
fn values_and_conditions_are_composable() {
    let value = Value::call(
        "add",
        [
            Value::global_variable("Score"),
            Value::Array(vec![Value::number(1.0), Value::number(2.0)]),
        ],
    );
    let condition = Condition::new(value.clone());
    let mut program = Program::default();
    program.rule(Rule::new("Composable", Event::Global).condition(condition));

    assert!(matches!(
        &program.rules[0].conditions[0].value,
        Value::Call { name, args } if name == "add" && args.len() == 2
    ));
}

#[test]
fn public_program_is_the_raw_parse_and_emit_boundary() {
    let catalog = catalog();
    let locale = workshop_rs::catalog::Locale::new("en-US");
    let source = "rule (\"public boundary\") {\n    event { Ongoing - Global; }\n    actions { Wait(1, Ignore Condition); }\n}\n";

    let parsed = workshop_rs::parser::parse(source, &catalog, &locale).expect("parses");
    assert_eq!(
        parsed
            .source(workshop_rs::source::FileId::from_index(0))
            .unwrap()
            .text(),
        source
    );
    let rule_span = parsed.rule_span(0).expect("public rule span");
    assert!(parsed.action_span(0, 0).is_some());
    let argument_span = parsed
        .action_argument_span(0, 0, 0)
        .expect("public action argument span");
    assert!(rule_span.start.line <= argument_span.start.line);
    let edit = parsed
        .edit_source(argument_span, "2")
        .expect("validated public source edit");
    let updated = parsed
        .source(workshop_rs::source::FileId::from_index(0))
        .unwrap()
        .apply(&[edit])
        .expect("apply public source edit");
    assert!(updated.text().contains("Wait(2, Ignore Condition)"));
    parsed.validate().expect("structurally validates");
    workshop_rs::validate::validate_canonical_ids(&parsed, &catalog).expect("catalog validates");
    assert!(parsed.semantic_issues(&catalog).is_empty());
    assert!(parsed.element_count(&catalog).expect("counts").total > 0);

    let emitted = workshop_rs::emitter::emit(&parsed, &catalog, &locale).expect("emits");
    let reparsed = workshop_rs::parser::parse(&emitted, &catalog, &locale).expect("reparses");
    assert!(workshop_rs::roundtrip::equivalent(&parsed, &reparsed));

    let edited =
        workshop_rs::parser::parse(updated.text(), &catalog, &locale).expect("reparses edit");
    assert!(!workshop_rs::roundtrip::equivalent(&parsed, &edited));
}

#[test]
fn independently_constructed_program_uses_the_same_operations() {
    let catalog = catalog();
    let locale = workshop_rs::catalog::Locale::new("en-US");
    let mut program = Program::new();
    program.global_variable(Variable::new("Score")).rule(
        Rule::new("constructed", Event::Global).action(Action::SetGlobalVariable {
            variable: "Score".to_string(),
            value: Value::number(1.0),
        }),
    );

    program.validate().expect("structurally validates");
    workshop_rs::validate::validate_canonical_ids(&program, &catalog).expect("catalog validates");
    let layout =
        workshop_rs::actions::action_width(&program, &catalog, &locale, &program.rules[0].actions)
            .expect("lays out");
    assert_eq!(layout.width, 1);
    let emitted = workshop_rs::emitter::emit(&program, &catalog, &locale).expect("emits");
    assert!(emitted.contains("Set Global Variable(Score, 1)"));
}

#[test]
fn disabled_public_semantics_fail_explicitly_at_the_storage_boundary() {
    let catalog = catalog();
    let locale = workshop_rs::catalog::Locale::new("en-US");

    let mut disabled_condition = Program::new();
    disabled_condition.rule(
        Rule::new("disabled condition", Event::Global)
            .condition(Condition::disabled(Value::Bool(true))),
    );
    assert!(matches!(
        disabled_condition.validate(),
        Err(workshop_rs::WorkshopError::Unsupported { .. })
    ));

    let mut disabled_action = Program::new();
    disabled_action.rule(
        Rule::new("disabled action", Event::Global)
            .action(Action::disabled(Action::call("Wait", [Value::number(1.0)]))),
    );
    assert!(matches!(
        workshop_rs::emitter::emit(&disabled_action, &catalog, &locale),
        Err(workshop_rs::WorkshopError::Unsupported { .. })
    ));
}
