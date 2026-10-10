//! Contract tests for the `workshop-rs/rule-content-v1` structured export.
//!
//! Each node form in the published schema gets coverage here, and the
//! variant table below fails to compile or to match when a `Value` or
//! `Action` variant is added without a node form.

use serde_json::{Value as JsonValue, json};
use workshop_rs::rules::content::{RULE_CONTENT_V1, RULE_CONTENT_V1_SCHEMA, rule_content};
use workshop_rs::{
    Action, Condition, Event, EventTarget, EventTeam, ModifyOp, PlayerEventKind, Program, Rule,
    Subroutine, Value, emitter, parser,
};

use crate::common::{catalog, en, zh};

fn schema() -> jsonschema::JSONSchema {
    let document: JsonValue =
        serde_json::from_str(RULE_CONTENT_V1_SCHEMA).expect("schema parses as JSON");
    jsonschema::JSONSchema::compile(&document).expect("schema compiles")
}

fn assert_valid(content: &JsonValue) {
    let validator = schema();
    if let Err(errors) = validator.validate(content) {
        let errors = errors.map(|error| error.to_string()).collect::<Vec<_>>();
        panic!("rule-content-v1 output does not validate: {errors:?}\n{content}");
    }
}

fn content_of(rule: &Rule) -> JsonValue {
    let content = rule_content(rule).expect("rule content renders");
    assert_eq!(content["format"], RULE_CONTENT_V1);
    assert_valid(&content);
    content
}

fn parse_rule(source: &str) -> Rule {
    let program = parser::parse(source, &catalog(), &en()).expect("Workshop source parses");
    let [rule] = <[Rule; 1]>::try_from(program.rules).expect("one rule");
    rule
}

fn program_with(rule: Rule) -> Program {
    let mut program = Program::new();
    program.rule(rule);
    program
}

fn assert_rejects(document: &JsonValue) {
    let validator = schema();
    assert!(
        validator.validate(document).is_err(),
        "schema unexpectedly accepts {document}"
    );
}

#[test]
fn every_node_form_appears_in_a_rendered_rule() {
    let rule = Rule::new("node forms", Event::Global)
        .condition(Condition::new(Value::from(1.5)))
        .condition(Condition::new(Value::Bool(true)))
        .condition(Condition::new(Value::Null))
        .condition(Condition::new(Value::string("custom {0}")))
        .condition(Condition::new(Value::LocalizedString("hello".to_string())))
        .condition(Condition::new(Value::Enum {
            value_type: "Status".to_string(),
            value: "STUNNED".to_string(),
        }))
        .condition(Condition::new(Value::EventPlayer))
        .condition(Condition::new(Value::Array(vec![
            Value::from(1.0),
            Value::from(2.0),
        ])))
        .condition(Condition::new(Value::Vector {
            x: Box::new(Value::from(1.0)),
            y: Box::new(Value::from(2.0)),
            z: Box::new(Value::from(3.0)),
        }))
        .condition(Condition::new(Value::call(
            "compare",
            [Value::from(1.0), Value::from(2.0)],
        )));

    let content = content_of(&rule);
    let conditions = content["conditions"].as_array().expect("conditions");
    let values: Vec<&JsonValue> = conditions.iter().map(|entry| &entry["value"]).collect();
    assert_eq!(values[0], &json!(1.5));
    assert_eq!(values[1], &json!(true));
    assert_eq!(values[2], &JsonValue::Null);
    assert_eq!(values[3], &json!({ "string": "custom {0}" }));
    assert_eq!(values[4], &json!({ "localizedString": "hello" }));
    assert_eq!(values[5], &json!({ "enum": "Status", "member": "STUNNED" }));
    assert_eq!(values[6], &json!({ "call": "eventPlayer", "args": [] }));
    assert_eq!(values[7], &json!({ "call": "array", "args": [1.0, 2.0] }));
    assert_eq!(
        values[8],
        &json!({ "call": "vector", "args": [1.0, 2.0, 3.0] })
    );
    assert_eq!(values[9], &json!({ "call": "compare", "args": [1.0, 2.0] }));
}

