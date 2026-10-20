//! The hero-values loader validates files against the gameplay dataset and the
//! public `hero_values` read path exposes them.

use workshop_rs::hero_values::{self, HealthPool, HeroValuesError};

fn document(values: &str) -> String {
    format!(
        r#"{{"format":"workshop-rs/hero-values-v1","balanceVersion":"test-version","heroes":[{{"id":"ana","values":[{values}]}}]}}"#
    )
}

fn load_values(values: &str) -> Result<workshop_rs::hero_values::HeroValues, HeroValuesError> {
    hero_values::load(&document(values))
}

#[test]
fn builtin_loads_the_pinned_version() {
    let values = hero_values::builtin(hero_values::CURRENT_VERSION).expect("pinned version loads");
    assert_eq!(values.version(), hero_values::CURRENT_VERSION);
}

#[test]
fn builtin_rejects_an_unknown_version() {
    assert!(matches!(
        hero_values::builtin("9.9.9.9 - 0"),
        Err(HeroValuesError::UnknownVersion { .. })
    ));
}

#[test]
fn absent_values_read_as_none_not_a_default() {
    let values = hero_values::builtin(hero_values::CURRENT_VERSION).expect("pinned version loads");
    assert_eq!(values.health("ana", HealthPool::Health), None);
    assert_eq!(values.health("ana", HealthPool::Shield), None);
    assert_eq!(values.cooldown("ana", "ability1", None), None);
    assert_eq!(values.ultimate_cost("ana"), None);
    assert_eq!(values.health("notAHero", HealthPool::Health), None);
}

#[test]
fn loaded_values_are_exposed() {
    let values = load_values(
        r#"{"kind":"health","pool":"health","amount":250},
           {"kind":"health","pool":"armor","amount":25},
           {"kind":"cooldown","slot":"ability1","seconds":14},
           {"kind":"ultimateCost","points":2150}"#,
    )
    .expect("valid file loads");
    assert_eq!(values.health("ana", HealthPool::Health), Some(250.0));
    assert_eq!(values.health("ana", HealthPool::Armor), Some(25.0));
    assert_eq!(values.health("ana", HealthPool::Shield), None);
    assert_eq!(values.cooldown("ana", "ability1", None), Some(14.0));
    assert_eq!(values.cooldown("ana", "ability2", None), None);
    assert_eq!(values.ultimate_cost("ana"), Some(2150.0));
    assert_eq!(values.version(), "test-version");
}

#[test]
fn variant_cooldowns_are_supported() {
    let json = r#"{"format":"workshop-rs/hero-values-v1","balanceVersion":"v","heroes":[
        {"id":"bastion","values":[
            {"kind":"cooldown","slot":"primaryFire","variant":"assault","seconds":2.5},
            {"kind":"cooldown","slot":"primaryFire","variant":"recon","seconds":3}
        ]}]}"#;
    let values = hero_values::load(json).expect("variant cooldowns load");
    assert_eq!(
        values.cooldown("bastion", "primaryFire", Some("assault")),
        Some(2.5)
    );
    assert_eq!(
        values.cooldown("bastion", "primaryFire", Some("recon")),
        Some(3.0)
    );
    assert_eq!(values.cooldown("bastion", "primaryFire", None), None);
}

#[test]
fn load_rejects_an_unknown_hero() {
    let json = r#"{"format":"workshop-rs/hero-values-v1","balanceVersion":"v","heroes":[
        {"id":"exampleHero","values":[]}]}"#;
    assert!(matches!(
        hero_values::load(json),
        Err(HeroValuesError::UnknownHero { .. })
    ));
}

#[test]
fn load_rejects_an_unknown_slot() {
    assert!(matches!(
        load_values(r#"{"kind":"cooldown","slot":"ability3","seconds":1}"#),
        Err(HeroValuesError::UnknownSlot { .. })
    ));
}

#[test]
fn load_rejects_an_unknown_variant() {
    let json = r#"{"format":"workshop-rs/hero-values-v1","balanceVersion":"v","heroes":[
        {"id":"bastion","values":[
            {"kind":"cooldown","slot":"ability1","variant":"assault","seconds":1}
        ]}]}"#;
    assert!(matches!(
        hero_values::load(json),
        Err(HeroValuesError::UnknownVariant { .. })
    ));
}

#[test]
fn load_rejects_an_ambiguous_slot_without_variant() {
    let json = r#"{"format":"workshop-rs/hero-values-v1","balanceVersion":"v","heroes":[
        {"id":"bastion","values":[
            {"kind":"cooldown","slot":"primaryFire","seconds":1}
        ]}]}"#;
    assert!(matches!(
        hero_values::load(json),
        Err(HeroValuesError::AmbiguousSlot { .. })
    ));
}

