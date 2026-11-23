use crate::common::{cases, catalog, en, locale, source, zh};
use workshop_rs::{Action, Program, Value, parser, source::Span};

const SOURCE: &str = r#"variables {
    global:
        0: cakePos
    player:
        1: playerScore
}

subroutines {
    0: tick
}

rule ("spans") {
    event { Ongoing - Global; }
    conditions {
        Add(Event Player.playerScore, Global.cakePos) > 0;
        Compare(Global.cakePos, ==, 0);
        disabled Global.cakePos > 0;
    }
    actions {
        Set Global Variable(cakePos, 1);
        Global.cakePos = Vector(1, 2, 3);
        Event Player.playerScore = 5;
        Global.cakePos += 1;
        Set Global Variable At Index(cakePos, 0, Add(1, 2));
        Global.cakePos[0] = 4;
        Call Subroutine(tick);
        Set Global Variable(cakePos,
            Add(Event Player.playerScore, Multiply(2, 3)));
        Abort;
        disabled Set Global Variable(cakePos, 0);
        If(Global.cakePos > 0);
            Abort;
        End;
    }
}
"#;

fn span_text(program: &Program, span: Span) -> &str {
    let document = program.source(span.file).expect("source file");
    &document.text()[document.byte_range(span).expect("span range")]
}

#[test]
fn condition_and_action_spans_cover_the_whole_statement() {
    let catalog = catalog();
    let program = parser::parse(SOURCE, &catalog, &en()).expect("parses");

    for (condition, expected) in [
        (0, "Add(Event Player.playerScore, Global.cakePos) > 0"),
        (1, "Compare(Global.cakePos, ==, 0)"),
        (2, "Global.cakePos > 0"),
    ] {
        let span = program.condition_span(0, condition).unwrap();
        assert_eq!(span_text(&program, span), expected, "condition {condition}");
        assert_eq!(program.condition_value_span(0, condition, &[]), Some(span));
    }

    for (action, expected) in [
        (0, "Set Global Variable(cakePos, 1)"),
        (1, "Global.cakePos = Vector(1, 2, 3)"),
        (2, "Event Player.playerScore = 5"),
        (3, "Global.cakePos += 1"),
        (4, "Set Global Variable At Index(cakePos, 0, Add(1, 2))"),
        (5, "Global.cakePos[0] = 4"),
        (6, "Call Subroutine(tick)"),
        (
            7,
            "Set Global Variable(cakePos,\n            Add(Event Player.playerScore, Multiply(2, 3)))",
        ),
        (8, "Abort"),
        (9, "Set Global Variable(cakePos, 0)"),
        (
            10,
            "If(Global.cakePos > 0);\n            Abort;\n        End",
        ),
        (11, "Abort"),
    ] {
        assert_eq!(
            span_text(&program, program.action_span(0, action).unwrap()),
            expected,
            "action {action}"
        );
    }
}

#[test]
fn nested_if_marker_spans_point_to_each_authored_keyword() {
    let input = r#"variables { player: 0: slot }
rule("nested markers") {
    event { Ongoing - Each Player; All; All; }
    actions {
        If(True); // outer if
            If(False); // inner if
                Abort;
            Else  If(True);
                Abort;
            Else; // inner else
                Abort;
            End; // inner end
        Else If(False);
            Abort;
        Else; // outer else
            Abort;
        End; // outer end
        For Player Variable(Event Player, slot, 0, 1, 1);
        End; // player loop end
    }
}"#;
    let program = parser::parse(input, &catalog(), &en()).unwrap();
    for (action, authored, keyword) in [
        (0, "If(True); // outer if", "If"),
        (1, "If(False); // inner if", "If"),
        (3, "Else  If(True)", "Else  If"),
        (5, "Else; // inner else", "Else"),
        (7, "End; // inner end", "End"),
        (8, "Else If(False)", "Else If"),
        (10, "Else; // outer else", "Else"),
        (12, "End; // outer end", "End"),
        (14, "End; // player loop end", "End"),
    ] {
        assert_marker(&program, 0, action, input.find(authored).unwrap(), keyword);
    }
    assert_eq!(program.action_marker_span(0, 2), None);
}