#[test]
fn variable_and_subroutine_names_use_tagged_nodes() {
    let rule = Rule::new("declared names", Event::Global).action(Action::call(
        "startRule",
        [
            Value::Subroutine("tick".to_string()),
            Value::Enum {
                value_type: "StartRuleBehavior".to_string(),
                value: "RESTART".to_string(),
            },
        ],
    ));
    let content = content_of(&rule);
    assert_eq!(
        content["actions"][0],
        json!({
            "call": "startRule",
            "args": [
                { "subroutine": "tick" },
                { "enum": "StartRuleBehavior", "member": "RESTART" }
            ]
        })
    );
}

#[test]
fn every_action_form_appears_in_a_rendered_rule() {
    let rule = Rule::new("action forms", Event::Global)
        .action(Action::SetGlobalVariable {
            variable: "score".to_string(),
            value: Value::from(7.0),
        })
        .action(Action::ModifyGlobalVariable {
            variable: "score".to_string(),
            op: ModifyOp::Add,
            value: Value::from(1.0),
        })
        .action(Action::SetPlayerVariable {
            player: Value::EventPlayer,
            variable: "streak".to_string(),
            value: Value::from(0.0),
        })
        .action(Action::ModifyPlayerVariable {
            player: Value::EventPlayer,
            variable: "streak".to_string(),
            op: ModifyOp::AppendToArray,
            value: Value::from(3.0),
        })
        .action(Action::AssignMember {
            target: Value::call(
                "valueInArray",
                [Value::global_variable("score"), Value::from(0.0)],
            ),
            op: None,
            value: Value::from(9.0),
        })
        .action(Action::AssignMember {
            target: Value::call(
                "valueInArray",
                [Value::global_variable("score"), Value::from(1.0)],
            ),
            op: Some(ModifyOp::Min),
            value: Value::from(4.0),
        })
        .action(Action::CallSubroutine {
            subroutine: "tick".to_string(),
        })
        .action(Action::If {
            condition: Value::Bool(true),
        })
        .action(Action::ElseIf {
            condition: Value::Bool(false),
        })
        .action(Action::Else)
        .action(Action::End)
        .action(Action::While {
            condition: Value::Bool(false),
        })
        .action(Action::End)
        .action(Action::ForGlobalVariable {
            variable: "index".to_string(),
            start: Value::from(0.0),
            stop: Value::from(3.0),
            step: Value::from(1.0),
        })
        .action(Action::End)
        .action(Action::ForPlayerVariable {
            player: Value::EventPlayer,
            variable: "slot".to_string(),
            start: Value::from(0.0),
            stop: Value::from(2.0),
            step: Value::from(1.0),
        })
        .action(Action::End)
        .action(Action::call("wait", [Value::from(0.016)]))
        .action(Action::disabled(Action::call("abort", [])));

    let content = content_of(&rule);
    let actions = content["actions"].as_array().expect("actions");
    assert_eq!(actions.len(), 19);

    assert_eq!(
        actions[0],
        json!({"call": "setGlobalVariable", "args": [{"variable": "score"}, 7.0]})
    );
    assert_eq!(
        actions[1],
        json!({
            "call": "modifyGlobalVariable",
            "args": [{"variable": "score"}, {"call": "add", "args": []}, 1.0]
        })
    );
    assert_eq!(
        actions[2],
        json!({
            "call": "setPlayerVariable",
            "args": [{"call": "eventPlayer", "args": []}, {"variable": "streak"}, 0.0]
        })
    );
    assert_eq!(
        actions[3],
        json!({
            "call": "modifyPlayerVariable",
            "args": [
                {"call": "eventPlayer", "args": []},
                {"variable": "streak"},
                {"call": "appendToArray", "args": []},
                3.0
            ]
        })
    );
    assert_eq!(
        actions[4],
        json!({
            "call": "assignMember",
            "args": [
                {"call": "valueInArray", "args": [
                    {"call": "globalVariable", "args": [{"variable": "score"}]},
                    0.0
                ]},
                9.0
            ]
        })
    );
    assert_eq!(
        actions[5],
        json!({
            "call": "modifyMember",
            "args": [
                {"call": "valueInArray", "args": [
                    {"call": "globalVariable", "args": [{"variable": "score"}]},
                    1.0
                ]},
                {"call": "min", "args": []},
                4.0
            ]
        })
    );
    assert_eq!(
        actions[6],
        json!({"call": "callSubroutine", "args": [{"subroutine": "tick"}]})
    );
    assert_eq!(actions[7], json!({"call": "if", "args": [true]}));
    assert_eq!(actions[8], json!({"call": "elseIf", "args": [false]}));
    assert_eq!(actions[9], json!({"call": "else", "args": []}));
    assert_eq!(actions[10], json!({"call": "end", "args": []}));
    assert_eq!(actions[11], json!({"call": "while", "args": [false]}));
    assert_eq!(actions[12], json!({"call": "end", "args": []}));
    assert_eq!(
        actions[13],
        json!({
            "call": "forGlobalVariable",
            "args": [{"variable": "index"}, 0.0, 3.0, 1.0]
        })
    );
    assert_eq!(actions[14], json!({"call": "end", "args": []}));
    assert_eq!(
        actions[15],
        json!({
            "call": "forPlayerVariable",
            "args": [
                {"call": "eventPlayer", "args": []},
                {"variable": "slot"},
                0.0,
                2.0,
                1.0
            ]
        })
    );
    assert_eq!(actions[16], json!({"call": "end", "args": []}));
    assert_eq!(actions[17], json!({"call": "wait", "args": [0.016]}));
    assert_eq!(
        actions[18],
        json!({"call": "abort", "args": [], "disabled": true})
    );
}

