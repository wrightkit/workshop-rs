use serde_json::{Value as JsonValue, json};
use workshop_rs::rules::content::rule_content;
use workshop_rs::rules::statement::{Statement, StatementContentError, statement_from_content};
use workshop_rs::{Action, Event, ModifyOp, Rule, Value, emitter, parser};

use crate::common::{catalog, corpus_text, fixture, locale};

fn convert(node: &JsonValue) -> Result<Statement, StatementContentError> {
    statement_from_content(&catalog(), node)
}

fn content_of(statement: Statement) -> JsonValue {
    let rule = Rule::new("statement", Event::Global);
    match statement {
        Statement::Condition(condition) => {
            let content = rule_content(&rule.condition(condition)).expect("content renders");
            content["conditions"][0].clone()
        }
        Statement::Action(action) => {
            let content = rule_content(&rule.action(action)).expect("content renders");
            content["actions"][0].clone()
        }
        _ => unreachable!("only conditions and actions exist"),
    }
}

fn statements(content: &JsonValue) -> impl Iterator<Item = &JsonValue> {
    ["conditions", "actions"]
        .into_iter()
        .flat_map(|list| content[list].as_array().expect("statement list"))
}

fn has_ambiguous(node: &JsonValue) -> bool {
    match node {
        JsonValue::Object(map) => map.contains_key("ambiguous") || map.values().any(has_ambiguous),
        JsonValue::Array(items) => items.iter().any(has_ambiguous),
        _ => false,
    }
}

#[derive(Default)]
struct Tally {
    converted: usize,
    refused_ambiguous: usize,
    locales: std::collections::BTreeSet<String>,
}

fn assert_round_trip(source: &str, source_locale: &str, tally: &mut Tally) {
    let catalog = catalog();
    let program = parser::parse(source, &catalog, &locale(source_locale)).expect("source parses");
    for target in catalog.locales() {
        let Ok(text) = emitter::emit(&program, &catalog, target) else {
            continue;
        };
        tally.locales.insert(target.to_string());
        let reparsed = parser::parse(&text, &catalog, target)
            .unwrap_or_else(|error| panic!("{target} reparse: {error}"));
        for rule in &reparsed.rules {
            let content = rule_content(rule).expect("rule content renders");
            for node in statements(&content) {
                if has_ambiguous(node) {
                    let error = statement_from_content(&catalog, node)
                        .expect_err("an ambiguous enum member is refused");
                    assert!(
                        matches!(error, StatementContentError::Unrepresentable { .. }),
                        "{error:?}"
                    );
                    tally.refused_ambiguous += 1;
                    continue;
                }
                let statement = statement_from_content(&catalog, node).unwrap_or_else(|error| {
                    panic!("{target} rule '{}': {error}\n{node}", rule.name)
                });
                assert_eq!(
                    &content_of(statement),
                    node,
                    "{target} rule '{}'",
                    rule.name
                );
                tally.converted += 1;
            }
        }
    }
}

#[test]
fn practice_range_statements_round_trip_in_every_locale() {
    let mut tally = Tally::default();
    for (tag, file) in [("en-US", "en-US.ow"), ("zh-CN", "zh-CN.ow")] {
        let source = std::fs::read_to_string(fixture("practice-range", file)).expect("fixture");
        assert_round_trip(&source, tag, &mut tally);
    }
    assert!(tally.converted > 1000, "converted {}", tally.converted);
    assert!(tally.refused_ambiguous > 0);
    assert!(tally.locales.contains("en-us") && tally.locales.contains("zh-cn"));
}

#[test]
fn corpus_statements_round_trip_in_every_locale() {
    let mut tally = Tally::default();
    for id in [
        "basic-rule",
        "control-flow",
        "declarations-rules",
        "expressions-values",
        "receiver-calls",
    ] {
        assert_round_trip(&corpus_text(id), "en-US", &mut tally);
    }
    assert!(tally.converted > 0);
}

fn error_of(node: JsonValue) -> StatementContentError {
    convert(&node).expect_err("the node is refused")
}

#[test]
fn unknown_id_names_the_node() {
    let node = json!({"call": "wait", "args": [1.0, {"call": "noSuchValue", "args": []}]});
    let error = error_of(node);
    assert!(matches!(&error, StatementContentError::UnknownId { id, .. } if id == "noSuchValue"));
    assert_eq!(error.path(), "/args/1");
    assert_eq!(error.node(), &json!({"call": "noSuchValue", "args": []}));
    let error = error_of(json!({"call": "noSuchAction", "args": []}));
    assert!(matches!(error, StatementContentError::UnknownId { .. }));
}

#[test]
fn wrong_argument_count_is_refused() {
    for node in [
        json!({"call": "wait", "args": []}),
        json!({"call": "wait", "args": [1.0, 2.0, 3.0]}),
        json!({"call": "end", "args": [1.0]}),
        json!({"call": "if", "args": []}),
        json!({"value": {"call": "vector", "args": [1.0]}}),
        json!({"value": {"call": "globalVariable", "args": []}}),
    ] {
        let error = error_of(node.clone());
        assert!(
            matches!(error, StatementContentError::ArgumentCount { .. }),
            "{node}: {error:?}"
        );
    }
}

