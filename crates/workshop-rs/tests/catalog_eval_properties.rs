use crate::common::catalog;
use workshop_rs::catalog::{Catalog, Kind};

#[test]
fn nondeterministic_values_carry_the_reviewed_flag() {
    let catalog = catalog();
    for id in [
        "randomInteger",
        "randomReal",
        "randomValueInArray",
        "randomizedArray",
        "getTotalTimeElapsed",
        "getMatchTime",
        "getServerLoad",
        "getAverageServerLoad",
        "getPeakServerLoad",
    ] {
        let entry = catalog
            .entry(Kind::Value, id)
            .unwrap_or_else(|| panic!("{id} is in the catalog"));
        assert_eq!(entry.nondeterministic(), Some(true), "{id}");
    }

    // A reviewed deterministic value reports `false`; an action or an
    // unreviewed entry reports `None` rather than `false`.
    let add = catalog.entry(Kind::Value, "add").expect("add");
    assert_eq!(add.nondeterministic(), Some(false));
    assert_eq!(
        catalog
            .entry(Kind::Action, "kill")
            .expect("kill")
            .nondeterministic(),
        None
    );
    assert_eq!(
        catalog
            .entry(Kind::Value, "getHealth")
            .expect("getHealth")
            .nondeterministic(),
        None
    );
}

#[test]
fn per_element_params_name_the_positions_rebinding_current_array_element() {
    let catalog = catalog();
    for id in [
        "filteredArray",
        "isTrueForAll",
        "isTrueForAny",
        "mappedArray",
        "sortedArray",
    ] {
        let entry = catalog
            .entry(Kind::Value, id)
            .unwrap_or_else(|| panic!("{id} is in the catalog"));
        assert_eq!(
            entry.per_element_params(),
            Some(&[1][..]),
            "{id} evaluates every argument after the array source per element"
        );
    }

    // A reviewed non-binder value reports an empty list; an action or an
    // unreviewed entry reports `None`.
    let add = catalog.entry(Kind::Value, "add").expect("add");
    assert_eq!(add.per_element_params(), Some(&[][..]));
    assert_eq!(
        catalog
            .entry(Kind::Action, "kill")
            .expect("kill")
            .per_element_params(),
        None
    );
    assert_eq!(
        catalog
            .entry(Kind::Value, "getHealth")
            .expect("getHealth")
            .per_element_params(),
        None
    );
}

#[test]
fn action_reevaluated_params_name_the_positions_the_action_reevaluates() {
    let catalog = catalog();
    for id in ["waitUntil", "loopIf"] {
        let entry = catalog
            .entry(Kind::Action, id)
            .unwrap_or_else(|| panic!("{id} is in the catalog"));
        assert_eq!(
            entry.action_reevaluated_params(),
            Some(&[0][..]),
            "{id} re-evaluates its condition argument itself"
        );
    }
    // The engine-level loop actions declare no parameters, so their
    // coverage is the reviewed empty list, not `None`.
    for id in ["loopIfConditionIsTrue", "__loopIfConditionIsFalse__"] {
        let entry = catalog
            .entry(Kind::Action, id)
            .unwrap_or_else(|| panic!("{id} is in the catalog"));
        assert_eq!(entry.action_reevaluated_params(), Some(&[][..]), "{id}");
    }

    // A plain action reports the reviewed empty list; a value or an
    // unreviewed action reports `None`.
    assert_eq!(
        catalog
            .entry(Kind::Action, "kill")
            .expect("kill")
            .action_reevaluated_params(),
        Some(&[][..])
    );
    assert_eq!(
        catalog
            .entry(Kind::Structural, "setGlobalVariable")
            .expect("setGlobalVariable structural entry")
            .action_reevaluated_params(),
        None
    );
    assert_eq!(
        catalog
            .entry(Kind::Value, "add")
            .expect("add")
            .action_reevaluated_params(),
        None
    );
    assert_eq!(
        catalog
            .entry(Kind::Action, "wait")
            .expect("wait")
            .action_reevaluated_params(),
        None
    );
}

