use std::collections::HashSet;

use workshop_rs::actions::{ElementCountError, ElementNodeKind};
use workshop_rs::catalog::{Catalog, Locale};
use workshop_rs::convert::{self, ConvertOptions};
use workshop_rs::parser;
use workshop_rs::settings::{Settings, SettingsNode};
use workshop_rs::{Action, Condition, Event, Program, Rule, Value, Variable};

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
        array_report.total, 5,
        "rule + action + (array 2 + literals 2 - top-level 1)"
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
    assert_eq!(hero_report.total, 6, "hero pair surcharge adds one element");
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
fn custom_settings_and_disabled_rules_do_not_change_cost() {
    let mut program = parser::parse(
        "disabled rule (\"disabled\") { event { Ongoing - Global; } actions { Disable Inspector Recording; } }",
        &catalog(),
        &Locale::new("en-US"),
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
    assert_eq!(program.element_count(&catalog()).unwrap().total, 2);
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
fn public_api_rejects_unsupported_and_invalid_programs_explicitly() {
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

    let mut invalid = Program::new();
    invalid.rule(
        Rule::new("invalid", Event::Global).condition(Condition::disabled(Value::Bool(true))),
    );
    let invalid_error = invalid
        .element_count(&catalog())
        .expect_err("unsupported condition must not produce an exact count");
    assert!(matches!(
        invalid_error,
        ElementCountError::InvalidProgram { message } if message.contains("disabled condition")
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
