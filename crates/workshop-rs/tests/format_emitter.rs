use super::{internal, wir};

#[test]
fn implicit_format_placeholders_renumber_to_the_oracle_form() {
    let mut program = wir::Program::default();
    let file = program
        .files
        .push(workshop_rs::source::SourceFile::new("workshop.txt"));
    let text = program.values.push(workshop_rs::wir::ValueNode::new(
        workshop_rs::wir::Value::String("v: {}".to_string()),
        None,
    ));
    let variable = program.values.push(workshop_rs::wir::ValueNode::new(
        workshop_rs::wir::Value::GlobalVariable(workshop_rs::core::ids::Id::from_index(0)),
        None,
    ));
    let call = program.values.push(workshop_rs::wir::ValueNode::new(
        workshop_rs::wir::Value::Call {
            name: "format".to_string(),
            args: vec![text, variable],
        },
        None,
    ));
    let action = program.actions.push(wir::Action::SetGlobalVariable {
        variable: workshop_rs::core::ids::Id::from_index(1),
        value: call,
        span: None,
        target_span: None,
    });
    program.global_variables.push(wir::WorkshopVariable {
        name: "x".into(),
        index: 0,
        span: None,
        name_span: None,
    });
    program.global_variables.push(wir::WorkshopVariable {
        name: "z".into(),
        index: 1,
        span: None,
        name_span: None,
    });
    program.rules.push(wir::Rule {
        name: "r".into(),
        span: None,
        name_span: None,
        disabled: false,
        event: wir::Event::Global,
        conditions: vec![],
        actions: vec![action],
    });
    let _ = file;
    let emitted = internal::emit(&program);
    assert!(
        emitted.contains("Set Global Variable(z, Custom String(\"v: {0}\", Global.x));"),
        "implicit placeholders renumber to the oracle form: {emitted}"
    );
    let reparsed = internal::parse(&emitted);
    let reemitted = internal::emit(&reparsed);
    assert_eq!(
        emitted, reemitted,
        "renumbered format must be a byte-identical fixed point"
    );
}

#[test]
fn format_placeholder_boundaries_preserve_fallback_and_reparse() {
    for (text, expected) in [
        ("{} {1}", "{} {1}\", 3, Global.x"),
        ("{2}", "{2}\", 3, Global.x"),
        ("{x}", "{x}\", 3, Global.x"),
        ("{} {unfinished", "3 {unfinished\""),
        ("{1} {1}", "{0} {1}\", Global.x, Global.x"),
        ("中文 {0}", "中文 3\""),
    ] {
        let mut program = wir::Program::default();
        let text = program
            .values
            .push(wir::ValueNode::new(wir::Value::String(text.into()), None));
        let number = program.values.push(wir::ValueNode::new(
            wir::Value::Number {
                value: 3.0,
                text: "3".into(),
            },
            None,
        ));
        let variable = program.values.push(wir::ValueNode::new(
            wir::Value::GlobalVariable(workshop_rs::core::ids::Id::from_index(0)),
            None,
        ));
        let call = program.values.push(wir::ValueNode::new(
            wir::Value::Call {
                name: "format".into(),
                args: vec![text, number, variable],
            },
            None,
        ));
        let action = program.actions.push(wir::Action::SetGlobalVariable {
            variable: workshop_rs::core::ids::Id::from_index(0),
            value: call,
            span: None,
            target_span: None,
        });
        program.global_variables.push(wir::WorkshopVariable {
            name: "x".into(),
            index: 0,
            span: None,
            name_span: None,
        });
        program.rules.push(wir::Rule {
            name: "r".into(),
            span: None,
            name_span: None,
            disabled: false,
            event: wir::Event::Global,
            conditions: vec![],
            actions: vec![action],
        });
        let emitted = internal::emit(&program);
        assert!(
            emitted.contains(&format!("Custom String(\"{expected})")),
            "{emitted}"
        );
        let reparsed = internal::parse(&emitted);
        assert_eq!(emitted, internal::emit(&reparsed));
    }
}
