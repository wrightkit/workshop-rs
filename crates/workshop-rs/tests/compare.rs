use super::common;
use workshop_rs::compare::{self, DeclarationTable, Difference, DifferenceKind, PathStep};
use workshop_rs::{emitter, parser, roundtrip};

fn parse(source: &str) -> workshop_rs::Program {
    parser::parse(source, &common::catalog(), &common::en()).expect("test source parses")
}

fn emit(program: &workshop_rs::Program) -> String {
    emitter::emit(program, &common::catalog(), &common::en()).expect("test program emits")
}

fn assert_equivalent(first: &str, second: &str) {
    let first = parse(first);
    let second = parse(second);
    assert!(
        compare::equivalent(&first, &second),
        "programs must be behaviorally equivalent: {:?}",
        compare::difference(&first, &second)
    );
    assert!(compare::equivalent(&second, &first));
}

fn first_difference(first: &str, second: &str) -> Difference {
    compare::difference(&parse(first), &parse(second)).expect("programs must differ")
}

fn rules_only(rule_body: &str) -> String {
    format!("rule (\"r\") {{\n    event {{ Ongoing - Global; }}\n{rule_body}\n}}\n")
}

#[test]
fn identical_programs_are_equivalent() {
    let source = common::corpus_text("declarations-rules");
    let program = parse(&source);
    assert!(compare::equivalent(&program, &program));
    assert!(compare::difference(&program, &program).is_none());
}

#[test]
fn rule_names_do_not_matter() {
    let first = r#"rule ("first name") {
    event { Ongoing - Global; }
    actions { Disable Inspector Recording; }
}
rule ("second name") {
    event { Ongoing - Global; }
    actions { Kill(All Players(All Teams), Null); }
}
"#;
    let second = r#"rule ("renamed") {
    event { Ongoing - Global; }
    actions { Disable Inspector Recording; }
}
rule ("also renamed") {
    event { Ongoing - Global; }
    actions { Kill(All Players(All Teams), Null); }
}
"#;
    assert_equivalent(first, second);
}

#[test]
fn variable_and_subroutine_names_do_not_matter() {
    let first = r#"variables {
    global:
        0: score
    player:
        0: started
}
subroutines {
    0: showStatus
}
rule ("r1") {
    event { Subroutine; showStatus; }
    actions { Set Global Variable(score, 0); }
}
rule ("r2") {
    event { Ongoing - Each Player; All; All; }
    actions {
        Set Player Variable(Event Player, started, True);
        Call Subroutine(showStatus);
    }
}
"#;
    let second = r#"variables {
    global:
        0: points
    player:
        0: flag
}
subroutines {
    0: display
}
rule ("r1") {
    event { Subroutine; display; }
    actions { Set Global Variable(points, 0); }
}
rule ("r2") {
    event { Ongoing - Each Player; All; All; }
    actions {
        Set Player Variable(Event Player, flag, True);
        Call Subroutine(display);
    }
}
"#;
    assert_equivalent(first, second);
    assert!(!roundtrip::equivalent(&parse(first), &parse(second)));
}

#[test]
fn declaration_table_order_does_not_matter() {
    let first = r#"variables {
    global:
        0: a
        1: b
    player:
        0: p
        1: q
}
rule ("r") {
    event { Ongoing - Global; }
    actions {
        Set Global Variable(a, 1);
        Set Global Variable(b, 2);
        Set Player Variable(All Players(All Teams), q, 3);
    }
}
"#;
    let second = r#"variables {
    global:
        1: b
        0: a
    player:
        1: q
        0: p
}
rule ("r") {
    event { Ongoing - Global; }
    actions {
        Set Global Variable(a, 1);
        Set Global Variable(b, 2);
        Set Player Variable(All Players(All Teams), q, 3);
    }
}
"#;
    assert_equivalent(first, second);
}

