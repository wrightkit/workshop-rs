use crate::common::{catalog, en};
use workshop_rs::source::{FileId, Position, SourceFile, Span};
use workshop_rs::{
    Action, Condition, Event, Program, Rule, SourceMap, Subroutine, Value, Variable, parser,
};

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
fn file_source_attaches_to_registered_files() {
    let mut program = Program::new();
    let file = program.add_file(SourceFile::new("main.opy"));
    assert!(program.source(file).is_none());

    assert!(program.set_file_source(file, "x = 1\n"));
    assert_eq!(program.source(file).unwrap().text(), "x = 1\n");

    let missing = workshop_rs::source::FileId::from_index(9);
    assert!(!program.set_file_source(missing, "x = 1\n"));
}

#[test]
fn applying_a_source_map_drops_stale_settings_spans() {
    // The settings tree is carried into the program inertly: a source map
    // holds no settings entries, so spans parsed against the emitted text
    // cannot be re-anchored to the authored members the applied file table
    // names. `SourceMap::apply` clears them; otherwise the stale positions
    // fail `Program::validate` once a member retains shorter authored source
    // (wrightkit/wright#583).
    // Rules first so the stale settings span ends past the short authored
    // text while the map-carried rule span stays in bounds.
    let workshop = "rule (\"tick\") {\n    event { Ongoing - Each Player; All; All; }\n    actions { Wait(1); }\n}\n\nsettings {\n    main {\n        Description: \"Streak race\"\n    }\n}\n";
    let mut reparsed = parser::parse(workshop, &catalog(), &en()).expect("parses");
    assert!(reparsed.settings.as_ref().is_some_and(|s| s.span.is_some()));

    let map = SourceMap::extract(&reparsed);
    map.apply(&mut reparsed).expect("map applies");

    let settings = reparsed.settings.as_ref().expect("settings survive");
    assert!(settings.span.is_none());
    assert!(settings.children.iter().all(|node| node.span().is_none()));

    // Retain authored text that covers every map-carried span (the rule
    // block at lines 1–4) but ends before the stale settings span
    // (lines 6–9): the dropped settings span cannot fail validation
    // (wrightkit/wright#583).
    let authored = workshop
        .lines()
        .take(5)
        .fold(String::new(), |mut text, line| {
            text.push_str(line);
            text.push('\n');
            text
        });
    assert!(reparsed.set_file_source(FileId::from_index(0), authored));
    reparsed.validate().expect("mapped program validates");
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
        }
        | workshop_rs::WorkshopError::UnknownWithCandidates {
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
        2: my var
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
        disabled Modify Global Variable(cakePos, Add, Event Player.playerScore);
        Start Rule(tick, Restart Rule);
        Set Global Variable At Index(cakePos, 0, 1);
        Big Message(All Players(All Teams), Custom String("cakePos in a string is not a reference", cakePos));
        // cakePos inside a comment is not a reference either
        For Global Variable(cakePos, 0, 10, 1);
            Abort;
        End;
        For Player Variable(Event Player, playerScore, 0, 5, 1);
            Abort;
        End;
        Global.cakePos[0] = 4;
        Event Player.playerScore[0] = 9;
        Set Global Variable At Index(my var, 1, 2);
        Set Player Variable At Index(Event Player, playerScore, 0, 3);
        Modify Player Variable At Index(Event Player, playerScore, 0, Add, 5);
        Set Global Variable(my var, Add(Global Variable(cakePos), Player Variable(Event Player, playerScore)));
        Set Global Variable(cakePos, Add((Event Player).playerScore, 0));
        Stop Chasing Player Variable(Event Player, playerScore);
        Chase Player Variable Over Time(Event Player, playerScore, 0, 30, None);
        Chase Global Variable Over Time(my var, 0, 30, None);
        Stop Chasing Global Variable(cakePos);
        If(cakePos > 0);
            Abort;
        Else If(playerScore > 1);
            Abort;
        Else;
            Abort;
        End;
    }
}

