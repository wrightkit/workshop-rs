//! Layout and boundary tests for `Program::edit_statements` (workshop-rs#532):
//! shared lines, line endings, indentation units, dangling annotations, and
//! appending past a parser-synthesized `End`.

use serde_json::json;
use workshop_rs::rules::content::rule_content;
use workshop_rs::rules::statement_edit::{
    StatementAddress, StatementEdit, StatementEditError, StatementList,
};

use crate::common::catalog;
use crate::statement_edit::{
    BASE, actions_source, edit, lists, ok, parse, plain_cond, wait, waits,
};

#[test]
fn layout_conventions_are_kept() {
    let a = StatementList::Actions;
    let node = wait(9.0);

    // Tab indentation and CRLF line endings.
    let crlf = BASE.replace("    ", "\t").replace('\n', "\r\n");
    let out = ok(
        &crlf,
        a,
        1,
        StatementEdit::Insert(std::slice::from_ref(&node)),
    );
    assert!(
        out.contains("\t\tWait(9, Ignore Condition);\r\n\t\tWait(2"),
        "{out:?}"
    );
    assert!(!out.contains("9, Ignore Condition);\n\t\tWait(2"));
    let out = ok(&crlf, a, 1, StatementEdit::Delete);
    assert_eq!(
        out,
        crlf.replace("\t\tWait(2, Ignore Condition); // tail\r\n", "")
    );

    // Disabled statements: replace swaps the whole statement, modifier
    // included; the export spells a disabled statement with "disabled".
    let source = BASE.replace(
        "Wait(2, Ignore Condition);",
        "Disabled Wait(2, Ignore Condition);",
    );
    let content = lists(&source, "en-US");
    assert_eq!(content["actions"][1]["disabled"], true);
    let out = ok(&source, a, 1, StatementEdit::Replace(&node));
    assert_eq!(waits(&lists(&out, "en-US")), [1.0, 9.0, 3.0]);
    assert!(lists(&out, "en-US")["actions"][1].get("disabled").is_none());
    let mut disabled = node.clone();
    disabled["disabled"] = json!(true);
    let out = ok(BASE, a, 1, StatementEdit::Replace(&disabled));
    assert_eq!(lists(&out, "en-US")["actions"][1], disabled);

    // Control flow: a balanced group inserted before an existing statement.
    let group = [
        json!({"call": "if", "args": [true]}),
        wait(1.0),
        json!({"call": "end", "args": []}),
    ];
    let out = ok(BASE, a, 1, StatementEdit::Insert(&group));
    let actions = lists(&out, "en-US")["actions"].clone();
    assert_eq!(actions[1], group[0]);
    assert_eq!(actions[3], group[2]);
    assert_eq!(actions.as_array().unwrap().len(), 6);
    assert!(
        out.contains(
            "        If(True);\n        Wait(1, Ignore Condition);\n        End;\n        Wait(2"
        ),
        "{out}"
    );
}

#[test]
fn delete_on_a_shared_line_takes_only_its_own_bytes() {
    let a = StatementList::Actions;
    let w = |n: u8| format!("Wait({n}, Ignore Condition);");
    let src = actions_source(&format!(
        "        {} {} // t\n        {}\n",
        w(1),
        w(2),
        w(3)
    ));
    let out = ok(&src, a, 1, StatementEdit::Delete);
    assert_eq!(
        out,
        actions_source(&format!("        {} // t\n        {}\n", w(1), w(3)))
    );
    let out = ok(&src, a, 0, StatementEdit::Delete);
    assert_eq!(
        out,
        actions_source(&format!("        {} // t\n        {}\n", w(2), w(3)))
    );

    let src = actions_source(&format!("        {} \"next\" {}\n", w(1), w(2)));
    let out = ok(&src, a, 1, StatementEdit::Delete);
    assert_eq!(out, actions_source(&format!("        {}\n", w(1))));
    let out = ok(&src, a, 0, StatementEdit::Delete);
    assert_eq!(out, actions_source(&format!("        \"next\" {}\n", w(2))));

    // The section's closing brace keeps its line and indentation.
    let src = format!(
        "rule (\"r\") {{\n    event {{\n        Ongoing - Global;\n    }}\n    actions {{ {}\n        {} }}\n}}\n",
        w(1),
        w(2)
    );
    let out = ok(&src, a, 1, StatementEdit::Delete);
    assert!(
        out.contains("\n        }\n}\n") || out.contains("\n    }\n}\n"),
        "{out}"
    );
    assert_eq!(waits(&lists(&out, "en-US")), [1.0]);

    // CRLF
    let src = src.replace('\n', "\r\n");
    let out = ok(&src, a, 0, StatementEdit::Delete);
    assert_eq!(waits(&lists(&out, "en-US")), [2.0]);
    assert!(!out.contains("\r\r") && !out.replace("\r\n", "").contains('\r'));
}

#[test]
fn insert_at_the_end_skips_a_dangling_annotation() {
    let src = actions_source("        Wait(1, Ignore Condition);\n        \"dangling\"\n");
    let out = ok(
        &src,
        StatementList::Actions,
        1,
        StatementEdit::Insert(&[wait(7.0)]),
    );
    assert!(
        out.contains("Wait(7, Ignore Condition);\n        \"dangling\""),
        "{out}"
    );
    assert_eq!(waits(&lists(&out, "en-US")), [1.0, 7.0]);
}

