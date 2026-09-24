use workshop_rs::catalog::{Catalog, Locale};
use workshop_rs::source::{FileId, Position, SourceFile, Span};
use workshop_rs::{
    Action, Condition, Event, MappedText, Program, Rule, SourceMap, SourceMapError, Value,
    Variable, emitter, parser,
};

use super::common;

fn catalog() -> Catalog {
    Catalog::builtin().expect("builtin catalog")
}

fn en() -> Locale {
    Locale::new("en-US")
}

fn span(file: FileId, line: u32, start: u32, end: u32) -> Span {
    Span::new(file, Position::new(line, start), Position::new(line, end))
}

/// Every mapped position of a program in a stable order.
fn mapped_positions(program: &Program) -> Vec<(String, Option<Span>)> {
    let mut positions = Vec::new();
    for (rule, public) in program.rules.iter().enumerate() {
        positions.push((format!("rule {rule}"), program.rule_span(rule)));
        for condition in 0..public.conditions.len() {
            positions.push((
                format!("rule {rule} condition {condition}"),
                program.condition_span(rule, condition),
            ));
        }
        for action in 0..public.actions.len() {
            positions.push((
                format!("rule {rule} action {action}"),
                program.action_span(rule, action),
            ));
            for argument in 0..6 {
                positions.push((
                    format!("rule {rule} action {action} argument {argument}"),
                    program.action_argument_span(rule, action, argument),
                ));
            }
        }
    }
    positions
}

fn parsed(source: &str) -> Program {
    parser::parse(source, &catalog(), &en()).expect("parses")
}

const TWO_RULES: &str = r#"rule ("first") {
    event { Ongoing - Global; }
    conditions { 1 == 1; }
    actions {
        Wait(1, Ignore Condition);
        Wait(2, Ignore Condition);
    }
}
rule ("second") {
    event { Ongoing - Global; }
    actions { Wait(3, Ignore Condition); }
}
"#;

fn authored_program() -> Program {
    let mut program = Program::new();
    let file = program.add_file(SourceFile::new("src/main.opy"));
    program.global_variable(Variable::new("counter"));
    program.rule(
        Rule::new("authored", Event::Global)
            .condition(Condition::new(Value::Bool(true)))
            .action(Action::SetGlobalVariable {
                variable: "counter".to_string(),
                value: Value::Number(1.0),
            }),
    );
    program
        .set_global_variable_spans(0, Some(span(file, 1, 1, 8)), Some(span(file, 1, 1, 8)))
        .unwrap();
    program
        .set_rule_span(0, Some(span(file, 3, 1, 40)))
        .unwrap();
    program
        .set_condition_span(0, 0, Some(span(file, 3, 10, 14)))
        .unwrap();
    program
        .set_action_span(0, 0, Some(span(file, 4, 5, 20)))
        .unwrap();
    program
        .set_action_argument_span(0, 0, 0, Some(span(file, 4, 10, 11)))
        .unwrap();
    program
}

#[test]
fn extract_emit_parse_apply_round_trips_every_mapped_position() {
    let catalog = catalog();
    let mut compared = 0;
    for case in common::cases() {
        let (source, locale) = common::source(case);
        let program = parser::parse_with_context(&source, &catalog, &locale, &catalog)
            .unwrap_or_else(|error| panic!("{} parse failed: {error:?}", case.id));
        let Ok(text) = emitter::emit(&program, &catalog, &locale) else {
            continue;
        };
        let artifact = MappedText {
            text,
            map: SourceMap::extract(&program),
        };
        let artifact = MappedText::from_json(&artifact.to_json()).expect("artifact decodes");
        let mut reparsed = parser::parse_with_context(&artifact.text, &catalog, &locale, &catalog)
            .unwrap_or_else(|error| panic!("{} reparse failed: {error:?}", case.id));
        artifact
            .map
            .apply(&mut reparsed)
            .unwrap_or_else(|error| panic!("{} apply failed: {error}", case.id));

        assert!(reparsed.source(FileId::from_index(0)).is_none());
        let expected = mapped_positions(&program);
        assert_eq!(mapped_positions(&reparsed), expected, "{}", case.id);
        compared += expected.iter().filter(|(_, span)| span.is_some()).count();
    }
    assert!(compared > 0, "no real-project positions were compared");
}

#[test]
fn programmatic_mapping_round_trips_through_text_and_declarations() {
    let catalog = catalog();
    let program = authored_program();
    let text = emitter::emit(&program, &catalog, &en()).expect("emits");
    let artifact = MappedText {
        text,
        map: SourceMap::extract(&program),
    };
    let decoded = MappedText::from_json(&artifact.to_json()).expect("decodes");
    assert_eq!(decoded, artifact);
    assert_eq!(decoded.map.files(), ["src/main.opy"]);

    let mut reparsed = parser::parse(&decoded.text, &catalog, &en()).expect("reparses");
    decoded.map.apply(&mut reparsed).expect("applies");
    assert_eq!(mapped_positions(&reparsed), mapped_positions(&program));
    assert!(reparsed.rule_span(0).is_some());
    assert!(reparsed.source(FileId::from_index(0)).is_none());
    assert_eq!(SourceMap::extract(&reparsed), decoded.map);
}

