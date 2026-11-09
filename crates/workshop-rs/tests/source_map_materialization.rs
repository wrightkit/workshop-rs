use workshop_rs::source::Span;
use workshop_rs::{Action, MappedText, SourceMap, Value, parser};

use super::common::{catalog, en};

fn parsed(source: &str) -> workshop_rs::Program {
    parser::parse(source, &catalog(), &en()).expect("parses")
}

#[test]
fn materialized_control_flow_keeps_each_rows_source() {
    use crate::wir;

    let source = r#"variables {
    global:
        0: g
    player:
        0: p
}
rule ("flow") {
    event { Ongoing - Global; }
    actions {
        disabled If(unknownIf());
            While(unknownWhile());
                For Global Variable(g, unknownStart(), unknownStop(), unknownStep());
                    For Player Variable(Event Player, p, unknownPStart(), unknownPStop(), unknownPStep());
                        Wait(unknownInner());
                    End;
                End;
            End;
        Else If(unknownElseIf());
            If(unknownNested());
                Wait(unknownThen());
            Else;
                Wait(unknownElse());
            End;
        Else;
            Wait(unknownOuterElse());
        End;
        Wait(unknownTail());
    }
}"#;
    let program = parsed(source);
    let materialized = program.to_wir().expect("materializes");
    let slice = |span: Option<Span>| {
        let span = span.expect("source span");
        let document = program.source(span.file).expect("source");
        &document.text()[document.byte_range(span).expect("range")]
    };
    let action = |id| materialized.actions.get(id).expect("action");
    let value = |id| slice(materialized.values.get(id).expect("value").span);
    let root = &materialized.rules.iter().next().unwrap().actions;
    let wir::Action::Disabled {
        action: inner,
        span,
    } = action(root[0])
    else {
        panic!("disabled")
    };
    assert!(slice(*span).contains("If(unknownIf())"));
    let wir::Action::If {
        branches,
        else_body: Some(outer_else),
        ..
    } = action(*inner)
    else {
        panic!("if")
    };
    assert_eq!(value(branches[0].condition), "unknownIf");
    assert_eq!(value(branches[1].condition), "unknownElseIf");
    let wir::Action::While {
        condition, body, ..
    } = action(branches[0].body[0])
    else {
        panic!("while")
    };
    assert_eq!(value(*condition), "unknownWhile");
    let wir::Action::ForGlobalVariable {
        start,
        stop,
        step,
        body,
        ..
    } = action(body[0])
    else {
        panic!("global for")
    };
    assert_eq!(
        [value(*start), value(*stop), value(*step)],
        ["unknownStart", "unknownStop", "unknownStep"]
    );
    let wir::Action::ForPlayerVariable {
        start,
        stop,
        step,
        body,
        ..
    } = action(body[0])
    else {
        panic!("player for")
    };
    assert_eq!(
        [value(*start), value(*stop), value(*step)],
        ["unknownPStart", "unknownPStop", "unknownPStep"]
    );
    assert_eq!(value(action(body[0]).value_args()[0]), "unknownInner");
    let wir::Action::If {
        branches: nested,
        else_body: Some(nested_else),
        ..
    } = action(branches[1].body[0])
    else {
        panic!("nested if")
    };
    assert_eq!(value(nested[0].condition), "unknownNested");
    assert_eq!(
        value(action(nested[0].body[0]).value_args()[0]),
        "unknownThen"
    );
    assert_eq!(value(action(nested_else[0]).value_args()[0]), "unknownElse");
    assert_eq!(
        value(action(outer_else[0]).value_args()[0]),
        "unknownOuterElse"
    );
    assert_eq!(value(action(root[1]).value_args()[0]), "unknownTail");

    let issues = materialized.semantic_issues(&catalog());
    for issue in &issues {
        assert_eq!(slice(issue.span), issue.name);
    }
    assert!(issues.iter().any(|issue| issue.name == "unknownElseIf"));
    assert!(issues.iter().any(|issue| issue.name == "unknownTail"));

    let artifact = MappedText {
        text: source.to_owned(),
        map: SourceMap::extract(&program),
    };
    let decoded = MappedText::from_json(&artifact.to_json()).expect("decode");
    let mut mapped = parsed(&decoded.text);
    decoded.map.apply(&mut mapped).expect("apply map");
    let mapped_issues = mapped
        .to_wir()
        .expect("materialize map")
        .semantic_issues(&catalog());
    assert_eq!(
        issues
            .iter()
            .map(|issue| (&issue.name, issue.span))
            .collect::<Vec<_>>(),
        mapped_issues
            .iter()
            .map(|issue| (&issue.name, issue.span))
            .collect::<Vec<_>>()
    );

    let mut identity_changed = program.clone();
    let Action::Call { name, .. } = identity_changed.rules[0].actions.last_mut().unwrap() else {
        panic!("tail")
    };
    *name = "differentAction".into();
    let lowered = identity_changed.to_wir().unwrap();
    let tail = lowered
        .actions
        .get(*lowered.rules.iter().next().unwrap().actions.last().unwrap())
        .unwrap();
    assert!(tail.span().is_none());
    assert_eq!(
        slice(lowered.values.get(tail.value_args()[0]).unwrap().span),
        "unknownTail"
    );
    let replacement = program.action_span(0, program.rules[0].actions.len() - 1);
    identity_changed
        .set_action_span(0, program.rules[0].actions.len() - 1, replacement)
        .unwrap();
    let lowered = identity_changed.to_wir().unwrap();
    assert_eq!(
        lowered
            .actions
            .get(*lowered.rules.iter().next().unwrap().actions.last().unwrap())
            .unwrap()
            .span(),
        replacement
    );

    let mut changed = program.clone();
    let original = changed.rules[0].actions.last().unwrap().clone();
    changed.rules[0].actions.pop();
    let reshaped = changed.to_wir().unwrap();
    assert!(
        reshaped
            .actions
            .get(reshaped.rules.iter().next().unwrap().actions[0])
            .unwrap()
            .span()
            .is_none()
    );
    changed.rules[0].actions.push(original.clone());
    let restored = changed.to_wir().unwrap();
    assert_eq!(
        restored
            .actions
            .get(restored.rules.iter().next().unwrap().actions[0])
            .unwrap()
            .span(),
        action(root[0]).span()
    );
    let Action::Call { args, .. } = changed.rules[0].actions.last_mut().unwrap() else {
        panic!("tail")
    };
    args[0] = Value::number(42.0);
    let modified = changed.to_wir().unwrap();
    assert!(
        modified
            .actions
            .get(
                *modified
                    .rules
                    .iter()
                    .next()
                    .unwrap()
                    .actions
                    .last()
                    .unwrap()
            )
            .unwrap()
            .span()
            .is_none()
    );
    assert_eq!(
        modified
            .actions
            .get(modified.rules.iter().next().unwrap().actions[0])
            .unwrap()
            .span(),
        action(root[0]).span()
    );
    *changed.rules[0].actions.last_mut().unwrap() = original;
    let restored = changed.to_wir().unwrap();
    assert_eq!(
        restored
            .actions
            .get(restored.rules.iter().next().unwrap().actions[1])
            .unwrap()
            .span(),
        action(root[1]).span()
    );
}
