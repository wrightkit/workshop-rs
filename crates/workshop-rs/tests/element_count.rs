use std::collections::HashSet;

use workshop_rs::actions::{ElementCountError, ElementNodeKind};
use workshop_rs::catalog::{Catalog, Locale};
use workshop_rs::convert::{self, ConvertOptions};
use workshop_rs::parser;
use workshop_rs::settings::{Settings, SettingsNode};
use workshop_rs::{Action, Event, Program, Rule, Value, Variable};

fn catalog() -> Catalog {
    Catalog::builtin().unwrap()
}

fn program_with_value(value: Value) -> Program {
    let mut program = Program::default();
    program.global_variable(Variable::new("result")).rule(
        Rule::new("count", Event::Global).action(Action::SetGlobalVariable {
            variable: "result".to_string(),
            value,
        }),
    );
    program
}

#[test]
fn ordinary_rule_and_action_have_structured_per_rule_counts() {
    let program = parser::parse(
        include_str!("fixtures/corpus/basic-rule.ws"),
        &catalog(),
        &Locale::new("en-US"),
    )
    .unwrap();
    let report = program.element_count(&catalog()).unwrap();

    assert_eq!(report.total, 2, "one rule plus one action");
    assert_eq!(report.rule_counts().collect::<Vec<_>>(), vec![("setup", 2)]);
    assert_eq!(report.rules[0].base_count, 1);
    assert_eq!(report.rules[0].children.len(), 1);
    assert_eq!(report.rules[0].children[0].count, 1);
}

#[test]
fn conditions_and_top_level_arguments_follow_the_documented_adjustments() {
    let program = parser::parse(
        "rule (\"condition\") { event { Ongoing - Global; } conditions { Is Game In Progress; } actions { Disable Inspector Recording; } }",
        &catalog(),
        &Locale::new("en-US"),
    )
    .unwrap();
    let report = program.element_count(&catalog()).unwrap();

    assert_eq!(report.total, 3, "rule + condition + action");
    let condition = &report.rules[0].children[0];
    assert_eq!(condition.kind, ElementNodeKind::Condition);
    assert_eq!(condition.count, 1);
    assert_eq!(condition.children[0].base_count, 1);
    assert_eq!(condition.children[0].adjustment, -1);
    assert_eq!(condition.children[0].count, 0);
}

#[test]
fn arrays_localized_strings_and_hero_pairs_are_visible_in_the_tree() {
    let array = Value::Array(vec![Value::number(1.0), Value::number(2.0)]);
    let array_program = program_with_value(array);
    let array_report = array_program.element_count(&catalog()).unwrap();
    assert_eq!(
        array_report.total, 7,
        "rule + action + (array 2 + numeric literals 4 - top-level 1)"
    );

    let hero_program = program_with_value(Value::Array(vec![
        Value::Enum {
            value_type: "Hero".to_string(),
            value: "ANA".to_string(),
        },
        Value::Enum {
            value_type: "Hero".to_string(),
            value: "DVA".to_string(),
        },
    ]));
    let hero_report = hero_program.element_count(&catalog()).unwrap();
    assert_eq!(
        hero_report.total, 8,
        "hero wrappers and the pair surcharge add elements"
    );
    assert_eq!(hero_report.rules[0].children[0].adjustment, 1);

    let localized_report = program_with_value(Value::LocalizedString("hello".to_string()))
        .element_count(&catalog())
        .unwrap();
    assert_eq!(
        localized_report.total, 3,
        "localized string costs two before top-level reduction"
    );
}

#[test]
fn wrapper_backed_enum_literals_count_the_wrapper() {
    let report = program_with_value(Value::Enum {
        value_type: "Map".to_string(),
        value: "AATLIS".to_string(),
    })
    .element_count(&catalog())
    .unwrap();

    assert_eq!(report.total, 3, "rule + action + Map wrapper/literal");
    assert_eq!(report.rules[0].children[0].children[0].base_count, 2);
}

