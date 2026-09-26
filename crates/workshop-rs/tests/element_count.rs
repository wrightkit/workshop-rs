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
        "rule + action + (array 2 - top-level 1) + two numbers at 2 each"
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
        "rule + action + array 1 + two hero constants at 2 each + the pair surcharge"
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
}

#[test]
fn disabled_conditions_and_actions_cost_what_enabled_ones_do() {
    let catalog = catalog();
    let locale = Locale::new("en-US");
    let enabled = parser::parse(
        r#"rule ("r") { event { Ongoing - Global; } conditions { Is Game In Progress; } actions { Wait(1, Ignore Condition); } }"#,
        &catalog,
        &locale,
    )
    .unwrap();
    let disabled = parser::parse(
        r#"rule ("r") { event { Ongoing - Global; } conditions { disabled Is Game In Progress; } actions { disabled Wait(1, Ignore Condition); } }"#,
        &catalog,
        &locale,
    )
    .unwrap();
    assert_eq!(
        disabled.element_count(&catalog).unwrap().total,
        enabled.element_count(&catalog).unwrap().total
    );
}

#[test]
fn player_variable_targets_count_nontrivial_player_expressions() {
    let catalog = catalog();
    let program = parser::parse(
        r#"variables {
            player: 0: state
        }
        rule ("player-variable chase") { event { Ongoing - Global; } actions {
            Stop Chasing Player Variable(First Of(All Players(All Teams)), state);
        } }
        rule ("indexed player-variable target") { event { Ongoing - Each Player; } actions {
            Set Player Variable At Index(First Of(All Players(All Teams)), state, false, 5);
        } }"#,
        &catalog,
        &Locale::new("en-US"),
    )
    .unwrap();

    let report = program.element_count(&catalog).unwrap();
    assert_eq!(
        report.rule_counts().collect::<Vec<_>>(),
        vec![("player-variable chase", 5), ("indexed player-variable target", 6)],
        "both target forms count the player expression with the direct-argument reduction"
    );
    assert_eq!(report.total, 11);
}

/// Workshop text compiled by pinned OverPy 9.7.10 with `#!debugElementCount`, and
/// the total that compiler reports. On a production project OverPy's total is
/// within 0.01% of the client's; each program here isolates one rule of the model. OverPy also closes the last `If`
/// of a subroutine rule with an `End`, which the canonical emitter omits, so no case
/// with such a rule is listed.
const OVERPY_COUNTS: &[(&str, usize, &str)] = &[
    (
        "wait_number",
        5,
        r#"rule ("r") {
    event {
        Ongoing - Each Player;
        All;
        All;
    }
    actions {
        Wait(4, Ignore Condition);
        Set Move Speed(Event Player, 50);
    }
}"#,
    ),
    (
        "custom_string_defaults",
        6,
        r#"rule ("r") {
    event {
        Ongoing - Each Player;
        All;
        All;
    }
    actions {
        Small Message(Event Player, Custom String("hello"));
    }
}"#,
    ),
    (
        "variable_reads",
        6,
        r#"variables {
    global:
        0: g
    player:
        0: p
}

rule ("r") {
    event {
        Ongoing - Each Player;
        All;
        All;
    }
    actions {
        Set Global Variable(g, (Event Player).p);
        Set Player Variable(Event Player, p, Global.g);
    }
}"#,
    ),
    (
        "compare_in_action",
        10,
        r#"variables {
    global:
        0: g
}

rule ("r") {
    event {
        Ongoing - Each Player;
        All;
        All;
    }
    actions {
        Wait Until(Compare(Global.g, ==, 3), 5);
        Set Global Variable(g, 1);
    }
}"#,
    ),
    (
        "if_else_end",
        19,
        r#"variables {
    global:
        0: g
}

rule ("r") {
    event {
        Ongoing - Each Player;
        All;
        All;
    }
    actions {
        Wait(1, Ignore Condition);
        If(Compare(Global.g, ==, 3));
            Set Global Variable(g, 1);
        Else;
            Set Global Variable(g, 2);
        End;
        Wait(2, Ignore Condition);
        Set Global Variable(g, 7);
    }
}"#,
    ),
    (
        "trailing_if_omits_end",
        11,
        r#"variables {
    global:
        0: g
}

