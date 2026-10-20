//! Contract tests for `Program::edit_statements`, the write direction of
//! `workshop-rs/rule-content-v1` (workshop-rs#532).

use serde_json::{Value as JsonValue, json};
use workshop_rs::catalog::Locale;
use workshop_rs::rules::content::rule_content;
use workshop_rs::rules::statement_edit::{
    StatementAddress, StatementEdit, StatementEditError, StatementList,
};
use workshop_rs::{Program, emitter, parser};

use crate::common::{catalog, corpus_text, fixture, locale};

pub(crate) const BASE: &str = r#"rule ("r") {
    event {
        Ongoing - Global;
    }
    conditions {
        Global.a == 1;
        Global.b == 2;
        Global.c == 3;
    }
    actions {
        "first" Wait(1, Ignore Condition);
        Wait(2, Ignore Condition); // tail
        "last" Wait(3, Ignore Condition);
    }
}
"#;

pub(crate) fn parse(source: &str, tag: &str) -> Program {
    parser::parse(source, &catalog(), &locale(tag)).expect("source parses")
}

pub(crate) fn edit(
    source: &str,
    tag: &str,
    list: StatementList,
    index: usize,
    edit: StatementEdit<'_>,
) -> Result<String, StatementEditError> {
    let program = parse(source, tag);
    let result = program.edit_statements(
        &catalog(),
        &locale(tag),
        StatementAddress::new(0, list, index),
        edit,
    )?;
    Ok(result.apply(source).expect("edit applies to its source"))
}

pub(crate) fn ok(source: &str, list: StatementList, index: usize, e: StatementEdit<'_>) -> String {
    edit(source, "en-US", list, index, e).expect("edit succeeds")
}

pub(crate) fn lists(source: &str, tag: &str) -> JsonValue {
    let program = parse(source, tag);
    rule_content(&program.rules[0]).expect("content renders")
}

pub(crate) fn wait(seconds: f64) -> JsonValue {
    json!({"call": "wait", "args": [seconds, {"enum": "Wait", "member": "IGNORE_CONDITION"}]})
}

fn cond(n: f64) -> JsonValue {
    json!({"value": {"call": "==", "args": [{"call": "globalVariable", "args": [{"variable": "a"}]}, n]}})
}

/// A condition that names no variable, for sources that declare none.
pub(crate) fn plain_cond() -> JsonValue {
    json!({"value": {"call": "==", "args": [{"call": "isGameInProgress", "args": []}, true]}})
}

