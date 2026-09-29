use workshop_rs::source::{Position, SourceFile, Span};
use workshop_rs::{Action, Condition, Event, Program, Rule, Subroutine, Value, Variable};

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
        workshop_rs::emitter::action_width(&program, &catalog, &locale, &program.rules[0].actions)
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
        Err(workshop_rs::SourceMappingError::UnknownFile(foreign.file))
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

const DISABLED_SOURCE: &str = r#"variables { global: 0: g }
subroutines { 0: sub }
rule ("disabled modifiers") {
    event { Ongoing - Global; }
    conditions { disabled Is Alive(Event Player) == True; }
    actions {
        Wait(1, Ignore Condition);
        disabled Abort;
        disabled If(True);
            Wait(1, Ignore Condition);
        End;
        disabled Set Global Variable(g, 1);
        disabled Modify Global Variable(g, Add, 1);
        disabled Call Subroutine(sub);
        disabled For Global Variable(g, 0, 2, 1);
            Wait(1, Ignore Condition);
        End;
        disabled Event Player.x = 1;
    }
}"#;

#[test]
fn disabled_actions_and_conditions_parse_validate_emit_and_round_trip() {
    let catalog = catalog();
    let locale = workshop_rs::catalog::Locale::new("en-US");

    let parsed = workshop_rs::parser::parse(DISABLED_SOURCE, &catalog, &locale).expect("parse");
    parsed.validate().expect("disabled modifiers validate");
    let rule = &parsed.rules[0];
    assert!(rule.conditions[0].disabled);
    assert!(matches!(&rule.actions[0], Action::Call { .. }));
    assert!(
        matches!(&rule.actions[1], Action::Disabled { action } if matches!(**action, Action::Call { .. }))
    );
    assert!(
        matches!(&rule.actions[2], Action::Disabled { action } if matches!(**action, Action::If { .. }))
    );

    let emitted = workshop_rs::emitter::emit(&parsed, &catalog, &locale).expect("emit");
    assert!(emitted.contains("        disabled Abort;\n"), "{emitted}");
    assert!(
        emitted.contains("        disabled If(True);\n"),
        "{emitted}"
    );
    assert!(
        emitted.contains("disabled Is Alive(Event Player) == True;"),
        "{emitted}"
    );
    let reparsed = workshop_rs::parser::parse(&emitted, &catalog, &locale).expect("reparse");
    assert!(workshop_rs::roundtrip::equivalent(&parsed, &reparsed));
}

#[test]
fn programs_built_with_disabled_modifiers_validate_and_emit() {
    let catalog = catalog();
    let locale = workshop_rs::catalog::Locale::new("en-US");

    let mut program = Program::new();
    program.rule(
        Rule::new("disabled", Event::Global)
            .condition(Condition::disabled(Value::Bool(true)))
            .action(Action::disabled(Action::call("abort", std::iter::empty()))),
    );
    program.validate().expect("disabled modifiers validate");
    let emitted = workshop_rs::emitter::emit(&program, &catalog, &locale).expect("emit");
    assert!(emitted.contains("disabled Abort;"), "{emitted}");
    assert!(emitted.contains("disabled True == True;"), "{emitted}");
}

#[test]
fn disabled_modifier_wrapping_a_terminator_is_rejected() {
    let mut program = Program::new();
    program.rule(Rule::new("bad", Event::Global).action(Action::disabled(Action::End)));
    assert!(matches!(
        program.validate(),
        Err(workshop_rs::WorkshopError::Unsupported { .. })
    ));
}

const IDENTIFIER_SOURCE: &str = r#"variables {
    global:
        0: cakePos
    player:
        1: playerScore
}

subroutines {
    0: tick
}

rule ("identifiers") {
    event { Ongoing - Global; }
    conditions {
        Add(Event Player.playerScore, Global.cakePos) > 0;
    }
    actions {
        Set Global Variable(cakePos, 1);
        Global.cakePos = Vector(1, 2, 3);
        Modify Global Variable(cakePos, Add, 1);
        Event Player.playerScore = 5;
        Set Player Variable(Event Player, playerScore, 7);
        Call Subroutine(tick);
        Set Global Variable(cakePos, Add(Event Player.playerScore, 1));
    }
}
"#;

fn span_text(program: &Program, span: Span) -> &str {
    let document = program.source(span.file).expect("source file");
    &document.text()[document.byte_range(span).expect("span range")]
}