#[test]
fn subroutine_rule_relative_order_does_not_matter() {
    let first = r#"variables {
    global:
        0: x
}
subroutines {
    0: setup
    1: report
}
rule ("sub one") {
    event { Subroutine; setup; }
    actions { Set Global Variable(x, 1); }
}
rule ("tick") {
    event { Ongoing - Global; }
    actions { Wait(0.016, Ignore Condition); }
}
rule ("sub two") {
    event { Subroutine; report; }
    actions { Set Global Variable(x, 2); }
}
"#;
    let second = r#"variables {
    global:
        0: x
}
subroutines {
    0: setup
    1: report
}
rule ("tick") {
    event { Ongoing - Global; }
    actions { Wait(0.016, Ignore Condition); }
}
rule ("sub two") {
    event { Subroutine; report; }
    actions { Set Global Variable(x, 2); }
}
rule ("sub one") {
    event { Subroutine; setup; }
    actions { Set Global Variable(x, 1); }
}
"#;
    assert_equivalent(first, second);
}

#[test]
fn set_self_operation_equals_modify() {
    let first = r#"variables {
    global:
        0: x
    player:
        0: p
}
rule ("r") {
    event { Ongoing - Global; }
    actions {
        Global.x = Global.x + 1;
        Set Player Variable(Event Player, p, Subtract((Event Player).p, 2));
    }
}
"#;
    let second = r#"variables {
    global:
        0: x
    player:
        0: p
}
rule ("r") {
    event { Ongoing - Global; }
    actions {
        Modify Global Variable(x, Add, 1);
        Modify Player Variable(Event Player, p, Subtract, 2);
    }
}
"#;
    assert_equivalent(first, second);
}

#[test]
fn each_player_equals_explicit_all_filters() {
    let first = r#"rule ("r") {
    event { Ongoing - Each Player; }
    actions { Disable Inspector Recording; }
}
"#;
    let second = r#"rule ("r") {
    event {
        Ongoing - Each Player;
        All;
        All;
    }
    actions { Disable Inspector Recording; }
}
"#;
    assert_equivalent(first, second);
}

#[test]
fn disabled_rule_presence_does_not_matter() {
    let second = r#"disabled rule ("debug") {
    event { Ongoing - Global; }
    actions { Set Global Variable(x, 1); }
}
"#
    .to_owned()
        + &rules_only("    actions { Wait(0.016, Ignore Condition); }");
    let first = r#"variables {
    global:
        0: x
}
"#
    .to_owned()
        + &rules_only("    actions { Wait(0.016, Ignore Condition); }");
    assert_equivalent(&first, &second);
}

#[test]
fn disabled_condition_presence_does_not_matter() {
    let first =
        rules_only("    conditions { Global.x == 0; }\n    actions { Wait(1, Ignore Condition); }");
    let second = rules_only(
        "    conditions { Global.x == 0; disabled Global.x == 1; }\n    actions { Wait(1, Ignore Condition); }",
    );
    let declarations = "variables {\n    global:\n        0: x\n}\n";
    assert_equivalent(
        &(declarations.to_owned() + &first),
        &(declarations.to_owned() + &second),
    );
}

#[test]
fn disabled_action_presence_does_not_matter() {
    let first = rules_only("    actions {\n        Set Global Variable(x, 1);\n    }");
    let second = rules_only(
        "    actions {\n        disabled Small Message(All Players(All Teams), Custom String(\"hi\"));\n        Set Global Variable(x, 1);\n        disabled If(True);\n            Wait(1, Ignore Condition);\n        End;\n    }",
    );
    let declarations = "variables {\n    global:\n        0: x\n}\n";
    assert_equivalent(
        &(declarations.to_owned() + &first),
        &(declarations.to_owned() + &second),
    );
}

#[test]
fn empty_rule_presence_does_not_matter() {
    let first = rules_only("    actions { Wait(1, Ignore Condition); }");
    let second = rules_only("    actions { Wait(1, Ignore Condition); }")
        + "rule (\"no actions\") {\n    event { Ongoing - Global; }\n    actions { }\n}\n"
        + "rule (\"no sections\") {\n    event { Ongoing - Each Player; All; All; }\n}\n";
    assert_equivalent(&first, &second);
}

