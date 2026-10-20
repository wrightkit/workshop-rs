//! Contract test: every `rule-content-v1` call id resolves through
//! `Catalog::lookup` to the catalog entry the call came from
//! (workshop-rs#544).
//!
//! A consumer names a call by looking its emitted id up — the schema's
//! `CallId` contract promises "a public catalog structural, action, value,
//! or operator id". workshop-rs#493 emitted the global chase's id for a
//! player-variable chase: the id resolved, but to the neighboring entry,
//! mislabeling the call. This module checks the whole class: every action
//! and value call form's emitted id resolves back to its own entry.

use workshop_rs::catalog::{Catalog, CatalogEntry, Kind, Locale};
use workshop_rs::lookup::LookupMatch;
use workshop_rs::rules::content::rule_content;
use workshop_rs::{Action, Condition, Event, Rule, Value, parser};

use crate::common::{catalog, en};

/// `(catalog id, en-US surface call)` — one call for every catalog entry
/// whose signature folds the leading `(player, variable)` parameters into
/// one `playerVariable` argument. The parser stores these calls under a
/// name that can differ from the catalog id: the player-variable chases
/// keep the global chase's WIR name (the workshop-rs#493 merge).
const FOLDED_PAIR_CALLS: &[(&str, &str)] = &[
    (
        "setPlayerVariableAtIndex",
        "Set Player Variable At Index(Event Player, probe_p, 0, 5);",
    ),
    (
        "modifyPlayerVariableAtIndex",
        "Modify Player Variable At Index(Event Player, probe_p, 0, Add, 5);",
    ),
    (
        "stopChasingPlayerVariable",
        "Stop Chasing Player Variable(Event Player, probe_p);",
    ),
    (
        "chasePlayerVariableAtRate",
        "Chase Player Variable At Rate(Event Player, probe_p, 0, 1, None);",
    ),
    (
        "chasePlayerVariableOverTime",
        "Chase Player Variable Over Time(Event Player, probe_p, 0, 1, None);",
    ),
];

/// Whether the signature's leading `(player, variable)` parameters fold
/// into one `playerVariable` argument — the same predicate
/// `CatalogEntry::folds_player_variable_pair` applies internally.
fn folds_player_variable_pair(entry: &CatalogEntry) -> bool {
    entry.param_type(0) == Some("Player|Array") && entry.param_type(1) == Some("Variable")
}

/// Whether `emitted` resolves through `Catalog::lookup` to the catalog
/// entry `(kind, id)`: the primary match — the one a consumer naming the
/// call takes — must be a builtin carrying `id`, and the match list must
/// contain the `(kind, id)` entry itself. One id can name catalog entries
/// of two kinds (the structural and action rows of `modifyGlobalVariable`,
/// the value and operator rows of `add`); the twin shares the id, so the
/// id — not the kind — identifies the primary match. `lookup` scores
/// unrelated entries only as secondary matches, so an id swapped for a
/// neighbor's fails the primary check.
fn resolves_to(catalog: &Catalog, locale: &Locale, emitted: &str, kind: Kind, id: &str) -> bool {
    let matches = catalog
        .lookup(locale, emitted)
        .expect("lookup of a declared locale");
    let primary_is_entry = matches!(
        matches.first(),
        Some(LookupMatch::Builtin { id: found, .. }) if found == id
    );
    let contains_entry = matches.iter().any(|lookup| {
        matches!(
            lookup,
            LookupMatch::Builtin {
                kind: found_kind,
                id: found_id,
                ..
            } if *found_kind == kind && found_id == id
        )
    });
    primary_is_entry && contains_entry
}

