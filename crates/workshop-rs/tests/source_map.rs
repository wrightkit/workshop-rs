use workshop_rs::source::{FileId, Position, SourceFile, Span};
use workshop_rs::{
    Action, Condition, Event, MappedText, Program, Rule, SourceMap, SourceMapError, Value,
    Variable, emitter, parser,
};

use super::common::{self, catalog, en};

fn span(file: FileId, line: u32, start: u32, end: u32) -> Span {
    Span::new(file, Position::new(line, start), Position::new(line, end))
}

/// Every mapped position of `program` in a stable order.
///
/// Value paths are enumerated on `public` so a reparsed program is compared
/// against the tree the map was extracted from: emission can canonicalize
/// value forms (a boolean condition becomes a comparison), and positions only
/// addressable in the reparsed tree hold no provenance by construction.
fn mapped_positions(program: &Program, public: &Program) -> Vec<(String, Option<Span>)> {
    let mut positions = Vec::new();
    for (rule, public) in public.rules.iter().enumerate() {
        positions.push((format!("rule {rule}"), program.rule_span(rule)));
        positions.push((format!("rule {rule} name"), program.rule_name_span(rule)));
        positions.push((
            format!("rule {rule} event name"),
            program.rule_event_name_span(rule),
        ));
        for (condition, public_condition) in public.conditions.iter().enumerate() {
            positions.push((
                format!("rule {rule} condition {condition}"),
                program.condition_span(rule, condition),
            ));
            for path in value_paths(&public_condition.value) {
                positions.push((
                    format!("rule {rule} condition {condition} value {path:?}"),
                    program.condition_value_span(rule, condition, &path),
                ));
            }
        }
        for (action, public_action) in public.actions.iter().enumerate() {
            positions.push((
                format!("rule {rule} action {action}"),
                program.action_span(rule, action),
            ));
            positions.push((
                format!("rule {rule} action {action} identifier"),
                program.action_identifier_span(rule, action),
            ));
            for argument in 0..6 {
                positions.push((
                    format!("rule {rule} action {action} argument {argument}"),
                    program.action_argument_span(rule, action, argument),
                ));
            }
            for (argument, argument_value) in action_arguments(public_action).iter().enumerate() {
                for path in value_paths(argument_value) {
                    positions.push((
                        format!("rule {rule} action {action} argument {argument} value {path:?}"),
                        program.action_argument_value_span(rule, action, argument, &path),
                    ));
                }
            }
        }
    }
    positions
}

fn value_paths(value: &Value) -> Vec<Vec<usize>> {
    fn walk(value: &Value, path: &mut Vec<usize>, paths: &mut Vec<Vec<usize>>) {
        paths.push(path.clone());
        for (index, child) in value_children(value).into_iter().enumerate() {
            path.push(index);
            walk(child, path, paths);
            path.pop();
        }
    }
    let mut paths = Vec::new();
    walk(value, &mut Vec::new(), &mut paths);
    paths
}

fn value_children(value: &Value) -> Vec<&Value> {
    match value {
        Value::Array(values) => values.iter().collect(),
        Value::Vector { x, y, z } => vec![x.as_ref(), y.as_ref(), z.as_ref()],
        Value::PlayerVariable { player, .. } => vec![player.as_ref()],
        Value::Call { args, .. } => args.iter().collect(),
        _ => Vec::new(),
    }
}

fn action_arguments(action: &Action) -> Vec<&Value> {
    match action {
        Action::SetGlobalVariable { value, .. } | Action::ModifyGlobalVariable { value, .. } => {
            vec![value]
        }
        Action::SetPlayerVariable { player, value, .. }
        | Action::ModifyPlayerVariable { player, value, .. } => vec![player, value],
        Action::AssignMember { target, value, .. } => vec![target, value],
        Action::If { condition } | Action::ElseIf { condition } | Action::While { condition } => {
            vec![condition]
        }
        Action::ForGlobalVariable {
            start, stop, step, ..
        } => vec![start, stop, step],
        Action::ForPlayerVariable {
            player,
            start,
            stop,
            step,
            ..
        } => vec![player, start, stop, step],
        Action::Call { args, .. } => args.iter().collect(),
        Action::CallSubroutine { .. } | Action::Else | Action::End => Vec::new(),
        Action::Disabled { action } => action_arguments(action),
    }
}

fn parsed(source: &str) -> Program {
    parser::parse(source, &catalog(), &en()).expect("parses")
}

const PROVENANCE_SOURCE: &str = r#"variables {
    global:
        0: counter
    player:
        1: score
}

subroutines {
    0: tick
}