#[test]
fn inserted_lines_follow_the_line_ending_of_their_anchor() {
    let src = BASE.replacen('\n', "\r\n", 1);
    let out = ok(
        &src,
        StatementList::Actions,
        1,
        StatementEdit::Insert(&[wait(7.0)]),
    );
    assert!(
        out.contains("Wait(7, Ignore Condition);\n        Wait(2"),
        "{out:?}"
    );
    assert!(!out.contains("Wait(7, Ignore Condition);\r"));
}

#[test]
fn append_to_a_list_ending_in_an_unclosed_if() {
    let a = StatementList::Actions;
    let src = actions_source("        If(True);\n        Wait(1, Ignore Condition);\n");
    let before = lists(&src, "en-US")["actions"].clone();
    assert_eq!(before.as_array().unwrap().len(), 3, "synthesized End");
    // Insert at len and at the synthesized End's own index both append
    // after the last source statement; replace and delete are refused there.
    for index in [2, 3] {
        let out = ok(&src, a, index, StatementEdit::Insert(&[wait(7.0)]));
        let after = lists(&out, "en-US")["actions"].clone();
        let mut expected = before.as_array().unwrap().clone();
        expected.insert(2, wait(7.0));
        assert_eq!(after.as_array().unwrap(), &expected, "index {index}");
        assert!(out.contains("Wait(1, Ignore Condition);\n        Wait(7, Ignore Condition);\n"));
    }
    for e in [StatementEdit::Delete, StatementEdit::Replace(&wait(7.0))] {
        assert!(matches!(
            edit(&src, "en-US", a, 2, e),
            Err(StatementEditError::StaleAddress { .. })
        ));
    }
    assert!(edit(&src, "en-US", a, 4, StatementEdit::Insert(&[wait(7.0)])).is_err());
}

#[test]
fn append_to_the_unclosed_if_rule_of_a_real_project() {
    let case = crate::common::cases()
        .iter()
        .find(|case| case.id == "ai-pve")
        .expect("ai-pve case");
    let (source, source_locale) = crate::common::source(case);
    let tag = source_locale.as_str();
    let program = parse(&source, tag);
    let rule = program
        .rules
        .iter()
        .position(|rule| rule.name == "[Zenyatta] Snap Kick (Self Wall)")
        .expect("rule with an unclosed If");
    let before = rule_content(&program.rules[rule]).unwrap()["actions"]
        .as_array()
        .unwrap()
        .clone();
    let len = before.len();
    // The source closes neither `If`: the model's two trailing `End`s are
    // synthesized, so the source has `len - 2` statements and an append
    // lands after the last of them, inside both `If` bodies.
    assert_eq!(len, 6);
    assert!(before[len - 2..].iter().all(|node| node["call"] == "end"));
    for index in [len - 2, len - 1, len] {
        let edit = program
            .edit_statements(
                &catalog(),
                &source_locale,
                StatementAddress::new(rule, StatementList::Actions, index),
                StatementEdit::Insert(&[wait(7.0)]),
            )
            .unwrap_or_else(|error| panic!("index {index}: {error}"));
        let out = edit.apply(&source).unwrap();
        let reparsed = parse(&out, tag);
        let after = rule_content(&reparsed.rules[rule]).unwrap()["actions"]
            .as_array()
            .unwrap()
            .clone();
        let mut expected = before.clone();
        expected.insert(len - 2, wait(7.0));
        let first = after.iter().zip(&expected).position(|(a, b)| a != b);
        assert_eq!(
            (after.len(), expected.len(), first),
            (expected.len(), expected.len(), None),
            "index {index}, rule {rule}"
        );
    }
}

#[test]
fn created_section_follows_the_source_indentation_unit() {
    let tab_crlf = "rule (\"r\") {\r\n\tevent {\r\n\t\tOngoing - Global;\r\n\t}\r\n\tactions {\r\n\t\tWait(1, Ignore Condition);\r\n\t}\r\n}\r\n";
    let out = ok(
        tab_crlf,
        StatementList::Conditions,
        0,
        StatementEdit::Insert(&[plain_cond()]),
    );
    assert!(
        out.contains("\r\n\tconditions {\r\n\t\tIs Game In Progress == True;\r\n\t}\r\n\tactions"),
        "{out:?}"
    );
    assert!(!out.contains("\t    "));

    // A rule with no actions section and tab nesting taken from `event`.
    let bare = "rule (\"r\") {\n\tevent {\n\t\tOngoing - Global;\n\t}\n}\n";
    let out = ok(
        bare,
        StatementList::Actions,
        0,
        StatementEdit::Insert(&[wait(1.0)]),
    );
    assert!(
        out.contains("\n\tactions {\n\t\tWait(1, Ignore Condition);\n\t}\n}"),
        "{out:?}"
    );

    // Two-space nesting is kept; four spaces only when nothing is learnable.
    let two = "rule (\"r\") {\n  event {\n    Ongoing - Global;\n  }\n}\n";
    let out = ok(
        two,
        StatementList::Actions,
        0,
        StatementEdit::Insert(&[wait(1.0)]),
    );
    assert!(
        out.contains("\n  actions {\n    Wait(1, Ignore Condition);\n  }\n}"),
        "{out:?}"
    );

    // Appending into an empty existing section follows the same unit.
    let empty = "rule (\"r\") {\n\tevent {\n\t\tOngoing - Global;\n\t}\n\tactions {\n\t}\n}\n";
    let out = ok(
        empty,
        StatementList::Actions,
        0,
        StatementEdit::Insert(&[wait(1.0)]),
    );
    assert!(
        out.contains("\tactions {\n\t\tWait(1, Ignore Condition);\n\t}"),
        "{out:?}"
    );
}
