//! Executable witnesses for contextual Workshop literal semantics.

use workshop_rs::catalog::{Catalog, Locale};
use workshop_rs::parser;
use workshop_rs::roundtrip;
use workshop_rs::validate;
use workshop_rs::wir::{self, Action, Value};

fn catalog() -> Catalog {
    Catalog::builtin().expect("built-in catalog")
}

fn program_with(conditions: &str, actions: &str) -> wir::Program {
    let source = format!(
        r#"variables
{{
    global:
        0: probe
}}
rule ("contextual semantics")
{{
    event {{ Ongoing - Global; }}
    conditions {{
        {conditions}
    }}
    actions {{
        {actions}
    }}
}}"#
    );
    parser::parse(&source, &catalog(), &Locale::new("en-US")).expect("contextual source parses")
}

fn program(actions: &str) -> wir::Program {
    program_with("", actions)
}

fn first_action_args(program: &wir::Program) -> &[wir::ValueId] {
    let rule = program.rules.iter().next().expect("rule");
    match program.actions.get(rule.actions[0]).expect("action") {
        Action::Call { args, .. } => args,
        other => panic!("expected a catalog action call, got {other:?}"),
    }
}

fn validate_program(program: &wir::Program) {
    validate::validate_canonical_ids(program, &catalog()).expect("canonical validation");
}