rule ("r") {
    event {
        Ongoing - Each Player;
        All;
        All;
    }
    actions {
        Wait(1, Ignore Condition);
        If(Compare(Global.g, ==, 3));
            Set Global Variable(g, 1);
    }
}"#,
    ),
    (
        "while_keeps_end",
        10,
        r#"variables {
    global:
        0: g
}

rule ("r") {
    event {
        Ongoing - Each Player;
        All;
        All;
    }
    actions {
        While(Compare(Global.g, <, 3));
            Modify Global Variable(g, Add, 1);
        End;
    }
}"#,
    ),
    (
        "hero_pairs_per_argument",
        19,
        r#"variables {
    global:
        0: g
}

rule ("r") {
    event {
        Ongoing - Each Player;
        All;
        All;
    }
    actions {
        Set Global Variable(g, Array(Hero(Ana), Hero(Mercy)));
        Set Player Allowed Heroes(Event Player, Array(Hero(Ana), Hero(Mercy)));
        Set Player Allowed Heroes(Event Player, Hero(Ana));
        Set Player Allowed Heroes(Event Player, Hero(Mercy));
    }
}"#,
    ),
    (
        "team_and_color_constants",
        9,
        r#"variables {
    global:
        0: g
}

rule ("r") {
    event {
        Ongoing - Each Player;
        All;
        All;
    }
    actions {
        Set Global Variable(g, Color(Team 1));
        Set Global Variable(g, Team 2);
        Set Global Variable(g, Button(Reload));
        Set Global Variable(g, Map(Ilios));
    }
}"#,
    ),
];

#[test]
fn a_written_and_an_omitted_trailing_end_are_one_program_with_one_cost() {
    let catalog = catalog();
    let locale = Locale::new("en-US");
    let rule = |end: &str| {
        format!(
            r#"rule ("r") {{ event {{ Ongoing - Global; }} actions {{ If(True); Wait(1, Ignore Condition); {end} }} }}"#
        )
    };
    let omitted = parser::parse(&rule(""), &catalog, &locale).unwrap();
    let written = parser::parse(&rule("End;"), &catalog, &locale).unwrap();
    let omitted = omitted.element_count(&catalog).unwrap().total;
    assert_eq!(omitted, written.element_count(&catalog).unwrap().total);
    // rule 1 + If 1 + its condition 0 + Wait (1 + number 1 + behavior 0); no `End`
    assert_eq!(omitted, 4);
}

#[test]
fn only_the_last_action_of_a_rule_is_closed_without_end() {
    let catalog = catalog();
    let locale = Locale::new("en-US");
    let text = |body: &str| {
        format!(r#"rule ("r") {{ event {{ Ongoing - Global; }} actions {{ {body} }} }}"#)
    };
    let count = |body: &str| {
        parser::parse(&text(body), &catalog, &locale)
            .unwrap()
            .element_count(&catalog)
            .unwrap()
            .total
    };
    // the middle `If` and a nested last `If` keep their `End`
    assert_eq!(count("If(True); End; Wait(1, Ignore Condition);"), 5);
    assert_eq!(count("If(True); If(True); End; End;"), 4);
    // a loop keeps its `End` even as the last action
    assert_eq!(count("While(True); End;"), 3);
}

#[test]
fn the_count_is_that_of_the_canonical_emitted_form() {
    let catalog = catalog();
    let locale = Locale::new("en-US");
    for (name, _, text) in OVERPY_COUNTS {
        let program = parser::parse(text, &catalog, &locale).unwrap();
        let emitted = workshop_rs::emitter::emit(&program, &catalog, &locale).unwrap();
        let reparsed = parser::parse(&emitted, &catalog, &locale).unwrap();
        assert_eq!(
            program.element_count(&catalog).unwrap().total,
            reparsed.element_count(&catalog).unwrap().total,
            "{name}"
        );
    }
}

#[test]
fn counts_match_the_pinned_overpy_element_counter() {
    let catalog = catalog();
    for (name, expected, text) in OVERPY_COUNTS {
        let program = parser::parse(text, &catalog, &Locale::new("en-US")).unwrap();
        let report = program.element_count(&catalog).unwrap();
        assert_eq!(report.total, *expected, "{name}");
    }
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