#[test]
fn internal_sentinel_is_refused_as_action_and_as_value() {
    let sentinel = json!({"call": "__ambiguous_enum", "args": []});
    for node in [sentinel.clone(), json!({"value": sentinel})] {
        let error = error_of(node);
        assert!(
            matches!(&error, StatementContentError::InternalSentinel { id, .. } if id == "__ambiguous_enum"),
            "{error:?}"
        );
    }
}

#[test]
fn nodes_that_form_no_statement_are_refused() {
    for node in [
        json!(1.0),
        json!({}),
        json!({"value": 1.0, "call": "end", "args": []}),
        json!({"call": "end"}),
        json!({"call": "end", "args": [], "disabled": false}),
        json!({"call": "end", "args": [], "extra": 1}),
        json!({"call": "rawWorkshopAction", "args": []}),
        json!({"value": {"variable": "x"}}),
        json!({"value": "text"}),
        json!({"value": {"enum": "NoDomain", "member": "X"}}),
        json!({"call": "setGlobalVariable", "args": [1.0, 2.0]}),
        json!({"call": "modifyGlobalVariable", "args": [{"variable": "x"}, {"call": "eventPlayer", "args": []}, 1.0]}),
    ] {
        let error = convert(&node).expect_err("refused");
        assert!(
            matches!(
                error,
                StatementContentError::Unrepresentable { .. }
                    | StatementContentError::UnknownId { .. }
            ),
            "{node}: {error:?}"
        );
    }
    let error = error_of(json!({"value": {"variable": "x"}}));
    assert!(matches!(
        error,
        StatementContentError::Unrepresentable { .. }
    ));
}

#[test]
fn unresolved_ambiguous_enum_is_refused_not_approximated() {
    let node = json!({"value": {"call": "compare", "args": [
        1.0,
        {"call": "==", "args": []},
        {"ambiguous": "Up", "candidates": [
            {"enum": "Vector", "member": "UP"}, {"enum": "Rounding", "member": "UP"}
        ]}
    ]}});
    let error = error_of(node);
    assert!(matches!(
        error,
        StatementContentError::Unrepresentable { .. }
    ));
    assert_eq!(error.path(), "/value/args/2");
}

#[test]
fn a_player_variable_chase_keeps_its_own_catalog_id() {
    // ADR-0019: the document's `chasePlayerVariableAtRate` id names the
    // player-variable chase entry; the model never aliases it to the
    // global chase's name.
    let catalog = catalog();
    for (id, source_call) in [
        (
            "chasePlayerVariableAtRate",
            "Chase Player Variable At Rate(Event Player, probe_p, 0, 1, None);",
        ),
        (
            "chasePlayerVariableOverTime",
            "Chase Player Variable Over Time(Event Player, probe_p, 0, 1, None);",
        ),
    ] {
        let source = format!(
            "variables {{\n    player:\n        0: probe_p\n}}\n\nrule (\"probe\") {{\n    event {{\n        Ongoing - Each Player;\n        All;\n        All;\n    }}\n    actions {{\n        {source_call}\n    }}\n}}\n"
        );
        let program = parser::parse(&source, &catalog, &crate::common::en()).expect("parses");
        let content = rule_content(&program.rules[0]).expect("content renders");
        let node = &content["actions"][0];
        assert_eq!(node["call"], id, "the document names the player id");
        let Statement::Action(Action::Call { name, args }) =
            statement_from_content(&catalog, node).expect("converts")
        else {
            panic!("{id}: the statement is an action call");
        };
        assert_eq!(name, id);
        assert!(
            matches!(args.first(), Some(Value::PlayerVariable { .. })),
            "{id}: the folded target is a playerVariable"
        );
        assert_eq!(
            &content_of(Statement::Action(Action::Call {
                name: name.clone(),
                args: args.clone(),
            })),
            node,
            "{id}: render -> convert -> render is stable"
        );
    }
}

#[test]
fn chase_ids_are_section_exact() {
    let player_target = json!({"call": "playerVariable", "args": [
        {"call": "eventPlayer", "args": []}, {"variable": "v"}
    ]});
    // The global chase id with a playerVariable target is not the
    // document's player chase: its `Variable` parameter rejects the
    // folded pair instead of aliasing to the player id.
    let wrong_target = json!({"call": "chaseAtRate", "args": [
        player_target, 0.0, 1.0, {"enum": "ChaseRateReeval", "member": "NONE"}
    ]});
    assert!(
        convert(&wrong_target).is_err(),
        "chaseAtRate takes a global variable target"
    );
    // The player id with too few arguments is refused by its own
    // signature (player, variable, destination, rate, reevaluation
    // fold to target + 3 trailing args).
    let short = json!({"call": "chasePlayerVariableAtRate", "args": [
        player_target, 0.0
    ]});
    let error = error_of(short);
    assert!(matches!(error, StatementContentError::ArgumentCount { .. }));
}

