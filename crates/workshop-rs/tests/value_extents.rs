use crate::common::{catalog, en, zh};
use workshop_rs::{Program, parser, source::Span};

const INDEX_SOURCE: &str = r#"variables {
    global:
        0: arr
        1: g
    player:
        0: p
}

rule ("extents") {
    event { Ongoing - Global; }
    conditions {
        Global.arr[0] == 1;
        Global.arr[Global.arr[0]] == 1;
        Add(1, 2)[0] == 1;
        Global.arr[0] + Global.arr[1] > 0;
        Global.g * (2 + 3) > 0;
        (Global.g + 1) * (2 + 3) > 0;
        Global.g == Multiply((1 + 2), 3);
        Global.g == (1 + 2) * 3;
        (Global.g).member == 3;
        (Event Player.p).member == 3;
    }
    actions {
        Set Global Variable(g, Vector(1, 2, 3)[0]);
    }
}
"#;

const SOURCE: &str = r#"variables {
    global:
        0: g
    player:
        0: p
}

rule ("extents") {
    event { Ongoing - Global; }
    conditions {
        1 == 1;
        Add(1, 2) > 0;
        True;
        Add(Multiply(2, 3), 4) > 0;
        Global.g + Event Player.p > 0;
        !Global.g;
        Count Of(Array(1, 2)) > 0;
        Distance Between(Vector(1, 2, 3), Vector(4, 5, 6)) > 0;
        Event Player.p == 5;
        Global.g == 1;
        (Global.g > 0 ? 1 : 2) == 1;
    }
    actions {
        Set Global Variable(g, Add(1, 2));
        Set Global Variable(g, Vector(1, 2, 3));
        Set Global Variable(g, Global.g * 2);
        Set Global Variable(g, Event Player.p);
    }
}
"#;

fn span_text(program: &Program, span: Span) -> &str {
    let document = program.source(span.file).expect("source file");
    &document.text()[document.byte_range(span).expect("span range")]
}

fn condition_extent(program: &Program, condition: usize, path: &[usize]) -> String {
    let span = program
        .condition_value_extent(0, condition, path)
        .unwrap_or_else(|| panic!("condition {condition} path {path:?} has no extent"));
    span_text(program, span).to_string()
}

fn argument_extent(program: &Program, action: usize, argument: usize, path: &[usize]) -> String {
    let span = program
        .action_argument_value_extent(0, action, argument, path)
        .unwrap_or_else(|| {
            panic!("action {action} argument {argument} path {path:?} has no extent")
        });
    span_text(program, span).to_string()
}