rule ("writes") {
    event { Ongoing - Global; }
    conditions {
        Global.counter > 0;
    }
    actions {
        Set Global Variable(counter, Add(Global.counter, Event Player.score));
        Call Subroutine(tick);
        Global.counter = 2;
    }
}

rule ("on tick") {
    event {
        Subroutine;
        tick;
    }
    actions {
        Wait(1);
    }
}
"#;

#[test]
fn extraction_and_application_preserve_identifier_and_nested_value_provenance() {
    let catalog = catalog();
    let program = parsed(PROVENANCE_SOURCE);
    let artifact = MappedText {
        text: emitter::emit(&program, &catalog, &en()).expect("emits"),
        map: SourceMap::extract(&program),
    };
    let json: serde_json::Value = serde_json::from_str(&artifact.to_json()).unwrap();

    let entries = json["spans"].as_array().unwrap();
    let entries_of = |node: &str| {
        entries
            .iter()
            .filter(|entry| entry["node"] == node)
            .collect::<Vec<_>>()
    };
    for rule in entries_of("rule") {
        assert!(rule["name_span"].is_object(), "{rule}");
    }
    let subroutine_rule = entries_of("rule")
        .into_iter()
        .find(|entry| entry["rule"] == 1)
        .expect("second rule entry");
    assert!(subroutine_rule["event_name_span"].is_object());
    assert!(
        entries_of("action")
            .iter()
            .any(|entry| entry["identifier_span"].is_object()),
        "no action entry carries an identifier span"
    );
    assert!(
        entries_of("action_argument")
            .iter()
            .any(|entry| entry["children"]
                .as_array()
                .is_some_and(|children| !children.is_empty())),
        "no action argument entry carries children"
    );
    assert!(
        entries_of("condition").iter().any(|entry| entry["children"]
            .as_array()
            .is_some_and(|children| !children.is_empty())),
        "no condition entry carries children"
    );

    let decoded = MappedText::from_json(&artifact.to_json()).expect("decodes");
    let mut reparsed = parsed(&decoded.text);
    decoded.map.apply(&mut reparsed).expect("applies");
    assert_eq!(
        mapped_positions(&reparsed, &program),
        mapped_positions(&program, &program)
    );
    assert_eq!(SourceMap::extract(&reparsed), decoded.map);
}

fn wire_span(file: usize, line: u32, start: u32, end: u32) -> serde_json::Value {
    serde_json::json!({
        "file": file,
        "start": {"line": line, "column": start},
        "end": {"line": line, "column": end},
    })
}

