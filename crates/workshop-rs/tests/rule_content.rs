use serde_json::{Value as JsonValue, json};
use sha2::{Digest, Sha256};
use workshop_rs::catalog::Kind;
use workshop_rs::lookup::{LookupMatch, Signature};
use workshop_rs::rules::content::{RULE_CONTENT_V1, RULE_CONTENT_V1_SCHEMA, rule_content};
use workshop_rs::{
    Action, Condition, Event, EventTarget, EventTeam, ModifyOp, PlayerEventKind, Program, Rule,
    Subroutine, Value, emitter, parser,
};

use crate::common::{catalog, en, fixture, locale, zh};

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

fn ambiguous_value(spelling: &str, candidates: &[(&str, &str)]) -> Value {
    Value::call(
        "__ambiguous_enum",
        [
            Value::string(spelling),
            Value::Array(
                candidates
                    .iter()
                    .map(|(value_type, value)| Value::Enum {
                        value_type: value_type.to_string(),
                        value: value.to_string(),
                    })
                    .collect(),
            ),
        ],
    )
}

fn node_forms_rule() -> Rule {
    Rule::new("node forms", Event::Global)
        .condition(Condition::new(Value::from(1.5)))
        .condition(Condition::new(Value::Bool(true)))
        .condition(Condition::new(Value::Null))
        .condition(Condition::new(Value::string("custom {0}")))
        .condition(Condition::new(Value::LocalizedString("hello".to_string())))
        .condition(Condition::new(Value::Enum {
            value_type: "Status".to_string(),
            value: "STUNNED".to_string(),
        }))
        .condition(Condition::new(ambiguous_value(
            "Up",
            &[("Vector", "UP"), ("Rounding", "UP")],
        )))
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
        )))
}