#[test]
fn disabled_marks_wrap_their_nodes() {
    let mut rule = Rule::new("disabled", Event::Global);
    rule.disabled = true;
    rule.conditions.push(Condition::disabled(Value::call(
        "isAlive",
        [Value::EventPlayer],
    )));
    rule.actions.push(Action::disabled(Action::If {
        condition: Value::Bool(true),
    }));

    let content = content_of(&rule);
    assert_eq!(content["disabled"], true);
    assert_eq!(
        content["conditions"][0],
        json!({
            "value": {"call": "isAlive", "args": [{"call": "eventPlayer", "args": []}]},
            "disabled": true
        })
    );
    assert_eq!(
        content["actions"][0],
        json!({"call": "if", "args": [true], "disabled": true})
    );
}

#[test]
fn every_event_form_renders_its_id_and_filters() {
    let cases = [
        (Event::Global, json!({"id": "global"})),
        (Event::EachPlayer, json!({"id": "eachPlayer"})),
        (
            Event::EachPlayerWithFilters {
                team: EventTeam::Team1,
                target: EventTarget::Hero("ANA".to_string()),
            },
            json!({"id": "eachPlayer", "team": "TEAM_1", "player": "ANA"}),
        ),
        (
            Event::Player {
                kind: PlayerEventKind::Died,
                team: EventTeam::All,
                target: EventTarget::Slot(3),
            },
            json!({"id": "playerDied", "team": "ALL", "player": "SLOT_3"}),
        ),
        (
            Event::Subroutine("tick".to_string()),
            json!({"id": "subroutine", "subroutine": "tick"}),
        ),
    ];
    for (event, expected) in cases {
        let content = content_of(&Rule::new("event", event));
        assert_eq!(content["event"], expected, "event {expected}");
    }
}

#[test]
fn modify_op_and_event_kind_catalog_ids_are_public() {
    for (op, id) in [
        (ModifyOp::Add, "add"),
        (ModifyOp::Subtract, "subtract"),
        (ModifyOp::Multiply, "multiply"),
        (ModifyOp::Divide, "divide"),
        (ModifyOp::Modulo, "modulo"),
        (ModifyOp::Min, "min"),
        (ModifyOp::Max, "max"),
        (ModifyOp::RaiseToPower, "raiseToPower"),
        (ModifyOp::AppendToArray, "appendToArray"),
        (ModifyOp::RemoveFromArrayByValue, "removeFromArrayByValue"),
        (ModifyOp::RemoveFromArrayByIndex, "removeFromArrayByIndex"),
    ] {
        assert_eq!(op.catalog_id(), id);
    }
    assert_eq!(
        PlayerEventKind::DealtKnockback.catalog_id(),
        "playerDealtKnockback"
    );
    assert_eq!(Event::Global.catalog_id(), "global");
    assert_eq!(Event::EachPlayer.catalog_id(), "eachPlayer");
    assert_eq!(
        Event::EachPlayerWithFilters {
            team: EventTeam::All,
            target: EventTarget::All,
        }
        .catalog_id(),
        "eachPlayer"
    );
    assert_eq!(
        Event::Player {
            kind: PlayerEventKind::Left,
            team: EventTeam::All,
            target: EventTarget::All,
        }
        .catalog_id(),
        "playerLeft"
    );
    assert_eq!(
        Event::Subroutine("tick".to_string()).catalog_id(),
        "subroutine"
    );
}

