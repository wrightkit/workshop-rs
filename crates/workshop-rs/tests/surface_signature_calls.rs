mod common;

use workshop_rs::catalog::{Catalog, Kind};
use workshop_rs::program::{Action, Program, Value};
use workshop_rs::{ModifyOp, WorkshopError, emitter, parser, roundtrip, validate};

use crate::common::{catalog, en};

const DIVERGENT_SURFACE_CALLS: &[(&str, &str, usize, &str, usize)] = &[
    (
        "setPlayerVariableAtIndex",
        "Set Player Variable At Index(Event Player, probe_p, 0, 5);",
        4,
        "setPlayerVariableAtIndex",
        3,
    ),
    (
        "modifyPlayerVariableAtIndex",
        "Modify Player Variable At Index(Event Player, probe_p, 0, Add, 5);",
        5,
        "modifyPlayerVariableAtIndex",
        4,
    ),
    (
        "stopChasingPlayerVariable",
        "Stop Chasing Player Variable(Event Player, probe_p);",
        2,
        "stopChasingPlayerVariable",
        1,
    ),
    (
        "chasePlayerVariableAtRate",
        "Chase Player Variable At Rate(Event Player, probe_p, 0, 1, None);",
        5,
        "chasePlayerVariableAtRate",
        4,
    ),
    (
        "chasePlayerVariableOverTime",
        "Chase Player Variable Over Time(Event Player, probe_p, 0, 1, None);",
        5,
        "chasePlayerVariableOverTime",
        4,
    ),
];

const OMITTED_PLAYER_CALLS: &[(&str, &str)] = &[
    (
        "setPlayerVariableAtIndex",
        "Set Player Variable At Index(probe_p, 0, 5);",
    ),
    (
        "modifyPlayerVariableAtIndex",
        "Modify Player Variable At Index(probe_p, 0, Add, 5);",
    ),
    (
        "stopChasingPlayerVariable",
        "Stop Chasing Player Variable(probe_p);",
    ),
    (
        "chasePlayerVariableAtRate",
        "Chase Player Variable At Rate(probe_p, 0, 1, None);",
    ),
    (
        "chasePlayerVariableOverTime",
        "Chase Player Variable Over Time(probe_p, 0, 1, None);",
    ),
];

fn source(action: &str) -> String {
    format!(
        "variables {{\n    player:\n        0: probe_p\n}}\n\nrule (\"probe\") {{\n    event {{\n        Ongoing - Each Player;\n    }}\n    actions {{\n        {action}\n    }}\n}}\n"
    )
}

fn parse_and_validate(source: &str, catalog: &Catalog) -> Result<Program, WorkshopError> {
    parser::parse(source, catalog, &en())
        .and_then(|program| validate::validate_canonical_ids(&program, catalog).map(|()| program))
}

#[test]
fn every_fold_divergent_action_has_hand_written_surface_coverage() {
    let catalog = catalog();
    let mut divergent: Vec<&str> = catalog
        .entries_of(Kind::Action)
        .filter(|entry| {
            entry.param_type(0) == Some("Player|Array") && entry.param_type(1) == Some("Variable")
        })
        .map(|entry| entry.id.as_str())
        .collect();
    divergent.sort_unstable();
    let mut covered: Vec<&str> = DIVERGENT_SURFACE_CALLS.iter().map(|(id, ..)| *id).collect();
    covered.sort_unstable();
    assert_eq!(
        divergent, covered,
        "catalog actions with a folded (player, variable) signature must be \
         covered one-to-one by hand-written surface calls"
    );
}

#[test]
fn hand_written_surface_calls_parse_and_validate_for_divergent_entries() {
    let catalog = catalog();
    for (id, call, surface_args, stored_id, stored_args) in DIVERGENT_SURFACE_CALLS {
        let entry = catalog
            .entry(Kind::Action, id)
            .unwrap_or_else(|| panic!("{id}: catalog action must exist"));
        assert_eq!(
            entry.param_count(),
            *surface_args,
            "{id}: `{call}` carries {surface_args} surface arguments; the \
             declared signature must declare exactly these (a dropped \
             parameter leaves the surface call unsigned)"
        );
        let program = parse_and_validate(&source(call), &catalog)
            .unwrap_or_else(|error| panic!("{id}: `{call}` must parse and validate: {error}"));
        let Action::Call { name, args } = &program.rules[0].actions[0] else {
            panic!("{id}: `{call}` must produce an action call");
        };
        assert_eq!(
            name, stored_id,
            "{id}: `{call}` must store as `{stored_id}`"
        );
        assert_eq!(
            args.len(),
            *stored_args,
            "{id}: `{call}` must fold to {stored_args} WIR arguments"
        );
        let emitted = emitter::emit(&program, &catalog, &en())
            .unwrap_or_else(|error| panic!("{id}: `{call}` must emit: {error}"));
        assert!(
            emitted.contains(call),
            "{id}: emitted program must reproduce `{call}`, got:\n{emitted}"
        );
    }
}

#[test]
fn surface_calls_missing_the_player_argument_are_rejected() {
    let catalog = catalog();
    for (id, call) in OMITTED_PLAYER_CALLS {
        let result = parse_and_validate(&source(call), &catalog);
        assert!(
            result.is_err(),
            "{id}: `{call}` omits the declared player argument and must be \
             rejected, but it parsed and validated"
        );
    }
}