#[test]
fn a_rule_with_only_disabled_actions_is_inert() {
    let first = rules_only("    actions { Wait(1, Ignore Condition); }");
    let second = rules_only("    actions { Wait(1, Ignore Condition); }")
        + "rule (\"inert\") {\n    event { Ongoing - Global; }\n    actions { disabled Wait(1, Ignore Condition); }\n}\n";
    assert_equivalent(&first, &second);
}

#[test]
fn source_layout_does_not_matter() {
    let first = rules_only("    actions { Wait(1, Ignore Condition); }");
    let second = "// element count: 99
rule(\"r\"){event{Ongoing - Global;}actions{Wait(1, Ignore Condition);}}
";
    assert_equivalent(&first, second);
}

#[test]
fn settings_difference_is_reported() {
    let first = "settings { main { Description: \"first\" } }\n".to_owned()
        + &rules_only("    actions { Wait(1, Ignore Condition); }");
    let second = "settings { main { Description: \"second\" } }\n".to_owned()
        + &rules_only("    actions { Wait(1, Ignore Condition); }");
    let difference = first_difference(&first, &second);
    assert!(matches!(difference.kind, DifferenceKind::Settings));
    assert!(difference.rules.is_none());
}

#[test]
fn declaration_table_index_sets_matter() {
    let first = "variables {\n    global:\n        0: x\n}\n".to_owned()
        + &rules_only("    actions { Wait(1, Ignore Condition); }");
    let second = "variables {\n    global:\n        0: x\n        1: y\n}\n".to_owned()
        + &rules_only("    actions { Wait(1, Ignore Condition); }");
    let difference = first_difference(&first, &second);
    assert_eq!(
        difference.kind,
        DifferenceKind::DeclarationTable {
            table: DeclarationTable::GlobalVariables,
            first: vec![0],
            second: vec![0, 1],
        }
    );
}

#[test]
fn variable_index_binding_differs() {
    let declarations = |order: &str| format!("variables {{\n    global:\n{order}}}\n");
    let body = rules_only("    actions { Set Global Variable(a, 1); }");
    let first = declarations("        0: a\n        1: b\n") + &body;
    let second = declarations("        1: a\n        0: b\n") + &body;
    let difference = first_difference(&first, &second);
    assert_eq!(difference.rules, Some((0, 0)));
    assert_eq!(
        difference.kind,
        DifferenceKind::Reference {
            table: DeclarationTable::GlobalVariables,
            first: 0,
            second: 1,
        }
    );
}

#[test]
fn rule_order_matters() {
    let wait = "rule (\"wait\") {\n    event { Ongoing - Global; }\n    actions { Wait(1, Ignore Condition); }\n}\n";
    let kill = "rule (\"kill\") {\n    event { Ongoing - Global; }\n    actions { Kill(All Players(All Teams), Null); }\n}\n";
    let difference = first_difference(&format!("{wait}{kill}"), &format!("{kill}{wait}"));
    assert_eq!(difference.rules, Some((0, 0)));
    assert!(matches!(
        difference.kind,
        DifferenceKind::Shape { .. } | DifferenceKind::Scalar { .. }
    ));
}

#[test]
fn initializer_rules_keep_their_position() {
    let init = r#"rule ("init") {
    event { Ongoing - Global; }
    actions {
        Set Global Variable(a, 1);
        Set Global Variable(b, 2);
    }
}
"#;
    let tick = r#"rule ("tick") {
    event { Ongoing - Global; }
    actions { Wait(0.016, Ignore Condition); }
}
"#;
    let declarations = "variables {\n    global:\n        0: a\n        1: b\n}\n";
    let difference = first_difference(
        &format!("{declarations}{init}{tick}"),
        &format!("{declarations}{tick}{init}"),
    );
    assert_eq!(difference.rules, Some((0, 0)));
}

