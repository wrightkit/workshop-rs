use workshop_rs::gameplay::{
    Ability, AbilityRef, AbilityVariant, EvidenceRef, Fact, GameplayCatalog,
    GameplayDatasetIdentity, Hero, HeroId, LocalizedText, LogicalSlot, Quantity, StatKey,
    StatValue, Unit, hero_ids, slots, units,
};
use workshop_rs::gameplay_data::builtin;
use workshop_rs::gameplay_query::{
    AbilityNameResolutionError, CooldownError, CooldownNonApplicability, CooldownPercentage,
    CooldownPercentageError, GameplayQueryError, StatOwner,
};

fn evidence(locator: &str) -> EvidenceRef {
    EvidenceRef {
        source: "test-fixture".to_string(),
        locator: locator.to_string(),
        note: None,
    }
}

fn names(en: &str, zh: &str, locator: &str) -> Fact<LocalizedText> {
    Fact::new(
        LocalizedText::new([
            ("en-US".to_string(), en.to_string()),
            ("zh-CN".to_string(), zh.to_string()),
        ]),
        vec![evidence(locator)],
    )
}

fn ability(en: &str, zh: &str, slot: &str, variant: Option<&str>) -> Ability {
    Ability::new(
        LogicalSlot::new(slot),
        variant.map(AbilityVariant::new),
        names(en, zh, &format!("ability.{slot}")),
        vec![evidence(&format!("ability.{slot}"))],
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
    let sleep = ability("Sleep Dart", "麻醉镖", "ability1", None)
        .with_keyword("crowd-control")
        .with_stat(
            StatKey::new("cooldown"),
            Fact::new(
                StatValue::Quantity(seconds(12.0)),
                vec![evidence("ana.sleep")],
            ),
        );
    let nano = ability("Nano Boost", "纳米激素", "ultimate", None)
        .with_keyword("buff")
        .with_stat(
            StatKey::new("description"),
            Fact::new(
                StatValue::Text("ultimate".to_string()),
                vec![evidence("ana.nano.description")],
            ),
        );
    let ana = Hero::new(
        HeroId::new("ana"),
        names("Ana", "安娜", "heroes.ana"),
        vec![sleep, nano],
        vec![evidence("heroes.ana")],
    );
    let ramattra = Hero::new(
        HeroId::new("ramattra"),
        names("Ramattra", "拉玛刹", "heroes.ramattra"),
        vec![
            ability("Void Barrier", "虚空屏障", "secondaryFire", Some("nemesis"))
                .with_keyword("barrier"),
            ability("Void Barrier", "虚空屏障", "secondaryFire", Some("omnic"))
                .with_keyword("barrier"),
        ],
        vec![evidence("heroes.ramattra")],
    );
    GameplayCatalog::new(identity(), vec![ramattra, ana]).unwrap()
}

#[test]
fn queries_use_open_typed_hero_slot_and_variant_semantics() {
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
            .kit(hero_ids::ANA)
            .unwrap()
            .iter()
            .map(|ability| ability.slot().as_str())
            .collect::<Vec<_>>(),
        ["ability1", "ultimate"]
    );
    assert_eq!(
        query
            .slot(hero_ids::RAMATTRA, slots::SECONDARY_FIRE)
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        query
            .variant(hero_ids::RAMATTRA, slots::SECONDARY_FIRE, "omnic")
            .unwrap()
            .variant()
            .unwrap()
            .as_str(),
        "omnic"
    );
    assert_eq!(query.keyword("barrier").len(), 2);
    assert_eq!(
        query
            .stat(hero_ids::ANA, slots::ABILITY_1, None, "cooldown")
            .unwrap()
            .value(),
        &StatValue::Quantity(seconds(12.0))
    );
    assert_eq!(
        query
            .quantity_stat(hero_ids::ANA, slots::ABILITY_1, None, "cooldown")
            .unwrap()
            .value,
        12.0
    );
}