fn indexed_source(action: &str) -> String {
    format!(
        r#"variables {{
    global:
        0: g
    player:
        0: p
}}
rule ("r") {{
    event {{ Ongoing - Each Player; All; All; }}
    actions {{ {action}; }}
}}"#
    )
}

fn assert_indexed_call(
    program: &Program,
    action: &str,
    index: &str,
    number: f64,
    op: Option<ModifyOp>,
) {
    let Action::Call { name, args } = &program.rules[0].actions[0] else {
        panic!("expected indexed action");
    };
    assert_eq!(name, action);
    assert_eq!(args.len(), if op.is_some() { 4 } else { 3 });
    assert!(
        matches!(
            (action, &args[0]),
            ("setGlobalVariableAtIndex" | "modifyGlobalVariableAtIndex", Value::GlobalVariable(variable))
                if variable == "g"
        ) || matches!(
            (action, &args[0]),
            ("setPlayerVariableAtIndex" | "modifyPlayerVariableAtIndex", Value::PlayerVariable { player, variable })
                if matches!(player.as_ref(), Value::EventPlayer) && variable == "p"
        )
    );
    assert!(
        matches!(
            &args[1],
            Value::Call { name, args }
                if name == index
                    && matches!(&args[..], [Value::GlobalVariable(variable), Value::Number(value)]
                        if variable == "g" && *value == number)
        ),
        "index call must be preserved: {:?}",
        args[1]
    );
    if let Some(op) = op {
        assert!(
            matches!(
                &args[2],
                Value::Call { name, args } if name == op.catalog_id() && args.is_empty()
            ),
            "operation must be preserved: {:?}",
            args[2]
        );
        assert!(matches!(&args[3], Value::Number(value) if *value == 10.0));
    } else {
        assert!(matches!(&args[2], Value::Number(value) if *value == 10.0));
    }
}

fn check_indexed_call(
    catalog: &Catalog,
    action: &str,
    expected: &str,
    index: &str,
    number: f64,
    op: Option<ModifyOp>,
) {
    let source = indexed_source(action);
    let program = parse_and_validate(&source, catalog)
        .unwrap_or_else(|error| panic!("{action} must parse and validate: {error}"));
    program
        .validate()
        .expect("parsed program must be structurally valid");
    assert_indexed_call(&program, expected, index, number, op);
    let emitted = emitter::emit(&program, catalog, &en()).expect("program emits");
    let reparsed = parse_and_validate(&emitted, catalog).unwrap_or_else(|error| {
        panic!("emitted {action} must parse and validate: {error}\n{emitted}")
    });
    reparsed
        .validate()
        .expect("reparsed program must be structurally valid");
    assert_indexed_call(&reparsed, expected, index, number, op);
    assert!(
        roundtrip::equivalent(&program, &reparsed),
        "{action} must round-trip"
    );
}

fn check_modify_call_indices(player: bool) {
    let catalog = catalog();
    for (op, spelling) in [
        (ModifyOp::Add, "Add"),
        (ModifyOp::Subtract, "Subtract"),
        (ModifyOp::Multiply, "Multiply"),
        (ModifyOp::Divide, "Divide"),
        (ModifyOp::Modulo, "Modulo"),
        (ModifyOp::Min, "Min"),
        (ModifyOp::Max, "Max"),
        (ModifyOp::RaiseToPower, "Raise To Power"),
        (ModifyOp::AppendToArray, "Append To Array"),
        (
            ModifyOp::RemoveFromArrayByValue,
            "Remove From Array By Value",
        ),
        (
            ModifyOp::RemoveFromArrayByIndex,
            "Remove From Array By Index",
        ),
    ] {
        for (index, id, number) in [
            ("Add(Global.g, 18)", "add", 18.0),
            ("Multiply(Global.g, 2)", "multiply", 2.0),
            ("Raise To Power(Global.g, 2)", "raiseToPower", 2.0),
        ] {
            let (action, expected) = if player {
                (
                    format!(
                        "Modify Player Variable At Index(Event Player, p, {index}, {spelling}, 10)"
                    ),
                    "modifyPlayerVariableAtIndex",
                )
            } else {
                (
                    format!("Modify Global Variable At Index(g, {index}, {spelling}, 10)"),
                    "modifyGlobalVariableAtIndex",
                )
            };
            check_indexed_call(&catalog, &action, expected, id, number, Some(op));
        }
    }
}

#[test]
fn modify_global_variable_at_call_index_preserves_every_operation() {
    check_modify_call_indices(false);
}

#[test]
fn modify_player_variable_at_call_index_preserves_every_operation() {
    check_modify_call_indices(true);
}

#[test]
fn set_variable_at_call_index_round_trips_for_both_scopes() {
    let catalog = catalog();
    for (action, expected) in [
        (
            "Set Global Variable At Index(g, Add(Global.g, 18), 10)",
            "setGlobalVariableAtIndex",
        ),
        (
            "Set Player Variable At Index(Event Player, p, Add(Global.g, 18), 10)",
            "setPlayerVariableAtIndex",
        ),
    ] {
        check_indexed_call(&catalog, action, expected, "add", 18.0, None);
    }
}

#[test]
fn indexed_modify_still_rejects_unknown_operations() {
    let catalog = catalog();
    for action in [
        "Modify Global Variable At Index(g, Add(Global.g, 18), Unknown Operation, 10)",
        "Modify Player Variable At Index(Event Player, p, Add(Global.g, 18), Unknown Operation, 10)",
    ] {
        assert!(
            parse_and_validate(&indexed_source(action), &catalog).is_err(),
            "{action}"
        );
    }
}
