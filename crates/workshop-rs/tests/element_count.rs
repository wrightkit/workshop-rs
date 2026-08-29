use workshop_rs::catalog::{Catalog, Locale};
use workshop_rs::convert::{self, ConvertOptions};
use workshop_rs::element_count::ElementNodeKind;
use workshop_rs::parser;
use workshop_rs::settings::{Settings, SettingsNode};
use workshop_rs::source::SourceFile;
use workshop_rs::wir::{self, Action, Event, Program, Rule, Value, ValueNode};

fn catalog() -> Catalog {
    Catalog::builtin().unwrap()
}

fn number(program: &mut Program, value: f64) -> wir::ValueId {
    program.values.push(ValueNode::new(
        Value::Number {
            value,
            text: value.to_string(),
        },
        None,
    ))
}

fn program_with_value(value: Value) -> Program {
    let mut program = Program::default();
    program.files.push(SourceFile::new("element-count.ws"));
    let variable = program.global_variables.push(wir::WorkshopVariable {
        name: "result".to_string(),
        index: 0,
        span: None,
        name_span: None,
    });
    let value = program.values.push(ValueNode::new(value, None));
    let action = program.actions.push(Action::SetGlobalVariable {
        variable,
        value,
        span: None,
        target_span: None,
    });
    program.rules.push(Rule {
        name: "count".to_string(),
        span: None,
        name_span: None,
        disabled: false,
        event: Event::Global,
        conditions: vec![],
        actions: vec![action],
    });
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
    let mut array_program = Program::default();
    let first = number(&mut array_program, 1.0);
    let second = number(&mut array_program, 2.0);
    let array = array_program
        .values
        .push(ValueNode::new(Value::Array(vec![first, second]), None));
    let array_program = attach_value(array_program, array);
    let array_report = array_program.element_count(&catalog()).unwrap();
    assert_eq!(
        array_report.total, 5,
        "rule + action + (array 2 + literals 2 - top-level 1)"
    );

    let mut hero_program = Program::default();
    let ana = hero_program.values.push(ValueNode::new(
        Value::Enum {
            value_type: "Hero".to_string(),
            value: "ANA".to_string(),
        },
        None,
    ));
    let dva = hero_program.values.push(ValueNode::new(
        Value::Enum {
            value_type: "Hero".to_string(),
            value: "DVA".to_string(),
        },
        None,
    ));
    let heroes = hero_program
        .values
        .push(ValueNode::new(Value::Array(vec![ana, dva]), None));
    let hero_program = attach_value(hero_program, heroes);
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

fn attach_value(mut program: Program, value: wir::ValueId) -> Program {
    let variable = program.global_variables.push(wir::WorkshopVariable {
        name: "result".to_string(),
        index: 0,
        span: None,
        name_span: None,
    });
    let action = program.actions.push(Action::SetGlobalVariable {
        variable,
        value,
        span: None,
        target_span: None,
    });
    program.rules.push(Rule {
        name: "count".to_string(),
        span: None,
        name_span: None,
        disabled: false,
        event: Event::Global,
        conditions: vec![],
        actions: vec![action],
    });
    program
}