#[test]
fn authored_entries_attach_identifier_and_nested_value_provenance() {
    // A provider authors the entries directly: `name_span`, `event_name_span`,
    // `identifier_span`, and `children` are optional members of the existing
    // entries, and spans may address any file in the table.
    let artifact = serde_json::json!({
        "format": "workshop-rs/mapped-text-v1",
        "text": PROVENANCE_SOURCE,
        "files": [{"path": "src/main.opy"}, {"path": "src/generated.opy"}],
        "shape": {
            "global_variables": 1,
            "player_variables": 1,
            "subroutines": 1,
            "rules": [
                {"conditions": 1, "actions": 3},
                {"conditions": 0, "actions": 1},
            ],
        },
        "spans": [
            {
                "node": "rule", "rule": 0,
                "span": wire_span(0, 1, 1, 30),
                "name_span": wire_span(0, 1, 7, 13),
            },
            {
                "node": "condition", "rule": 0, "condition": 0,
                "span": wire_span(0, 2, 1, 20),
                "children": [
                    {"identifier_span": wire_span(0, 2, 8, 15)},
                    {"span": wire_span(0, 2, 19, 20)},
                ],
            },
            {
                "node": "action", "rule": 0, "action": 0,
                "span": wire_span(0, 3, 1, 50),
                "identifier_span": wire_span(0, 3, 22, 29),
            },
            {
                "node": "action_argument", "rule": 0, "action": 0, "argument": 0,
                "span": wire_span(0, 3, 31, 49),
                "children": [
                    {"identifier_span": wire_span(0, 3, 35, 48)},
                    {"span": wire_span(0, 3, 50, 68)},
                ],
            },
            {
                "node": "action", "rule": 0, "action": 1,
                "span": wire_span(0, 4, 1, 20),
                "identifier_span": wire_span(0, 4, 16, 20),
            },
            {"node": "action", "rule": 0, "action": 2, "span": wire_span(0, 5, 1, 15)},
            {
                "node": "rule", "rule": 1,
                "span": wire_span(1, 1, 1, 25),
                "name_span": wire_span(1, 1, 7, 14),
                "event_name_span": wire_span(1, 3, 9, 13),
            },
            {
                "node": "action", "rule": 1, "action": 0,
                "span": wire_span(1, 5, 5, 12),
                "identifier_span": wire_span(1, 5, 9, 11),
            },
        ],
    });
    let decoded = MappedText::from_json(&artifact.to_string()).expect("decodes");
    let mut program = parsed(&decoded.text);
    decoded.map.apply(&mut program).expect("applies");

    let authored = FileId::from_index(0);
    let generated = FileId::from_index(1);
    assert_eq!(program.rule_span(0), Some(span(authored, 1, 1, 30)));
    assert_eq!(program.rule_name_span(0), Some(span(authored, 1, 7, 13)));
    assert_eq!(program.rule_name_span(1), Some(span(generated, 1, 7, 14)));
    assert_eq!(program.rule_event_name_span(0), None);
    assert_eq!(
        program.rule_event_name_span(1),
        Some(span(generated, 3, 9, 13))
    );
    assert_eq!(
        program.action_identifier_span(0, 0),
        Some(span(authored, 3, 22, 29))
    );
    assert_eq!(
        program.action_identifier_span(0, 1),
        Some(span(authored, 4, 16, 20))
    );
    assert_eq!(program.action_identifier_span(0, 2), None);
    assert_eq!(
        program.action_identifier_span(1, 0),
        Some(span(generated, 5, 9, 11))
    );
    assert_eq!(
        program.condition_value_span(0, 0, &[]),
        Some(span(authored, 2, 1, 20))
    );
    assert_eq!(
        program.condition_value_span(0, 0, &[0]),
        Some(span(authored, 2, 8, 15))
    );
    assert_eq!(
        program.condition_value_span(0, 0, &[1]),
        Some(span(authored, 2, 19, 20))
    );
    assert_eq!(program.condition_value_span(0, 0, &[2]), None);
    assert_eq!(
        program.action_argument_value_span(0, 0, 0, &[]),
        Some(span(authored, 3, 31, 49))
    );
    assert_eq!(
        program.action_argument_value_span(0, 0, 0, &[0]),
        Some(span(authored, 3, 35, 48))
    );
    assert_eq!(
        program.action_argument_value_span(0, 0, 0, &[1]),
        Some(span(authored, 3, 50, 68))
    );
    assert_eq!(SourceMap::extract(&program), decoded.map);
}

#[test]
fn nested_value_entries_are_validated_against_the_value_tree() {
    let map = SourceMap::extract(&parsed(PROVENANCE_SOURCE));
    let base: serde_json::Value =
        serde_json::from_str(&MappedText::new("", map).to_json()).unwrap();
    let argument_with_children = base["spans"]
        .as_array()
        .unwrap()
        .iter()
        .position(|entry| {
            entry["node"] == "action_argument"
                && entry["children"]
                    .as_array()
                    .is_some_and(|children| !children.is_empty())
        })
        .expect("a child-bearing action argument entry");
    let mut target = parsed(PROVENANCE_SOURCE);
    let before = mapped_positions(&target, &target);

    let mut unknown_child_file = base.clone();
    unknown_child_file["spans"][argument_with_children]["children"][0]["identifier_span"]["file"] =
        serde_json::json!(9);
    let mut invalid_child_span = base.clone();
    invalid_child_span["spans"][argument_with_children]["children"][0]["identifier_span"]["end"] =
        serde_json::json!({"line": 0, "column": 0});
    let mut invalid_name_span = base.clone();
    invalid_name_span["spans"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|entry| entry["node"] == "rule")
        .expect("a rule entry")["name_span"]["file"] = serde_json::json!(9);
    let mut empty_entry = base.clone();
    let fieldless = serde_json::json!({"node": "action", "rule": 1, "action": 0});
    empty_entry["spans"].as_array_mut().unwrap().push(fieldless);

    for (artifact, matches) in [
        (
            unknown_child_file,
            (|error: &SourceMapError| matches!(error, SourceMapError::UnknownFile(9)))
                as fn(&SourceMapError) -> bool,
        ),
        (invalid_child_span, |error| {
            matches!(error, SourceMapError::InvalidSpan(_))
        }),
        (invalid_name_span, |error| {
            matches!(error, SourceMapError::UnknownFile(9))
        }),
        (empty_entry, |error| {
            matches!(error, SourceMapError::DuplicateEntry)
        }),
    ] {
        let decoded = MappedText::from_json(&artifact.to_string()).expect("structure decodes");
        let error = decoded
            .map
            .apply(&mut target)
            .expect_err("entry is invalid");
        assert!(matches(&error), "{error:?}");
        assert_eq!(mapped_positions(&target, &target), before);
    }

    // Children beyond the applied value's tree are dropped rather than
    // rejected: emission can canonicalize a value into a form with fewer
    // children (a member access reparses as a variable read).
    let mut extra_child = base.clone();
    extra_child["spans"][argument_with_children]["children"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!({"span": wire_span(0, 1, 1, 2)}));
    let mut grandchild_of_a_leaf = base.clone();
    grandchild_of_a_leaf["spans"][argument_with_children]["children"][0]["children"] =
        serde_json::json!([{"span": wire_span(0, 1, 1, 2)}]);
    for artifact in [extra_child, grandchild_of_a_leaf] {
        let decoded = MappedText::from_json(&artifact.to_string()).expect("structure decodes");
        let mut reparsed = parsed(PROVENANCE_SOURCE);
        decoded
            .map
            .apply(&mut reparsed)
            .expect("excess children attach positionally");
        assert_eq!(reparsed.action_argument_value_span(0, 0, 0, &[2]), None);
        assert!(reparsed.action_argument_value_span(0, 0, 0, &[0]).is_some());
    }

    // An entry carrying no position at all is rejected.
    let mut no_position = base.clone();
    let action = no_position["spans"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|entry| entry["node"] == "action")
        .expect("an action entry");
    action.as_object_mut().unwrap().remove("span");
    action.as_object_mut().unwrap().remove("identifier_span");
    let decoded = MappedText::from_json(&no_position.to_string()).expect("structure decodes");
    assert_eq!(
        decoded.map.apply(&mut target),
        Err(SourceMapError::EmptyEntry)
    );
    assert_eq!(mapped_positions(&target, &target), before);
}