#[test]
fn independent_sets_do_not_commute() {
    let declarations = "variables {\n    global:\n        0: a\n        1: b\n}\n";
    let first = declarations.to_owned()
        + &rules_only(
            "    actions {\n        Set Global Variable(a, 1);\n        Set Global Variable(b, 2);\n    }",
        );
    let second = declarations.to_owned()
        + &rules_only(
            "    actions {\n        Set Global Variable(b, 2);\n        Set Global Variable(a, 1);\n    }",
        );
    let difference = first_difference(&first, &second);
    assert_eq!(difference.rules, Some((0, 0)));
    assert_eq!(difference.path, vec![PathStep::Action(0)]);
    assert_eq!(
        difference.kind,
        DifferenceKind::Reference {
            table: DeclarationTable::GlobalVariables,
            first: 0,
            second: 1,
        }
    );
}

#[test]
fn event_kind_difference_is_reported() {
    let first = rules_only("    actions { Wait(1, Ignore Condition); }");
    let second = "rule (\"r\") {\n    event { Ongoing - Each Player; All; All; }\n    actions { Wait(1, Ignore Condition); }\n}\n";
    let difference = first_difference(&first, second);
    assert_eq!(difference.rules, Some((0, 0)));
    assert!(difference.path.is_empty());
    assert_eq!(
        difference.kind,
        DifferenceKind::Event {
            first: "global".to_string(),
            second: "eachPlayer(team=All, target=All)".to_string(),
        }
    );
}

#[test]
fn event_filters_are_compared() {
    let rule = |filters: &str| {
        format!(
            "rule (\"r\") {{\n    event {{\n        Ongoing - Each Player;\n{filters}    }}\n    actions {{ Wait(1, Ignore Condition); }}\n}}\n"
        )
    };
    let difference = first_difference(
        &rule("        All;\n        All;\n"),
        &rule("        Team 1;\n        All;\n"),
    );
    assert!(matches!(difference.kind, DifferenceKind::Event { .. }));
    let difference = first_difference(
        &rule("        All;\n        All;\n"),
        &rule("        All;\n        Slot 3;\n"),
    );
    assert!(matches!(difference.kind, DifferenceKind::Event { .. }));
    let player_rule = |filters: &str| {
        format!(
            "rule (\"r\") {{\n    event {{\n        Player Died;\n{filters}    }}\n    actions {{ Wait(1, Ignore Condition); }}\n}}\n"
        )
    };
    let difference = first_difference(
        &player_rule("        Team 1;\n        All;\n"),
        &player_rule("        Team 1;\n        Ana;\n"),
    );
    assert!(matches!(difference.kind, DifferenceKind::Event { .. }));
}

#[test]
fn condition_count_difference_is_reported() {
    let declarations = "variables {\n    global:\n        0: x\n}\n";
    let first = declarations.to_owned()
        + &rules_only(
            "    conditions { Global.x == 0; }\n    actions { Wait(1, Ignore Condition); }",
        );
    let second = declarations.to_owned()
        + &rules_only(
            "    conditions { Global.x == 0; Global.x < 9; }\n    actions { Wait(1, Ignore Condition); }",
        );
    let difference = first_difference(&first, &second);
    assert_eq!(difference.rules, Some((0, 0)));
    assert_eq!(
        difference.kind,
        DifferenceKind::Count {
            element: "conditions",
            first: 1,
            second: 2,
        }
    );
}

#[test]
fn condition_order_matters() {
    let declarations = "variables {\n    global:\n        0: x\n        1: y\n}\n";
    let first = declarations.to_owned()
        + &rules_only(
            "    conditions { Global.x == 0; Global.y == 0; }\n    actions { Wait(1, Ignore Condition); }",
        );
    let second = declarations.to_owned()
        + &rules_only(
            "    conditions { Global.y == 0; Global.x == 0; }\n    actions { Wait(1, Ignore Condition); }",
        );
    let difference = first_difference(&first, &second);
    assert_eq!(
        difference.path,
        vec![PathStep::Condition(0), PathStep::Value(0)]
    );
    assert_eq!(
        difference.kind,
        DifferenceKind::Reference {
            table: DeclarationTable::GlobalVariables,
            first: 0,
            second: 1,
        }
    );
}

