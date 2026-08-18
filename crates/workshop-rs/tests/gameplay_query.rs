use workshop_rs::gameplay::{
    Ability, AbilityId, EvidenceRef, Fact, GameplayCatalog, GameplayDatasetIdentity, Hero, HeroId,
    LocalizedText, LogicalSlot, Quantity, StatKey, StatValue, Unit, units,
};
use workshop_rs::gameplay_query::{
    CooldownError, CooldownNonApplicability, CooldownPercentage, CooldownPercentageError,
    GameplayQueryError, StatOwner,
};

fn evidence(locator: &str) -> EvidenceRef {
    EvidenceRef {
        source: "workshop-data".to_string(),
        locator: locator.to_string(),
        note: None,
    }
}

fn names(name: &str, locator: &str) -> Fact<LocalizedText> {
    Fact::new(
        LocalizedText::new([("en-US".to_string(), name.to_string())]),
        vec![evidence(locator)],
    )
}

fn ability(id: &str, slot: &str, variant: Option<&str>) -> Ability {
    Ability::new(
        AbilityId::new(id),
        LogicalSlot::new(slot),
        variant.map(Into::into),
        names(id, &format!("abilities.{id}")),
        vec![evidence(&format!("abilities.{id}"))],
    )
}

fn seconds(value: f64) -> Quantity {
    Quantity::new(value, Unit::from(units::SECONDS)).unwrap()
}

fn identity() -> GameplayDatasetIdentity {
    GameplayDatasetIdentity {
        dataset_id: "gameplay-test".to_string(),
        version: "2026-08-18".to_string(),
        digest: "sha256:test".to_string(),
        source: "test-fixture".to_string(),
        license: "MIT".to_string(),
        target: "gameplay query tests".to_string(),
        reviewed: true,
    }
}

fn catalog() -> GameplayCatalog {
    let ana = Hero::new(
        HeroId::new("ana"),
        names("Ana", "heroes.ana"),
        vec![
            ability("sleepDart", "ability1", None)
                .with_keyword("crowd-control")
                .with_stat(
                    StatKey::new("cooldown"),
                    Fact::new(
                        StatValue::Quantity(seconds(12.0)),
                        vec![evidence("ana.sleep")],
                    ),
                ),
            ability("nanoBoost", "ultimate", None)
                .with_keyword("buff")
                .with_stat(
                    StatKey::new("description"),
                    Fact::new(
                        StatValue::Text("ultimate".to_string()),
                        vec![evidence("ana.nano.description")],
                    ),
                ),
        ],
        vec![evidence("heroes.ana")],
    );
    let ramattra = Hero::new(
        HeroId::new("ramattra"),
        names("Ramattra", "heroes.ramattra"),
        vec![
            ability("voidBarrierNemesis", "secondaryFire", Some("nemesis")).with_keyword("barrier"),
            ability("voidBarrierOmnic", "secondaryFire", Some("omnic")).with_keyword("barrier"),
        ],
        vec![evidence("heroes.ramattra")],
    );
    GameplayCatalog::new(identity(), vec![ramattra, ana]).unwrap()
}

#[test]
fn queries_are_deterministic_and_cover_hero_kit_slot_variant_keyword_and_stat() {
    let catalog = catalog();
    let query = catalog.query();

    assert_eq!(
        query
            .heroes()
            .iter()
            .map(|hero| hero.id().as_str())
            .collect::<Vec<_>>(),
        ["ana", "ramattra"]
    );
    assert_eq!(
        query
            .kit("ana")
            .unwrap()
            .iter()
            .map(|ability| ability.id().as_str())
            .collect::<Vec<_>>(),
        ["sleepDart", "nanoBoost"]
    );
    assert_eq!(
        query
            .slot("ramattra", "secondaryFire")
            .unwrap()
            .iter()
            .map(|ability| ability.id().as_str())
            .collect::<Vec<_>>(),
        ["voidBarrierNemesis", "voidBarrierOmnic"]
    );
    assert_eq!(
        query
            .variant("ramattra", "secondaryFire", "omnic")
            .unwrap()
            .id()
            .as_str(),
        "voidBarrierOmnic"
    );
    let keyword = query.keyword("barrier");
    assert_eq!(keyword.len(), 2);
    assert_eq!(keyword[0].hero.id().as_str(), "ramattra");
    assert_eq!(keyword[0].ability.id().as_str(), "voidBarrierNemesis");
    assert_eq!(
        query.stat("ana", "sleepDart", "cooldown").unwrap().value(),
        &StatValue::Quantity(seconds(12.0))
    );
    assert_eq!(
        query
            .quantity_stat("ana", "sleepDart", "cooldown")
            .unwrap()
            .value,
        12.0
    );
}

