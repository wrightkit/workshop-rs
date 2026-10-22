use crate::common::{catalog, en};
use workshop_rs::{Program, parser, source::Span};

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
    }
    actions {
        Set Global Variable(cakePos, 1);
        Global.cakePos = Vector(1, 2, 3);
        Event Player.playerScore = 5;
        Set Global Variable At Index(cakePos, 0, Add(1, 2));
        Global.cakePos[0] = 4;
        Call Subroutine(tick);
        Set Global Variable(cakePos, Add(Event Player.playerScore, Multiply(2, 3)));
        Abort;
        disabled Set Global Variable(cakePos, 0);
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
    ] {
        let span = program.condition_span(0, condition).unwrap();
        assert_eq!(span_text(&program, span), expected, "condition {condition}");
        assert_eq!(program.condition_value_span(0, condition, &[]), Some(span));
    }

    for (action, expected) in [
        (0, "Set Global Variable(cakePos, 1)"),
        (1, "Global.cakePos = Vector(1, 2, 3)"),
        (2, "Event Player.playerScore = 5"),
        (3, "Set Global Variable At Index(cakePos, 0, Add(1, 2))"),
        (4, "Global.cakePos[0] = 4"),
        (5, "Call Subroutine(tick)"),
        (
            6,
            "Set Global Variable(cakePos, Add(Event Player.playerScore, Multiply(2, 3)))",
        ),
        (7, "Abort"),
        (8, "Set Global Variable(cakePos, 0)"),
    ] {
        assert_eq!(
            span_text(&program, program.action_span(0, action).unwrap()),
            expected,
            "action {action}"
        );
    }
}