/// The call id `rule-content-v1` emits for a call named by `entry`'s own
/// id: an action entry renders an action node, a value entry renders the
/// value of a condition. A folded entry carries its `playerVariable` target
/// first, the shape the stored call holds.
fn emitted_call_id(catalog_entry: &CatalogEntry) -> String {
    let folded = folds_player_variable_pair(catalog_entry);
    let mut args = Vec::new();
    if folded {
        args.push(Value::player_variable(Value::EventPlayer, "probe_p"));
    }
    args.resize_with(catalog_entry.param_count() - usize::from(folded), || {
        Value::from(0.0)
    });
    let rule = Rule::new("call id contract", Event::Global);
    let content = match catalog_entry.kind {
        Kind::Action => rule_content(&rule.action(Action::call(catalog_entry.id.clone(), args))),
        Kind::Value => rule_content(
            &rule.condition(Condition::new(Value::call(catalog_entry.id.clone(), args))),
        ),
        other => unreachable!("only actions and values are enumerated: {other:?}"),
    }
    .expect("rule content renders");
    let node = match catalog_entry.kind {
        Kind::Action => &content["actions"][0],
        Kind::Value => &content["conditions"][0]["value"],
        _ => unreachable!(),
    };
    node["call"]
        .as_str()
        .expect("a rendered call node carries a call id")
        .to_string()
}

/// Parse `call` (an action statement ending in `;`) as the only action of a
/// player-variable rule and return the rule as stored.
fn parsed_action_rule(call: &str) -> Rule {
    let source = format!(
        "variables {{\n    player:\n        0: probe_p\n}}\n\nrule (\"probe\") {{\n    event {{\n        Ongoing - Each Player;\n    }}\n    actions {{\n        {call}\n    }}\n}}\n"
    );
    let program = parser::parse(&source, &catalog(), &en())
        .unwrap_or_else(|error| panic!("`{call}` must parse: {error}"));
    let [rule] = <[Rule; 1]>::try_from(program.rules).expect("one rule");
    rule
}

#[test]
fn every_emitted_call_id_resolves_to_the_entry_it_came_from() {
    let catalog = catalog();
    let english = en();

    // The check must catch an id swapped for a neighbor's (workshop-rs#493):
    // a player chase emitted under the global chase's id must NOT resolve
    // back to the player chase entry.
    for (neighbor, own) in [
        ("chaseAtRate", "chasePlayerVariableAtRate"),
        ("chaseOverTime", "chasePlayerVariableOverTime"),
    ] {
        assert!(
            !resolves_to(&catalog, &english, neighbor, Kind::Action, own),
            "a call emitted under '{neighbor}' must not resolve to '{own}'"
        );
    }

    // Every catalog action and value entry renders a `{"call": id}` node;
    // the emitted id must resolve back to that same entry.
    for entry in catalog
        .entries_of(Kind::Action)
        .chain(catalog.entries_of(Kind::Value))
    {
        let emitted = emitted_call_id(entry);
        assert!(
            resolves_to(&catalog, &english, &emitted, entry.kind, &entry.id),
            "{} '{}' renders the call id '{emitted}', which resolves to a \
             different catalog entry",
            entry.kind.as_str(),
            entry.id
        );
    }

    // The parsed stored call for each folded `(player, variable)` action —
    // the shape the parser actually produces, including a stored WIR name
    // shared with a neighbor entry.
    for (id, call) in FOLDED_PAIR_CALLS {
        let content = rule_content(&parsed_action_rule(call)).expect("rule content renders");
        let emitted = content["actions"][0]["call"]
            .as_str()
            .expect("a rendered call node carries a call id");
        assert!(
            resolves_to(&catalog, &english, emitted, Kind::Action, id),
            "'{id}' parses to a call emitted as '{emitted}', which resolves \
             to a different catalog entry"
        );
    }
}

#[test]
fn every_folded_pair_call_has_surface_coverage() {
    // `FOLDED_PAIR_CALLS` must name every catalog entry whose signature
    // folds a leading `(player, variable)` pair — the entries whose stored
    // call name can diverge from their id. A newly folding entry fails here
    // until a surface call covers the name it actually stores under.
    let catalog = catalog();
    let mut folded: Vec<&str> = catalog
        .entries_of(Kind::Action)
        .chain(catalog.entries_of(Kind::Value))
        .filter(|entry| folds_player_variable_pair(entry))
        .map(|entry| entry.id.as_str())
        .collect();
    folded.sort_unstable();
    let mut covered: Vec<&str> = FOLDED_PAIR_CALLS.iter().map(|(id, _)| *id).collect();
    covered.sort_unstable();
    assert_eq!(
        folded, covered,
        "every entry folding a (player, variable) pair must have a \
         surface call in FOLDED_PAIR_CALLS"
    );
}