#[test]
fn variable_reads_render_as_reserved_calls() {
    let rule = Rule::new("reads", Event::Global).condition(Condition::new(Value::call(
        "==",
        [
            Value::GlobalVariable("score".to_string()),
            Value::PlayerVariable {
                player: Box::new(Value::EventPlayer),
                variable: "streak".to_string(),
            },
        ],
    )));
    let content = content_of(&rule);
    assert_eq!(
        content["conditions"][0]["value"],
        json!({
            "call": "==",
            "args": [
                {"call": "globalVariable", "args": [{"variable": "score"}]},
                {"call": "playerVariable", "args": [
                    {"call": "eventPlayer", "args": []},
                    {"variable": "streak"}
                ]}
            ]
        })
    );
}

#[test]
fn proximity_repel_actions_match_the_acceptance_examples() {
    let rule = parse_rule(
        r#"rule ("repel") {
    event {
        Ongoing - Each Player;
        All;
        All;
    }
    actions {
        Apply Impulse(Event Player, Direction Towards(Closest Player To(Event Player, Team 2), Event Player), 150, To World, Cancel Contrary Motion);
        Set Status(Event Player, Null, Stunned, 0.75);
    }
}
"#,
    );
    let content = content_of(&rule);
    let actions = content["actions"].as_array().expect("actions");

    assert_eq!(actions[0]["call"], "applyImpulse");
    assert_eq!(actions[0]["args"][2], json!(150.0));
    assert_eq!(
        actions[0]["args"][4],
        json!({"enum": "Impulse", "member": "CANCEL_CONTRARY_MOTION"})
    );

    assert_eq!(actions[1]["call"], "setStatusEffect");
    assert_eq!(
        actions[1]["args"][2],
        json!({"enum": "Status", "member": "STUNNED"})
    );
    assert_eq!(actions[1]["args"][3], json!(0.75));
}

#[test]
fn output_is_identical_across_locales() {
    let source = r#"rule ("repel") {
    event {
        Ongoing - Each Player;
        All;
        All;
    }
    conditions {
        Distance Between(Event Player, Closest Player To(Event Player, Team 2)) < 6;
    }
    actions {
        Apply Impulse(Event Player, Direction Towards(Closest Player To(Event Player, Team 2), Event Player), 150, To World, Cancel Contrary Motion);
        Set Status(Event Player, Null, Stunned, 0.75);
    }
}
"#;
    let catalog = catalog();
    let english = parser::parse(source, &catalog, &en()).expect("en-US parses");
    let chinese_source = emitter::emit(&english, &catalog, &zh()).expect("zh-CN emission");
    let chinese = parser::parse(&chinese_source, &catalog, &zh()).expect("zh-CN parses");

    let english_content = content_of(&english.rules[0]);
    let chinese_content = content_of(&chinese.rules[0]);
    assert_eq!(english_content, chinese_content);
}

#[test]
fn output_is_deterministic_and_flat() {
    let rule = Rule::new("flat", Event::Global)
        .action(Action::If {
            condition: Value::Bool(true),
        })
        .action(Action::call("wait", [Value::from(0.5)]))
        .action(Action::End);
    let program = program_with(rule);
    let first = content_of(&program.rules[0]);
    let second = content_of(&program.rules[0]);
    assert_eq!(first, second);
    // Index i in actions is Rule.actions[i]: the If body does not nest.
    assert_eq!(first["actions"].as_array().expect("actions").len(), 3);
}