pub(crate) fn waits(content: &JsonValue) -> Vec<f64> {
    content["actions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| a["args"][0].as_f64().unwrap())
        .collect()
}

#[test]
fn base_node_shapes_match_the_export() {
    // Guards the literals above against drift from the export.
    let content = lists(BASE, "en-US");
    assert_eq!(content["actions"][0], wait(1.0));
    assert_eq!(content["conditions"][0], cond(1.0));
}

#[test]
fn insert_replace_delete_at_start_middle_end_of_actions() {
    let nodes = [wait(9.0)];
    let a = StatementList::Actions;

    // Insert at start / middle / end (index == len appends).
    let out = ok(BASE, a, 0, StatementEdit::Insert(&nodes));
    assert_eq!(
        out,
        BASE.replace(
            "        \"first\" Wait(1",
            "        Wait(9, Ignore Condition);\n        \"first\" Wait(1"
        )
    );
    assert_eq!(waits(&lists(&out, "en-US")), [9.0, 1.0, 2.0, 3.0]);

    let out = ok(BASE, a, 1, StatementEdit::Insert(&nodes));
    assert_eq!(waits(&lists(&out, "en-US")), [1.0, 9.0, 2.0, 3.0]);
    assert!(out.contains("        Wait(9, Ignore Condition);\n        Wait(2"));

    let out = ok(BASE, a, 3, StatementEdit::Insert(&nodes));
    assert_eq!(
        out,
        BASE.replace(
            "    }\n}\n",
            "        Wait(9, Ignore Condition);\n    }\n}\n"
        )
    );
    assert_eq!(waits(&lists(&out, "en-US")), [1.0, 2.0, 3.0, 9.0]);

    // Replace at start / middle / end.
    let node = wait(7.0);
    let out = ok(BASE, a, 0, StatementEdit::Replace(&node));
    assert_eq!(
        out,
        BASE.replace("Wait(1, Ignore Condition)", "Wait(7, Ignore Condition)")
    );
    let out = ok(BASE, a, 1, StatementEdit::Replace(&node));
    assert_eq!(
        out,
        BASE.replace("Wait(2, Ignore Condition)", "Wait(7, Ignore Condition)")
    );
    let out = ok(BASE, a, 2, StatementEdit::Replace(&node));
    assert_eq!(
        out,
        BASE.replace("Wait(3, Ignore Condition)", "Wait(7, Ignore Condition)")
    );

    // Delete at start / middle / end.
    let out = ok(BASE, a, 0, StatementEdit::Delete);
    assert_eq!(
        out,
        BASE.replace("        \"first\" Wait(1, Ignore Condition);\n", "")
    );
    let out = ok(BASE, a, 1, StatementEdit::Delete);
    assert_eq!(
        out,
        BASE.replace("        Wait(2, Ignore Condition); // tail\n", "")
    );
    let out = ok(BASE, a, 2, StatementEdit::Delete);
    assert_eq!(
        out,
        BASE.replace("        \"last\" Wait(3, Ignore Condition);\n", "")
    );
    assert_eq!(waits(&lists(&out, "en-US")), [1.0, 2.0]);
}

#[test]
fn conditions_edits_at_start_middle_end() {
    let c = StatementList::Conditions;
    let node = cond(9.0);
    let nodes = [node.clone()];
    for index in 0..=3 {
        let out = ok(BASE, c, index, StatementEdit::Insert(&nodes));
        let content = lists(&out, "en-US");
        assert_eq!(content["conditions"].as_array().unwrap().len(), 4, "{out}");
        assert_eq!(content["conditions"][index]["value"]["args"][1], 9.0);
        assert_eq!(content["actions"], lists(BASE, "en-US")["actions"]);
    }
    for index in 0..3 {
        let out = ok(BASE, c, index, StatementEdit::Replace(&node));
        let content = lists(&out, "en-US");
        assert_eq!(content["conditions"][index]["value"]["args"][1], 9.0);
        let out = ok(BASE, c, index, StatementEdit::Delete);
        assert_eq!(
            lists(&out, "en-US")["conditions"].as_array().unwrap().len(),
            2
        );
    }
    let out = ok(BASE, c, 1, StatementEdit::Delete);
    assert_eq!(out, BASE.replace("        Global.b == 2;\n", ""));
}

#[test]
fn comments_are_kept_removed_or_absent() {
    let a = StatementList::Actions;
    let node = wait(7.0);
    // replace keeps the leading comment
    let out = ok(BASE, a, 0, StatementEdit::Replace(&node));
    assert!(out.contains("\"first\" Wait(7, Ignore Condition);"));
    // delete removes the statement together with its comment
    let out = ok(BASE, a, 2, StatementEdit::Delete);
    assert!(!out.contains("last"));
    // delete also removes a tail line comment on the statement's line
    let out = ok(BASE, a, 1, StatementEdit::Delete);
    assert!(!out.contains("tail"));
    // insert before a commented statement does not take its comment
    let out = ok(
        BASE,
        a,
        0,
        StatementEdit::Insert(std::slice::from_ref(&node)),
    );
    assert!(out.contains("Wait(7, Ignore Condition);\n        \"first\" Wait(1"));
    // insert writes no comment
    assert_eq!(out.matches('"').count(), BASE.matches('"').count());
}

#[test]
fn missing_conditions_section_is_created_and_last_delete_leaves_empty_list() {
    let source = "rule (\"r\") {\n    event {\n        Ongoing - Global;\n    }\n    actions {\n        Wait(1, Ignore Condition);\n    }\n}\n";
    let c = StatementList::Conditions;
    let nodes = [plain_cond()];
    let out = ok(source, c, 0, StatementEdit::Insert(&nodes));
    let content = lists(&out, "en-US");
    assert_eq!(content["conditions"].as_array().unwrap().len(), 1);
    assert!(out.find("conditions").unwrap() < out.find("actions").unwrap());
    assert!(out.contains("    conditions {\n        "), "{out}");

    // Only the section is added: everything else is byte-identical.
    let removed = ok(&out, c, 0, StatementEdit::Delete);
    assert_eq!(lists(&removed, "en-US")["conditions"], json!([]));
    assert_eq!(
        lists(&removed, "en-US")["actions"],
        lists(source, "en-US")["actions"]
    );

    // A rule with no actions section either: created before the closing brace.
    let bare = "rule (\"r\") {\n    event {\n        Ongoing - Global;\n    }\n}\n";
    let out = ok(
        bare,
        StatementList::Actions,
        0,
        StatementEdit::Insert(&[wait(1.0)]),
    );
    assert_eq!(waits(&lists(&out, "en-US")), [1.0]);

    // Deleting the last action reparses with an empty list.
    let out = ok(
        "rule (\"r\") {\n    event {\n        Ongoing - Global;\n    }\n    actions {\n        Wait(1, Ignore Condition);\n    }\n}\n",
        StatementList::Actions,
        0,
        StatementEdit::Delete,
    );
    assert_eq!(lists(&out, "en-US")["actions"], json!([]));
}

#[test]
fn one_line_rule_edits_inline() {
    let source =
        "rule (\"r\") { event { Ongoing - Global; } actions { Wait(1, Ignore Condition); } }\n";
    let out = ok(
        source,
        StatementList::Actions,
        1,
        StatementEdit::Insert(&[wait(2.0)]),
    );
    assert_eq!(waits(&lists(&out, "en-US")), [1.0, 2.0]);
    let out = ok(source, StatementList::Actions, 0, StatementEdit::Delete);
    assert_eq!(lists(&out, "en-US")["actions"], json!([]));
    let out = ok(
        source,
        StatementList::Conditions,
        0,
        StatementEdit::Insert(&[plain_cond()]),
    );
    assert_eq!(
        lists(&out, "en-US")["conditions"].as_array().unwrap().len(),
        1
    );
}

#[test]
fn locale_is_the_one_passed_in() {
    let c = catalog();
    let en_program = parse(BASE, "en-US");
    let zh_source = emitter::emit(&en_program, &c, &locale("zh-CN")).expect("zh emits");
    let node = wait(9.0);
    let zh_edit = edit(
        &zh_source,
        "zh-CN",
        StatementList::Actions,
        1,
        StatementEdit::Insert(std::slice::from_ref(&node)),
    )
    .expect("zh edit");
    assert_eq!(waits(&lists(&zh_edit, "zh-CN")), [1.0, 9.0, 2.0, 3.0]);
    let inserted: String = zh_edit
        .lines()
        .find(|line| line.contains('9'))
        .unwrap()
        .to_string();
    assert!(!inserted.contains("Wait"), "{inserted}");
    // The same statement in en-US uses English spellings.
    let en_edit = ok(
        BASE,
        StatementList::Actions,
        1,
        StatementEdit::Insert(&[node]),
    );
    assert!(en_edit.contains("Wait(9, Ignore Condition)"));

    // Conditions as well.
    let zh_cond = edit(
        &zh_source,
        "zh-CN",
        StatementList::Conditions,
        3,
        StatementEdit::Insert(&[cond(9.0)]),
    )
    .expect("zh condition edit");
    assert_eq!(
        lists(&zh_cond, "zh-CN")["conditions"]
            .as_array()
            .unwrap()
            .len(),
        4
    );
}

#[test]
fn stale_or_out_of_range_addresses_are_refused() {
    let c = catalog();
    let l = locale("en-US");
    let node = wait(1.0);
    let program = parse(BASE, "en-US");
    let at = |rule, list, index| StatementAddress::new(rule, list, index);
    let a = StatementList::Actions;

    for (address, e) in [
        (at(5, a, 0), StatementEdit::Delete),
        (at(0, a, 3), StatementEdit::Delete),
        (at(0, a, 3), StatementEdit::Replace(&node)),
        (
            at(0, a, 4),
            StatementEdit::Insert(std::slice::from_ref(&node)),
        ),
        (
            at(0, StatementList::Conditions, 4),
            StatementEdit::Insert(std::slice::from_ref(&node)),
        ),
    ] {
        assert!(
            program.edit_statements(&c, &l, address, e).is_err(),
            "{address:?}"
        );
    }
    // Empty insert, wrong statement kind.
    assert!(matches!(
        program.edit_statements(&c, &l, at(0, a, 0), StatementEdit::Insert(&[])),
        Err(StatementEditError::EmptyInsert { .. })
    ));
    assert!(matches!(
        program.edit_statements(&c, &l, at(0, a, 0), StatementEdit::Replace(&cond(1.0))),
        Err(StatementEditError::StatementKind { .. })
    ));
    assert!(matches!(
        program.edit_statements(
            &c,
            &l,
            at(0, StatementList::Conditions, 0),
            StatementEdit::Replace(&node)
        ),
        Err(StatementEditError::StatementKind { .. })
    ));
    // A node the conversion refuses.
    assert!(matches!(
        program.edit_statements(
            &c,
            &l,
            at(0, a, 0),
            StatementEdit::Replace(&json!({"call": "noSuch", "args": []}))
        ),
        Err(StatementEditError::Content(_))
    ));

    // The model drifted from the retained source: a statement was removed
    // from the public model after parsing.
    let mut drifted = parse(BASE, "en-US");
    drifted.rules[0].actions.remove(0);
    for e in [StatementEdit::Delete, StatementEdit::Replace(&node)] {
        assert!(matches!(
            drifted.edit_statements(&c, &l, at(0, a, 0), e),
            Err(StatementEditError::StaleAddress { .. })
        ));
    }
    // The model drifted by reordering, same length.
    let mut swapped = parse(BASE, "en-US");
    swapped.rules[0].actions.swap(0, 1);
    // Spans still describe source positions, so the addressed model entry
    // no longer sits at the source's chunk (or the content differs).
    let _ = swapped.edit_statements(&c, &l, at(0, a, 0), StatementEdit::Delete);

    // A programmatic program has no retained source.
    let mut bare = Program::new();
    bare.rule(
        workshop_rs::Rule::new("x", workshop_rs::Event::Global).action(workshop_rs::Action::End),
    );
    assert!(matches!(
        bare.edit_statements(&c, &l, at(0, a, 0), StatementEdit::Delete),
        Err(StatementEditError::SourceUnavailable { .. })
    ));
}

#[test]
fn inexpressible_neighbours_do_not_block_other_edits() {
    // `rawWorkshopAction` is the parser-preserved opaque action: the export
    // cannot express it, but the rule still accepts edits to other statements.
    let source = "rule (\"r\") {\n    event {\n        Ongoing - Global;\n    }\n    actions {\n        End;\n        Wait(1, Ignore Condition);\n        Wait(2, Ignore Condition);\n    }\n}\n";
    let content = lists(source, "en-US");
    assert_eq!(
        content["actions"][0]["call"], "rawWorkshopAction",
        "{content}"
    );
    let a = StatementList::Actions;
    let out = ok(source, a, 1, StatementEdit::Replace(&wait(4.0)));
    assert!(out.contains("End;") && out.contains("Wait(4, Ignore Condition);"));
    let out = ok(source, a, 2, StatementEdit::Insert(&[wait(5.0)]));
    assert!(out.contains("End;"));
    assert_eq!(lists(&out, "en-US")["actions"].as_array().unwrap().len(), 4);
    let out = ok(source, a, 0, StatementEdit::Delete);
    assert!(!out.contains("End;"));
}

fn has_ambiguous(node: &JsonValue) -> bool {
    match node {
        JsonValue::Object(map) => map.contains_key("ambiguous") || map.values().any(has_ambiguous),
        JsonValue::Array(items) => items.iter().any(has_ambiguous),
        _ => false,
    }
}

/// Replace every expressible statement of `source`, rendered in `target`,
/// by its own exported content, in one batch of disjoint edits at `stride`,
/// and require the reparsed program to have unchanged content. Returns the
/// number of statements replaced, or `None` when `target` lacks a spelling
/// the program needs.
///
/// The program is converted to `target` first and its own content is the
/// reference, so the check is `content(parse(emit(statement_from_content(c))))
/// == c` with the neighbours already in `target`.
fn round_trip(source: &str, tag: &str, target: &Locale, stride: usize) -> Option<usize> {
    let c = catalog();
    let text = emitter::emit(&parse(source, tag), &c, target).ok()?;
    let parsed = parser::parse(&text, &c, target).expect("converted source reparses");
    let contents: Vec<JsonValue> = parsed
        .rules
        .iter()
        .map(|rule| rule_content(rule).expect("content renders"))
        .collect();
    let mut edits = Vec::new();
    let mut seen = 0;
    for (rule, content) in contents.iter().enumerate() {
        for (list, key) in [
            (StatementList::Conditions, "conditions"),
            (StatementList::Actions, "actions"),
        ] {
            for (index, node) in content[key].as_array().unwrap().iter().enumerate() {
                // Opaque raw actions and ambiguous enum members have no
                // content; they stay as authored.
                if workshop_rs::rules::statement::statement_from_content(&c, node).is_err() {
                    continue;
                }
                seen += 1;
                if seen % stride != 0 {
                    continue;
                }
                let result = parsed.edit_statements(
                    &c,
                    target,
                    StatementAddress::new(rule, list, index),
                    StatementEdit::Replace(node),
                );
                let edit = match result {
                    Ok(edit) => edit,
                    // An unclosed `If` carries an `End` in the model that the
                    // source does not: it has no text to replace.
                    Err(StatementEditError::StaleAddress { .. }) if node["call"] == "end" => {
                        continue;
                    }
                    Err(error) => {
                        panic!("{tag}->{target} rule {rule} {key}[{index}]: {error}\n{node}")
                    }
                };
                edits.push(edit);
            }
        }
    }
    let document = parsed
        .source(parsed.rule_span(0).expect("rule span").file)
        .expect("converted source retained");
    let edited = document.apply(&edits).expect("edits are disjoint");
    let reparsed = parser::parse(edited.text(), &c, target)
        .unwrap_or_else(|error| panic!("{tag}->{target}: {error}"));
    assert_eq!(reparsed.rules.len(), contents.len());
    for (rule, expected) in reparsed.rules.iter().zip(&contents) {
        let actual = rule_content(rule).unwrap();
        assert_eq!(&actual, expected, "{tag}->{target} rule '{}'", rule.name);
    }
    Some(edits.len())
}

fn round_trip_source(
    source: &str,
    tag: &str,
    stride: usize,
    covered: &mut usize,
    locales: &mut std::collections::BTreeSet<String>,
) {
    for target in catalog().locales() {
        if let Some(checked) = round_trip(source, tag, target, stride) {
            *covered += checked;
            locales.insert(target.to_string());
        }
    }
}

#[test]
fn edited_statements_round_trip_in_every_locale() {
    let mut covered = 0;
    let mut locales = Default::default();
    for (tag, file) in [("en-US", "en-US.ow"), ("zh-CN", "zh-CN.ow")] {
        let source = std::fs::read_to_string(fixture("practice-range", file)).expect("fixture");
        round_trip_source(&source, tag, 3, &mut covered, &mut locales);
    }
    for id in [
        "basic-rule",
        "control-flow",
        "declarations-rules",
        "expressions-values",
        "receiver-calls",
    ] {
        round_trip_source(&corpus_text(id), "en-US", 1, &mut covered, &mut locales);
    }
    assert!(covered > 1000, "covered {covered}");
    assert!(
        locales.contains("en-us") && locales.contains("zh-cn"),
        "{locales:?}"
    );
}

#[test]
fn real_project_statements_round_trip_and_edit_in_place() {
    let c = catalog();
    let mut checked = 0;
    for case in crate::common::cases() {
        let (source, source_locale) = crate::common::source(case);
        let target = crate::common::target_locale(&source_locale);
        for locale in [&source_locale, &target] {
            if let Some(count) = round_trip(&source, source_locale.as_str(), locale, 11) {
                checked += count;
            }
        }

        // In-place edits against the authored text, comments and layout
        // included: replace then delete each rule's first plain action.
        let program = parse(&source, source_locale.as_str());
        let targets: Vec<(usize, usize)> = program
            .rules
            .iter()
            .enumerate()
            .filter_map(|(rule, r)| {
                let content = rule_content(r).ok()?;
                let first = content["actions"].as_array()?.first()?.clone();
                let plain = first.get("call").is_some_and(|id| {
                    ![
                        "if",
                        "elseIf",
                        "else",
                        "end",
                        "while",
                        "forGlobalVariable",
                        "forPlayerVariable",
                        "rawWorkshopAction",
                    ]
                    .contains(&id.as_str().unwrap_or_default())
                }) && !has_ambiguous(&first)
                    && first.get("disabled").is_none();
                (plain && program.action_span(rule, 0).is_some()).then_some((rule, 0))
            })
            .collect();
        assert!(!targets.is_empty(), "{}", case.id);
        let replacement = wait(0.25);
        for deleting in [false, true] {
            let mut edits = Vec::new();
            for &(rule, index) in &targets {
                let edit = if deleting {
                    StatementEdit::Delete
                } else {
                    StatementEdit::Replace(&replacement)
                };
                edits.push(
                    program
                        .edit_statements(
                            &c,
                            &source_locale,
                            StatementAddress::new(rule, StatementList::Actions, index),
                            edit,
                        )
                        .unwrap_or_else(|error| panic!("{} rule {rule}: {error}", case.id)),
                );
            }
            let document = program.source(program.rule_span(0).unwrap().file).unwrap();
            let edited = document.apply(&edits).expect("disjoint edits");
            let reparsed = parser::parse(edited.text(), &c, &source_locale)
                .unwrap_or_else(|error| panic!("{} reparse: {error}", case.id));
            assert_eq!(reparsed.rules.len(), program.rules.len());
            for (rule, (before, after)) in program.rules.iter().zip(&reparsed.rules).enumerate() {
                let before = rule_content(before).unwrap();
                let after = rule_content(after).unwrap();
                assert_eq!(after["conditions"], before["conditions"]);
                let mut expected = before["actions"].as_array().unwrap().clone();
                if targets.iter().any(|&(r, _)| r == rule) {
                    if deleting {
                        expected.remove(0);
                    } else {
                        expected[0] = replacement.clone();
                    }
                }
                assert_eq!(
                    after["actions"].as_array().unwrap(),
                    &expected,
                    "{} rule {rule}",
                    case.id
                );
            }
        }
    }
    assert!(checked > 500, "checked {checked}");
}

pub(crate) fn actions_source(body: &str) -> String {
    format!(
        "rule (\"r\") {{\n    event {{\n        Ongoing - Global;\n    }}\n    actions {{\n{body}    }}\n}}\n"
    )
}

#[test]
fn undeclared_names_are_refused_without_an_edit() {
    let node = json!({
        "call": "setGlobalVariable",
        "args": [{"variable": "neverDeclared"}, 1.0]
    });
    let result = edit(
        BASE,
        "en-US",
        StatementList::Actions,
        0,
        StatementEdit::Replace(&node),
    );
    assert!(
        matches!(result, Err(StatementEditError::Emission(_))),
        "{result:?}"
    );
}
