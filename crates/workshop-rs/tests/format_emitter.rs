use super::{internal, wir};
use workshop_rs::{Action, Event, Program, Rule, Value, Variable, emitter, parser, roundtrip};

#[test]
fn numeric_format_arguments_match_the_pinned_overpy_program_in_both_locales() {
    let mut program = Program::new();
    program.global_variable(Variable::new("g"));
    program.rule(
        Rule::new("optimized", Event::Global).action(Action::SetGlobalVariable {
            variable: "g".into(),
            value: Value::string("1 2"),
        }),
    );
    let mut disabled = Rule::new("disabled", Event::Global);
    for value in [
        Value::call(
            "format",
            [
                Value::string("{} {}"),
                Value::number(1.0),
                Value::number(2.0),
            ],
        ),
        Value::call(
            "customString",
            [
                Value::string("{0} {1} {2}"),
                Value::number(0.5),
                Value::global_variable("g"),
                Value::number(-3.0),
            ],
        ),
        Value::call(
            "customString",
            [
                Value::string("{0} x"),
                Value::call("add", [Value::number(1.0), Value::number(2.0)]),
            ],
        ),
    ] {
        disabled = disabled.action(Action::SetGlobalVariable {
            variable: "g".into(),
            value,
        });
    }
    program.rule(disabled);

    // OverPy 9.7.17, gitHead ec763c1d245aa5485ef61af5b2bda4975bce4877.
    let references = [
        (
            "en-US",
            r#"variables { global: 0: g }
rule ("optimized") { event { Ongoing - Global; } actions {
    Set Global Variable(g, Custom String("1 2"));
} }
rule ("disabled") { event { Ongoing - Global; } actions {
    Set Global Variable(g, Custom String("{0} {1}", 1, 2));
    Set Global Variable(g, Custom String("{0} {1} {2}", 0.5, Global.g, -3));
    Set Global Variable(g, Custom String("{0} x", Add(1, 2)));
} }"#,
        ),
        (
            "zh-CN",
            r#"变量 { 全局: 0: g }
规则 ("optimized") { 事件 { 持续 - 全局; } 动作 {
    设置全局变量(g, 自定义字符串("1 2"));
} }
规则 ("disabled") { 事件 { 持续 - 全局; } 动作 {
    设置全局变量(g, 自定义字符串("{0} {1}", 1, 2));
    设置全局变量(g, 自定义字符串("{0} {1} {2}", 0.5, 全局.g, -3));
    设置全局变量(g, 自定义字符串("{0} x", 加(1, 2)));
} }"#,
        ),
    ];
    let catalog = super::super::common::catalog();
    let mut differences = Vec::new();
    for (language, reference) in references {
        let locale = workshop_rs::catalog::Locale::new(language);
        let expected = parser::parse(reference, &catalog, &locale).expect("oracle output parses");
        let emitted = emitter::emit(&program, &catalog, &locale).expect("program emits");
        let actual = parser::parse(&emitted, &catalog, &locale).expect("emitted output parses");
        if !roundtrip::equivalent(&expected, &actual) {
            differences.push(format!("{language}: {emitted}"));
        }
    }
    assert!(
        differences.is_empty(),
        "canonical differences:\n{}",
        differences.join("\n")
    );
}

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
        ("{} {unfinished", "{0} {unfinished\", 3"),
        ("{1} {1}", "{1} {1}\", 3, Global.x"),
        ("中文 {0}", "中文 {0}\", 3, Global.x"),
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
