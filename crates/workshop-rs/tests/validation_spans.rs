use crate::common::catalog;
use workshop_rs::source::{Position, SourceFile, Span};
use workshop_rs::{Action, Event, Program, Rule, Value, Variable, WorkshopError};

fn rejected_slice_program() -> (Program, Span, Span) {
    let mut program = Program::new();
    let vector = Value::call(
        "vector",
        [Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)],
    );
    let slice = Value::call(
        "stringSlice",
        [vector, Value::Number(1.0), Value::Number(2.0)],
    );
    program
        .global_variable(Variable::new("s"))
        .rule(
            Rule::new("slice", Event::Global).action(Action::SetGlobalVariable {
                variable: "s".to_string(),
                value: slice,
            }),
        );
    let file = program.add_file(SourceFile::new("main.opy"));
    let rule_span = Span::new(file, Position::new(1, 1), Position::new(2, 40));
    let call_span = Span::new(file, Position::new(2, 9), Position::new(2, 40));
    program.set_rule_span(0, Some(rule_span)).unwrap();
    (program, rule_span, call_span)
}

fn rejected_span(program: &Program) -> Option<Span> {
    let error = workshop_rs::validate::validate_canonical_ids(program, &catalog())
        .expect_err("a Vector is not a String Slice string");
    assert!(
        matches!(error, WorkshopError::Unsupported { .. }),
        "{error:?}"
    );
    error.span()
}

#[test]
fn an_unmapped_argument_reports_its_nearest_mapped_ancestor() {
    let (mut program, _, call_span) = rejected_slice_program();
    program
        .set_action_argument_span(0, 0, 0, Some(call_span))
        .unwrap();
    assert_eq!(rejected_span(&program), Some(call_span));
}

#[test]
fn an_unmapped_action_reports_its_rule() {
    let (program, rule_span, _) = rejected_slice_program();
    assert_eq!(rejected_span(&program), Some(rule_span));
}