#[test]
fn numeric_literals_and_player_variable_reads_match_reference_counts() {
    let numeric_report = program_with_value(Value::number(1.0))
        .element_count(&catalog())
        .unwrap();
    assert_eq!(numeric_report.total, 3, "rule + action + numeric literal");
    let number = &numeric_report.rules[0].children[0].children[0];
    assert_eq!(number.base_count, 2);
    assert_eq!(number.adjustment, -1);

    let program_with_global_value = |value| {
        let mut program = Program::new();
        program
            .global_variable(Variable::new("source"))
            .global_variable(Variable::new("result"))
            .rule(
                Rule::new("read global", Event::Global).action(Action::SetGlobalVariable {
                    variable: "result".to_string(),
                    value,
                }),
            );
        program
    };

    let global_report = program_with_global_value(Value::GlobalVariable("source".to_string()))
        .element_count(&catalog())
        .unwrap();
    assert_eq!(global_report.total, 3, "rule + action + global read");

    let array_read = program_with_global_value(Value::Call {
        name: "valueInArray".to_string(),
        args: vec![
            Value::GlobalVariable("source".to_string()),
            Value::number(12.0),
        ],
    })
    .element_count(&catalog())
    .unwrap();
    assert_eq!(array_read.total, 6, "rule + action + indexed global read");

    let player_variable = |name| Value::player_variable(Value::EventPlayer, name);
    let mut program = Program::new();
    for name in [
        "eventDurationHud",
        "eventDuration",
        "combatRegen",
        "nanoEffect",
        "hasNano",
    ] {
        program.player_variable(Variable::new(name));
    }
    program.subroutine(workshop_rs::Subroutine::new("setEventDuration"));
    program.rule(
        Rule::new(
            "duration",
            Event::Subroutine("setEventDuration".to_string()),
        )
        .action(Action::SetPlayerVariable {
            player: Value::EventPlayer,
            variable: "eventDurationHud".to_string(),
            value: player_variable("eventDuration"),
        }),
    );
    program.rule(
        Rule::new("regen", Event::EachPlayer)
            .action(Action::call(
                "skipIf",
                [player_variable("combatRegen"), Value::Bool(true)],
            ))
            .action(Action::SetPlayerVariable {
                player: Value::EventPlayer,
                variable: "combatRegen".to_string(),
                value: Value::Bool(true),
            }),
    );
    program.rule(
        Rule::new("nano", Event::EachPlayer)
            .action(Action::call(
                "destroyEffect",
                [player_variable("nanoEffect")],
            ))
            .action(Action::SetPlayerVariable {
                player: Value::EventPlayer,
                variable: "nanoEffect".to_string(),
                value: Value::Null,
            })
            .action(Action::SetPlayerVariable {
                player: Value::EventPlayer,
                variable: "hasNano".to_string(),
                value: Value::Bool(false),
            }),
    );

    let report = program.element_count(&catalog()).unwrap();
    assert_eq!(
        report
            .rule_counts()
            .map(|(_, count)| count)
            .collect::<Vec<_>>(),
        vec![4, 5, 6]
    );
}

#[test]
fn custom_string_counts_unused_format_slots() {
    let report = program_with_value(Value::Call {
        name: "customString".to_string(),
        args: vec![Value::String("abc".to_string())],
    })
    .element_count(&catalog())
    .unwrap();

    assert_eq!(
        report.total, 6,
        "rule + action + five-element custom string"
    );
    let custom_string = &report.rules[0].children[0].children[0];
    assert_eq!(custom_string.base_count, 4);
    assert_eq!(custom_string.children[0].count, 1);
}

#[test]
fn workshop_setting_lowerings_match_pinned_overpy_counts() {
    let integer_args = || {
        vec![
            Value::String("category".to_string()),
            Value::String("name".to_string()),
            Value::number(1.0),
            Value::number(0.0),
            Value::number(10.0),
            Value::number(1.0),
        ]
    };
    let cases = [
        ("workshopSettingInteger", integer_args(), 9, -4),
        ("createWorkshopSettingFloat", integer_args(), 9, -4),
        (
            "workshopSettingCombo",
            vec![
                Value::String("category".to_string()),
                Value::String("name".to_string()),
                Value::number(0.0),
                Value::Array(vec![
                    Value::String("one".to_string()),
                    Value::String("two".to_string()),
                ]),
                Value::number(1.0),
            ],
            10,
            -3,
        ),
        (
            "workshopSettingToggle",
            vec![
                Value::String("category".to_string()),
                Value::String("name".to_string()),
                Value::Bool(false),
                Value::number(1.0),
            ],
            7,
            -1,
        ),
    ];

    for (name, args, expected_total, expected_adjustment) in cases {
        let report = program_with_value(Value::Call {
            name: name.to_string(),
            args,
        })
        .element_count(&catalog())
        .unwrap();
        let setting = &report.rules[0].children[0].children[0];
        assert_eq!(report.total, expected_total, "{name}");
        assert_eq!(setting.base_count, 1, "{name}");
        assert_eq!(setting.adjustment, expected_adjustment, "{name}");
    }

    let hero_setting = program_with_value(Value::Call {
        name: "createWorkshopSettingHero".to_string(),
        args: vec![
            Value::String("category".to_string()),
            Value::String("name".to_string()),
            Value::Enum {
                value_type: "Hero".to_string(),
                value: "ANA".to_string(),
            },
            Value::number(1.0),
        ],
    })
    .element_count(&catalog())
    .unwrap();
    let setting = &hero_setting.rules[0].children[0].children[0];
    assert_eq!(hero_setting.total, 8, "createWorkshopSettingHero");
    assert_eq!(setting.base_count, 1);
    assert_eq!(setting.adjustment, -1);
}