#[test]
fn missing_unknown_ambiguous_and_wrong_stat_paths_are_explicit() {
    let catalog = catalog();
    let query = catalog.query();
    assert!(matches!(
        query.hero("missing"),
        Err(GameplayQueryError::MissingHero { .. })
    ));
    assert!(matches!(
        query.slot(hero_ids::ANA, slots::ABILITY_2),
        Err(GameplayQueryError::MissingSlot { .. })
    ));
    assert!(
        matches!(query.ability(hero_ids::RAMATTRA, slots::SECONDARY_FIRE), Err(GameplayQueryError::AmbiguousSlot { candidates, .. }) if candidates.iter().all(|reference| reference.variant().is_some()))
    );
    assert!(matches!(
        query.variant(hero_ids::RAMATTRA, slots::SECONDARY_FIRE, "missing"),
        Err(GameplayQueryError::MissingVariant { .. })
    ));
    assert!(matches!(
        query.stat(hero_ids::ANA, slots::ABILITY_1, None, "damage"),
        Err(GameplayQueryError::MissingStat {
            owner: StatOwner::Ability { .. },
            ..
        })
    ));
    assert!(matches!(
        query.quantity_stat(hero_ids::ANA, slots::ULTIMATE, None, "description"),
        Err(GameplayQueryError::WrongStatType { .. })
    ));
    let unknown = AbilityRef::new(HeroId::new("ana"), LogicalSlot::new("ability2"), None);
    assert!(matches!(
        query.ability_ref(&unknown),
        Err(GameplayQueryError::MissingSlot { .. } | GameplayQueryError::MissingAbility { .. })
    ));
}

#[test]
fn locale_forward_and_inverse_resolution_is_exact_and_preserves_reference() {
    let catalog = catalog();
    let query = catalog.query();
    let sleep = AbilityRef::new(
        HeroId::from(hero_ids::ANA),
        LogicalSlot::from(slots::ABILITY_1),
        None,
    );
    assert_eq!(
        query
            .ability_name(hero_ids::ANA, slots::ABILITY_1, None, "en-US")
            .unwrap(),
        "Sleep Dart"
    );
    assert_eq!(
        query
            .ability_name(hero_ids::ANA, slots::ABILITY_1, None, "zh-CN")
            .unwrap(),
        "麻醉镖"
    );
    assert_eq!(
        query
            .resolve_ability_name(hero_ids::ANA, "zh-CN", "麻醉镖")
            .unwrap(),
        sleep
    );
    assert!(matches!(
        query.ability_name(hero_ids::ANA, slots::ABILITY_1, None, "fr-FR"),
        Err(AbilityNameResolutionError::UnsupportedLocale { .. })
    ));
    assert!(matches!(
        query.resolve_ability_name(hero_ids::ANA, "en-US", "unknown"),
        Err(AbilityNameResolutionError::MissingDisplayName { .. })
    ));
    assert!(matches!(
        query.ability_name(hero_ids::RAMATTRA, slots::SECONDARY_FIRE, None, "en-US"),
        Err(AbilityNameResolutionError::AmbiguousSlot { .. })
    ));
    assert!(matches!(
        query.resolve_ability_name(hero_ids::RAMATTRA, "en-US", "Void Barrier"),
        Err(AbilityNameResolutionError::AmbiguousName { .. })
    ));
}

#[test]
fn cooldown_calculations_are_unit_safe_bounded_and_non_mutating() {
    let catalog = catalog();
    let query = catalog.query();
    let reference = AbilityRef::new(
        HeroId::from(hero_ids::ANA),
        LogicalSlot::from(slots::ABILITY_1),
        None,
    );
    assert_eq!(query.cooldown(&reference).unwrap().value, 12.0);
    assert_eq!(
        query
            .effective_cooldown(&reference, CooldownPercentage::new(50.0).unwrap())
            .unwrap()
            .value,
        6.0
    );
    assert_eq!(
        query
            .effective_cooldown(&reference, CooldownPercentage::new(500.0).unwrap())
            .unwrap()
            .value,
        60.0
    );
    assert_eq!(
        query
            .required_cooldown_percentage(&reference, &seconds(3.0))
            .unwrap()
            .value(),
        25.0
    );
    assert_eq!(query.cooldown(&reference).unwrap().value, 12.0);
}

