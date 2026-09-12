use workshop_rs::source::{Position, SourceFile, Span};
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
fn independently_constructed_program_preserves_near_integer_numbers() {
    let catalog = catalog();
    let locale = workshop_rs::catalog::Locale::new("en-US");
    let value = 1.0000000000000004;
    let mut program = Program::new();
    program.global_variable(Variable::new("Score")).rule(
        Rule::new("near integer", Event::Global).action(Action::SetGlobalVariable {
            variable: "Score".to_string(),
            value: Value::number(value),
        }),
    );

    let emitted = workshop_rs::emitter::emit(&program, &catalog, &locale).expect("emits");
    assert!(
        emitted.contains("Set Global Variable(Score, 1.0000000000000004)"),
        "non-integral values must not be emitted as integers: {emitted}"
    );
    let reparsed = workshop_rs::parser::parse(&emitted, &catalog, &locale).expect("reparses");
    assert!(workshop_rs::roundtrip::equivalent(&program, &reparsed));
}

#[test]
fn externally_constructed_program_attaches_source_and_preserves_diagnostic_span() {
    let catalog = catalog();
    let mut program = Program::new();
    program.global_variable(Variable::new("Score")).rule(
        Rule::new("external", Event::Global).action(Action::SetGlobalVariable {
            variable: "Score".to_string(),
            value: Value::call("notCanonicalValue", std::iter::empty()),
        }),
    );

    let file = program.add_file(SourceFile::with_source(
        "main.opy",
        "Score = notCanonicalValue()\n",
    ));
    let action_span = Span::new(file, Position::new(1, 1), Position::new(1, 29));
    let value_span = Span::new(file, Position::new(1, 9), Position::new(1, 28));
    program
        .set_rule_span(0, Some(action_span))
        .expect("rule span attaches");
    program
        .set_action_span(0, 0, Some(action_span))
        .expect("action span attaches");
    program
        .set_action_argument_span(0, 0, 0, Some(value_span))
        .expect("action argument span attaches");

    assert_eq!(
        program.source(file).unwrap().text(),
        "Score = notCanonicalValue()\n"
    );
    assert_eq!(program.action_span(0, 0), Some(action_span));
    assert_eq!(program.action_argument_span(0, 0, 0), Some(value_span));

    let error = workshop_rs::validate::validate_canonical_ids(&program, &catalog)
        .expect_err("unknown canonical value is rejected");
    assert!(matches!(
        error,
        workshop_rs::WorkshopError::Unknown {
            kind: "value",
            span: Some(span),
            ..
        } if span == value_span
    ));
}

#[test]
fn external_action_provenance_reaches_structural_validation() {
    let source = "Wait(1)\n";
    let action_span = Span::new(
        workshop_rs::source::FileId::from_index(0),
        Position::new(2, 1),
        Position::new(2, 5),
    );
    let argument_span = Span::new(
        workshop_rs::source::FileId::from_index(0),
        Position::new(2, 6),
        Position::new(2, 7),
    );

    let mut action_program = Program::new();
    action_program.rule(
        Rule::new("action span", Event::Global).action(Action::call("Wait", [Value::number(1.0)])),
    );
    let file = action_program.add_file(SourceFile::with_source("main.opy", source));
    let action_span = Span::new(file, action_span.start, action_span.end);
    action_program
        .set_action_span(0, 0, Some(action_span))
        .expect("action span attaches");
    assert!(matches!(
        action_program.validate(),
        Err(workshop_rs::WorkshopError::Malformed {
            span: Some(span), ..
        }) if span == action_span
    ));

    let mut argument_program = Program::new();
    argument_program.rule(
        Rule::new("argument span", Event::Global)
            .action(Action::call("Wait", [Value::number(1.0)])),
    );
    let file = argument_program.add_file(SourceFile::with_source("main.opy", source));
    let argument_span = Span::new(file, argument_span.start, argument_span.end);
    argument_program
        .set_action_argument_span(0, 0, 0, Some(argument_span))
        .expect("action argument span attaches");
    assert!(matches!(
        argument_program.validate(),
        Err(workshop_rs::WorkshopError::Malformed {
            span: Some(span), ..
        }) if span == argument_span
    ));
}

#[test]
fn provenance_attachment_rejects_foreign_files() {
    let mut program = Program::new();
    program.rule(Rule::new("external", Event::Global));
    let file = program.add_file(SourceFile::new("main.opy"));
    let foreign = Span::new(
        workshop_rs::source::FileId::from_index(file.index() + 1),
        Position::new(1, 1),
        Position::new(1, 2),
    );

    assert_eq!(
        program.set_rule_span(0, Some(foreign)),
        Err(workshop_rs::ProvenanceError::UnknownFile(foreign.file))
    );
    assert!(program.rule_span(0).is_none());
}

#[test]
fn declaration_provenance_reaches_structural_diagnostics() {
    let mut program = Program::new();
    program.global_variable(Variable::new("Score"));
    let file = program.add_file(SourceFile::with_source("main.opy", "Score\n"));
    let declaration_span = Span::new(file, Position::new(2, 1), Position::new(2, 6));

    program
        .set_global_variable_spans(0, Some(declaration_span), None)
        .expect("declaration span attaches");
    let error = program
        .validate()
        .expect_err("source range outside the document is rejected");
    assert!(matches!(
        error,
        workshop_rs::WorkshopError::Malformed {
            span: Some(span), ..
        } if span == declaration_span
    ));
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