#[test]
fn every_value_variant_has_a_node_form() {
    // Exhaustive over the public `Value` enum: a new variant that lacks a
    // node form stops compiling here instead of rendering silently wrong.
    let forms = |value: &Value| match value {
        Value::Number(_) | Value::String(_) | Value::Bool(_) | Value::Null => "literal",
        Value::LocalizedString(_) => "localizedString",
        Value::Enum { .. } => "enum",
        Value::Array(_) => "array",
        Value::Vector { .. } => "vector",
        Value::GlobalVariable(_) | Value::PlayerVariable { .. } => "variable read",
        Value::Subroutine(_) => "subroutine",
        Value::EventPlayer => "eventPlayer",
        Value::Call { .. } => "call",
    };
    for value in [
        Value::from(1.0),
        Value::string("s"),
        Value::Bool(true),
        Value::Null,
        Value::LocalizedString("hello".to_string()),
        Value::Enum {
            value_type: "Status".to_string(),
            value: "STUNNED".to_string(),
        },
        Value::Array(vec![]),
        Value::Vector {
            x: Box::new(Value::from(0.0)),
            y: Box::new(Value::from(0.0)),
            z: Box::new(Value::from(0.0)),
        },
        Value::GlobalVariable("g".to_string()),
        Value::PlayerVariable {
            player: Box::new(Value::EventPlayer),
            variable: "p".to_string(),
        },
        Value::Subroutine("s".to_string()),
        Value::EventPlayer,
        Value::call(
            "vector",
            [Value::from(1.0), Value::from(2.0), Value::from(3.0)],
        ),
    ] {
        let _ = forms(&value);
    }
}

#[test]
fn every_action_variant_has_a_node_form() {
    // Exhaustive over the public `Action` enum for the same reason as the
    // `Value` coverage above.
    let forms = |action: &Action| match action {
        Action::SetGlobalVariable { .. } | Action::SetPlayerVariable { .. } => "set",
        Action::ModifyGlobalVariable { .. } | Action::ModifyPlayerVariable { .. } => "modify",
        Action::AssignMember { .. } => "assignMember",
        Action::CallSubroutine { .. } => "callSubroutine",
        Action::If { .. } | Action::ElseIf { .. } | Action::Else | Action::While { .. } => {
            "control flow"
        }
        Action::ForGlobalVariable { .. } | Action::ForPlayerVariable { .. } => "for",
        Action::End => "end",
        Action::Disabled { .. } => "disabled",
        Action::Call { .. } => "call",
    };
    let _ = forms(&Action::End);
}

#[test]
fn schema_rejects_missing_and_unknown_fields() {
    let rule = Rule::new("schema", Event::Global).action(Action::call("wait", [Value::from(1.0)]));
    let content = content_of(&rule);

    let mut missing_format = content.clone();
    missing_format
        .as_object_mut()
        .expect("object")
        .remove("format");
    assert_rejects(&missing_format);

    let mut wrong_format = content.clone();
    *wrong_format
        .as_object_mut()
        .expect("object")
        .get_mut("format")
        .expect("format") = json!("workshop-rs/rule-content-v2");
    assert_rejects(&wrong_format);

    let mut extra_field = content.clone();
    extra_field
        .as_object_mut()
        .expect("object")
        .insert("spans".to_string(), json!([]));
    assert_rejects(&extra_field);

    let mut unknown_node = content.clone();
    unknown_node["actions"][0]["args"]
        .as_array_mut()
        .expect("args")
        .push(json!({"surprise": 1.0}));
    assert_rejects(&unknown_node);

    let mut node_missing_args = content.clone();
    node_missing_args["actions"][0]
        .as_object_mut()
        .expect("action")
        .remove("args");
    assert_rejects(&node_missing_args);
}

#[test]
fn parsed_program_rules_render_through_the_public_api() {
    let source = r#"rule ("api") {
    event {
        Ongoing - Global;
    }
    actions {
        Wait(0.25, Ignore Condition);
    }
}
"#;
    let program = parser::parse(source, &catalog(), &en()).expect("source parses");
    let content = program.rules[0].content().expect("Rule::content renders");
    assert_valid(&content);
    assert_eq!(
        content["actions"][0],
        json!({
            "call": "wait",
            "args": [0.25, {"enum": "Wait", "member": "IGNORE_CONDITION"}]
        })
    );
}

#[test]
fn internal_sentinels_never_reach_the_document() {
    let mut program = Program::new();
    program.subroutine(Subroutine::new("tick"));
    program.rule(Rule::new("sentinel", Event::Global).action(Action::Call {
        name: "__ambiguous_enum".to_string(),
        args: Vec::new(),
    }));
    let error = rule_content(&program.rules[0]).expect_err("sentinel is rejected");
    assert!(error.to_string().contains("__ambiguous_enum"), "{error}");
}