#[test]
fn every_node_form_appears_in_a_rendered_rule() {
    let rule = node_forms_rule();
    let content = content_of(&rule);
    let conditions = content["conditions"].as_array().expect("conditions");
    let values: Vec<&JsonValue> = conditions.iter().map(|entry| &entry["value"]).collect();
    assert_eq!(values[0], &json!(1.5));
    assert_eq!(values[1], &json!(true));
    assert_eq!(values[2], &JsonValue::Null);
    assert_eq!(values[3], &json!({ "string": "custom {0}" }));
    assert_eq!(values[4], &json!({ "localizedString": "hello" }));
    assert_eq!(values[5], &json!({ "enum": "Status", "member": "STUNNED" }));
    assert_eq!(
        values[6],
        &json!({
            "ambiguous": "Up",
            "candidates": [
                {"enum": "Vector", "member": "UP"},
                {"enum": "Rounding", "member": "UP"}
            ]
        })
    );
    assert_eq!(values[7], &json!({ "call": "eventPlayer", "args": [] }));
    assert_eq!(values[8], &json!({ "call": "array", "args": [1.0, 2.0] }));
    assert_eq!(
        values[9],
        &json!({ "call": "vector", "args": [1.0, 2.0, 3.0] })
    );
    assert_eq!(
        values[10],
        &json!({ "call": "compare", "args": [1.0, 2.0] })
    );
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

fn action_forms_rule() -> Rule {
    Rule::new("action forms", Event::Global)
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
        .action(Action::disabled(Action::call("abort", [])))
}

#[test]
fn every_action_form_appears_in_a_rendered_rule() {
    let rule = action_forms_rule();
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

const STRUCTURAL_CALL_IDS: &[&str] = &[
    "setGlobalVariable",
    "modifyGlobalVariable",
    "setPlayerVariable",
    "modifyPlayerVariable",
    "forGlobalVariable",
    "forPlayerVariable",
    "if",
    "elseIf",
    "else",
    "while",
    "end",
    "callSubroutine",
];

#[test]
fn every_emitted_structural_call_matches_its_lookup_signature_arity() {
    let catalog = catalog();
    let content = content_of(&action_forms_rule());
    let mut covered = std::collections::BTreeSet::new();
    for action in content["actions"].as_array().expect("actions") {
        let call = action["call"].as_str().expect("a call id");
        if !STRUCTURAL_CALL_IDS.contains(&call) {
            continue;
        }
        covered.insert(call);
        let arg_count = action["args"].as_array().expect("args").len();
        let matches = catalog.lookup(&en(), call).expect("lookup answers");
        let signature = matches
            .iter()
            .find_map(|lookup| match lookup {
                LookupMatch::Builtin {
                    kind,
                    id,
                    signature,
                    ..
                } if *kind == Kind::Structural && id == call => signature.as_ref(),
                _ => None,
            })
            .unwrap_or_else(|| {
                panic!("'{call}' must resolve to a structural signature: {matches:?}")
            });
        assert_eq!(
            signature.params.len(),
            arg_count,
            "'{call}' declares {} parameter(s) for an emitted {arg_count}-argument call",
            signature.params.len()
        );
    }
    assert_eq!(
        covered.len(),
        STRUCTURAL_CALL_IDS.len(),
        "the fixture must emit every structural call id: {covered:?}"
    );
}

const FOLDED_TARGET_SOURCE: &str = r#"variables {
    global:
        0: probe_g
    player:
        0: probe_p
}

rule ("fold") {
    event {
        Ongoing - Each Player;
        All;
        All;
    }
    actions {
        Set Global Variable At Index(probe_g, 0, 5);
        Set Player Variable At Index(Event Player, probe_p, 0, 5);
        Modify Global Variable At Index(probe_g, 0, Add, 5);
        Modify Player Variable At Index(Event Player, probe_p, 0, Add, 5);
        Stop Chasing Global Variable(probe_g);
        Stop Chasing Player Variable(Event Player, probe_p);
        Chase Global Variable At Rate(probe_g, 0, 1, None);
        Chase Global Variable Over Time(probe_g, 0, 1, None);
        Chase Player Variable At Rate(Event Player, probe_p, 0, 1, None);
        Chase Player Variable Over Time(Event Player, probe_p, 0, 1, None);
    }
}
"#;

const FOLDED_TARGET_CALLS: &[(&str, &str, usize)] = &[
    ("setGlobalVariableAtIndex", "setGlobalVariableAtIndex", 3),
    ("setPlayerVariableAtIndex", "setPlayerVariableAtIndex", 3),
    (
        "modifyGlobalVariableAtIndex",
        "modifyGlobalVariableAtIndex",
        4,
    ),
    (
        "modifyPlayerVariableAtIndex",
        "modifyPlayerVariableAtIndex",
        4,
    ),
    ("stopChasingGlobalVariable", "stopChasingGlobalVariable", 1),
    ("stopChasingPlayerVariable", "stopChasingPlayerVariable", 1),
    ("chaseAtRate", "chaseAtRate", 4),
    ("chaseOverTime", "chaseOverTime", 4),
    ("chasePlayerVariableAtRate", "chasePlayerVariableAtRate", 4),
    (
        "chasePlayerVariableOverTime",
        "chasePlayerVariableOverTime",
        4,
    ),
];

fn folded_target(signature: &Signature) -> Option<JsonValue> {
    let mut call = None;
    let mut args: Vec<JsonValue> = Vec::new();
    for param in &signature.params {
        let Some(fold) = param
            .content
            .as_ref()
            .and_then(|content| content.fold.as_ref())
        else {
            continue;
        };
        match call {
            Some(call) => assert_eq!(
                call, fold.call,
                "folded parameters must share one target call"
            ),
            None => call = Some(fold.call.as_str()),
        }
        let node = if matches!(
            param.param_type.as_deref(),
            Some("Variable") | Some("Global Variable")
        ) {
            let variable = if fold.call == "globalVariable" {
                "probe_g"
            } else {
                "probe_p"
            };
            json!({ "variable": variable })
        } else {
            json!({"call": "eventPlayer", "args": []})
        };
        if args.len() <= fold.arg {
            args.resize(fold.arg + 1, JsonValue::Null);
        }
        args[fold.arg] = node;
    }
    call.map(|call| json!({"call": call, "args": args}))
}

#[test]
fn folded_variable_target_signatures_describe_the_content_layout() {
    let catalog = catalog();
    let mut folded: Vec<&str> = catalog
        .entries_of(Kind::Action)
        .filter(|entry| {
            (0..entry.param_count())
                .any(|index| entry.param_content(index).is_some_and(|c| c.fold.is_some()))
        })
        .map(|entry| entry.id.as_str())
        .collect();
    folded.sort_unstable();
    let mut covered: Vec<&str> = FOLDED_TARGET_CALLS.iter().map(|(id, ..)| *id).collect();
    covered.sort_unstable();
    assert_eq!(
        folded, covered,
        "the actions declaring a folded variable target must be exactly the \
         covered set — a new folded action needs coverage"
    );

    let rule = parse_rule(FOLDED_TARGET_SOURCE);
    let content = content_of(&rule);
    let actions = content["actions"].as_array().expect("actions");
    assert_eq!(actions.len(), FOLDED_TARGET_CALLS.len());
    for (index, (id, call, arg_count)) in FOLDED_TARGET_CALLS.iter().enumerate() {
        let matches = catalog.lookup(&en(), id).expect("lookup answers");
        let signature = matches
            .iter()
            .find_map(|lookup| match lookup {
                LookupMatch::Builtin {
                    id: matched,
                    signature,
                    ..
                } if matched == id => signature.as_ref(),
                _ => None,
            })
            .unwrap_or_else(|| panic!("'{id}' must resolve to its own signature"));

        let target = folded_target(signature)
            .unwrap_or_else(|| panic!("'{id}' must declare a folded variable target"));
        let action = &actions[index];
        assert_eq!(
            action["call"].as_str(),
            Some(*call),
            "{id}: emitted call id"
        );
        let args = action["args"].as_array().expect("args");
        assert_eq!(
            args.len(),
            *arg_count,
            "{id}: the fold facts must predict the emitted argument count"
        );
        assert_eq!(
            args[0], target,
            "{id}: the signature's fold facts must derive the emitted target node"
        );
    }
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

const OPERATION_FACT_POSITIONS: &[(Kind, &str, usize)] = &[
    (Kind::Action, "modifyGlobalVariable", 1),
    (Kind::Action, "modifyGlobalVariableAtIndex", 2),
    (Kind::Action, "modifyPlayerVariableAtIndex", 3),
    (Kind::Structural, "modifyGlobalVariable", 1),
    (Kind::Structural, "modifyPlayerVariable", 2),
];

#[test]
fn modify_operations_are_operator_call_ids_not_enum_members() {
    let catalog = catalog();
    assert!(
        catalog.enum_domain("Operation").is_none(),
        "the misleading Operation enum domain must be gone"
    );
    let mut declared = Vec::new();
    for kind in [Kind::Structural, Kind::Action, Kind::Value] {
        for entry in catalog.entries_of(kind) {
            for index in 0..entry.param_count() {
                assert_ne!(
                    entry.param_domain(index),
                    Some("Operation"),
                    "{} '{}': parameter {} still declares the Operation enum domain",
                    kind.as_str(),
                    entry.id,
                    index + 1
                );
                if entry
                    .param_content(index)
                    .is_some_and(|content| !content.operations.is_empty())
                {
                    declared.push((kind, entry.id.as_str(), index));
                }
            }
        }
    }
    declared.sort_unstable_by_key(|(kind, id, index)| (kind.as_str(), *id, *index));
    let mut expected = OPERATION_FACT_POSITIONS.to_vec();
    expected.sort_unstable_by_key(|(kind, id, index)| (kind.as_str(), *id, *index));
    assert_eq!(
        declared, expected,
        "the operations fact must sit on exactly the modify-operation \
         parameters — a new operator-typed parameter needs coverage"
    );

    let modify_ops: Vec<&str> = {
        let mut ids = [
            ModifyOp::Add,
            ModifyOp::Subtract,
            ModifyOp::Multiply,
            ModifyOp::Divide,
            ModifyOp::Modulo,
            ModifyOp::Min,
            ModifyOp::Max,
            ModifyOp::RaiseToPower,
            ModifyOp::AppendToArray,
            ModifyOp::RemoveFromArrayByValue,
            ModifyOp::RemoveFromArrayByIndex,
        ]
        .map(ModifyOp::catalog_id);
        ids.sort_unstable();
        ids.to_vec()
    };
    assert_eq!(modify_ops.len(), 11);
    for (kind, id, index) in OPERATION_FACT_POSITIONS {
        let entry = catalog
            .entries_of(*kind)
            .find(|entry| entry.id == *id)
            .unwrap_or_else(|| panic!("{id} exists"));
        let operations = &entry
            .param_content(*index)
            .expect("operations fact")
            .operations;
        let mut sorted = operations.iter().map(String::as_str).collect::<Vec<_>>();
        sorted.sort_unstable();
        assert_eq!(
            sorted,
            modify_ops,
            "{id} parameter {} must accept exactly the ModifyOp set",
            index + 1
        );
        for operation in operations {
            assert!(
                ModifyOp::from_catalog_id(operation).is_some(),
                "{operation} must name a ModifyOp"
            );
        }
    }
    assert!(
        ModifyOp::from_catalog_id("==").is_none(),
        "comparison operators are not modify operations"
    );
}

fn first_action_content(source: &str) -> JsonValue {
    let rule = parse_rule(source);
    let content = content_of(&rule);
    let actions = content["actions"].as_array().expect("actions");
    actions[0].clone()
}

#[test]
fn every_modify_operation_spelling_parses_emits_and_renders_its_call_id() {
    let catalog = catalog();
    for (action, surface) in [
        (
            "modifyGlobalVariableAtIndex",
            "Modify Global Variable At Index(probe_g, 0, {}, 5)",
        ),
        (
            "modifyPlayerVariableAtIndex",
            "Modify Player Variable At Index(Event Player, probe_p, 0, {}, 5)",
        ),
    ] {
        let signature = catalog
            .lookup(&en(), action)
            .expect("lookup answers")
            .into_iter()
            .find_map(|lookup| match lookup {
                LookupMatch::Builtin { id, signature, .. } if id == action => signature,
                _ => None,
            })
            .unwrap_or_else(|| panic!("'{action}' must resolve to its own signature"));
        let operations = signature
            .params
            .iter()
            .find_map(|param| {
                param
                    .content
                    .as_ref()
                    .filter(|content| !content.operations.is_empty())
            })
            .unwrap_or_else(|| panic!("'{action}' must expose its operations fact through lookup"));
        for operation in &operations.operations {
            let spelling = catalog
                .spelling(Kind::Operator, &en(), operation)
                .unwrap_or_else(|| panic!("'{operation}' needs an en-US spelling"));
            let source = format!(
                "variables {{\n    global:\n        0: probe_g\n    player:\n        0: probe_p\n}}\n\n\
                 rule (\"op\") {{\n    event {{\n        Ongoing - Each Player;\n        All;\n        All;\n    }}\n    actions {{\n        {};\n    }}\n}}\n",
                surface.replace("{}", spelling)
            );
            let program = parser::parse(&source, &catalog, &en())
                .unwrap_or_else(|error| panic!("{action}({spelling}) must parse: {error}"));
            assert!(
                program.semantic_issues(&catalog).is_empty(),
                "{action}({spelling}) must validate cleanly"
            );
            let emitted = emitter::emit(&program, &catalog, &en()).expect("emission");
            let reparsed = parser::parse(&emitted, &catalog, &en())
                .unwrap_or_else(|error| panic!("emitted {spelling} must reparse: {error}"));
            assert!(
                workshop_rs::roundtrip::equivalent(&program, &reparsed),
                "{action}({spelling}) must round-trip"
            );

            let action_node = first_action_content(&source);
            assert_eq!(
                action_node["args"][2],
                json!({"call": operation, "args": []}),
                "{action}({spelling}) must render the operand as the \
                 zero-argument '{operation}' call"
            );
        }
    }
}

fn variable_reads_rule() -> Rule {
    Rule::new("reads", Event::Global).condition(Condition::new(Value::call(
        "==",
        [
            Value::GlobalVariable("score".to_string()),
            Value::PlayerVariable {
                player: Box::new(Value::EventPlayer),
                variable: "streak".to_string(),
            },
        ],
    )))
}

#[test]
fn variable_reads_render_as_reserved_calls() {
    let content = content_of(&variable_reads_rule());
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

const REPEL_SOURCE: &str = r#"rule ("repel") {
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
"#;

#[test]
fn proximity_repel_actions_match_the_acceptance_examples() {
    let rule = parse_rule(REPEL_SOURCE);
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
    assert_eq!(first["actions"].as_array().expect("actions").len(), 3);
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

fn reserved_ids_rule() -> Rule {
    let member = Value::call(
        "memberAccess",
        [
            Value::GlobalVariable("myArray".to_string()),
            Value::string("length"),
            Value::from(0.0),
        ],
    );
    Rule::new("reserved ids", Event::Global)
        .action(Action::AssignMember {
            target: member,
            op: None,
            value: Value::from(1.0),
        })
        .action(Action::call("rawWorkshopAction", []))
        .condition(Condition::new(Value::call(
            "+",
            [Value::from(1.0), Value::from(2.0)],
        )))
}

#[test]
fn documented_reserved_ids_render_for_member_access_and_opaque_residuals() {
    let content = content_of(&reserved_ids_rule());
    assert_eq!(
        content["conditions"][0]["value"],
        json!({"call": "+", "args": [1.0, 2.0]})
    );
    assert_eq!(
        content["actions"][0],
        json!({
            "call": "assignMember",
            "args": [
                {
                    "call": "memberAccess",
                    "args": [
                        {"call": "globalVariable", "args": [{"variable": "myArray"}]},
                        {"string": "length"},
                        0.0
                    ]
                },
                1.0
            ]
        })
    );
    assert_eq!(
        content["actions"][1],
        json!({ "call": "rawWorkshopAction", "args": [] })
    );
}

#[test]
fn a_non_finite_number_never_reaches_the_document() {
    for number in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let rule =
            Rule::new("non-finite", Event::Global).condition(Condition::new(Value::from(number)));
        rule_content(&rule).expect_err("non-finite numbers are rejected");
    }
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

const AMBIGUOUS_MEMBERS_SOURCE: &str = r#"variables {
    global:
        0: probe
}

rule ("ambiguous members") {
    event {
        Ongoing - Global;
    }
    actions {
        Set Global Variable(probe, Up);
        Create Icon(Event Player, Left, No, Visible To and Position, Color(Yellow), True);
    }
}
"#;

#[test]
fn a_preserved_ambiguous_member_renders_its_spelling_and_candidates() {
    let rule = parse_rule(AMBIGUOUS_MEMBERS_SOURCE);
    let content = content_of(&rule);
    assert_eq!(
        content["actions"][0],
        json!({
            "call": "setGlobalVariable",
            "args": [
                {"variable": "probe"},
                {
                    "ambiguous": "Up",
                    "candidates": [
                        {"enum": "Vector", "member": "UP"},
                        {"enum": "Rounding", "member": "UP"}
                    ]
                }
            ]
        })
    );
    assert_eq!(
        content["actions"][1]["args"][1],
        json!({
            "ambiguous": "Left",
            "candidates": [
                {"enum": "Vector", "member": "LEFT"},
                {"enum": "HudPosition", "member": "LEFT"}
            ]
        })
    );
}

#[test]
fn a_context_pinned_ambiguous_member_renders_a_plain_enum_node() {
    let source = r#"rule ("pinned member") {
    event {
        Ongoing - Global;
    }
    actions {
        Create Progress Bar HUD Text(Event Player, 1, 0, Left, 1, Color(Yellow), Color(Yellow), Color, Default Visibility);
    }
}
"#;
    let content = content_of(&parse_rule(source));
    assert_eq!(
        content["actions"][0]["args"][3],
        json!({"enum": "HudPosition", "member": "LEFT"})
    );
}

fn strip_ambiguous_spelling(node: &mut JsonValue) {
    match node {
        JsonValue::Object(map) => {
            if let Some(spelling) = map.get_mut("ambiguous") {
                *spelling = JsonValue::Null;
            }
            for value in map.values_mut() {
                strip_ambiguous_spelling(value);
            }
        }
        JsonValue::Array(items) => {
            for item in items {
                strip_ambiguous_spelling(item);
            }
        }
        _ => {}
    }
}

#[test]
fn ambiguous_member_content_is_identical_across_locales_except_the_spelling() {
    let catalog = catalog();
    let english = parser::parse(AMBIGUOUS_MEMBERS_SOURCE, &catalog, &en()).expect("en-US parses");
    let chinese_source = emitter::emit(&english, &catalog, &zh()).expect("zh-CN emission");
    let chinese = parser::parse(&chinese_source, &catalog, &zh()).expect("zh-CN parses");

    let english_content = content_of(&english.rules[0]);
    let chinese_content = content_of(&chinese.rules[0]);
    assert_eq!(
        english_content["actions"][0]["args"][1]["ambiguous"],
        json!("Up")
    );
    assert_eq!(
        chinese_content["actions"][0]["args"][1]["ambiguous"],
        json!("上")
    );
    assert_eq!(
        english_content["actions"][1]["args"][1]["ambiguous"],
        json!("Left")
    );
    assert_eq!(
        chinese_content["actions"][1]["args"][1]["ambiguous"],
        json!("左")
    );

    let mut stripped_english = english_content.clone();
    let mut stripped_chinese = chinese_content.clone();
    strip_ambiguous_spelling(&mut stripped_english);
    strip_ambiguous_spelling(&mut stripped_chinese);
    assert_eq!(stripped_english, stripped_chinese);
}

#[test]
fn malformed_ambiguous_sentinels_fail_the_render() {
    for args in [
        Vec::new(),
        vec![Value::from(1.0), Value::Array(vec![])],
        vec![Value::string("Up"), Value::Array(vec![])],
        vec![Value::string("Up"), Value::Array(vec![Value::from(1.0)])],
    ] {
        let rule = Rule::new("malformed", Event::Global).condition(Condition::new(Value::Call {
            name: "__ambiguous_enum".to_string(),
            args,
        }));
        rule_content(&rule).expect_err("malformed ambiguous sentinel is rejected");
    }
}

const CHASE_SOURCE: &str = r#"variables {
    global:
        0: probe_g
    player:
        0: probe_p
}

rule ("chase") {
    event {
        Ongoing - Each Player;
        All;
        All;
    }
    actions {
        Chase Global Variable At Rate(probe_g, 0, 1, None);
        Chase Global Variable Over Time(probe_g, 0, 1, None);
        Chase Player Variable At Rate(Event Player, probe_p, 0, 1, None);
        Chase Player Variable Over Time(Event Player, probe_p, 0, 1, None);
    }
}
"#;

#[test]
fn player_variable_chases_emit_their_catalog_call_ids() {
    let catalog = catalog();
    let english = parser::parse(CHASE_SOURCE, &catalog, &en()).expect("en-US parses");
    let content = content_of(&english.rules[0]);
    let actions = content["actions"].as_array().expect("actions");

    let global_target = json!({"call": "globalVariable", "args": [{"variable": "probe_g"}]});
    let player_target = json!({"call": "playerVariable", "args": [
        {"call": "eventPlayer", "args": []},
        {"variable": "probe_p"}
    ]});
    assert_eq!(
        actions[0],
        json!({"call": "chaseAtRate", "args": [
            global_target, 0.0, 1.0, {"enum": "ChaseRateReeval", "member": "NONE"}
        ]})
    );
    assert_eq!(
        actions[1],
        json!({"call": "chaseOverTime", "args": [
            global_target, 0.0, 1.0, {"enum": "ChaseTimeReeval", "member": "NONE"}
        ]})
    );
    assert_eq!(
        actions[2],
        json!({"call": "chasePlayerVariableAtRate", "args": [
            player_target, 0.0, 1.0, {"enum": "ChaseRateReeval", "member": "NONE"}
        ]})
    );
    assert_eq!(
        actions[3],
        json!({"call": "chasePlayerVariableOverTime", "args": [
            player_target, 0.0, 1.0, {"enum": "ChaseTimeReeval", "member": "NONE"}
        ]})
    );

    let chinese_source = emitter::emit(&english, &catalog, &zh()).expect("zh-CN emission");
    let chinese = parser::parse(&chinese_source, &catalog, &zh()).expect("zh-CN parses");
    assert_eq!(content_of(&chinese.rules[0]), content);
}

const PRACTICE_RANGE_MODES: &[(&str, &str, &str)] = &[
    (
        "en-US",
        "en-US.ow",
        "9f1ef1328e923f205f09a8a006d40be47defac2c7c92416e8091134925fca07a",
    ),
    (
        "zh-CN",
        "zh-CN.ow",
        "b6e099c29068b7b2e0664c67ddd17d4dec51d1b2b2084acb50a30a96e1e3b98c",
    ),
];

fn count_ambiguous_nodes(node: &JsonValue) -> usize {
    match node {
        JsonValue::Object(map) => {
            usize::from(map.contains_key("ambiguous"))
                + map.values().map(count_ambiguous_nodes).sum::<usize>()
        }
        JsonValue::Array(items) => items.iter().map(count_ambiguous_nodes).sum(),
        _ => 0,
    }
}

#[test]
fn every_practice_range_rule_renders_a_valid_document() {
    let validator = schema();
    for (locale_tag, file, sha256) in PRACTICE_RANGE_MODES {
        let path = fixture("practice-range", file);
        let bytes = std::fs::read(&path).expect("practice-range fixture reads");
        assert_eq!(
            format!("{:x}", Sha256::digest(&bytes)),
            *sha256,
            "pinned digest mismatch for {file}"
        );
        let source = String::from_utf8(bytes).expect("fixture is UTF-8 Workshop text");
        let catalog = catalog();
        let program = parser::parse(&source, &catalog, &locale(locale_tag))
            .unwrap_or_else(|error| panic!("{file} parses: {error}"));
        program
            .validate()
            .unwrap_or_else(|error| panic!("{file} validates: {error}"));
        let mut ambiguous_nodes = 0;
        for rule in &program.rules {
            let content = rule
                .content()
                .unwrap_or_else(|error| panic!("{file} rule '{}': {error}", rule.name));
            if let Err(errors) = validator.validate(&content) {
                let errors = errors.map(|error| error.to_string()).collect::<Vec<_>>();
                panic!(
                    "{file} rule '{}' does not validate: {errors:?}\n{content}",
                    rule.name
                );
            }
            ambiguous_nodes += count_ambiguous_nodes(&content);
        }
        assert!(
            ambiguous_nodes > 0,
            "{file} must exercise the ambiguous node form"
        );
        for name in ["Cover values 74", "Cover enums 83"] {
            let rule = program
                .rules
                .iter()
                .find(|rule| rule.name == name)
                .unwrap_or_else(|| panic!("{file} must contain rule '{name}'"));
            let content = rule.content().expect("rule content renders");
            assert!(
                count_ambiguous_nodes(&content) > 0,
                "{file} rule '{name}' must render ambiguous member nodes"
            );
        }
    }
}