#[test]
fn cooldown_percentage_and_data_errors_never_default() {
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
    let missing = Hero::new(
        HeroId::new("missing"),
        names("Missing", "缺失", "heroes.missing"),
        vec![ability("No Cooldown", "无冷却", "ability1", None)],
        vec![evidence("heroes.missing")],
    );
    let wrong_type = ability("Text Cooldown", "文本冷却", "ability1", None).with_stat(
        StatKey::new("cooldown"),
        Fact::new(StatValue::Text("12".to_string()), vec![evidence("text")]),
    );
    let wrong_unit = ability("Meter Cooldown", "米冷却", "ability2", None).with_stat(
        StatKey::new("cooldown"),
        Fact::new(
            StatValue::Quantity(Quantity::new(12.0, Unit::new("meters")).unwrap()),
            vec![evidence("meters")],
        ),
    );
    let zero = ability("Zero Cooldown", "零冷却", "ability3", None).with_stat(
        StatKey::new("cooldown"),
        Fact::new(StatValue::Quantity(seconds(0.0)), vec![evidence("zero")]),
    );
    let edge = Hero::new(
        HeroId::new("edge"),
        names("Edge", "边界", "heroes.edge"),
        vec![wrong_type, wrong_unit, zero],
        vec![evidence("heroes.edge")],
    );
    let catalog = GameplayCatalog::new(identity(), vec![missing, edge]).unwrap();
    let query = catalog.query();
    let no = AbilityRef::new(HeroId::new("missing"), LogicalSlot::new("ability1"), None);
    assert!(matches!(
        query.cooldown(&no),
        Err(CooldownError::Missing { .. })
    ));
    for (slot, reason) in [
        ("ability1", CooldownNonApplicability::WrongValueType),
        (
            "ability2",
            CooldownNonApplicability::WrongUnit {
                actual: Unit::new("meters"),
            },
        ),
        ("ability3", CooldownNonApplicability::NonPositiveBase),
    ] {
        let reference = AbilityRef::new(HeroId::new("edge"), LogicalSlot::new(slot), None);
        assert!(
            matches!(query.cooldown(&reference), Err(CooldownError::NonApplicable { reason: actual, .. }) if actual == reason)
        );
    }
}

#[test]
fn builtin_query_and_locale_resolution_use_real_records_without_fabricated_facts() {
    let catalog = builtin().unwrap();
    let query = catalog.query();
    let ana = query.ability(hero_ids::ANA, slots::ABILITY_1).unwrap();
    assert_eq!(ana.name().value().get("en-US"), Some("Sleep Dart"));
    assert_eq!(
        query
            .ability_name(hero_ids::ANA, slots::ABILITY_1, None, "en-US")
            .unwrap(),
        "Sleep Dart"
    );
    assert_eq!(
        query
            .ability_name(hero_ids::ANA, slots::ABILITY_1, None, "zh-CN")
            .unwrap(),
        "麻醉镖"
    );
    assert_eq!(
        query
            .resolve_ability_name(hero_ids::ANA, "zh-CN", "麻醉镖")
            .unwrap()
            .slot()
            .as_str(),
        "ability1"
    );
    assert!(matches!(
        query.ability(hero_ids::RAMATTRA, slots::PRIMARY_FIRE),
        Err(GameplayQueryError::AmbiguousSlot { .. })
    ));
    let venture = AbilityRef::new(
        HeroId::from(hero_ids::VENTURE),
        LogicalSlot::from(slots::SECONDARY_FIRE),
        None,
    );
    assert!(matches!(
        query.cooldown(&venture),
        Err(CooldownError::Missing { .. })
    ));
}
