use workshop_rs::gameplay::{
    Ability, AbilityLookupError, AbilityRef, AbilityVariant, EvidenceRef, Fact, GameplayCatalog,
    GameplayDataError, GameplayDatasetIdentity, Hero, HeroId, LocalizedText, LogicalSlot, Quantity,
    StatKey, StatValue, Unit, hero_ids, slots,
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

fn ability(slot: &str, variant: Option<&str>, name: &str) -> Ability {
    Ability::new(
        LogicalSlot::new(slot),
        variant.map(AbilityVariant::new),
        names(name, &format!("heroes.ana.{slot}")),
        vec![evidence(&format!("heroes.ana.{slot}"))],
    )
}

fn hero(id: &str, abilities: Vec<Ability>) -> Hero {
    Hero::new(
        HeroId::new(id),
        names(id, &format!("heroes.{id}")),
        abilities,
        vec![evidence(&format!("heroes.{id}"))],
    )
}

fn identity() -> GameplayDatasetIdentity {
    GameplayDatasetIdentity {
        dataset_id: "overwatch-workshop-hero-gameplay".to_string(),
        version: "2026-08-12".to_string(),
        digest: "sha256:test".to_string(),
        source: "workshop-data@d854bf01fc7bbf3b2169f67408c07a8da8989ad6".to_string(),
        license: "MIT-compatible user-provided export".to_string(),
        target: "Overwatch Workshop hero identity and gameplay facts".to_string(),
        reviewed: true,
    }
}

#[test]
fn open_ids_have_typed_constants_without_being_closed_enums() {
    assert_eq!(HeroId::from(hero_ids::ANA).as_str(), "ana");
    assert_eq!(LogicalSlot::from(slots::ABILITY_3).as_str(), "ability3");
    assert_eq!(HeroId::new("future-hero").as_str(), "future-hero");
}

#[test]
fn non_uniform_kits_and_explicit_variant_lookup_are_supported() {
    let brigitte = hero(
        "brigitte",
        vec![
            ability("ability1", None, "Whip Shot"),
            ability("ability3", None, "Shield Bash"),
        ],
    );
    let ramattra = hero(
        "ramattra",
        vec![
            ability("secondaryFire", Some("omnic"), "Void Barrier"),
            ability("secondaryFire", Some("nemesis"), "Void Barrier"),
        ],
    );
    let catalog = GameplayCatalog::new(identity(), vec![ramattra, brigitte]).unwrap();
    assert_eq!(catalog.heroes()[0].id().as_str(), "brigitte");
    assert_eq!(catalog.hero_by_id("ramattra").unwrap().abilities().len(), 2);
    assert!(matches!(
        catalog
            .hero_by_id("ramattra")
            .unwrap()
            .ability(&LogicalSlot::new("secondaryFire")),
        Err(AbilityLookupError::Ambiguous { .. })
    ));
    assert_eq!(
        catalog
            .hero_by_id("ramattra")
            .unwrap()
            .ability_variant(
                &LogicalSlot::from(slots::SECONDARY_FIRE),
                &AbilityVariant::new("nemesis")
            )
            .unwrap()
            .slot(),
        &LogicalSlot::from(slots::SECONDARY_FIRE)
    );
    assert_eq!(
        catalog
            .hero_by_id("brigitte")
            .unwrap()
            .ability(&LogicalSlot::from(slots::ABILITY_3))
            .unwrap()
            .slot()
            .as_str(),
        "ability3"
    );
}

#[test]
fn missing_and_unknown_data_are_explicit() {
    let catalog = GameplayCatalog::new(
        identity(),
        vec![hero("ana", vec![ability("ability1", None, "Sleep Dart")])],
    )
    .unwrap();
    assert!(catalog.hero_by_id("unknown-hero").is_none());
    assert!(
        catalog
            .hero_by_id("ana")
            .unwrap()
            .stat(&StatKey::new("health"))
            .is_none()
    );
    assert!(matches!(
        catalog
            .hero_by_id("ana")
            .unwrap()
            .ability(&LogicalSlot::from(slots::ULTIMATE)),
        Err(AbilityLookupError::Missing { .. })
    ));
}

#[test]
fn facts_carry_evidence_and_quantities_reject_non_finite_values() {
    let stat = Fact::new(
        StatValue::Quantity(Quantity::new(12.0, Unit::new("seconds")).unwrap()),
        vec![evidence("heroes.ana.ability1.cooldown")],
    );
    let sleep = ability("ability1", None, "Sleep Dart")
        .with_keyword("crowd-control")
        .with_stat(StatKey::new("cooldown"), stat);
    let catalog = GameplayCatalog::new(identity(), vec![hero("ana", vec![sleep])]).unwrap();
    let cooldown = catalog
        .hero_by_id("ana")
        .unwrap()
        .ability(&LogicalSlot::from(slots::ABILITY_1))
        .unwrap()
        .stat(&StatKey::new("cooldown"))
        .unwrap();
    assert_eq!(
        cooldown.evidence()[0].locator,
        "heroes.ana.ability1.cooldown"
    );
    assert!(Quantity::new(f64::NAN, Unit::new("seconds")).is_err());
}

#[test]
fn gameplay_serialization_preserves_dataset_identity_and_open_data() {
    let catalog = GameplayCatalog::new(identity(), vec![hero("dva", vec![])]).unwrap();
    let encoded = serde_json::to_string(catalog.identity()).unwrap();
    assert!(encoded.contains("datasetId"));
    assert!(encoded.contains("2026-08-12"));
    assert_eq!(
        catalog.identity().dataset_id,
        "overwatch-workshop-hero-gameplay"
    );
}

#[test]
fn duplicate_slot_variant_is_rejected() {
    let ana = hero(
        "ana",
        vec![
            ability("ability1", Some("default"), "Sleep Dart"),
            ability("ability1", Some("default"), "Sleep Dart"),
        ],
    );
    assert!(GameplayCatalog::new(identity(), vec![ana]).is_err());
}

#[test]
fn multiple_entries_require_variants_and_display_names_are_not_identity() {
    let missing_variant = hero(
        "ramattra",
        vec![
            ability("secondaryFire", None, "Void Barrier"),
            ability("secondaryFire", Some("nemesis"), "Void Barrier"),
        ],
    );
    assert!(matches!(
        GameplayCatalog::new(identity(), vec![missing_variant]),
        Err(GameplayDataError::VariantRequired { .. })
    ));
    let first = ability("ability1", None, "Sleep Dart").reference(&HeroId::new("ana"));
    let renamed = ability("ability1", None, "麻醉镖").reference(&HeroId::new("ana"));
    assert_eq!(first, renamed);
    let explicit = AbilityRef::new(
        HeroId::new("ana"),
        LogicalSlot::from(slots::ABILITY_1),
        None,
    );
    assert_eq!(explicit, first);
    let encoded = serde_json::to_value(&first).unwrap();
    assert_eq!(encoded["hero"], "ana");
    assert_eq!(encoded["slot"], "ability1");
    assert!(encoded.get("name").is_none());
    assert!(
        serde_json::from_value::<AbilityRef>(serde_json::json!({
            "hero": "ana",
            "slot": "ability1",
            "name": "Sleep Dart"
        }))
        .is_err()
    );
    assert!(
        serde_json::from_value::<AbilityRef>(serde_json::json!({
            "hero": "ana",
            "slot": "ability1",
            "abilityId": "sleepDart"
        }))
        .is_err()
    );
}

#[test]
fn empty_id_and_empty_evidence_are_rejected() {
    let empty_id = Hero::new(
        HeroId::new(""),
        names("Unnamed", "heroes.unnamed"),
        vec![],
        vec![evidence("heroes.unnamed")],
    );
    assert!(GameplayCatalog::new(identity(), vec![empty_id]).is_err());
    let empty_evidence = Hero::new(
        HeroId::new("ana"),
        Fact::new(
            LocalizedText::new([("en-US".to_string(), "Ana".to_string())]),
            vec![EvidenceRef {
                source: String::new(),
                locator: "heroes.ana".to_string(),
                note: None,
            }],
        ),
        vec![],
        vec![evidence("heroes.ana")],
    );
    assert!(GameplayCatalog::new(identity(), vec![empty_evidence]).is_err());
}