#[test]
fn a_modify_operand_resolves_the_operator_section_not_the_same_named_value() {
    // ADR-0019: `add` is declared under `values` and `operators`; an
    // operand-position call resolves the operator section, so a
    // zero-argument `add` node is an operand while `add(x)` is the value
    // call.
    let operand = json!({"call": "add", "args": []});
    let statement = json!({"call": "modifyGlobalVariable", "args": [
        {"variable": "x"}, operand, 1.0
    ]});
    let Statement::Action(Action::ModifyGlobalVariable { op, .. }) =
        convert(&statement).expect("the modify action converts")
    else {
        panic!("the statement is a modify action");
    };
    assert_eq!(op, ModifyOp::Add);

    // The same id with its declared arguments reads as the `values`
    // entry, not an operand: `add(1, 2)` inside a condition is the value
    // call.
    let value_call = json!({"value": {"call": "add", "args": [1.0, 2.0]}});
    let statement = convert(&value_call).expect("the value call converts");
    let Statement::Condition(condition) = statement else {
        panic!("the statement is a condition");
    };
    assert!(matches!(
        condition.value,
        Value::Call { ref name, .. } if name == "add"
    ));
    // A partial application is still refused: the `values` signature
    // decides argument count, never the operand shape.
    let partial = json!({"value": {"call": "add", "args": [1.0]}});
    let error = error_of(partial);
    assert!(matches!(error, StatementContentError::ArgumentCount { .. }));

    // `removeFromArray` is a `values`-only id that names no ModifyOp: a
    // zero-argument call of it is neither a valid value call (its
    // signature takes two arguments) nor an operand.
    let unapplied = json!({"call": "modifyGlobalVariable", "args": [
        {"variable": "x"}, {"call": "removeFromArray", "args": []}, 1.0
    ]});
    assert!(
        convert(&unapplied).is_err(),
        "removeFromArray is not a modify operand"
    );
}

#[test]
fn spans_are_refused_wherever_they_appear() {
    let span = json!({"start": {"line": 1, "column": 1}});
    for (node, path) in [
        (json!({"call": "end", "args": [], "span": span}), ""),
        (json!({"value": true, "span": span}), ""),
        (
            json!({"call": "if", "args": [{"call": "eventPlayer", "args": [], "span": span}]}),
            "/args/0",
        ),
    ] {
        let error = error_of(node);
        assert!(
            matches!(error, StatementContentError::Spans { .. }),
            "{error:?}"
        );
        assert_eq!(error.path(), path);
    }
}

#[test]
fn disabled_statements_keep_their_modifier() {
    for node in [
        json!({"call": "abort", "args": [], "disabled": true}),
        json!({"value": true, "disabled": true}),
    ] {
        assert_eq!(content_of(convert(&node).expect("converts")), node);
    }
}

#[test]
fn conversion_and_validation_agree_on_argument_counts() {
    use workshop_rs::catalog::Kind;
    use workshop_rs::{Action, Condition, Program, Value, Variable};

    let catalog = catalog();
    let all = || {
        catalog
            .entries_of(Kind::Action)
            .chain(catalog.entries_of(Kind::Value))
    };
    let folded = all()
        .find(|e| e.param_type(0) == Some("Player|Array") && e.param_type(1) == Some("Variable"))
        .expect("a folded player-variable entry");
    let variadic = all().find(|e| e.is_variadic()).expect("a variadic entry");
    let undeclared = all()
        .find(|e| e.param_count() == 0 && e.required_param_count() == 0)
        .expect("an undeclared-signature entry");

    let player_target = json!({"call": "playerVariable", "args": [
        {"call": "eventPlayer", "args": []}, {"variable": "v"}
    ]});
    for (entry, fold) in [(folded, true), (variadic, false), (undeclared, false)] {
        for count in 0..=entry.param_count() + 3 {
            let (mut args, mut values) = (Vec::new(), Vec::new());
            for index in 0..count {
                if fold && index == 0 {
                    args.push(player_target.clone());
                    values.push(Value::player_variable(Value::EventPlayer, "v"));
                } else {
                    args.push(json!(0.0));
                    values.push(Value::from(0.0));
                }
            }
            let call = json!({"call": entry.id, "args": args});
            let (node, rule) = if entry.kind == Kind::Action {
                (
                    call,
                    Rule::new("agreement", Event::Global)
                        .action(Action::call(entry.id.clone(), values)),
                )
            } else {
                (
                    json!({"value": call}),
                    Rule::new("agreement", Event::Global)
                        .condition(Condition::new(Value::call(entry.id.clone(), values))),
                )
            };
            let converted = statement_from_content(&catalog, &node);
            let converter_rejects =
                matches!(converted, Err(StatementContentError::ArgumentCount { .. }));

            let mut program = Program::new();
            program.player_variable(Variable::new("v"));
            program.rule(rule);
            let validator_rejects =
                workshop_rs::validate::validate_canonical_ids(&program, &catalog)
                    .is_err_and(|error| error.to_string().contains("argument(s)"));

            assert_eq!(
                converter_rejects, validator_rejects,
                "'{}' with {count} argument(s)",
                entry.id
            );
        }
    }
}
