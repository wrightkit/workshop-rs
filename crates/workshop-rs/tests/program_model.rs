use workshop_rs::{Action, Condition, Event, Program, Rule, Value, Variable};

#[test]
fn program_is_constructible_without_storage_ids() {
    let condition = Condition::new(Value::call("isAlive", [Value::global_variable("Target")]));
    let rule = Rule::new("Linear control flow", Event::Global)
        .condition(condition.clone())
        .action(Action::If {
            condition: condition.clone(),
        })
        .action(Action::call("Wait", [Value::number(1.0)]))
        .action(Action::ElseIf {
            condition: Condition::new(Value::Bool(false)),
        })
        .action(Action::Else)
        .action(Action::While { condition })
        .action(Action::disabled(Action::call("Abort", std::iter::empty())))
        .action(Action::End);

    let mut program = Program::new();
    program.global_variable(Variable::new("Target")).rule(rule);

    assert_eq!(program.global_variables[0].name, "Target");
    assert!(matches!(program.rules[0].actions[0], Action::If { .. }));
    assert!(matches!(program.rules[0].actions[2], Action::ElseIf { .. }));
    assert!(matches!(program.rules[0].actions[3], Action::Else));
    assert!(matches!(
        program.rules[0].actions[5],
        Action::Disabled { .. }
    ));
    assert!(matches!(program.rules[0].actions[6], Action::End));
}

#[test]
fn values_and_conditions_are_composable() {
    let value = Value::call(
        "add",
        [
            Value::global_variable("Score"),
            Value::Array(vec![Value::number(1.0), Value::number(2.0)]),
        ],
    );
    let condition = Condition::new(value.clone());
    let mut program = Program::default();
    program.rule(Rule::new("Composable", Event::Global).condition(condition));

    assert!(matches!(
        &program.rules[0].conditions[0].value,
        Value::Call { name, args } if name == "add" && args.len() == 2
    ));
}
