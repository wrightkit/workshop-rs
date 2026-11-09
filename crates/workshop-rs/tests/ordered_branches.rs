use crate::common::{catalog, en};
use workshop_rs::compare::{self, PathStep};
use workshop_rs::{Action, emitter, parser};

fn source(body: &str) -> String {
    format!("rule(\"r\") {{ event {{ Ongoing - Global; }} actions {{ {body} }} }}")
}

fn parse(input: &str) -> workshop_rs::Program {
    parser::parse(input, &catalog(), &en()).unwrap()
}

#[test]
fn pinned_dispatcher_preserves_the_authored_sequence() {
    let case = crate::common::cases()
        .iter()
        .find(|case| case.id == "ai-pve")
        .unwrap();
    let (input, locale) = crate::common::source(case);
    let section = input
        .split("规则 (\"[hero_init/dispatcher.opy]: heroInitDispatcher()\")")
        .nth(1)
        .unwrap()
        .split("\n}")
        .next()
        .unwrap();
    let expected: Vec<String> = section
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.starts_with("If(") {
                Some("if".into())
            } else if line.starts_with("跳过(") {
                Some("skip".into())
            } else if let Some(name) = line
                .strip_prefix("调用子程序(")
                .and_then(|s| s.strip_suffix(");"))
            {
                Some(format!("call:{name}"))
            } else if line == "Else;" {
                Some("else".into())
            } else if line == "End;" {
                Some("end".into())
            } else {
                None
            }
        })
        .collect();
    assert!(expected.iter().filter(|row| *row == "else").count() > 1);
    let program = parser::parse(&input, &catalog(), &locale).unwrap();
    let sequence = |program: &workshop_rs::Program| -> Vec<String> {
        let rule = program
            .rules
            .iter()
            .find(|rule| rule.name == "[hero_init/dispatcher.opy]: heroInitDispatcher()")
            .unwrap();
        rule.actions
            .iter()
            .map(|action| match action {
                Action::If { .. } => "if".into(),
                Action::Else => "else".into(),
                Action::End => "end".into(),
                Action::Call { name, .. } if name == "skip" => "skip".into(),
                Action::CallSubroutine { subroutine } => format!("call:{subroutine}"),
                other => panic!("unexpected dispatcher action: {other:?}"),
            })
            .collect()
    };
    assert_eq!(sequence(&program), expected);
    program.validate().unwrap();
    let emitted = emitter::emit(&program, &catalog(), &locale).unwrap();
    let reparsed = parser::parse(&emitted, &catalog(), &locale).unwrap();
    assert_eq!(sequence(&reparsed), expected);
    assert!(workshop_rs::roundtrip::equivalent(&program, &reparsed));
}

#[test]
fn multiple_else_bodies_and_later_else_if_round_trip() {
    let input = source(
        "If(True); Wait(1, Ignore Condition); Else; Wait(2, Ignore Condition); Else; Wait(3, Ignore Condition); Else If(False); Wait(4, Ignore Condition); End; Wait(5, Ignore Condition);",
    );
    let program = parse(&input);
    let emitted = emitter::emit(&program, &catalog(), &en()).unwrap();
    let reparsed = parse(&emitted);
    assert!(workshop_rs::roundtrip::equivalent(&program, &reparsed));
    assert_eq!(
        emitter::action_width(&program, &catalog(), &en(), &program.rules[0].actions)
            .unwrap()
            .width,
        10
    );
    for number in 1..=5 {
        assert!(emitted.contains(&format!("Wait({number}, Ignore Condition)")));
    }
}

#[test]
fn comparison_identifies_each_else_body() {
    let first = parse(&source(
        "If(True); Else; Wait(1, Ignore Condition); Else; Wait(2, Ignore Condition); End;",
    ));
    for (ordinal, body) in [
        (
            0,
            "If(True); Else; Wait(9, Ignore Condition); Else; Wait(2, Ignore Condition); End;",
        ),
        (
            1,
            "If(True); Else; Wait(1, Ignore Condition); Else; Wait(9, Ignore Condition); End;",
        ),
    ] {
        let difference = compare::difference(&first, &parse(&source(body))).unwrap();
        assert!(
            difference.path.contains(&PathStep::ElseBranch(ordinal)),
            "{difference:?}"
        );
    }
}

#[test]
fn single_else_comparison_keeps_its_existing_path() {
    let first = parse(&source("If(True); Else; Wait(1, Ignore Condition); End;"));
    let second = parse(&source("If(True); Else; Wait(2, Ignore Condition); End;"));
    let difference = compare::difference(&first, &second).unwrap();
    assert_eq!(
        difference.path,
        vec![
            PathStep::Action(0),
            PathStep::ElseBody,
            PathStep::Action(0),
            PathStep::Value(0)
        ]
    );
}

#[test]
fn added_else_and_action_each_contribute_one_element() {
    let original = parse(&source("If(True); Else; End; Disable Inspector Recording;"));
    let added = parse(&source(
        "If(True); Else; Else; Disable Inspector Recording; End; Disable Inspector Recording;",
    ));
    assert_eq!(
        added.element_count(&catalog()).unwrap().total
            - original.element_count(&catalog()).unwrap().total,
        2
    );
}

#[test]
fn branches_preserve_source_spans_and_later_condition() {
    let program = parse(&source(
        "If(True); Else; Wait(2, Ignore Condition); Else; disabled Wait(3, Ignore Condition); Else If(False); Wait(4, Ignore Condition); End;",
    ));
    let text = |span: workshop_rs::source::Span| {
        let doc = program.source(span.file).unwrap();
        &doc.text()[doc.byte_range(span).unwrap()]
    };
    assert!(text(program.action_span(0, 2).unwrap()).starts_with("Wait(2"));
    assert_eq!(
        text(program.action_argument_span(0, 5, 0).unwrap()),
        "False"
    );
    assert!(text(program.action_span(0, 6).unwrap()).starts_with("Wait(4"));
    assert!(program.action_span(0, 1).is_none());
    assert!(program.action_span(0, 3).is_none());
    assert!(program.action_span(0, 7).is_none());
    let emitted = emitter::emit(&program, &catalog(), &en()).unwrap();
    let reparsed = parse(&emitted);
    assert!(workshop_rs::roundtrip::equivalent(&program, &reparsed));
}

#[test]
fn comparing_condition_after_else_preserves_conditional_indices() {
    let first = parse(&source("If(True); Else; Else; Else If(False); End;"));
    let second = parse(&source("If(True); Else; Else; Else If(True); End;"));
    let difference = compare::difference(&first, &second).unwrap();
    assert_eq!(
        difference.path,
        vec![PathStep::Action(0), PathStep::Value(1)]
    );
    let reordered = parse(&source("If(True); Else If(False); Else; Else; End;"));
    let difference = compare::difference(&first, &reordered).unwrap();
    assert_eq!(difference.path, vec![PathStep::Action(0)]);
}