#[test]
fn artifacts_without_identifier_or_children_members_apply_as_before() {
    // An artifact produced before these members existed decodes and applies
    // unchanged: coarse spans map and the finer positions report unmapped.
    fn strip_new_members(value: &mut serde_json::Value) {
        if let Some(object) = value.as_object_mut() {
            for member in [
                "name_span",
                "event_name_span",
                "identifier_span",
                "children",
            ] {
                object.remove(member);
            }
        }
        if let Some(array) = value.as_array_mut() {
            for item in array {
                strip_new_members(item);
            }
        } else if let Some(object) = value.as_object_mut() {
            for item in object.values_mut() {
                strip_new_members(item);
            }
        }
    }

    let map = SourceMap::extract(&parsed(PROVENANCE_SOURCE));
    let mut json: serde_json::Value =
        serde_json::from_str(&MappedText::new("", map).to_json()).unwrap();
    strip_new_members(&mut json);
    json["spans"][0]["future_member"] = serde_json::json!(true);
    let decoded = MappedText::from_json(&json.to_string()).expect("decodes");
    let mut program = parsed(PROVENANCE_SOURCE);
    decoded.map.apply(&mut program).expect("applies");

    assert_eq!(program.rule_name_span(0), None);
    assert_eq!(program.rule_event_name_span(1), None);
    assert_eq!(program.action_identifier_span(0, 0), None);
    let coarse_condition = program.condition_span(0, 0);
    assert!(coarse_condition.is_some());
    assert_eq!(program.condition_value_span(0, 0, &[]), coarse_condition);
    assert_eq!(program.condition_value_span(0, 0, &[0]), None);
    assert!(program.action_argument_span(0, 0, 0).is_some());
    assert_eq!(program.action_argument_value_span(0, 0, 0, &[0]), None);
    assert_eq!(SourceMap::extract(&program), decoded.map);
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
        let expected = mapped_positions(&program, &program);
        assert_eq!(
            mapped_positions(&reparsed, &program),
            expected,
            "{}",
            case.id
        );
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
    assert_eq!(
        mapped_positions(&reparsed, &program),
        mapped_positions(&program, &program)
    );
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
        let before = mapped_positions(&mutated, &mutated);
        assert_eq!(map.apply(&mut mutated), Err(expected));
        assert_eq!(
            mapped_positions(&mutated, &mutated),
            before,
            "no partial application"
        );
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
    let before = mapped_positions(&target, &target);

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
        assert_eq!(mapped_positions(&target, &target), before);
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
        assert_eq!(program.rule_name_span(0), None);
        assert_eq!(program.condition_span(0, 0), None);
        assert_eq!(program.condition_value_span(0, 0, &[]), None);
        assert_eq!(program.action_span(0, 0), None);
        assert_eq!(program.action_identifier_span(0, 0), None);
        assert_eq!(program.action_argument_span(0, 0, 0), None);
        assert_eq!(program.action_argument_value_span(0, 0, 0, &[]), None);
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