#[test]
fn marker_spans_keep_the_spelling_in_localized_source() {
    let catalog = catalog();
    for (locale, input, markers) in [
        (
            zh(),
            r#"规则("分支") { 事件 { 持续 - 全局; } 动作 { If(真); Else If(假); Else; End; } }"#,
            [
                ("If(真)", "If"),
                ("Else If(假)", "Else If"),
                ("Else;", "Else"),
                ("End;", "End"),
            ],
        ),
        (
            locale("tr-TR"),
            r#"kural("dallar") { etkinlik { Devam Eden - Küresel; } eylemler { Eğer(Doğru); Başka Eğer(Yanlış); Değilse; Son; } }"#,
            [
                ("Eğer(Doğru)", "Eğer"),
                ("Başka Eğer(Yanlış)", "Başka Eğer"),
                ("Değilse;", "Değilse"),
                ("Son;", "Son"),
            ],
        ),
    ] {
        let program = parser::parse(input, &catalog, &locale).unwrap();
        for (action, (authored, keyword)) in markers.into_iter().enumerate() {
            assert_marker(&program, 0, action, input.find(authored).unwrap(), keyword);
        }
    }
}

#[test]
fn pinned_illari_rule_marker_spans_follow_nested_branches_and_loops() {
    let case = cases().iter().find(|case| case.id == "illari").unwrap();
    let (input, locale) = source(case);
    let program = parser::parse(&input, &catalog(), &locale).unwrap();
    let rule = program
        .rules
        .iter()
        .position(|rule| rule.name == "Healthkit Cooldown")
        .unwrap();
    let rule_start = input.find("规则(\"Healthkit Cooldown\")").unwrap();
    for (action, authored, keyword) in [
        (6, "If(全局.Healthkit_CD[全局.Healthkit_Index] <= 0);", "If"),
        (
            7,
            "If(全局.Used_Healthkit_Type[全局.Healthkit_Index] == 1);",
            "If",
        ),
        (
            9,
            "Else If(全局.Used_Healthkit_Type[全局.Healthkit_Index] == 2);",
            "Else If",
        ),
        (11, "Else;\n\t\t\t\t\t\t修改全局变量(Healthkits,", "Else"),
        (13, "End;\n\t\t\t\t\t修改全局变量(Healthkit_Used,", "End"),
        (20, "End;\n\t\t\t等待(1, 无视条件);", "End"),
        (22, "End;\n\t}\n}", "End"),
    ] {
        let start = rule_start + input[rule_start..].find(authored).unwrap();
        assert_marker(&program, rule, action, start, keyword);
    }
}

#[test]
fn marker_spans_require_matching_authored_provenance() {
    let input = r#"rule("r") { event { Ongoing - Global; } actions { disabled If(True); } }"#;
    let mut program = parser::parse(input, &catalog(), &en()).unwrap();
    assert_marker(&program, 0, 0, input.find("If(True)").unwrap(), "If");
    assert!(matches!(program.rules[0].actions[1], Action::End));
    assert_eq!(program.action_marker_span(0, 1), None);
    assert_eq!(program.action_marker_span(0, 2), None);
    assert_eq!(program.action_marker_span(1, 0), None);

    let file = program.action_marker_span(0, 0).unwrap().file;
    let mut with_empty_source = program.clone();
    assert!(with_empty_source.set_file_source(file, ""));
    assert_eq!(with_empty_source.action_span(0, 0), None);
    assert_eq!(with_empty_source.action_marker_span(0, 0), None);

    program.rules[0].actions[0] = Action::disabled(Action::If {
        condition: Value::Bool(false),
    });
    assert_eq!(program.action_span(0, 0), None);
    assert_eq!(program.action_marker_span(0, 0), None);
    program.set_action_span(0, 0, None).unwrap();
    assert_eq!(program.action_marker_span(0, 0), None);
}

fn assert_marker(program: &Program, rule: usize, action: usize, start: usize, keyword: &str) {
    let span = program.action_marker_span(rule, action).unwrap_or_else(|| {
        panic!("missing marker span for rule {rule}, action {action} ({keyword})")
    });
    assert_eq!(span.file.index(), 0);
    let document = program.source(span.file).unwrap();
    let range = document.byte_range(span).unwrap();
    assert_eq!(range.start, start, "rule {rule}, action {action}");
    assert_eq!(
        &document.text()[range],
        keyword,
        "rule {rule}, action {action}"
    );
}