#[test]
fn condition_value_extents_cover_the_authored_subexpression() {
    let catalog = catalog();
    let program = parser::parse(SOURCE, &catalog, &en()).expect("parses");

    // literal
    assert_eq!(condition_extent(&program, 0, &[]), "1 == 1");
    assert_eq!(condition_extent(&program, 0, &[0]), "1");
    assert_eq!(condition_extent(&program, 0, &[1]), "1");
    // call with arguments
    assert_eq!(condition_extent(&program, 1, &[]), "Add(1, 2) > 0");
    assert_eq!(condition_extent(&program, 1, &[0]), "Add(1, 2)");
    assert_eq!(condition_extent(&program, 1, &[0, 0]), "1");
    assert_eq!(condition_extent(&program, 1, &[0, 1]), "2");
    assert_eq!(condition_extent(&program, 1, &[1]), "0");
    // call without arguments
    assert_eq!(condition_extent(&program, 2, &[]), "True");
    // nested call
    assert_eq!(
        condition_extent(&program, 3, &[0]),
        "Add(Multiply(2, 3), 4)"
    );
    assert_eq!(condition_extent(&program, 3, &[0, 0]), "Multiply(2, 3)");
    assert_eq!(condition_extent(&program, 3, &[0, 0, 0]), "2");
    assert_eq!(condition_extent(&program, 3, &[0, 0, 1]), "3");
    assert_eq!(condition_extent(&program, 3, &[0, 1]), "4");
    // infix binary
    assert_eq!(
        condition_extent(&program, 4, &[0]),
        "Global.g + Event Player.p"
    );
    assert_eq!(condition_extent(&program, 4, &[0, 0]), "Global.g");
    assert_eq!(condition_extent(&program, 4, &[0, 1]), "Event Player.p");
    assert_eq!(condition_extent(&program, 4, &[0, 1, 0]), "Event Player");
    // infix unary
    assert_eq!(condition_extent(&program, 5, &[]), "!Global.g");
    assert_eq!(condition_extent(&program, 5, &[0]), "Global.g");
    // array inside a call
    assert_eq!(condition_extent(&program, 6, &[0]), "Count Of(Array(1, 2))");
    assert_eq!(condition_extent(&program, 6, &[0, 0]), "Array(1, 2)");
    assert_eq!(condition_extent(&program, 6, &[0, 0, 0]), "1");
    assert_eq!(condition_extent(&program, 6, &[0, 0, 1]), "2");
    // vectors
    assert_eq!(
        condition_extent(&program, 7, &[]),
        "Distance Between(Vector(1, 2, 3), Vector(4, 5, 6)) > 0"
    );
    assert_eq!(
        condition_extent(&program, 7, &[0]),
        "Distance Between(Vector(1, 2, 3), Vector(4, 5, 6))"
    );
    assert_eq!(condition_extent(&program, 7, &[0, 0]), "Vector(1, 2, 3)");
    assert_eq!(condition_extent(&program, 7, &[0, 0, 0]), "1");
    assert_eq!(condition_extent(&program, 7, &[0, 0, 2]), "3");
    assert_eq!(condition_extent(&program, 7, &[0, 1]), "Vector(4, 5, 6)");
    // player-variable read
    assert_eq!(condition_extent(&program, 8, &[0]), "Event Player.p");
    // Global.name
    assert_eq!(condition_extent(&program, 9, &[0]), "Global.g");
    // conditional `? :` inside a parenthesized infix operand
    assert_eq!(
        condition_extent(&program, 10, &[]),
        "(Global.g > 0 ? 1 : 2) == 1"
    );
    assert_eq!(
        condition_extent(&program, 10, &[0]),
        "(Global.g > 0 ? 1 : 2)"
    );
    assert_eq!(condition_extent(&program, 10, &[0, 0]), "Global.g > 0");
    assert_eq!(condition_extent(&program, 10, &[0, 1]), "1");
    assert_eq!(condition_extent(&program, 10, &[0, 2]), "2");
}

#[test]
fn value_extents_cover_array_indexing_and_parenthesized_operands() {
    let catalog = catalog();
    let program = parser::parse(INDEX_SOURCE, &catalog, &en()).expect("parses");

    // `receiver[index]` extents start at the receiver, not the `[`.
    assert_eq!(condition_extent(&program, 0, &[0]), "Global.arr[0]");
    assert_eq!(
        condition_extent(&program, 1, &[0]),
        "Global.arr[Global.arr[0]]"
    );
    assert_eq!(condition_extent(&program, 1, &[0, 1]), "Global.arr[0]");
    assert_eq!(condition_extent(&program, 2, &[0]), "Add(1, 2)[0]");
    // The postfix `[` binds looser than infix `+`, so the add is the
    // receiver: `valueInArray(Global.arr[0] + Global.arr, 1)`.
    assert_eq!(
        condition_extent(&program, 3, &[0]),
        "Global.arr[0] + Global.arr[1]"
    );
    assert_eq!(
        condition_extent(&program, 3, &[0, 0]),
        "Global.arr[0] + Global.arr"
    );
    assert_eq!(condition_extent(&program, 3, &[0, 0, 0]), "Global.arr[0]");
    assert_eq!(condition_extent(&program, 3, &[0, 0, 1]), "Global.arr");
    assert_eq!(condition_extent(&program, 3, &[0, 1]), "1");
    // Parenthesized operands keep their enclosing parens.
    assert_eq!(condition_extent(&program, 4, &[0]), "Global.g * (2 + 3)");
    assert_eq!(condition_extent(&program, 4, &[0, 1]), "(2 + 3)");
    assert_eq!(condition_extent(&program, 4, &[0, 1, 0]), "2");
    assert_eq!(condition_extent(&program, 4, &[0, 1, 1]), "3");
    assert_eq!(
        condition_extent(&program, 5, &[0]),
        "(Global.g + 1) * (2 + 3)"
    );
    assert_eq!(condition_extent(&program, 5, &[0, 0]), "(Global.g + 1)");
    assert_eq!(condition_extent(&program, 5, &[0, 0, 0]), "Global.g");
    // authored parens around a call argument or infix operand
    assert_eq!(condition_extent(&program, 6, &[0]), "Global.g");
    assert_eq!(condition_extent(&program, 6, &[1]), "Multiply((1 + 2), 3)");
    assert_eq!(condition_extent(&program, 6, &[1, 0]), "(1 + 2)");
    // `Global.g == (1 + 2) * 3` parses as `(Global.g == (1 + 2)) * 3`:
    // the `==` extent ends at the authored `)` instead of mid-operand.
    assert_eq!(condition_extent(&program, 7, &[0]), "Global.g == (1 + 2)");
    assert_eq!(condition_extent(&program, 7, &[0, 1]), "(1 + 2)");
    assert_eq!(condition_extent(&program, 7, &[1]), "3");
    // `(expr).member`
    assert_eq!(condition_extent(&program, 8, &[0]), "(Global.g).member");
    assert_eq!(
        condition_extent(&program, 9, &[0]),
        "(Event Player.p).member"
    );
    assert_eq!(condition_extent(&program, 9, &[0, 0]), "(Event Player.p)");
    // an indexed whole expression used as an action argument
    assert_eq!(argument_extent(&program, 0, 0, &[]), "Vector(1, 2, 3)[0]");
    assert_eq!(argument_extent(&program, 0, 0, &[0]), "Vector(1, 2, 3)");
    assert_eq!(argument_extent(&program, 0, 0, &[1]), "0");
}

