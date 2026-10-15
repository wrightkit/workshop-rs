//! Hand-written surface calls for catalog actions whose declared signature
//! diverges from the WIR call shape (workshop-rs#461).
//!
//! `docs/language-support/actions.md` is generated from the catalog, so
//! conformance cases derived from it cannot notice a signature that declares
//! the WIR shape instead of the source syntax — #445 shipped exactly that
//! defect when `Set/Modify Player Variable At Index` declared
//! `(Variable, Index, [Operation,] Value)` and omitted the leading `player`
//! argument the surface form requires.
//!
//! The calls below are written by hand from Workshop syntax, not derived
//! from catalog-generated docs. Each entry's leading `(player, variable)`
//! parameters fold into a single `PlayerVariable` WIR argument (the chase
//! calls additionally store under the canonical `chaseAtRate`/`chaseOverTime`
//! call ids); the same calls with the player argument omitted must be
//! rejected.

mod common;

use workshop_rs::catalog::{Catalog, Kind};
use workshop_rs::program::{Action, Program};
use workshop_rs::{WorkshopError, emitter, parser, validate};

use crate::common::{catalog, en};

/// One hand-written en-US surface call per catalog action whose declared
/// signature diverges from the stored WIR call:
/// `(catalog id, surface call, surface arg count, stored call id, stored arg
/// count)`.
///
/// The stored shape is what `parser::parse` produces: the folded
/// `PlayerVariable` argument replaces the surface's `player, variable`
/// pair, so the stored arg count is the surface arg count minus one. The
/// surface arg count pins the declared signature: dropping any one of its
/// parameters makes this table disagree with `param_count`.
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
        "chaseAtRate",
        4,
    ),
    (
        "chasePlayerVariableOverTime",
        "Chase Player Variable Over Time(Event Player, probe_p, 0, 1, None);",
        5,
        "chaseOverTime",
        4,
    ),
];

/// The same calls with the leading `player` argument omitted — the WIR-shaped
/// signatures these entries wrongly declared before #445.
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

/// The coverage above must name every catalog action whose declared surface
/// signature folds the `(player, variable)` pair — the predicate
/// `CatalogEntry::folds_player_variable_pair` applies internally. A newly
/// divergent entry fails here until a hand-written call covers it.
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