#[test]
fn mapped_text_uses_the_documented_json_shape() {
    let program = authored_program();
    let artifact = MappedText {
        text: "rule".to_string(),
        map: SourceMap::extract(&program),
    };
    let json: serde_json::Value = serde_json::from_str(&artifact.to_json()).unwrap();
    assert_eq!(json["format"], "workshop-rs/mapped-text-v1");
    assert_eq!(json["text"], "rule");
    assert_eq!(json["files"], serde_json::json!([{"path": "src/main.opy"}]));
    assert_eq!(
        json["shape"],
        serde_json::json!({
            "global_variables": 1,
            "player_variables": 0,
            "subroutines": 0,
            "rules": [{"conditions": 1, "actions": 1}],
        })
    );
    assert_eq!(
        json["spans"][1],
        serde_json::json!({
            "node": "rule",
            "rule": 0,
            "span": {
                "file": 0,
                "start": {"line": 3, "column": 1},
                "end": {"line": 3, "column": 40},
            },
        })
    );
}

#[test]
fn decoding_rejects_other_formats_and_malformed_artifacts() {
    assert!(matches!(
        MappedText::from_json(r#"{"format":"workshop-rs/mapped-text-v2"}"#),
        Err(SourceMapError::UnsupportedFormat(format)) if format == "workshop-rs/mapped-text-v2"
    ));
    assert!(matches!(
        MappedText::from_json(r#"{"format":"workshop-rs/mapped-text-v1"}"#),
        Err(SourceMapError::Malformed(_))
    ));
    assert!(matches!(
        MappedText::from_json("not json"),
        Err(SourceMapError::Malformed(_))
    ));
}

#[test]
fn shape_mismatch_rejects_the_whole_mapping() {
    let program = parsed(TWO_RULES);
    let map = SourceMap::extract(&program);

    let mut fewer_rules = parsed(TWO_RULES);
    fewer_rules.rules.pop();
    let mut fewer_conditions = parsed(TWO_RULES);
    fewer_conditions.rules[0].conditions.clear();
    let mut fewer_actions = parsed(TWO_RULES);
    fewer_actions.rules[1].actions.clear();
    let mut extra_variable = parsed(TWO_RULES);
    extra_variable.global_variable(Variable::new("extra"));

    for (mut mutated, expected) in [
        (
            fewer_rules,
            SourceMapError::RuleCount {
                expected: 2,
                found: 1,
            },
        ),
        (
            fewer_conditions,
            SourceMapError::ConditionCount {
                rule: 0,
                expected: 1,
                found: 0,
            },
        ),
        (
            fewer_actions,
            SourceMapError::ActionCount {
                rule: 1,
                expected: 1,
                found: 0,
            },
        ),
        (
            extra_variable,
            SourceMapError::GlobalVariableCount {
                expected: 0,
                found: 1,
            },
        ),
    ] {
        let before = mapped_positions(&mutated);
        assert_eq!(map.apply(&mut mutated), Err(expected));
        assert_eq!(mapped_positions(&mutated), before, "no partial application");
    }
}

#[test]
fn invalid_entries_reject_the_whole_mapping_without_changing_the_program() {
    let map = SourceMap::extract(&parsed(TWO_RULES));
    let json = MappedText {
        text: String::new(),
        map,
    }
    .to_json();
    let mut target = parsed(TWO_RULES);
    let before = mapped_positions(&target);

    let mut unknown_file: serde_json::Value = serde_json::from_str(&json).unwrap();
    unknown_file["spans"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!({
            "node": "rule", "rule": 0,
            "span": {"file": 9, "start": {"line": 1, "column": 1}, "end": {"line": 1, "column": 2}},
        }));
    let mut reversed: serde_json::Value = serde_json::from_str(&json).unwrap();
    reversed["spans"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!({
            "node": "rule", "rule": 0,
            "span": {"file": 0, "start": {"line": 2, "column": 1}, "end": {"line": 1, "column": 1}},
        }));
    let mut outside_shape: serde_json::Value = serde_json::from_str(&json).unwrap();
    outside_shape["spans"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!({
            "node": "action", "rule": 0, "action": 99,
            "span": {"file": 0, "start": {"line": 1, "column": 1}, "end": {"line": 1, "column": 2}},
        }));

    let mut duplicate: serde_json::Value = serde_json::from_str(&json).unwrap();
    let first_rule = duplicate["spans"][0].clone();
    assert_eq!(first_rule["node"], "rule");
    duplicate["spans"].as_array_mut().unwrap().push(first_rule);

    for (artifact, matches) in [
        (
            unknown_file,
            (|error: &SourceMapError| matches!(error, SourceMapError::UnknownFile(9)))
                as fn(&SourceMapError) -> bool,
        ),
        (reversed, |error| {
            matches!(error, SourceMapError::InvalidSpan(_))
        }),
        (outside_shape, |error| {
            matches!(error, SourceMapError::InvalidPosition)
        }),
        (duplicate, |error| {
            matches!(error, SourceMapError::DuplicateEntry)
        }),
    ] {
        let decoded = MappedText::from_json(&artifact.to_string()).expect("structure decodes");
        let error = decoded
            .map
            .apply(&mut target)
            .expect_err("entry is invalid");
        assert!(matches(&error), "{error:?}");
        assert_eq!(mapped_positions(&target), before);
    }
}

#[test]
fn inserting_or_removing_nodes_hides_displaced_spans() {
    let base = parsed(TWO_RULES);
    assert!(base.rule_span(0).is_some());
    assert!(base.rule_span(1).is_some());
    assert!(base.condition_span(0, 0).is_some());
    assert!(base.action_span(0, 1).is_some());
    assert!(base.action_argument_span(0, 1, 0).is_some());

    let mut inserted_rule = base.clone();
    inserted_rule
        .rules
        .insert(0, inserted_rule.rules[1].clone());
    let mut removed_rule = base.clone();
    removed_rule.rules.remove(0);
    for program in [&inserted_rule, &removed_rule] {
        assert_eq!(program.rule_span(0), None);
        assert_eq!(program.rule_span(1), None);
        assert_eq!(program.condition_span(0, 0), None);
        assert_eq!(program.action_span(0, 0), None);
        assert_eq!(program.action_argument_span(0, 0, 0), None);
    }

    let mut inserted_condition = base.clone();
    inserted_condition.rules[0]
        .conditions
        .insert(0, Condition::new(Value::Bool(true)));
    assert_eq!(inserted_condition.condition_span(0, 0), None);
    assert_eq!(inserted_condition.condition_span(0, 1), None);
    assert_eq!(inserted_condition.rule_span(0), base.rule_span(0));
    assert_eq!(inserted_condition.action_span(0, 0), base.action_span(0, 0));
    let mut removed_condition = base.clone();
    removed_condition.rules[0].conditions.clear();
    assert_eq!(removed_condition.condition_span(0, 0), None);

    let mut inserted_action = base.clone();
    let action = inserted_action.rules[0].actions[0].clone();
    inserted_action.rules[0].actions.insert(0, action);
    assert_eq!(inserted_action.action_span(0, 0), None);
    assert_eq!(inserted_action.action_span(0, 1), None);
    assert_eq!(inserted_action.action_argument_span(0, 1, 0), None);
    assert_eq!(
        inserted_action.condition_span(0, 0),
        base.condition_span(0, 0)
    );
    assert_eq!(inserted_action.action_span(1, 0), base.action_span(1, 0));
    let mut removed_action = base.clone();
    removed_action.rules[0].actions.pop();
    assert_eq!(removed_action.action_span(0, 0), None);

    let drifted = MappedText {
        text: String::new(),
        map: SourceMap::extract(&inserted_rule),
    };
    let json: serde_json::Value = serde_json::from_str(&drifted.to_json()).unwrap();
    assert_eq!(json["spans"], serde_json::json!([]));
}

#[test]
fn columns_count_unicode_scalar_values() {
    let source = "rule (\"héllo 世界 😀\") { event { Ongoing - Global; } actions { Wait(1, Ignore Condition); } }";
    let program = parsed(source);
    let action_column =
        |needle: &str| source[..source.find(needle).unwrap()].chars().count() as u32 + 1;
    let action = program.action_span(0, 0).expect("action span");
    assert_eq!(action.start, Position::new(1, action_column("Wait")));
    let argument = program
        .action_argument_span(0, 0, 0)
        .expect("argument span");
    assert_eq!(argument.start, Position::new(1, action_column("1, Ignore")));

    let artifact = MappedText {
        text: emitter::emit(&program, &catalog(), &en()).expect("emits"),
        map: SourceMap::extract(&program),
    };
    let decoded = MappedText::from_json(&artifact.to_json()).expect("decodes");
    let mut reparsed = parsed(&decoded.text);
    decoded.map.apply(&mut reparsed).expect("applies");
    assert_eq!(reparsed.action_span(0, 0), Some(action));
}

#[test]
fn declaration_entries_need_a_span_and_are_unique() {
    let program = authored_program();
    let json = MappedText {
        text: String::new(),
        map: SourceMap::extract(&program),
    }
    .to_json();
    let mut target = authored_program();
    target.set_rule_span(0, None).unwrap();

    let mut empty: serde_json::Value = serde_json::from_str(&json).unwrap();
    empty["spans"][0] = serde_json::json!({"node": "global_variable", "index": 0});
    let mut duplicate: serde_json::Value = serde_json::from_str(&json).unwrap();
    let declaration = duplicate["spans"][0].clone();
    duplicate["spans"].as_array_mut().unwrap().push(declaration);

    for (artifact, expected) in [
        (empty, SourceMapError::EmptyEntry),
        (duplicate, SourceMapError::DuplicateEntry),
    ] {
        let decoded = MappedText::from_json(&artifact.to_string()).expect("structure decodes");
        assert_eq!(decoded.map.apply(&mut target), Err(expected));
    }
}