fn first_value_call<'a>(program: &'a wir::Program, name: &str) -> &'a [wir::ValueId] {
    program
        .values
        .iter()
        .find_map(|node| match &node.value {
            Value::Call {
                name: value_name,
                args,
            } if value_name == name => Some(args.as_slice()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("missing value call {name}"))
}

#[test]
fn numeric_boolean_aliases_share_canonical_wir_only_when_declared() {
    for (boolean, number, expected) in [("True", "1", 1.0), ("False", "0", 0.0)] {
        let boolean_program = program(&format!("Wait({boolean}, Ignore Condition);"));
        let number_program = program(&format!("Wait({number}, Ignore Condition);"));
        validate_program(&boolean_program);
        validate_program(&number_program);
        assert!(roundtrip::equivalent(&boolean_program, &number_program));
        assert!(matches!(
            &boolean_program
                .values
                .get(first_action_args(&boolean_program)[0])
                .expect("wait duration")
                .value,
            Value::Number { value, .. } if *value == expected
        ));
    }
}

#[test]
fn one_sided_contextual_aliases_reject_the_other_boolean() {
    let accepted = program("Start Forcing Spawn Room(Team 1, False);");
    validate_program(&accepted);
    assert!(matches!(
        &accepted
            .values
            .get(first_action_args(&accepted)[1])
            .expect("spawn room")
            .value,
        Value::Number { value, .. } if *value == 0.0
    ));

    let rejected = program("Start Forcing Spawn Room(Team 1, True);");
    let error = validate::validate_canonical_ids(&rejected, &catalog())
        .expect_err("True is not a documented alias for the spawn-room parameter");
    assert!(format!("{error:?}").contains("semantic type"));
}

#[test]
fn wait_until_keeps_numeric_exception_without_global_truthiness() {
    let numeric = program("Wait Until(1, 2);");
    validate_program(&numeric);
    assert!(matches!(
        &numeric
            .values
            .get(first_action_args(&numeric)[0])
            .expect("continue condition")
            .value,
        Value::Number { value, .. } if *value == 1.0
    ));

    let vector = program("Wait Until(Vector(1, 2, 3), 2);");
    let error = validate::validate_canonical_ids(&vector, &catalog())
        .expect_err("Wait Until must not accept arbitrary values as conditions");
    assert!(format!("{error:?}").contains("semantic type"));
}

#[test]
fn null_and_empty_string_contexts_normalize_at_their_positions() {
    let teleport = program("Teleport(Event Player, 0);");
    validate_program(&teleport);
    assert!(matches!(
        &teleport
            .values
            .get(first_action_args(&teleport)[1])
            .expect("teleport position")
            .value,
        Value::Null
    ));

    let position = program("Start Forcing Player Position(Event Player, 0, False);");
    validate_program(&position);
    assert!(matches!(
        &position
            .values
            .get(first_action_args(&position)[1])
            .expect("forced position")
            .value,
        Value::Null
    ));

    let dummy =
        program("Create Dummy Bot(Hero(Bastion), Team 2, 0, Vector(0, 0, 0), Vector(0, 0, 0));");
    validate_program(&dummy);
    let dummy_args = first_action_args(&dummy);
    assert!(matches!(
        &dummy
            .values
            .get(dummy_args[3])
            .expect("dummy position")
            .value,
        Value::Null
    ));
    assert!(matches!(
        &dummy
            .values
            .get(dummy_args[4])
            .expect("dummy direction")
            .value,
        Value::Null
    ));

    let name = program("Start Forcing Dummy Bot Name(Event Player, Empty Array);");
    validate_program(&name);
    assert!(matches!(
        &name.values
            .get(first_action_args(&name)[1])
            .expect("forced name")
            .value,
        Value::String(value) if value.is_empty()
    ));

    let string_source = r#"variables
{
    global:
        0: probe
}
rule ("empty string")
{
    event { Ongoing - Global; }
    actions { Set Global Variable(probe, String Replace(Empty Array, Empty Array, Empty Array)); }
}"#;
    let reparsed = roundtrip::round_trip_with_context(
        string_source,
        &catalog(),
        &Locale::new("en-US"),
        &catalog(),
    );
    assert!(
        reparsed.equivalent,
        "empty string contextual alias must round-trip"
    );
}

#[test]
fn nested_numeric_boolean_aliases_normalize_inside_vector_components() {
    let program = program("Set Global Variable(probe, Vector(1, True, False));");
    validate_program(&program);
    let args = first_value_call(&program, "vector");
    assert_eq!(args.len(), 3);
    for (value_id, expected) in [(args[0], 1.0), (args[1], 1.0), (args[2], 0.0)] {
        assert!(matches!(
            &program.values.get(value_id).expect("vector component").value,
            Value::Number { value, .. } if *value == expected
        ));
    }
}

#[test]
fn comparisons_preserve_polymorphic_operand_types() {
    let program = program_with("1 == True;", "Wait(1, Ignore Condition);");
    validate_program(&program);
    let condition = program
        .values
        .get(program.rules.iter().next().expect("rule").conditions[0])
        .expect("comparison");
    let Value::Call { name, args } = &condition.value else {
        panic!("expected comparison call, got {:?}", condition.value);
    };
    assert_eq!(name, "==");
    assert!(matches!(
        &program.values.get(args[0]).expect("left operand").value,
        Value::Number { value, .. } if *value == 1.0
    ));
    assert!(matches!(
        &program.values.get(args[1]).expect("right operand").value,
        Value::Bool(true)
    ));
}

#[test]
fn arithmetic_operators_apply_contextual_aliases_but_comparisons_do_not() {
    let program = program("Set Global Variable(probe, True + 1);");
    validate_program(&program);
    let args = first_value_call(&program, "add");
    assert!(matches!(
        &program.values.get(args[0]).expect("left operand").value,
        Value::Number { value, .. } if *value == 1.0
    ));
}

#[test]
fn indexed_player_variable_sugar_uses_canonical_parameter_positions() {
    let source = r#"variables
{
    global:
        0: probe
    player:
        0: indexed
}

rule ("indexed contextual semantics")
{
    event { Ongoing - Global; }
    actions { Set Player Variable At Index(Event Player, indexed, True, Null); }
}"#;
    let parsed =
        parser::parse(source, &catalog(), &Locale::new("en-US")).expect("indexed source parses");
    validate::validate_canonical_ids(&parsed, &catalog()).expect("canonical validation");
    let action = parsed
        .actions
        .get(parsed.rules.iter().next().expect("rule").actions[0])
        .expect("indexed action");
    let Action::Call { args, .. } = action else {
        panic!("expected indexed action call, got {action:?}");
    };
    assert!(matches!(
        &parsed.values.get(args[1]).expect("index").value,
        Value::Number { value, .. } if *value == 1.0
    ));
}

#[test]
fn conditional_operator_applies_branch_contextual_aliases() {
    let parsed = program("Set Global Variable(probe, True ? 0 : 1);");
    validate_program(&parsed);
    let args = first_value_call(&parsed, "ifThenElse");
    assert!(matches!(
        &parsed.values.get(args[1]).expect("true branch").value,
        Value::Null
    ));
}

#[test]
fn audited_catalog_parameters_cover_boolean_numeric_aliases() {
    for (source, expected) in [
        ("Set Gravity(Event Player, True);", 1.0),
        ("Set Ultimate Charge(Event Player, False);", 0.0),
        ("Set Team Score(Team 1, True);", 1.0),
    ] {
        let parsed = program(source);
        validate_program(&parsed);
        assert!(matches!(
            &parsed
                .values
                .get(first_action_args(&parsed)[1])
                .expect("numeric parameter")
                .value,
            Value::Number { value, .. } if *value == expected
        ));
    }

    let parsed = program("Set Global Variable(probe, Is Objective Complete(True));");
    validate_program(&parsed);
    let args = first_value_call(&parsed, "isObjectiveComplete");
    assert!(matches!(
        &parsed.values.get(args[0]).expect("objective index").value,
        Value::Number { value, .. } if *value == 1.0
    ));
}

#[test]
fn array_index_sugar_applies_value_in_array_context() {
    let parsed = program("Set Global Variable(probe, Array(1)[True]);");
    validate_program(&parsed);
    let args = first_value_call(&parsed, "valueInArray");
    assert!(matches!(
        &parsed.values.get(args[1]).expect("array index").value,
        Value::Number { value, .. } if *value == 1.0
    ));
}

#[test]
fn modify_contexts_apply_operation_specific_replacements() {
    let source = r#"variables
{
    global:
        0: g
    player:
        0: p
}
rule ("modify contextual semantics")
{
    event { Ongoing - Global; }
    actions {
        Modify Global Variable(g, Add, False);
        Modify Global Variable(g, Append To Array, 0);
        Modify Player Variable(Event Player, p, Subtract, True);
        Modify Global Variable At Index(g, 0, Append To Array, 0);
        Global.g[False] += 1;
    }
}"#;
    let parsed =
        parser::parse(source, &catalog(), &Locale::new("en-US")).expect("modify source parses");
    validate_program(&parsed);

    let direct_values: Vec<_> = parsed
        .actions
        .iter()
        .filter_map(|action| match action {
            Action::ModifyGlobalVariable { value, .. }
            | Action::ModifyPlayerVariable { value, .. } => Some(value),
            _ => None,
        })
        .collect();
    assert!(matches!(
        &parsed.values.get(*direct_values[0]).expect("add value").value,
        Value::Number { value, .. } if *value == 0.0
    ));
    assert!(matches!(
        &parsed
            .values
            .get(*direct_values[1])
            .expect("append value")
            .value,
        Value::Null
    ));
    assert!(matches!(
        &parsed.values.get(*direct_values[2]).expect("subtract value").value,
        Value::Number { value, .. } if *value == 1.0
    ));

    let indexed: Vec<_> = parsed
        .actions
        .iter()
        .filter_map(|action| match action {
            Action::Call { name, args, .. } if name == "modifyGlobalVariableAtIndex" => Some(args),
            _ => None,
        })
        .collect();
    assert_eq!(indexed.len(), 2);
    assert!(matches!(
        &parsed
            .values
            .get(indexed[0][3])
            .expect("indexed modify value")
            .value,
        Value::Null
    ));
    assert!(matches!(
        &parsed
            .values
            .get(indexed[1][1])
            .expect("indexed assignment index")
            .value,
        Value::Number { value, .. } if *value == 0.0
    ));
}