#[test]
fn action_argument_value_extents_cover_the_authored_subexpression() {
    let catalog = catalog();
    let program = parser::parse(SOURCE, &catalog, &en()).expect("parses");

    assert_eq!(argument_extent(&program, 0, 0, &[]), "Add(1, 2)");
    assert_eq!(argument_extent(&program, 0, 0, &[0]), "1");
    assert_eq!(argument_extent(&program, 1, 0, &[]), "Vector(1, 2, 3)");
    assert_eq!(argument_extent(&program, 1, 0, &[1]), "2");
    assert_eq!(argument_extent(&program, 2, 0, &[]), "Global.g * 2");
    assert_eq!(argument_extent(&program, 2, 0, &[0]), "Global.g");
    assert_eq!(argument_extent(&program, 2, 0, &[1]), "2");
    assert_eq!(argument_extent(&program, 3, 0, &[]), "Event Player.p");
    assert_eq!(argument_extent(&program, 3, 0, &[0]), "Event Player");
}

#[test]
fn value_extents_cover_comments_inside_the_expression() {
    let source = r#"variables {
    global:
        0: g
}

rule ("comments") {
    event { Ongoing - Global; }
    conditions {
        Add(1, // left
            2) > 0;
        Global.g + // tail
            2 > 0;
    }
    actions {
        Set Global Variable(g, Add(1, // arg
            2));
    }
}
"#;
    let catalog = catalog();
    let program = parser::parse(source, &catalog, &en()).expect("parses");

    assert_eq!(
        condition_extent(&program, 0, &[]),
        "Add(1, // left\n            2) > 0"
    );
    assert_eq!(
        condition_extent(&program, 0, &[0]),
        "Add(1, // left\n            2)"
    );
    assert_eq!(condition_extent(&program, 0, &[0, 0]), "1");
    assert_eq!(condition_extent(&program, 0, &[0, 1]), "2");
    assert_eq!(
        condition_extent(&program, 1, &[0]),
        "Global.g + // tail\n            2"
    );
    assert_eq!(
        argument_extent(&program, 0, 0, &[]),
        "Add(1, // arg\n            2)"
    );
    assert_eq!(argument_extent(&program, 0, 0, &[1]), "2");
}

#[test]
fn value_extents_cover_non_english_locale_source() {
    let source = "变量\n{\n    全局:\n        0: g\n}\n规则(\"r\")\n{\n    事件\n    {\n        持续 - 全局;\n    }\n    条件\n    {\n        全局.g + 2 > 0;\n    }\n    动作\n    {\n        设置全局变量(g, 相加(1, 2));\n    }\n}\n";
    let catalog = catalog();
    let program = parser::parse(source, &catalog, &zh()).expect("parses");

    assert_eq!(condition_extent(&program, 0, &[0]), "全局.g + 2");
    assert_eq!(condition_extent(&program, 0, &[0, 0]), "全局.g");
    assert_eq!(condition_extent(&program, 0, &[0, 1]), "2");
    assert_eq!(argument_extent(&program, 0, 0, &[]), "相加(1, 2)");
    assert_eq!(argument_extent(&program, 0, 0, &[0]), "1");
    assert_eq!(argument_extent(&program, 0, 0, &[1]), "2");
}