#[test]
fn condition_value_difference_is_reported() {
    let declarations = "variables {\n    global:\n        0: x\n}\n";
    let first = declarations.to_owned()
        + &rules_only(
            "    conditions { Global.x == 0; }\n    actions { Wait(1, Ignore Condition); }",
        );
    let second = declarations.to_owned()
        + &rules_only(
            "    conditions { Global.x == 1; }\n    actions { Wait(1, Ignore Condition); }",
        );
    let difference = first_difference(&first, &second);
    assert_eq!(
        difference.path,
        vec![PathStep::Condition(0), PathStep::Value(1)]
    );
    assert!(matches!(difference.kind, DifferenceKind::Scalar { .. }));
}

#[test]
fn action_count_difference_is_reported() {
    let declarations = "variables {\n    global:\n        0: x\n}\n";
    let first = declarations.to_owned() + &rules_only("    actions { Set Global Variable(x, 1); }");
    let second = declarations.to_owned()
        + &rules_only("    actions { Set Global Variable(x, 1); Wait(1, Ignore Condition); }");
    let difference = first_difference(&first, &second);
    assert_eq!(difference.rules, Some((0, 0)));
    assert_eq!(
        difference.kind,
        DifferenceKind::Count {
            element: "actions",
            first: 1,
            second: 2,
        }
    );
}

#[test]
fn nested_value_difference_is_reported() {
    let declarations = "variables {\n    global:\n        0: x\n}\n";
    let first = declarations.to_owned()
        + &rules_only("    actions { Set Global Variable(x, Add(Global.x, 1)); }");
    let second = declarations.to_owned()
        + &rules_only("    actions { Set Global Variable(x, Add(Global.x, 2)); }");
    let difference = first_difference(&first, &second);
    assert_eq!(
        difference.path,
        vec![PathStep::Action(0), PathStep::Value(0), PathStep::Value(1)]
    );
    assert_eq!(
        difference.kind,
        DifferenceKind::Scalar {
            first: "1".to_string(),
            second: "2".to_string(),
        }
    );
}

#[test]
fn player_variable_index_difference_is_reported() {
    let declarations = "variables {\n    player:\n        0: p\n        1: q\n}\n";
    let first = declarations.to_owned()
        + &rules_only("    actions { Set Player Variable(Event Player, p, 1); }");
    let second = declarations.to_owned()
        + &rules_only("    actions { Set Player Variable(Event Player, q, 1); }");
    let difference = first_difference(&first, &second);
    assert_eq!(difference.path, vec![PathStep::Action(0)]);
    assert_eq!(
        difference.kind,
        DifferenceKind::Reference {
            table: DeclarationTable::PlayerVariables,
            first: 0,
            second: 1,
        }
    );
}

#[test]
fn subroutine_call_index_difference_is_reported() {
    let subroutines = "subroutines {\n    0: setup\n    1: report\n}\n";
    let sub_rules = "rule (\"s0\") {\n    event { Subroutine; setup; }\n    actions { Wait(1, Ignore Condition); }\n}\nrule (\"s1\") {\n    event { Subroutine; report; }\n    actions { Wait(2, Ignore Condition); }\n}\n";
    let first =
        subroutines.to_owned() + sub_rules + &rules_only("    actions { Call Subroutine(setup); }");
    let second = subroutines.to_owned()
        + sub_rules
        + &rules_only("    actions { Call Subroutine(report); }");
    let difference = first_difference(&first, &second);
    assert_eq!(difference.path, vec![PathStep::Action(0)]);
    assert_eq!(
        difference.kind,
        DifferenceKind::Reference {
            table: DeclarationTable::Subroutines,
            first: 0,
            second: 1,
        }
    );
}

#[test]
fn subroutine_event_binding_difference_is_reported() {
    let subroutines = "subroutines {\n    0: setup\n    1: report\n}\n";
    let first = subroutines.to_owned()
        + "rule (\"s\") {\n    event { Subroutine; setup; }\n    actions { Wait(1, Ignore Condition); }\n}\n";
    let second = subroutines.to_owned()
        + "rule (\"s\") {\n    event { Subroutine; report; }\n    actions { Wait(1, Ignore Condition); }\n}\n";
    let difference = first_difference(&first, &second);
    assert_eq!(difference.rules, Some((0, 0)));
    assert_eq!(
        difference.kind,
        DifferenceKind::Event {
            first: "subroutine(index=0)".to_string(),
            second: "subroutine(index=1)".to_string(),
        }
    );
}