#[test]
fn declaration_and_use_identifiers_slice_to_the_recorded_text() {
    let catalog = catalog();
    let locale = workshop_rs::catalog::Locale::new("en-US");
    let program = workshop_rs::parser::parse(IDENTIFIER_SOURCE, &catalog, &locale).expect("parses");

    assert_eq!(
        span_text(&program, program.global_variable_name_span(0).unwrap()),
        "cakePos"
    );
    assert_eq!(
        span_text(&program, program.player_variable_name_span(0).unwrap()),
        "playerScore"
    );
    assert_eq!(
        span_text(&program, program.subroutine_name_span(0).unwrap()),
        "tick"
    );

    // Infix assignments record a target span: `Event Player.name` records the
    // identifier itself, while `Global.name` records the qualified target.
    assert_eq!(
        span_text(&program, program.action_identifier_span(0, 3).unwrap()),
        "playerScore"
    );
    assert_eq!(
        span_text(&program, program.action_identifier_span(0, 1).unwrap()),
        "Global.cakePos"
    );
    // Standard-form variable writes and `Call Subroutine` record no
    // identifier span.
    assert_eq!(program.action_identifier_span(0, 0), None);
    assert_eq!(program.action_identifier_span(0, 2), None);
    assert_eq!(program.action_identifier_span(0, 5), None);

    // A read nested inside another value is addressed by a path into the
    // public value tree: `Add(Event Player.playerScore, 1)` argument 0.
    assert_eq!(
        span_text(
            &program,
            program.action_argument_value_span(0, 6, 0, &[0]).unwrap(),
        ),
        "playerScore"
    );
    // The same spelling inside a condition resolves to the identifier.
    assert_eq!(
        span_text(
            &program,
            program.condition_value_span(0, 0, &[0, 0]).unwrap(),
        ),
        "playerScore"
    );
    // A `Global.name` read records the `Global` keyword, not the identifier.
    assert_eq!(
        span_text(
            &program,
            program.condition_value_span(0, 0, &[0, 1]).unwrap(),
        ),
        "Global"
    );

    // An empty path addresses the argument or condition value itself.
    assert_eq!(
        program.action_argument_value_span(0, 6, 0, &[]),
        program.action_argument_span(0, 6, 0)
    );
    assert_eq!(
        program.condition_value_span(0, 0, &[]),
        program.condition_span(0, 0)
    );
    // Positions outside the value tree carry no span.
    assert_eq!(
        program.action_argument_value_span(0, 6, 0, &[0, 0, 0]),
        None
    );
    assert_eq!(program.condition_value_span(0, 0, &[9]), None);
}

#[test]
fn programs_without_source_carry_no_identifier_provenance() {
    let mut program = Program::new();
    program
        .global_variable(Variable::new("cakePos"))
        .player_variable(Variable::new("playerScore"))
        .subroutine(Subroutine::new("tick"))
        .rule(
            Rule::new("no source", Event::Global)
                .condition(Condition::new(Value::call(
                    "add",
                    [
                        Value::player_variable(Value::EventPlayer, "playerScore"),
                        Value::number(1.0),
                    ],
                )))
                .action(Action::SetGlobalVariable {
                    variable: "cakePos".to_string(),
                    value: Value::number(1.0),
                })
                .action(Action::CallSubroutine {
                    subroutine: "tick".to_string(),
                }),
        );

    assert_eq!(program.global_variable_name_span(0), None);
    assert_eq!(program.player_variable_name_span(0), None);
    assert_eq!(program.subroutine_name_span(0), None);
    assert_eq!(program.action_identifier_span(0, 0), None);
    assert_eq!(program.action_identifier_span(0, 1), None);
    assert_eq!(program.condition_value_span(0, 0, &[]), None);
    assert_eq!(program.condition_value_span(0, 0, &[0]), None);
    assert_eq!(program.action_argument_value_span(0, 0, 0, &[]), None);
    assert_eq!(program.action_argument_value_span(0, 0, 0, &[0]), None);
}

#[test]
fn identifier_provenance_drops_out_when_the_shape_changes() {
    let catalog = catalog();
    let locale = workshop_rs::catalog::Locale::new("en-US");
    let mut program =
        workshop_rs::parser::parse(IDENTIFIER_SOURCE, &catalog, &locale).expect("parses");

    program.global_variable(Variable::new("extra"));
    assert_eq!(program.global_variable_name_span(0), None);

    program.rules[0].actions.push(Action::End);
    assert_eq!(program.action_identifier_span(0, 3), None);
    assert_eq!(program.action_argument_value_span(0, 6, 0, &[0]), None);
}

#[test]
fn attached_identifier_spans_take_priority_over_declaration_spans() {
    let mut program = Program::new();
    program.global_variable(Variable::new("Score"));
    let file = program.add_file(SourceFile::with_source("main.opy", "globalvar 0: Score\n"));
    let declaration_span = Span::new(file, Position::new(1, 1), Position::new(1, 18));
    let name_span = Span::new(file, Position::new(1, 14), Position::new(1, 19));

    program
        .set_global_variable_spans(0, Some(declaration_span), Some(name_span))
        .expect("declaration spans attach");

    assert_eq!(program.global_variable_name_span(0), Some(name_span));
}

#[test]
fn round_trip_equivalence_distinguishes_disabled_from_active() {
    let catalog = catalog();
    let locale = workshop_rs::catalog::Locale::new("en-US");
    let parse = |body: &str| {
        let source = format!(
            "rule (\"r\") {{ event {{ Ongoing - Global; }} conditions {{ {} }} actions {{ {} }} }}",
            body.split('|').next().unwrap(),
            body.split('|').nth(1).unwrap(),
        );
        workshop_rs::parser::parse(&source, &catalog, &locale).expect("parse")
    };
    let active = parse("Is Alive(Event Player) == True;|Abort;");
    let disabled_action = parse("Is Alive(Event Player) == True;|disabled Abort;");
    let disabled_condition = parse("disabled Is Alive(Event Player) == True;|Abort;");
    assert!(workshop_rs::roundtrip::equivalent(&active, &active));
    assert!(!workshop_rs::roundtrip::equivalent(
        &active,
        &disabled_action
    ));
    assert!(!workshop_rs::roundtrip::equivalent(
        &active,
        &disabled_condition
    ));
}
