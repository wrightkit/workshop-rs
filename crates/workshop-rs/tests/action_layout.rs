use workshop_rs::catalog::{Catalog, Locale};
use workshop_rs::emitter;
use workshop_rs::wir::{self, Action, Event, Value, ValueNode};

fn program_with_structured_actions() -> (wir::Program, Vec<wir::ActionId>) {
    let mut program = wir::Program::default();
    let global = program.global_variables.push(wir::WorkshopVariable {
        name: "index".into(),
        index: 0,
        span: None,
        name_span: None,
    });
    let player = program.player_variables.push(wir::WorkshopVariable {
        name: "index".into(),
        index: 0,
        span: None,
        name_span: None,
    });
    let condition = program.values.push(ValueNode::new(Value::Bool(true), None));
    let number = program.values.push(ValueNode::new(
        Value::Number {
            value: 0.0,
            text: "0".into(),
        },
        None,
    ));
    fn leaf(program: &mut wir::Program, number: wir::ValueId) -> wir::ActionId {
        program.actions.push(Action::Call {
            name: "wait".into(),
            args: vec![number],
            span: None,
        })
    }
    let if_body = leaf(&mut program, number);
    let else_body = leaf(&mut program, number);
    let if_action = program.actions.push(Action::If {
        branches: vec![wir::IfBranch {
            condition,
            body: vec![if_body],
        }],
        else_body: Some(vec![else_body]),
        span: None,
    });
    let while_body = leaf(&mut program, number);
    let while_action = program.actions.push(Action::While {
        condition,
        body: vec![while_body],
        span: None,
    });
    let for_global_body = leaf(&mut program, number);
    let for_global = program.actions.push(Action::ForGlobalVariable {
        variable: global,
        start: number,
        stop: number,
        step: number,
        body: vec![for_global_body],
        span: None,
        target_span: None,
    });
    let nested_if_body = if_action;
    let for_player = program.actions.push(Action::ForPlayerVariable {
        player: program
            .values
            .push(ValueNode::new(Value::EventPlayer, None)),
        variable: player,
        start: number,
        stop: number,
        step: number,
        body: vec![nested_if_body],
        span: None,
        target_span: None,
    });
    let trailing_leaf = leaf(&mut program, number);
    let actions = vec![
        if_action,
        while_action,
        for_global,
        for_player,
        trailing_leaf,
    ];
    program.rules.push(wir::Rule {
        name: "layout".into(),
        span: None,
        name_span: None,
        disabled: false,
        event: Event::Global,
        conditions: vec![],
        actions: actions.clone(),
    });
    (program, actions)
}

#[test]
fn structured_action_widths_count_native_expansion() {
    let (program, actions) = program_with_structured_actions();
    let catalog = Catalog::builtin().unwrap();
    let locale = Locale::new("en-US");

    assert_eq!(
        emitter::action_width(&program, &catalog, &locale, &actions[..1])
            .unwrap()
            .width,
        5
    );
    assert_eq!(
        emitter::action_width(&program, &catalog, &locale, &actions[1..2])
            .unwrap()
            .width,
        3
    );
    assert_eq!(
        emitter::action_width(&program, &catalog, &locale, &actions[2..3])
            .unwrap()
            .width,
        3
    );
    assert_eq!(
        emitter::action_width(&program, &catalog, &locale, &actions[3..4])
            .unwrap()
            .width,
        7
    );
}

#[test]
fn layout_matches_canonical_emission_for_a_nested_sequence() {
    let (program, actions) = program_with_structured_actions();
    let catalog = Catalog::builtin().unwrap();
    let locale = Locale::new("en-US");
    let emitted = emitter::emit(&program, &catalog, &locale).unwrap();
    let action_text = emitted
        .split_once("actions {\n")
        .unwrap()
        .1
        .split_once("\n    }")
        .unwrap()
        .0;
    let emitted_width = action_text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .count();
    let layout = emitter::action_width(&program, &catalog, &locale, &actions).unwrap();
    assert_eq!(layout.width, emitted_width);
    assert_eq!(layout.width, 19);
}

#[test]
fn invalid_layout_requests_fail_explicitly() {
    let mut program = wir::Program::default();
    let dangling = wir::ActionId::from_index(0);
    program.rules.push(wir::Rule {
        name: "invalid".into(),
        span: None,
        name_span: None,
        disabled: false,
        event: Event::Global,
        conditions: vec![],
        actions: vec![dangling],
    });
    let error = emitter::action_width(
        &program,
        &Catalog::builtin().unwrap(),
        &Locale::new("en-US"),
        &[dangling],
    )
    .unwrap_err();
    assert!(matches!(
        error,
        emitter::ActionLayoutError::InvalidWIR(wir::error::IrError::DanglingReference {
            what: "action",
            id: 0
        })
    ));
}