#[test]
fn subroutine_rule_body_difference_is_reported() {
    let subroutines = "subroutines {\n    0: setup\n}\n";
    let rule = |value: u32| {
        format!(
            "rule (\"s\") {{\n    event {{ Subroutine; setup; }}\n    actions {{ Wait({value}, Ignore Condition); }}\n}}\n"
        )
    };
    let difference = first_difference(
        &(subroutines.to_owned() + &rule(1)),
        &(subroutines.to_owned() + &rule(2)),
    );
    assert_eq!(difference.rules, Some((0, 0)));
}

#[test]
fn a_subroutine_rule_is_not_an_ordered_rule() {
    let subroutines = "subroutines {\n    0: setup\n}\n";
    let first = subroutines.to_owned()
        + "rule (\"s\") {\n    event { Subroutine; setup; }\n    actions { Wait(1, Ignore Condition); }\n}\n"
        + &rules_only("    actions { Wait(2, Ignore Condition); }");
    let second = subroutines.to_owned()
        + &rules_only("    actions { Wait(1, Ignore Condition); }")
        + &rules_only("    actions { Wait(2, Ignore Condition); }");
    let difference = first_difference(&first, &second);
    assert_eq!(
        difference.kind,
        DifferenceKind::Count {
            element: "rules",
            first: 1,
            second: 2,
        }
    );
}

#[test]
fn set_modify_operation_must_match() {
    let declarations = "variables {\n    global:\n        0: x\n}\n";
    let first = declarations.to_owned() + &rules_only("    actions { Global.x = Global.x + 1; }");
    let second = declarations.to_owned() + &rules_only("    actions { Global.x -= 1; }");
    let difference = first_difference(&first, &second);
    assert_eq!(
        difference.kind,
        DifferenceKind::Operation {
            first: "add".to_string(),
            second: "subtract".to_string(),
        }
    );
}

#[test]
fn set_modify_must_reference_the_same_variable() {
    let declarations = "variables {\n    global:\n        0: x\n        1: y\n}\n";
    let first = declarations.to_owned() + &rules_only("    actions { Global.x = Global.y + 1; }");
    let second = declarations.to_owned() + &rules_only("    actions { Global.x += 1; }");
    let difference = first_difference(&first, &second);
    assert_eq!(
        difference.kind,
        DifferenceKind::Reference {
            table: DeclarationTable::GlobalVariables,
            first: 1,
            second: 0,
        }
    );
}

#[test]
fn plain_set_is_not_a_modify() {
    let declarations = "variables {\n    global:\n        0: x\n}\n";
    let first = declarations.to_owned() + &rules_only("    actions { Global.x = 1; }");
    let second = declarations.to_owned() + &rules_only("    actions { Global.x += 1; }");
    let difference = first_difference(&first, &second);
    assert_eq!(
        difference.kind,
        DifferenceKind::Shape {
            first: "setGlobalVariable",
            second: "modifyGlobalVariable",
        }
    );
}

#[test]
fn modify_operations_differ() {
    let declarations = "variables {\n    global:\n        0: x\n}\n";
    let first = declarations.to_owned() + &rules_only("    actions { Global.x += 1; }");
    let second = declarations.to_owned() + &rules_only("    actions { Global.x -= 1; }");
    let difference = first_difference(&first, &second);
    assert_eq!(
        difference.kind,
        DifferenceKind::Operation {
            first: "add".to_string(),
            second: "subtract".to_string(),
        }
    );
}