#[test]
fn param_default_enum_member_decodes_the_typed_default() {
    let catalog = catalog();
    let wait = catalog.entry(Kind::Action, "wait").expect("wait");
    assert_eq!(
        wait.param_default_enum_member(1),
        Some(("Wait", "IGNORE_CONDITION"))
    );
    assert_eq!(wait.param_default_enum_member(0), None);

    let hud = catalog
        .entry(Kind::Action, "createHudText")
        .expect("createHudText");
    assert_eq!(hud.param_default_enum_member(7), Some(("Color", "WHITE")));
    // `null` is a literal default, not an enum member.
    assert_eq!(hud.param_default_enum_member(2), None);

    // A value-call default is not an enum member.
    let message = catalog
        .entry(Kind::Action, "bigMessage")
        .expect("bigMessage");
    assert_eq!(message.param_default(0), Some("allPlayers"));
    assert_eq!(message.param_default_enum_member(0), None);

    // No declared default reports `None`.
    assert_eq!(
        catalog
            .entry(Kind::Value, "add")
            .expect("add")
            .param_default_enum_member(0),
        None
    );
}

#[test]
fn evaluation_facts_reject_bad_records() {
    let fixture = |entry_kind: &str, fact: &str| {
        format!(
            r#"{{
            "schemaVersion": 1,
            "locales": ["en-US"],
            "target": {{ "game": "g", "format": "f", "surface": "s" }},
            "provenance": {{ "generator": "g", "generatorVersion": "0", "source": "s", "license": "l", "reviewed": true }},
            "{entry_kind}": [
                {{ "id": "some{entry_kind}", "aliases": {{ "en-US": "Some {entry_kind}" }}, "params": ["Player"]{fact} }}
            ]
        }}"#
        )
    };
    for (entry_kind, fact) in [
        // Evaluation facts on the wrong entry kind.
        ("actions", r#", "nondeterministic": true"#),
        ("actions", r#", "perElementParams": [0]"#),
        ("values", r#", "actionReevaluatedParams": [0]"#),
        // Positions out of range or duplicated.
        ("actions", r#", "actionReevaluatedParams": [1]"#),
        ("actions", r#", "actionReevaluatedParams": [0, 0]"#),
        ("values", r#", "perElementParams": [9]"#),
        ("values", r#", "perElementParams": [0, 0]"#),
    ] {
        let json = fixture(entry_kind, fact);
        assert!(
            Catalog::load(&json).is_err(),
            "fact should be rejected: {fact} on {entry_kind}"
        );
    }

    // Reviewed facts on the right entry kind load, including the explicit
    // empty review.
    for (entry_kind, fact) in [
        ("values", r#", "nondeterministic": false"#),
        ("values", r#", "perElementParams": []"#),
        ("actions", r#", "actionReevaluatedParams": []"#),
        ("actions", r#", "actionReevaluatedParams": [0]"#),
    ] {
        let json = fixture(entry_kind, fact);
        assert!(
            Catalog::load(&json).is_ok(),
            "reviewed fact should load: {fact} on {entry_kind}"
        );
    }
}

#[test]
fn param_display_names_target_the_structural_entry_of_a_dual_declared_id() {
    // ADR-0019: `modifyGlobalVariable` is declared under both `structural`
    // and `actions`; display-name data describes the structural signature,
    // so records resolve that section, not every entry carrying the id.
    let catalog = catalog();
    let en = workshop_rs::catalog::Locale::new("en-US");
    assert_eq!(
        catalog
            .entry(Kind::Structural, "modifyGlobalVariable")
            .and_then(|entry| entry.param_display_name(&en, 0)),
        Some("Variable")
    );
    assert_eq!(
        catalog
            .entry(Kind::Action, "modifyGlobalVariable")
            .and_then(|entry| entry.param_display_name(&en, 0)),
        None,
        "the action entry of the id keeps no display-name record"
    );
}
