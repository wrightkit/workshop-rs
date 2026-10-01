use crate::common::en;
use workshop_rs::WorkshopError;
use workshop_rs::catalog::Catalog;
use workshop_rs::parser;

#[test]
fn malformed_number_literals_are_reported_by_the_public_parser() {
    let catalog = Catalog::builtin().expect("catalog");
    let locale = en();

    for source in ["0X", "1.2.3"] {
        let error = parser::parse(source, &catalog, &locale).expect_err("invalid number");
        let WorkshopError::Malformed { message, span } = error else {
            panic!("invalid number should be a malformed Workshop diagnostic");
        };
        assert_eq!(message, format!("invalid number '{source}'"));
        assert_eq!(span.expect("source span").start.col, 1);
    }
}

fn parse_error(source: &str) -> WorkshopError {
    let catalog = Catalog::builtin().expect("catalog");
    parser::parse(source, &catalog, &en()).expect_err("source must not parse")
}

fn program_with_condition(condition: &str) -> String {
    format!(
        "rule (\"x\") {{\n    event {{\n        Ongoing - Global;\n    }}\n    conditions {{\n        {condition}\n    }}\n}}\n"
    )
}

#[test]
fn an_action_call_used_as_a_condition_reports_the_action_at_its_call() {
    let error = parse_error(&program_with_condition("Wait(1, Ignore Condition);"));
    let WorkshopError::Malformed { message, span } = error else {
        panic!("an action used as a condition is a malformed diagnostic");
    };
    assert_eq!(message, "action 'Wait' cannot be used as a condition");
    let span = span.expect("the misused action call carries a span");
    // `Wait` starts at column 9 of the conditions entry and the diagnostic
    // covers the whole `Wait(1, Ignore Condition)` call.
    assert_eq!((span.start.line, span.start.col), (6, 9));
    assert_eq!((span.end.line, span.end.col), (6, 34));
}

#[test]
fn action_statements_used_as_conditions_name_the_construct() {
    // `Set Global Variable` is a structural action spelling rather than a
    // catalog action; it still cannot be a condition.
    let error = parse_error(&program_with_condition("Set Global Variable(x, 1);"));
    let WorkshopError::Malformed { message, .. } = error else {
        panic!("an action used as a condition is a malformed diagnostic");
    };
    assert_eq!(
        message,
        "action 'Set Global Variable' cannot be used as a condition"
    );

    // Nested inside another expression the same construct reports a value
    // position instead.
    let error = parse_error(&program_with_condition(
        "Compare(Wait(1, Ignore Condition), ==, 1);",
    ));
    let WorkshopError::Malformed { message, .. } = error else {
        panic!("an action inside a condition argument is malformed");
    };
    assert_eq!(message, "action 'Wait' cannot be used as a value");

    // An infix operand is likewise a value slot inside the condition.
    let error = parse_error(&program_with_condition("x == Wait(1, Ignore Condition);"));
    let WorkshopError::Malformed { message, .. } = error else {
        panic!("an action operand inside a condition is malformed");
    };
    assert_eq!(message, "action 'Wait' cannot be used as a value");
}

#[test]
fn an_enum_call_sharing_an_action_name_still_parses_as_a_condition() {
    // `Wait` is both the Wait action and the Wait enum domain; a well-formed
    // enum member call keeps its value reading.
    let catalog = Catalog::builtin().expect("catalog");
    parser::parse(
        &program_with_condition("Wait(Ignore Condition);"),
        &catalog,
        &en(),
    )
    .expect("a single-member enum call is a valid condition");
}

#[test]
fn a_missing_statement_semicolon_reports_the_end_of_the_statement() {
    let source = concat!(
        "rule (\"x\") {\n",
        "    event {\n",
        "        Ongoing - Global;\n",
        "    }\n",
        "    actions {\n",
        "        Set Global Variable(x, 1)\n",
        "    }\n",
        "}\n",
    );
    let error = parse_error(source);
    let WorkshopError::Malformed { message, span } = error else {
        panic!("a missing semicolon is a malformed diagnostic");
    };
    assert_eq!(message, "expected ';'");
    // The `}` closing `actions` sits on line 7; the missing `;` belongs at
    // the end of the `Set Global Variable(x, 1)` statement on line 6.
    let span = span.expect("source span");
    assert_eq!(span.start.line, 6);
    assert_eq!(span.end.line, 6);
}

#[test]
fn a_missing_semicolon_before_a_same_line_token_still_names_that_token() {
    let source = "rule (\"x\") { event { Ongoing - Global; } actions { Set Global Variable(x, 1) Set Global Variable(x, 2); } }\n";
    let error = parse_error(source);
    let WorkshopError::Malformed { message, span } = error else {
        panic!("a missing semicolon is a malformed diagnostic");
    };
    assert_eq!(message, "expected ';'");
    let span = span.expect("source span");
    let offending = source.find("Set Global Variable(x, 2)").unwrap() as u32 + 1;
    assert_eq!(span.start.col, offending);
}

#[test]
fn a_missing_rule_paren_keeps_its_existing_diagnostic() {
    let error = parse_error("rule \"no parens\" {\n}\n");
    let WorkshopError::Malformed { message, span } = error else {
        panic!("a missing rule paren is a malformed diagnostic");
    };
    assert_eq!(message, "expected '(' after 'rule'");
    let span = span.expect("source span");
    assert_eq!((span.start.line, span.start.col), (1, 6));
}
