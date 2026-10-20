//! Contract tests for `rules::statement::statement_from_content`, the input
//! direction of `workshop-rs/rule-content-v1` (workshop-rs#531).

use serde_json::{Value as JsonValue, json};
use workshop_rs::rules::content::rule_content;
use workshop_rs::rules::statement::{Statement, StatementContentError, statement_from_content};
use workshop_rs::{Event, Rule, emitter, parser};

use crate::common::{catalog, corpus_text, fixture, locale};

fn convert(node: &JsonValue) -> Result<Statement, StatementContentError> {
    statement_from_content(&catalog(), node)
}

/// The statement's content as the export renders it inside a rule.
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

/// Round-trip every statement of `source` in every catalog locale.
fn assert_round_trip(source: &str, source_locale: &str, tally: &mut Tally) {
    let catalog = catalog();
    let program = parser::parse(source, &catalog, &locale(source_locale)).expect("source parses");
    for target in catalog.locales() {
        // A locale without every spelling the program needs cannot render
        // it; the statement content does not depend on the locale otherwise.
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
    // The locales with complete spellings for the corpus; emission skips
    // any locale that lacks a mapping the program needs.
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
fn a_refusal_produces_no_model_value() {
    // The result type is `Result`: an `Err` carries no statement. A partly
    // valid statement is refused as a whole.
    let node = json!({"call": "wait", "args": [1.0, {"call": "noSuchValue", "args": []}]});
    assert!(convert(&node).is_err());
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