#[test]
fn structured_comparisons_and_control_markers_match_reference_counts() {
    let catalog = catalog();
    let program = parser::parse(
        r#"rule ("control") { event { Ongoing - Global; } actions {
            If(Global.foo == 2);
                Set Global Variable(foo, 0);
            Else If(Global.foo == 3);
                Set Global Variable(foo, 1);
            Else;
                Set Global Variable(foo, 2);
            End;
            While(Global.foo == 4);
                Set Global Variable(foo, 3);
            End;
        } }"#,
        &catalog,
        &Locale::new("en-US"),
    )
    .unwrap();

    assert_eq!(
        program.element_count(&catalog).unwrap().total,
        30,
        "matches pinned OverPy 9.7.10 action annotations for this control flow"
    );

    let boolean_program = parser::parse(
        r#"rule ("boolean control") { event { Ongoing - Global; } actions {
            If(Or(Global.foo == 2, Global.foo == 3));
                Set Global Variable(foo, 0);
            End;
        } }"#,
        &catalog,
        &Locale::new("en-US"),
    )
    .unwrap();
    assert_eq!(
        boolean_program.element_count(&catalog).unwrap().total,
        17,
        "matches pinned OverPy 9.7.10 nested comparison annotations"
    );

    let numeric_program = parser::parse(
        r#"rule ("numeric comparison") { event { Ongoing - Global; } actions {
            If(2 < 3);
                Set Global Variable(foo, 0);
            End;
        } }"#,
        &catalog,
        &Locale::new("en-US"),
    )
    .unwrap();
    assert_eq!(
        numeric_program.element_count(&catalog).unwrap().total,
        10,
        "numeric comparisons do not add a boolean projection"
    );
}

#[test]
fn indexed_variable_targets_are_excluded_from_element_costs() {
    let catalog = catalog();
    let program = parser::parse(
        r#"variables {
            global: 0: values
            player: 0: state
        }
        rule ("indexed writes") { event { Ongoing - Each Player; } actions {
            Set Global Variable At Index(values, false, 5);
            Set Global Variable At Index(values, true, 6);
            Modify Global Variable At Index(values, 2, Add, 7);
            Set Player Variable At Index(Event Player, state, false, 5);
            Set Player Variable At Index(Event Player, state, true, 6);
            Modify Player Variable At Index(Event Player, state, 2, Add, 7);
            Stop Chasing Player Variable(Event Player, state);
        } }"#,
        &catalog,
        &Locale::new("en-US"),
    )
    .unwrap();

    assert_eq!(
        program.element_count(&catalog).unwrap().total,
        16,
        "matches the seven pinned OverPy 9.7.10 action annotations"
    );
}

#[test]
fn custom_settings_and_disabled_rules_actions_and_conditions_do_not_change_cost() {
    let catalog = catalog();
    let locale = Locale::new("en-US");
    let active = parser::parse(
        "rule (\"active\") { event { Ongoing - Global; } conditions { Is Game In Progress; } actions { Disable Inspector Recording; } }",
        &catalog,
        &locale,
    )
    .unwrap();
    let mut program = parser::parse(
        "disabled rule (\"disabled\") { event { Ongoing - Global; } conditions { disabled Is Game In Progress; } actions { disabled Disable Inspector Recording; } }",
        &catalog,
        &locale,
    )
    .unwrap();
    program.settings = Some(Settings {
        span: None,
        children: vec![SettingsNode::Raw {
            name: "project-defined".to_string(),
            value: "value".to_string(),
            span: None,
        }],
    });
    assert_eq!(
        active.element_count(&catalog).unwrap().total,
        program.element_count(&catalog).unwrap().total,
    );
    assert_eq!(program.element_count(&catalog).unwrap().total, 3);
}