#[test]
fn load_rejects_duplicate_heroes() {
    let json = r#"{"format":"workshop-rs/hero-values-v1","balanceVersion":"v","heroes":[
        {"id":"ana","values":[]},{"id":"ana","values":[]}]}"#;
    assert!(matches!(
        hero_values::load(json),
        Err(HeroValuesError::DuplicateHero { .. })
    ));
}

#[test]
fn load_rejects_duplicate_values() {
    for values in [
        r#"{"kind":"health","pool":"health","amount":1},{"kind":"health","pool":"health","amount":2}"#,
        r#"{"kind":"cooldown","slot":"ability1","seconds":1},{"kind":"cooldown","slot":"ability1","seconds":2}"#,
        r#"{"kind":"ultimateCost","points":1},{"kind":"ultimateCost","points":2}"#,
    ] {
        assert!(
            matches!(
                load_values(values),
                Err(HeroValuesError::DuplicateValue { .. })
            ),
            "{values} unexpectedly loads"
        );
    }
}

#[test]
fn load_rejects_non_positive_amounts() {
    for values in [
        r#"{"kind":"health","pool":"health","amount":0}"#,
        r#"{"kind":"health","pool":"shield","amount":-5}"#,
        r#"{"kind":"cooldown","slot":"ability1","seconds":0}"#,
        r#"{"kind":"ultimateCost","points":-1}"#,
    ] {
        assert!(
            matches!(
                load_values(values),
                Err(HeroValuesError::NonPositiveAmount { .. })
            ),
            "{values} unexpectedly loads"
        );
    }
}

#[test]
fn load_rejects_unknown_and_missing_kinds() {
    assert!(matches!(
        load_values(r#"{"kind":"damage","amount":1}"#),
        Err(HeroValuesError::UnknownKind { .. })
    ));
    assert!(matches!(
        load_values(r#"{"amount":1}"#),
        Err(HeroValuesError::Malformed(_))
    ));
}

#[test]
fn load_rejects_schema_shape_violations() {
    for values in [
        // an unknown pool
        r#"{"kind":"health","pool":"barrier","amount":1}"#,
        // a field another kind owns
        r#"{"kind":"health","pool":"health","amount":1,"slot":"ability1"}"#,
        // a missing required field
        r#"{"kind":"cooldown","slot":"ability1"}"#,
        r#"{"kind":"ultimateCost"}"#,
        // an empty slot or variant
        r#"{"kind":"cooldown","slot":"","seconds":1}"#,
        r#"{"kind":"cooldown","slot":"ability1","variant":"","seconds":1}"#,
    ] {
        assert!(load_values(values).is_err(), "{values} unexpectedly loads");
    }
    assert!(matches!(
        load_values(r#"{"kind":"cooldown","slot":"","seconds":1}"#),
        Err(HeroValuesError::EmptyId("slot"))
    ));
    assert!(matches!(
        load_values(r#"{"kind":"cooldown","slot":"ability1","variant":"","seconds":1}"#),
        Err(HeroValuesError::EmptyId("variant"))
    ));
}

#[test]
fn load_rejects_malformed_documents() {
    for json in [
        "{",
        r#"{"format":"other","balanceVersion":"v","heroes":[]}"#,
        r#"{"format":"workshop-rs/hero-values-v1","balanceVersion":"","heroes":[]}"#,
        r#"{"format":"workshop-rs/hero-values-v1","heroes":[]}"#,
        r#"{"format":"workshop-rs/hero-values-v1","balanceVersion":"v","heroes":[{"id":"","values":[]}]}"#,
        r#"{"format":"workshop-rs/hero-values-v1","balanceVersion":"v","heroes":[],"extra":1}"#,
        r#"{"format":"workshop-rs/hero-values-v1","balanceVersion":"v","heroes":[{"id":"ana","values":[{"kind":"health","pool":"health","amount":1e999}]}]}"#,
    ] {
        assert!(
            hero_values::load(json).is_err(),
            "{json} unexpectedly loads"
        );
    }
    assert!(matches!(
        hero_values::load(r#"{"format":"other","balanceVersion":"v","heroes":[]}"#),
        Err(HeroValuesError::InvalidFormat { .. })
    ));
    assert!(matches!(
        hero_values::load(r#"{"format":"workshop-rs/hero-values-v1","heroes":[]}"#),
        Err(HeroValuesError::Malformed(_))
    ));
}