#[test]
fn query_failures_are_explicit_and_no_match_keyword_is_empty() {
    let catalog = catalog();
    let query = catalog.query();
    assert!(matches!(
        query.hero("missing"),
        Err(GameplayQueryError::MissingHero { .. })
    ));
    assert!(matches!(
        query.slot("ana", "ability2"),
        Err(GameplayQueryError::MissingSlot { .. })
    ));
    assert!(matches!(
        query.slot_ability("ramattra", "secondaryFire"),
        Err(GameplayQueryError::AmbiguousSlot { candidates, .. }) if candidates == [
            AbilityId::new("voidBarrierNemesis"),
            AbilityId::new("voidBarrierOmnic")
        ]
    ));
    assert!(matches!(
        query.variant("ramattra", "secondaryFire", "missing"),
        Err(GameplayQueryError::MissingVariant { .. })
    ));
    assert!(matches!(
        query.stat("ana", "sleepDart", "damage"),
        Err(GameplayQueryError::MissingStat {
            owner: StatOwner::Ability { .. },
            ..
        })
    ));
    assert!(matches!(
        query.quantity_stat("ana", "nanoBoost", "description"),
        Err(GameplayQueryError::WrongStatType { .. })
    ));
    assert!(matches!(
        query.ability("ana", "missing"),
        Err(GameplayQueryError::MissingAbility { .. })
    ));
    assert!(query.keyword("missing").is_empty());
}

#[test]
fn cooldown_calculations_are_unit_safe_bounded_and_non_mutating() {
    let catalog = catalog();
    let query = catalog.query();
    let ability = query.ability("ana", "sleepDart").unwrap();
    let base = query.cooldown(ability).unwrap();
    assert_eq!(base.value, 12.0);
    assert_eq!(
        query
            .effective_cooldown(ability, CooldownPercentage::new(50.0).unwrap())
            .unwrap()
            .value,
        6.0
    );
    assert_eq!(
        query
            .effective_cooldown(ability, CooldownPercentage::new(100.0).unwrap())
            .unwrap()
            .value,
        12.0
    );
    assert_eq!(
        query
            .effective_cooldown(ability, CooldownPercentage::new(0.0).unwrap())
            .unwrap()
            .value,
        0.0
    );
    assert_eq!(
        query
            .effective_cooldown(ability, CooldownPercentage::new(500.0).unwrap())
            .unwrap()
            .value,
        60.0
    );
    assert_eq!(
        query
            .required_cooldown_percentage(ability, &seconds(3.0))
            .unwrap()
            .value(),
        25.0
    );
    assert_eq!(query.cooldown(ability).unwrap().value, 12.0);
}

#[test]
fn cooldown_percentage_rejects_invalid_values_and_targets() {
    assert!(matches!(
        CooldownPercentage::new(f64::NAN),
        Err(CooldownPercentageError::NotFinite { .. })
    ));
    assert!(matches!(
        CooldownPercentage::new(-0.1),
        Err(CooldownPercentageError::OutOfRange { .. })
    ));
    assert!(matches!(
        CooldownPercentage::new(500.1),
        Err(CooldownPercentageError::OutOfRange { .. })
    ));

    let catalog = catalog();
    let query = catalog.query();
    let ability = query.ability("ana", "sleepDart").unwrap();
    assert!(matches!(
        query.required_cooldown_percentage(
            ability,
            &Quantity::new(1.0, Unit::new("meters")).unwrap()
        ),
        Err(CooldownError::TargetWrongUnit { .. })
    ));
    assert!(matches!(
        query.required_cooldown_percentage(ability, &seconds(-1.0)),
        Err(CooldownError::InvalidTarget { .. })
    ));
    assert!(matches!(
        query.required_cooldown_percentage(ability, &seconds(61.0)),
        Err(CooldownError::InvalidPercentage(
            CooldownPercentageError::OutOfRange { .. }
        ))
    ));
}

#[test]
fn cooldown_missing_and_non_applicable_data_is_not_defaulted() {
    let missing = Hero::new(
        HeroId::new("missing"),
        names("Missing", "heroes.missing"),
        vec![ability("noCooldown", "ability1", None)],
        vec![evidence("heroes.missing")],
    );
    let wrong_type = ability("textCooldown", "ability1", None).with_stat(
        StatKey::new("cooldown"),
        Fact::new(StatValue::Text("12".to_string()), vec![evidence("text")]),
    );
    let wrong_unit = ability("meterCooldown", "ability2", None).with_stat(
        StatKey::new("cooldown"),
        Fact::new(
            StatValue::Quantity(Quantity::new(12.0, Unit::new("meters")).unwrap()),
            vec![evidence("meters")],
        ),
    );
    let zero = ability("zeroCooldown", "ability3", None).with_stat(
        StatKey::new("cooldown"),
        Fact::new(StatValue::Quantity(seconds(0.0)), vec![evidence("zero")]),
    );
    let hero = Hero::new(
        HeroId::new("edge"),
        names("Edge", "heroes.edge"),
        vec![wrong_type, wrong_unit, zero],
        vec![evidence("heroes.edge")],
    );
    let catalog = GameplayCatalog::new(identity(), vec![missing, hero]).unwrap();
    let query = catalog.query();

    assert!(matches!(
        query.cooldown(query.ability("missing", "noCooldown").unwrap()),
        Err(CooldownError::Missing { .. })
    ));
    assert!(matches!(
        query.cooldown(query.ability("edge", "textCooldown").unwrap()),
        Err(CooldownError::NonApplicable {
            reason: CooldownNonApplicability::WrongValueType,
            ..
        })
    ));
    assert!(matches!(
        query.cooldown(query.ability("edge", "meterCooldown").unwrap()),
        Err(CooldownError::NonApplicable {
            reason: CooldownNonApplicability::WrongUnit { .. },
            ..
        })
    ));
    assert!(matches!(
        query.cooldown(query.ability("edge", "zeroCooldown").unwrap()),
        Err(CooldownError::NonApplicable {
            reason: CooldownNonApplicability::NonPositiveBase,
            ..
        })
    ));
}