#[test]
fn algebraic_identities_are_not_allowed() {
    let declarations = "variables {\n    global:\n        0: x\n}\n";
    let first = declarations.to_owned()
        + &rules_only("    actions { Set Global Variable(x, Add(Global.x, 0)); }");
    let second =
        declarations.to_owned() + &rules_only("    actions { Set Global Variable(x, Global.x); }");
    let difference = first_difference(&first, &second);
    assert!(matches!(difference.kind, DifferenceKind::Shape { .. }));
}

#[test]
fn if_else_presence_is_a_difference() {
    let declarations = "variables {\n    global:\n        0: x\n}\n";
    let first = declarations.to_owned()
        + &rules_only(
            "    actions {\n        If(Global.x == 1);\n            Wait(1, Ignore Condition);\n        End;\n    }",
        );
    let second = declarations.to_owned()
        + &rules_only(
            "    actions {\n        If(Global.x == 1);\n            Wait(1, Ignore Condition);\n        Else;\n            Wait(2, Ignore Condition);\n        End;\n    }",
        );
    let difference = first_difference(&first, &second);
    assert_eq!(
        difference.kind,
        DifferenceKind::Shape {
            first: "if",
            second: "if/else",
        }
    );
}

#[test]
fn branch_body_difference_is_reported() {
    let declarations = "variables {\n    global:\n        0: x\n}\n";
    let rule = |wait: u32| {
        rules_only(&format!(
            "    actions {{\n        If(Global.x == 1);\n            Wait(1, Ignore Condition);\n        Else If(Global.x == 2);\n            Wait({wait}, Ignore Condition);\n        End;\n    }}"
        ))
    };
    let difference = first_difference(
        &(declarations.to_owned() + &rule(2)),
        &(declarations.to_owned() + &rule(3)),
    );
    assert_eq!(
        difference.path,
        vec![
            PathStep::Action(0),
            PathStep::Branch(1),
            PathStep::Action(0),
            PathStep::Value(0)
        ]
    );
    assert_eq!(
        difference.kind,
        DifferenceKind::Scalar {
            first: "2".to_string(),
            second: "3".to_string(),
        }
    );
}

#[test]
fn while_body_difference_is_reported() {
    let declarations = "variables {\n    global:\n        0: x\n}\n";
    let rule = |wait: u32| {
        rules_only(&format!(
            "    actions {{\n        While(Global.x < 3);\n            Wait({wait}, Ignore Condition);\n        End;\n    }}"
        ))
    };
    let difference = first_difference(
        &(declarations.to_owned() + &rule(1)),
        &(declarations.to_owned() + &rule(2)),
    );
    assert_eq!(
        difference.path,
        vec![
            PathStep::Action(0),
            PathStep::Body,
            PathStep::Action(0),
            PathStep::Value(0)
        ]
    );
}

#[test]
fn value_shape_difference_is_reported() {
    let declarations = "variables {\n    global:\n        0: x\n}\n";
    let first = declarations.to_owned() + &rules_only("    actions { Set Global Variable(x, 1); }");
    let second = declarations.to_owned()
        + &rules_only("    actions { Set Global Variable(x, Custom String(\"one\")); }");
    let difference = first_difference(&first, &second);
    assert_eq!(
        difference.kind,
        DifferenceKind::Shape {
            first: "number",
            second: "call",
        }
    );
}

#[test]
fn action_name_difference_is_reported() {
    let first = rules_only("    actions { Small Message(All Players(All Teams), 1); }");
    let second = rules_only("    actions { Big Message(All Players(All Teams), 1); }");
    let difference = first_difference(&first, &second);
    assert!(matches!(difference.kind, DifferenceKind::Scalar { .. }));
}

#[test]
fn difference_reports_positions_skipping_inert_rules() {
    let declarations = "variables {\n    global:\n        0: x\n}\n";
    let inert = "disabled rule (\"d\") {\n    event { Ongoing - Global; }\n    actions { Wait(1, Ignore Condition); }\n}\nrule (\"empty\") {\n    event { Ongoing - Global; }\n    actions { }\n}\n";
    let first =
        declarations.to_owned() + inert + &rules_only("    actions { Set Global Variable(x, 1); }");
    let second =
        declarations.to_owned() + inert + &rules_only("    actions { Set Global Variable(x, 2); }");
    let difference = first_difference(&first, &second);
    assert_eq!(difference.rules, Some((2, 2)));
    assert_eq!(
        difference.path,
        vec![PathStep::Action(0), PathStep::Value(0)]
    );
}