#[test]
fn locale_conversion_preserves_the_canonical_count() {
    let catalog = catalog();
    let source = include_str!("fixtures/corpus/basic-rule.ws");
    let english = parser::parse(source, &catalog, &Locale::new("en-US")).unwrap();
    let converted = convert::convert(
        source,
        &catalog,
        &Locale::new("en-US"),
        &Locale::new("zh-CN"),
        &ConvertOptions::default(),
    )
    .unwrap();
    let chinese = parser::parse(&converted.text, &catalog, &Locale::new("zh-CN")).unwrap();

    assert_eq!(
        english.element_count(&catalog).unwrap().total,
        chinese.element_count(&catalog).unwrap().total
    );
}

#[test]
fn representative_corpus_program_produces_a_report() {
    let catalog = catalog();
    let program = parser::parse(
        include_str!("fixtures/corpus/expressions-values.ws"),
        &catalog,
        &Locale::new("en-US"),
    )
    .unwrap();
    let report = program.element_count(&catalog).unwrap();

    assert_eq!(report.rules.len(), 2);
    assert!(report.total > 2);
}

#[test]
fn public_report_preserves_rule_condition_and_action_order_with_spans() {
    let catalog = catalog();
    let source = "rule (\"ordered\") {\n    event { Ongoing - Global; }\n    conditions {\n        Is Game In Progress;\n        Is Game In Progress;\n    }\n    actions {\n        Disable Inspector Recording;\n        Disable Inspector Recording;\n    }\n}\n";
    let program = parser::parse(source, &catalog, &Locale::new("en-US")).unwrap();
    let report = program.element_count(&catalog).unwrap();
    let rule = &report.rules[0];

    assert_eq!(rule.kind, ElementNodeKind::Rule);
    assert_eq!(rule.span, program.rule_span(0));
    let mut ids = HashSet::new();
    assert_unique_node_ids(rule, &mut ids);

    let conditions = rule
        .children
        .iter()
        .filter(|node| node.kind == ElementNodeKind::Condition)
        .collect::<Vec<_>>();
    assert_eq!(conditions.len(), 2);
    for (index, condition) in conditions.iter().enumerate() {
        assert_eq!(condition.span, program.condition_span(0, index));
    }

    let actions = rule
        .children
        .iter()
        .filter(|node| node.kind == ElementNodeKind::Action)
        .collect::<Vec<_>>();
    assert_eq!(actions.len(), 2);
    for (index, action) in actions.iter().enumerate() {
        assert_eq!(action.span, program.action_span(0, index));
    }
}

#[test]
fn public_report_keeps_nested_values_and_actions_inspectable() {
    let mut program = Program::new();
    program.rule(
        Rule::new("nested", Event::Global)
            .action(Action::If {
                condition: Value::Bool(true),
            })
            .action(Action::call("wait", [Value::number(1.0)]))
            .action(Action::End),
    );

    let report = program.element_count(&catalog()).unwrap();
    let conditional = &report.rules[0].children[0];
    assert_eq!(conditional.kind, ElementNodeKind::Action);
    assert_eq!(conditional.name, "if");
    assert_eq!(conditional.children[0].kind, ElementNodeKind::Value);
    assert_eq!(conditional.children[1].kind, ElementNodeKind::Action);
    assert_eq!(conditional.children[1].name, "wait");
    assert_eq!(
        conditional.children[1].children[0].kind,
        ElementNodeKind::Value
    );
}

#[test]
fn public_api_rejects_unknown_actions_explicitly() {
    let mut unknown_action = Program::new();
    unknown_action.rule(
        Rule::new("unknown", Event::Global)
            .action(Action::call("notCanonicalAction", std::iter::empty())),
    );
    let unknown_error = unknown_action
        .element_count(&catalog())
        .expect_err("unknown action must not produce an exact count");
    assert!(matches!(
        unknown_error,
        ElementCountError::InvalidProgram { message } if message.contains("unknown action")
    ));
}

fn assert_unique_node_ids(node: &workshop_rs::actions::ElementCountNode, ids: &mut HashSet<usize>) {
    assert!(
        ids.insert(node.id),
        "duplicate report-local node id {}",
        node.id
    );
    for child in &node.children {
        assert_unique_node_ids(child, ids);
    }
}