rule ("subroutine runner") {
    event {
        Subroutine;
        tick;
    }
    actions {
        Call Subroutine(tick);
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
        span_text(&program, program.global_variable_name_span(1).unwrap()),
        "my var"
    );
    assert_eq!(
        span_text(&program, program.player_variable_name_span(0).unwrap()),
        "playerScore"
    );
    assert_eq!(
        span_text(&program, program.subroutine_name_span(0).unwrap()),
        "tick"
    );
    // Rule names sit inside their `rule("name")` string token.
    assert_eq!(
        span_text(&program, program.rule_name_span(0).unwrap()),
        "identifiers"
    );
    assert_eq!(
        span_text(&program, program.rule_name_span(1).unwrap()),
        "subroutine runner"
    );

    // Every variable write names its target identifier: standard-form
    // `Set`/`Modify`, infix `Global.name`/`Event Player.name` assignments,
    // `For` loops, and `Call Subroutine` callees.
    for action in [0, 1, 2, 6, 7, 11] {
        assert_eq!(
            span_text(&program, program.action_identifier_span(0, action).unwrap()),
            "cakePos",
            "action {action}"
        );
    }
    for action in [3, 4, 14] {
        assert_eq!(
            span_text(&program, program.action_identifier_span(0, action).unwrap()),
            "playerScore",
            "action {action}"
        );
    }
    assert_eq!(
        span_text(&program, program.action_identifier_span(0, 5).unwrap()),
        "tick"
    );
    // Generic calls carry no action-level identifier; the subroutine or
    // variable they name is provenance of a value argument instead.
    assert_eq!(program.action_identifier_span(0, 8), None);
    assert_eq!(program.action_identifier_span(0, 9), None);
    assert_eq!(program.action_identifier_span(0, 10), None);

    // A read nested inside another value is addressed by a path into the
    // public value tree: `Add(Event Player.playerScore, 1)` argument 0.
    assert_eq!(
        span_text(
            &program,
            program.action_argument_value_span(0, 6, 0, &[0]).unwrap(),
        ),
        "playerScore"
    );
    // `PlayerVariable` exposes its player expression as child 0.
    assert_eq!(
        span_text(
            &program,
            program
                .action_argument_value_span(0, 6, 0, &[0, 0])
                .unwrap(),
        ),
        "Event Player"
    );
    // `Vector` components are addressed as children 0/1/2.
    assert_eq!(
        span_text(
            &program,
            program.action_argument_value_span(0, 1, 0, &[2]).unwrap(),
        ),
        "3"
    );
    // A `disabled` action's provenance is that of the wrapped action.
    assert_eq!(
        span_text(
            &program,
            program.action_argument_value_span(0, 7, 0, &[]).unwrap(),
        ),
        "playerScore"
    );
    // The same spellings inside a condition resolve to their identifiers.
    assert_eq!(
        span_text(
            &program,
            program.condition_value_span(0, 0, &[0, 0]).unwrap(),
        ),
        "playerScore"
    );
    assert_eq!(
        span_text(
            &program,
            program.condition_value_span(0, 0, &[0, 1]).unwrap(),
        ),
        "cakePos"
    );
    // `Start Rule`'s subroutine argument and an `... At Index` variable name
    // record their identifiers as value provenance.
    assert_eq!(
        span_text(
            &program,
            program.action_argument_value_span(0, 8, 0, &[]).unwrap(),
        ),
        "tick"
    );
    assert_eq!(
        span_text(
            &program,
            program.action_argument_value_span(0, 9, 0, &[]).unwrap(),
        ),
        "cakePos"
    );
    // A string literal carrying the same spelling is string provenance, not
    // the identifier; the real argument next to it inside `Custom String` is.
    assert_eq!(
        span_text(
            &program,
            program.action_argument_value_span(0, 10, 1, &[0]).unwrap(),
        ),
        "\"cakePos in a string is not a reference\""
    );
    assert_eq!(
        span_text(
            &program,
            program.action_argument_value_span(0, 10, 1, &[1]).unwrap(),
        ),
        "cakePos"
    );

    // `For` action spans cover the full `For ... End;` block, starting at
    // the `For` keyword rather than the last word of its phrase.
    assert!(
        span_text(&program, program.action_span(0, 11).unwrap())
            .starts_with("For Global Variable(cakePos, 0, 10, 1);")
    );
    assert!(span_text(&program, program.action_span(0, 11).unwrap()).ends_with("End;"));
    assert!(
        span_text(&program, program.action_span(0, 14).unwrap())
            .starts_with("For Player Variable(Event Player, playerScore, 0, 5, 1);")
    );

    // Indexed writes are `... Variable At Index` calls: the action itself
    // carries no identifier, the variable argument records the name.
    for action in [17, 18, 19, 20, 21] {
        assert_eq!(
            program.action_identifier_span(0, action),
            None,
            "action {action}"
        );
    }
    for (action, expected) in [
        (17, "cakePos"),
        (18, "playerScore"),
        (19, "my var"),
        (20, "playerScore"),
        (21, "playerScore"),
    ] {
        assert_eq!(
            span_text(
                &program,
                program
                    .action_argument_value_span(0, action, 0, &[])
                    .unwrap(),
            ),
            expected,
            "action {action}"
        );
    }

    // `Global Variable(name)` and `Player Variable(player, name)` read forms
    // inside a `Set` value argument.
    assert_eq!(
        span_text(&program, program.action_identifier_span(0, 22).unwrap(),),
        "my var"
    );
    assert_eq!(
        span_text(
            &program,
            program.action_argument_value_span(0, 22, 0, &[0]).unwrap(),
        ),
        "cakePos"
    );
    assert_eq!(
        span_text(
            &program,
            program.action_argument_value_span(0, 22, 0, &[1]).unwrap(),
        ),
        "playerScore"
    );
    assert_eq!(
        span_text(
            &program,
            program
                .action_argument_value_span(0, 22, 0, &[1, 0])
                .unwrap(),
        ),
        "Event Player"
    );
    // A parenthesized `(Event Player).name` read.
    assert_eq!(
        span_text(
            &program,
            program.action_argument_value_span(0, 23, 0, &[0]).unwrap(),
        ),
        "playerScore"
    );
    // `Stop Chasing`/`Chase` variable forms record the name on the folded
    // variable argument.
    for (action, expected) in [
        (24, "playerScore"),
        (25, "playerScore"),
        (26, "my var"),
        (27, "cakePos"),
    ] {
        assert_eq!(
            span_text(
                &program,
                program
                    .action_argument_value_span(0, action, 0, &[])
                    .unwrap(),
            ),
            expected,
            "action {action}"
        );
    }
    // `If` and `Else If` conditions carry their reads as value provenance.
    assert_eq!(
        span_text(
            &program,
            program.action_argument_value_span(0, 28, 0, &[0]).unwrap(),
        ),
        "cakePos"
    );
    assert_eq!(
        span_text(
            &program,
            program.action_argument_value_span(0, 30, 0, &[0]).unwrap(),
        ),
        "playerScore"
    );

    // A `Subroutine` event binding records the bound name.
    assert_eq!(
        span_text(&program, program.rule_event_name_span(1).unwrap()),
        "tick"
    );
    assert_eq!(program.rule_event_name_span(0), None);

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
    assert_eq!(program.action_argument_value_span(0, 6, 1, &[]), None);
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
    assert_eq!(program.rule_event_name_span(0), None);
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

// Residual inspection is defined over the public canonical model and is
// independent from structural validation (#365): residuals observable in a
// program `validate` rejects must still be inventoried instead of projecting
// an internal materialization failure to an empty report.

#[test]
fn semantic_issues_reports_residuals_when_control_flow_is_unterminated() {
    let catalog = catalog();
    let mut program = Program::new();
    program.rule(
        Rule::new("missing terminator", Event::Global)
            .action(Action::call("unrecognizedProducerAction", []))
            .action(Action::While {
                condition: Value::Bool(true),
            }),
    );

    assert!(matches!(
        program.validate(),
        Err(workshop_rs::WorkshopError::Malformed { .. })
    ));
    for issues in [
        program.semantic_issues(&catalog),
        workshop_rs::rules::inspect(&program, &catalog),
    ] {
        let [issue] = issues.as_slice() else {
            panic!("expected exactly one residual, got {issues:?}");
        };
        assert_eq!(
            issue.kind,
            workshop_rs::rules::IncompletenessKind::UnknownAction
        );
        assert_eq!(issue.name, "unrecognizedProducerAction");
        assert_eq!(
            issue.classification,
            workshop_rs::rules::ResidualClassification::ProducerExtension
        );
    }
}

#[test]
fn semantic_issues_survives_unresolved_variable_reference() {
    let catalog = catalog();
    let mut program = Program::new();
    program.rule(
        Rule::new("unresolved variable", Event::Global)
            .condition(Condition::new(Value::call(
                "isAlive",
                [Value::global_variable("undeclared")],
            )))
            .action(Action::call("unrecognizedProducerAction", [])),
    );

    assert!(program.validate().is_err());
    let issues = program.semantic_issues(&catalog);
    assert!(
        issues.iter().any(|issue| issue.kind
            == workshop_rs::rules::IncompletenessKind::UnknownAction
            && issue.name == "unrecognizedProducerAction"),
        "unresolved variable must not suppress observable residuals: {issues:?}"
    );
}

#[test]
fn semantic_issues_survives_unresolved_subroutine_reference() {
    let catalog = catalog();
    let mut program = Program::new();
    program.rule(
        Rule::new("unresolved subroutine", Event::Global)
            .action(Action::CallSubroutine {
                subroutine: "missing".to_string(),
            })
            .action(Action::call("unrecognizedProducerAction", [])),
    );

    assert!(program.validate().is_err());
    let issues = program.semantic_issues(&catalog);
    assert!(
        issues.iter().any(|issue| issue.kind
            == workshop_rs::rules::IncompletenessKind::UnknownAction
            && issue.name == "unrecognizedProducerAction"),
        "unresolved subroutine must not suppress observable residuals: {issues:?}"
    );
}

#[test]
fn fully_understood_program_reports_an_empty_residual_inventory() {
    let catalog = catalog();
    let mut program = Program::new();
    program.global_variable(Variable::new("Score")).rule(
        Rule::new("understood", Event::Global).action(Action::SetGlobalVariable {
            variable: "Score".to_string(),
            value: Value::number(1.0),
        }),
    );

    program.validate().expect("structurally validates");
    assert!(program.semantic_issues(&catalog).is_empty());
    assert!(workshop_rs::rules::inspect(&program, &catalog).is_empty());
}

#[test]
fn residual_ordering_preserves_control_flow_materialization_order() {
    // Value residuals follow the order value nodes take when materialized:
    // `While`, `For`, and `Else If` headers land after their bodies, while an
    // `If` header precedes its body.
    let catalog = catalog();
    let locale = workshop_rs::catalog::Locale::new("en-US");
    let source = r#"variables { global: 0: g }
rule ("ordering") {
    event { Ongoing - Global; }
    actions {
        While(unknownWhileCond());
            Set Global Variable(g, unknownWhileBody());
        End;
        For Global Variable(g, unknownForStart(), unknownForStop(), unknownForStep());
            Set Global Variable(g, unknownForBody());
        End;
        If(unknownIfCond());
            Set Global Variable(g, unknownIfBody());
        Else If(unknownElseIfCond());
            Set Global Variable(g, unknownElseIfBody());
        Else;
            Set Global Variable(g, unknownElseBody());
        End;
    }
}"#;
    let program = workshop_rs::parser::parse(source, &catalog, &locale).expect("parses");
    program.validate().expect("structurally validates");

    let issues = program.semantic_issues(&catalog);
    let names: Vec<_> = issues.iter().map(|issue| issue.name.as_str()).collect();
    assert_eq!(
        names,
        [
            "unknownWhileBody",
            "unknownWhileCond",
            "unknownForBody",
            "unknownForStart",
            "unknownForStop",
            "unknownForStep",
            "unknownIfCond",
            "unknownIfBody",
            "unknownElseIfBody",
            "unknownElseIfCond",
            "unknownElseBody",
        ]
    );
    assert!(issues.iter().all(|issue| {
        issue.kind == workshop_rs::rules::IncompletenessKind::UnknownValue
            && issue.classification
                == workshop_rs::rules::ResidualClassification::UnresolvedIdentifier
            && issue.span.is_some()
    }));
}

#[test]
fn empty_settings_member_does_not_consume_the_following_member() {
    // A bare `name:` member is its own member with an empty value; the next
    // line still parses as its own member (issue 402).
    let catalog = catalog();
    let locale = workshop_rs::catalog::Locale::new("en-US");
    let source = r#"settings
{
    lobby
    {
        Project Empty:
        Map Rotation: After A Game
    }
}
rule("r")
{
    event
    {
        Ongoing - Global;
    }
    actions
    {
        Wait(1, Ignore Condition);
    }
}
"#;
    let program = workshop_rs::parser::parse(source, &catalog, &locale).expect("parses");
    let emitted = workshop_rs::emitter::emit(&program, &catalog, &locale).expect("emits");
    let lines: Vec<&str> = emitted.lines().map(str::trim).collect();
    assert!(lines.contains(&"Project Empty:"), "{emitted}");
    assert!(lines.contains(&"Map Rotation: After A Game"), "{emitted}");
    let reparsed = workshop_rs::parser::parse(&emitted, &catalog, &locale).expect("reparses");
    assert!(workshop_rs::roundtrip::equivalent(&program, &reparsed));
}