#[test]
fn unmaterializable_program_reports_malformed() {
    use workshop_rs::compare::ProgramSide;
    use workshop_rs::{Action, Event, Program, Rule, Value};

    let mut broken = Program::default();
    broken.rules.push(
        Rule::new("r", Event::Global).action(Action::SetGlobalVariable {
            variable: "ghost".to_string(),
            value: Value::number(1.0),
        }),
    );
    let program = parse(&rules_only("    actions { Wait(1, Ignore Condition); }"));
    let difference = compare::difference(&broken, &program).expect("programs must differ");
    assert!(matches!(
        difference.kind,
        DifferenceKind::Malformed {
            program: ProgramSide::First,
            ..
        }
    ));
    let difference = compare::difference(&program, &broken).expect("programs must differ");
    assert!(matches!(
        difference.kind,
        DifferenceKind::Malformed {
            program: ProgramSide::Second,
            ..
        }
    ));
    assert!(!compare::equivalent(&broken, &program));
}

#[test]
fn member_access_spelling_binds_by_index() {
    use workshop_rs::{Action, Event, Program, Rule, Value, Variable};

    let program = |name: &str, member_access: bool| {
        let mut program = Program::default();
        program.global_variables.push(Variable::with_index("g", 0));
        program.player_variables.push(Variable::with_index(name, 0));
        let read = if member_access {
            Value::call("memberAccess", [Value::EventPlayer, Value::string(name)])
        } else {
            Value::player_variable(Value::EventPlayer, name)
        };
        program.rules.push(
            Rule::new("r", Event::Global).action(Action::SetGlobalVariable {
                variable: "g".to_string(),
                value: read,
            }),
        );
        program
    };
    let variable_read = program("p", false);
    let member_read = program("renamed", true);
    assert!(compare::equivalent(&variable_read, &member_read));
    assert!(compare::equivalent(&member_read, &variable_read));
    let mut variable_read = Program::default();
    variable_read
        .global_variables
        .push(Variable::with_index("g", 0));
    variable_read
        .player_variables
        .push(Variable::with_index("p", 0));
    variable_read
        .player_variables
        .push(Variable::with_index("q", 1));
    variable_read.rules.push(
        Rule::new("r", Event::Global).action(Action::SetGlobalVariable {
            variable: "g".to_string(),
            value: Value::player_variable(Value::EventPlayer, "p"),
        }),
    );
    let mut member_read = Program::default();
    member_read
        .global_variables
        .push(Variable::with_index("g", 0));
    member_read
        .player_variables
        .push(Variable::with_index("p", 0));
    member_read
        .player_variables
        .push(Variable::with_index("q", 1));
    member_read.rules.push(
        Rule::new("r", Event::Global).action(Action::SetGlobalVariable {
            variable: "g".to_string(),
            value: Value::call("memberAccess", [Value::EventPlayer, Value::string("q")]),
        }),
    );
    let difference = compare::difference(&variable_read, &member_read).expect("must differ");
    assert_eq!(
        difference.kind,
        DifferenceKind::Reference {
            table: DeclarationTable::PlayerVariables,
            first: 0,
            second: 1,
        }
    );
}

#[test]
fn strict_equivalence_implies_behavioral_equivalence() {
    for fixture in ["declarations-rules", "control-flow", "expressions-values"] {
        let source = common::corpus_text(fixture);
        let program = parse(&source);
        let emitted = emit(&program);
        let reparsed = parse(&emitted);
        assert!(
            roundtrip::equivalent(&program, &reparsed),
            "{fixture}: emit → reparse must stay structurally equivalent"
        );
        assert!(
            compare::equivalent(&program, &reparsed),
            "{fixture}: structurally equivalent programs must be behaviorally equivalent"
        );
    }
}
