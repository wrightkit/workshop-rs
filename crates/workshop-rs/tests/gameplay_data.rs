use std::collections::BTreeSet;

use workshop_rs::gameplay::{
    Ability, AbilityId, AbilityVariant, EvidenceRef, Fact, GameplayCatalog, GameplayDataError,
    GameplayDatasetIdentity, Hero, HeroId, LocalizedText, LogicalSlot, Quantity, StatKey,
    StatValue, Unit,
};
use workshop_rs::gameplay_data::{GAMEPLAY_DATA, builtin, content_digest, load};

const SOURCE: &str = "workshop-data/workshop-data.json@d854bf01fc7bbf3b2169f67408c07a8da8989ad6";
const OFFICIAL_HERO_SOURCE: &str = "Blizzard Entertainment official Overwatch hero detail";
const OFFICIAL_PATCH_SOURCE: &str = "Blizzard Entertainment official Overwatch live patch notes";
const PATCH_URL: &str = "https://overwatch.blizzard.com/en-us/news/patch-notes/live/2026/6/";

fn evidence(locator: &str) -> EvidenceRef {
    EvidenceRef {
        source: SOURCE.to_string(),
        locator: locator.to_string(),
        note: Some("commitDate=2026-08-12".to_string()),
    }
}

fn names(name: &str, locator: &str) -> Fact<LocalizedText> {
    Fact::new(
        LocalizedText::new([("en-US".to_string(), name.to_string())]),
        vec![evidence(locator)],
    )
}

fn identity() -> GameplayDatasetIdentity {
    GameplayDatasetIdentity {
        dataset_id: "test-gameplay".to_string(),
        version: "test".to_string(),
        digest: "sha256:test".to_string(),
        source: SOURCE.to_string(),
        license: "MIT-compatible test data".to_string(),
        target: "test".to_string(),
        reviewed: true,
    }
}

fn assert_evidence(evidence: &[EvidenceRef], source: &str, locator: &str) {
    assert!(
        evidence
            .iter()
            .any(|item| item.source == source && item.locator == locator),
        "missing evidence {source} at {locator}"
    );
}

fn assert_quantity(fact: &Fact<StatValue>, value: f64, unit: &str) {
    match fact.value() {
        StatValue::Quantity(quantity) => {
            assert_eq!(quantity.value, value);
            assert_eq!(quantity.unit.as_str(), unit);
        }
        other => panic!("expected quantity, got {other:?}"),
    }
}

fn ability(id: &str, slot: &str, variant: Option<&str>) -> Ability {
    Ability::new(
        AbilityId::new(id),
        LogicalSlot::new(slot),
        variant.map(AbilityVariant::new),
        names(id, "data.heroes.test.ability1"),
        vec![evidence("data.heroes.test.ability1")],
    )
}

#[test]
fn embedded_catalog_covers_the_pinned_roster_and_named_slots() {
    let catalog = builtin().expect("embedded gameplay data should load");
    assert_eq!(catalog.heroes().len(), 53);

    let expected = [
        ("ana", &["ability1", "ability2", "ultimate"][..]),
        ("anran", &["ability1", "ability2", "ultimate"][..]),
        ("ashe", &["ability1", "ability2", "ultimate"][..]),
        ("baptiste", &["ability1", "ability2", "ultimate"][..]),
        (
            "bastion",
            &["primaryFire", "secondaryFire", "ability1", "ultimate"][..],
        ),
        (
            "brigitte",
            &[
                "secondaryFire",
                "ability1",
                "ability2",
                "ability3",
                "ultimate",
            ][..],
        ),
        ("cassidy", &["ability1", "ability2", "ultimate"][..]),
        (
            "dmon",
            &["secondaryFire", "ability1", "ability2", "ultimate"][..],
        ),
        (
            "domina",
            &[
                "primaryFire",
                "secondaryFire",
                "ability1",
                "ability2",
                "ultimate",
                "passive",
            ][..],
        ),
        (
            "doomfist",
            &["secondaryFire", "ability1", "ability2", "ultimate"][..],
        ),
        (
            "dva",
            &[
                "primaryFire",
                "secondaryFire",
                "ability1",
                "ability2",
                "ultimate",
            ][..],
        ),
        (
            "echo",
            &[
                "secondaryFire",
                "ability1",
                "ability2",
                "ultimate",
                "passive",
            ][..],
        ),
        ("emre", &["ability1", "ability2", "ultimate"][..]),
        (
            "freja",
            &["secondaryFire", "ability1", "ability2", "ultimate"][..],
        ),
        ("genji", &["ability1", "ability2", "ultimate"][..]),
        ("illari", &["ability1", "ability2", "ultimate"][..]),
        (
            "wreckingBall",
            &[
                "secondaryFire",
                "ability1",
                "ability2",
                "ability3",
                "ultimate",
            ][..],
        ),
        (
            "hanzo",
            &["ability1", "ability2", "ability3", "ultimate"][..],
        ),
        (
            "jetpackCat",
            &["secondaryFire", "ability1", "ability2", "ultimate"][..],
        ),
        (
            "junkerQueen",
            &["secondaryFire", "ability1", "ability2", "ultimate"][..],
        ),
        ("junkrat", &["ability1", "ability2", "ultimate"][..]),
        ("kiriko", &["ability1", "ability2", "ultimate"][..]),
        (
            "lucio",
            &["secondaryFire", "ability1", "ability2", "ultimate"][..],
        ),
        (
            "mauga",
            &[
                "primaryFire",
                "secondaryFire",
                "ability1",
                "ability2",
                "ultimate",
            ][..],
        ),
        ("mei", &["ability1", "ability2", "ultimate"][..]),
        (
            "mercy",
            &["ability1", "ability2", "ultimate", "passive"][..],
        ),
        (
            "mizuki",
            &["secondaryFire", "ability1", "ability2", "ultimate"][..],
        ),
        ("moira", &["ability1", "ability2", "ultimate"][..]),
        (
            "orisa",
            &["secondaryFire", "ability1", "ability2", "ultimate"][..],
        ),
        (
            "pharah",
            &[
                "secondaryFire",
                "ability1",
                "ability2",
                "ultimate",
                "passive",
            ][..],
        ),
        ("reaper", &["ability1", "ability2", "ultimate"][..]),
        (
            "reinhardt",
            &["secondaryFire", "ability1", "ability2", "ultimate"][..],
        ),
        ("roadhog", &["ability1", "ability2", "ultimate"][..]),
        (
            "shion",
            &["secondaryFire", "ability1", "ability2", "ultimate"][..],
        ),
        (
            "sierra",
            &["secondaryFire", "ability1", "ability2", "ultimate"][..],
        ),
        (
            "sigma",
            &["secondaryFire", "ability1", "ability2", "ultimate"][..],
        ),
        (
            "sojourn",
            &["secondaryFire", "ability1", "ability2", "ultimate"][..],
        ),
        (
            "soldier",
            &["secondaryFire", "ability1", "ability2", "ultimate"][..],
        ),
        (
            "sombra",
            &["secondaryFire", "ability1", "ability2", "ultimate"][..],
        ),
        ("symmetra", &["ability1", "ability2", "ultimate"][..]),
        ("torbjorn", &["ability1", "ability2", "ultimate"][..]),
        ("tracer", &["ability1", "ability2", "ultimate"][..]),
        ("widowmaker", &["ability1", "ability2", "ultimate"][..]),
        ("winston", &["ability1", "ability2", "ultimate"][..]),
        ("zarya", &["ability1", "ability2", "ultimate"][..]),
        ("zenyatta", &["ability1", "ability2", "ultimate"][..]),
        (
            "ramattra",
            &[
                "primaryFire",
                "secondaryFire",
                "ability1",
                "ability2",
                "ultimate",
            ][..],
        ),
        (
            "lifeweaver",
            &["secondaryFire", "ability1", "ability2", "ultimate"][..],
        ),
        (
            "venture",
            &["primaryFire", "secondaryFire", "ability1", "ultimate"][..],
        ),
        (
            "juno",
            &[
                "primaryFire",
                "secondaryFire",
                "ability1",
                "ability2",
                "ultimate",
                "passive",
            ][..],
        ),
        (
            "hazard",
            &[
                "primaryFire",
                "secondaryFire",
                "ability1",
                "ability2",
                "ultimate",
            ][..],
        ),
        (
            "wuyang",
            &["secondaryFire", "ability1", "ability2", "ultimate"][..],
        ),
        (
            "vendetta",
            &[
                "primaryFire",
                "secondaryFire",
                "ability1",
                "ability2",
                "ultimate",
            ][..],
        ),
    ];
    let expected_ids: BTreeSet<_> = expected.iter().map(|(hero_id, _)| *hero_id).collect();
    let actual_ids: BTreeSet<_> = catalog
        .heroes()
        .iter()
        .map(|hero| hero.id().as_str())
        .collect();
    assert_eq!(actual_ids, expected_ids);

    let ability_count: usize = catalog
        .heroes()
        .iter()
        .map(|hero| hero.abilities().len())
        .sum();
    let keyword_count = catalog
        .heroes()
        .iter()
        .flat_map(|hero| hero.abilities())
        .filter(|ability| ability.keywords().next().is_some())
        .count();
    let variant_count = catalog
        .heroes()
        .iter()
        .flat_map(|hero| hero.abilities())
        .filter(|ability| ability.variant().is_some())
        .count();
    let stat_count = catalog
        .heroes()
        .iter()
        .map(|hero| {
            hero.stats().count()
                + hero
                    .abilities()
                    .iter()
                    .map(|ability| ability.stats().count())
                    .sum::<usize>()
        })
        .sum::<usize>();
    assert_eq!(ability_count, 207);
    assert!(keyword_count > 0);
    assert!(variant_count > 0);
    assert!(stat_count > 0);

    for (hero_id, expected_slots) in expected {
        let hero = catalog.hero_by_id(hero_id).expect("expected hero identity");
        let actual_slots: BTreeSet<_> = hero
            .abilities()
            .iter()
            .map(|ability| ability.slot().as_str())
            .collect();
        let expected_slots: BTreeSet<_> = expected_slots.iter().copied().collect();
        assert_eq!(actual_slots, expected_slots, "slot topology for {hero_id}");
        let role = hero.role().expect("official role evidence for every hero");
        assert!(matches!(
            role.value().as_str(),
            "tank" | "damage" | "support"
        ));
        assert_eq!(role.evidence().len(), 1);
        for ability in hero.abilities() {
            assert!(ability.name().value().get("en-US").is_some());
        }
    }
}

#[test]
fn embedded_catalog_is_deterministic_and_preserves_representative_ids() {
    let first = builtin().unwrap();
    let second = load(GAMEPLAY_DATA).unwrap();
    let first_shape: Vec<_> = first
        .heroes()
        .iter()
        .map(|hero| {
            (
                hero.id().as_str().to_string(),
                hero.abilities()
                    .iter()
                    .map(|ability| {
                        (
                            ability.id().as_str().to_string(),
                            ability.slot().as_str().to_string(),
                        )
                    })
                    .collect::<Vec<_>>(),
            )
        })
        .collect();
    let second_shape: Vec<_> = second
        .heroes()
        .iter()
        .map(|hero| {
            (
                hero.id().as_str().to_string(),
                hero.abilities()
                    .iter()
                    .map(|ability| {
                        (
                            ability.id().as_str().to_string(),
                            ability.slot().as_str().to_string(),
                        )
                    })
                    .collect::<Vec<_>>(),
            )
        })
        .collect();
    assert_eq!(first_shape, second_shape);
    assert_eq!(first.identity(), second.identity());
    assert_eq!(
        first.identity().digest,
        content_digest(GAMEPLAY_DATA).unwrap()
    );

    assert_eq!(
        first.hero_by_id("ana").unwrap().abilities()[0]
            .id()
            .as_str(),
        "sleepDart"
    );
    assert_eq!(
        first
            .hero_by_id("brigitte")
            .unwrap()
            .ability_by_id(&AbilityId::new("shieldBash"))
            .unwrap()
            .slot()
            .as_str(),
        "ability3"
    );
    assert!(
        first
            .hero_by_id("ramattra")
            .unwrap()
            .ability_by_id(&AbilityId::new("voidBarrierOmnic"))
            .is_some()
    );
    assert!(
        first
            .hero_by_id("dva")
            .unwrap()
            .ability_by_id(&AbilityId::new("boosters"))
            .is_some()
    );
    assert!(
        first
            .hero_by_id("bastion")
            .unwrap()
            .ability_by_id(&AbilityId::new("reconfigure"))
            .is_some()
    );
}

#[test]
fn embedded_facts_have_representative_names_values_and_official_provenance() {
    let catalog = builtin().unwrap();

    let ana = catalog.hero_by_id("ana").unwrap();
    assert_eq!(ana.role().unwrap().value().as_str(), "support");
    for (id, name, keyword) in [
        ("sleepDart", "Sleep Dart", "crowdControl"),
        ("bioticGrenade", "Biotic Grenade", "healing"),
        ("nanoBoost", "Nano Boost", "buff"),
    ] {
        let ability = ana.ability_by_id(&AbilityId::new(id)).unwrap();
        assert_eq!(ability.name().value().get("en-US"), Some(name));
        assert!(ability.has_keyword(keyword));
        assert_evidence(
            ability.evidence(),
            OFFICIAL_HERO_SOURCE,
            "https://overwatch.blizzard.com/en-us/heroes/ana/",
        );
    }

    let brigitte = catalog.hero_by_id("brigitte").unwrap();
    assert_eq!(brigitte.role().unwrap().value().as_str(), "support");
    for (id, name, keyword) in [
        ("whipShot", "Whip Shot", "knockback"),
        ("shieldBash", "Shield Bash", "mobility"),
    ] {
        let ability = brigitte.ability_by_id(&AbilityId::new(id)).unwrap();
        assert_eq!(ability.name().value().get("en-US"), Some(name));
        assert!(ability.has_keyword(keyword));
        assert_evidence(
            ability.evidence(),
            OFFICIAL_HERO_SOURCE,
            "https://overwatch.blizzard.com/en-us/heroes/brigitte/",
        );
    }

    let ramattra = catalog.hero_by_id("ramattra").unwrap();
    assert_eq!(ramattra.role().unwrap().value().as_str(), "tank");
    let vortex = ramattra
        .ability_by_id(&AbilityId::new("ravenousVortex"))
        .unwrap();
    assert_eq!(vortex.name().value().get("en-US"), Some("Ravenous Vortex"));
    assert!(vortex.has_keyword("crowdControl"));
    assert_evidence(
        vortex.evidence(),
        OFFICIAL_HERO_SOURCE,
        "https://overwatch.blizzard.com/en-us/heroes/ramattra/",
    );
    for (id, variant, name) in [
        ("voidAccelerator", "omnic", "Void Accelerator"),
        ("pummel", "nemesis", "Pummel"),
    ] {
        let ability = ramattra.ability_by_id(&AbilityId::new(id)).unwrap();
        assert_eq!(ability.variant().unwrap().as_str(), variant);
        assert_eq!(ability.name().value().get("en-US"), Some(name));
        assert_evidence(
            ability.evidence(),
            OFFICIAL_HERO_SOURCE,
            "https://overwatch.blizzard.com/en-us/heroes/ramattra/",
        );
    }

    let dva = catalog.hero_by_id("dva").unwrap();
    assert_eq!(dva.role().unwrap().value().as_str(), "tank");
    let matrix = dva.ability_by_id(&AbilityId::new("defenseMatrix")).unwrap();
    assert!(matrix.has_keyword("barrier"));
    assert!(matrix.has_keyword("resource"));
    assert_evidence(
        matrix.evidence(),
        OFFICIAL_HERO_SOURCE,
        "https://overwatch.blizzard.com/en-us/heroes/dva/",
    );
    for (id, variant, name) in [
        ("fusionCannons", "mech", "Fusion Cannons"),
        ("lightGun", "pilot", "Light Gun"),
    ] {
        let ability = dva.ability_by_id(&AbilityId::new(id)).unwrap();
        assert_eq!(ability.variant().unwrap().as_str(), variant);
        assert_eq!(ability.name().value().get("en-US"), Some(name));
        assert_evidence(
            ability.evidence(),
            OFFICIAL_HERO_SOURCE,
            "https://overwatch.blizzard.com/en-us/heroes/dva/",
        );
    }

    let bastion = catalog.hero_by_id("bastion").unwrap();
    assert_eq!(bastion.role().unwrap().value().as_str(), "damage");
    let reconfigure = bastion
        .ability_by_id(&AbilityId::new("reconfigure"))
        .unwrap();
    assert!(reconfigure.has_keyword("form"));
    assert_evidence(
        reconfigure.evidence(),
        OFFICIAL_HERO_SOURCE,
        "https://overwatch.blizzard.com/en-us/heroes/bastion/",
    );
    for (id, variant, name) in [
        ("configurationAssault", "assault", "Configuration: Assault"),
        ("configurationRecon", "recon", "Configuration: Recon"),
    ] {
        let ability = bastion.ability_by_id(&AbilityId::new(id)).unwrap();
        assert_eq!(ability.variant().unwrap().as_str(), variant);
        assert_eq!(ability.name().value().get("en-US"), Some(name));
        assert_evidence(
            ability.evidence(),
            OFFICIAL_HERO_SOURCE,
            "https://overwatch.blizzard.com/en-us/heroes/bastion/",
        );
    }

    let venture = catalog.hero_by_id("venture").unwrap();
    assert_eq!(venture.role().unwrap().value().as_str(), "damage");
    let health = venture.stat(&StatKey::new("health")).unwrap();
    assert_quantity(health, 225.0, "health");
    assert_evidence(health.evidence(), OFFICIAL_PATCH_SOURCE, PATCH_URL);
    let drill_dash = venture.ability_by_id(&AbilityId::new("drillDash")).unwrap();
    assert!(drill_dash.has_keyword("mobility"));
    assert_evidence(
        drill_dash.evidence(),
        OFFICIAL_HERO_SOURCE,
        "https://overwatch.blizzard.com/en-us/heroes/venture/",
    );
    let cooldown = drill_dash.stat(&StatKey::new("cooldown")).unwrap();
    assert_quantity(cooldown, 6.0, "seconds");
    assert_evidence(cooldown.evidence(), OFFICIAL_PATCH_SOURCE, PATCH_URL);
    let damage = drill_dash.stat(&StatKey::new("damage")).unwrap();
    assert_quantity(damage, 35.0, "damage");
    assert_evidence(damage.evidence(), OFFICIAL_PATCH_SOURCE, PATCH_URL);
}

#[test]
fn embedded_records_keep_pinned_provenance_on_every_fact() {
    let catalog = builtin().unwrap();
    for hero in catalog.heroes() {
        for evidence in hero.evidence().iter().chain(hero.name().evidence()) {
            assert_eq!(evidence.source, SOURCE);
            assert_eq!(evidence.note.as_deref(), Some("commitDate=2026-08-12"));
        }
        let role = hero.role().expect("official role evidence for every hero");
        let role_evidence = &role.evidence()[0];
        assert_eq!(
            role_evidence.source,
            "Blizzard Entertainment official Overwatch hero detail"
        );
        assert!(
            role_evidence
                .locator
                .starts_with("https://overwatch.blizzard.com/en-us/heroes/")
        );
        assert_eq!(
            role_evidence.note.as_deref(),
            Some("role metadata; accessed 2026-08-18")
        );
        for ability in hero.abilities() {
            for evidence in ability.evidence().iter().chain(ability.name().evidence()) {
                if evidence.source == SOURCE {
                    assert!(evidence.locator.starts_with("data.heroes."));
                    assert_eq!(evidence.note.as_deref(), Some("commitDate=2026-08-12"));
                } else if evidence.source == OFFICIAL_HERO_SOURCE {
                    assert!(
                        evidence
                            .locator
                            .starts_with("https://overwatch.blizzard.com/en-us/heroes/")
                    );
                    assert_eq!(
                        evidence.note.as_deref(),
                        Some("official ability description; accessed 2026-08-18")
                    );
                } else {
                    panic!("unexpected ability evidence source: {}", evidence.source);
                }
            }
        }
    }
}

#[test]
fn loader_rejects_stale_digest_and_unsupported_schema() {
    let stale = GAMEPLAY_DATA.replacen(
        "d15bf17d413e7057bc7ef25e90a6e33df1a79e279a9dbff41e643a30fb9f7635",
        "e15bf17d413e7057bc7ef25e90a6e33df1a79e279a9dbff41e643a30fb9f7635",
        1,
    );
    assert!(matches!(
        load(&stale),
        Err(GameplayDataError::DigestMismatch { .. })
    ));

    let unsupported = GAMEPLAY_DATA.replacen("\"schemaVersion\": 1", "\"schemaVersion\": 2", 1);
    assert!(matches!(
        load(&unsupported),
        Err(GameplayDataError::UnsupportedSchema(2))
    ));
}

#[test]
fn catalog_rejects_invalid_slots_unqualified_variants_and_non_finite_values() {
    let empty_slot = Hero::new(
        HeroId::new("test"),
        names("Test", "data.heroes.test"),
        vec![ability("testAbility", " ", None)],
        vec![evidence("data.heroes.test")],
    );
    assert!(matches!(
        GameplayCatalog::new(identity(), vec![empty_slot]),
        Err(GameplayDataError::EmptyId("ability slot"))
    ));

    let future_slot = Hero::new(
        HeroId::new("test"),
        names("Test", "data.heroes.test"),
        vec![ability("testAbility", "futureSlot", None)],
        vec![evidence("data.heroes.test")],
    );
    assert!(GameplayCatalog::new(identity(), vec![future_slot]).is_ok());

    let unqualified = Hero::new(
        HeroId::new("test"),
        names("Test", "data.heroes.test"),
        vec![
            ability("one", "ability1", None),
            ability("two", "ability1", None),
        ],
        vec![evidence("data.heroes.test")],
    );
    assert!(matches!(
        GameplayCatalog::new(identity(), vec![unqualified]),
        Err(GameplayDataError::VariantRequired { .. })
    ));

    let non_finite = Hero::new(
        HeroId::new("test"),
        names("Test", "data.heroes.test"),
        vec![ability("testAbility", "ability1", None).with_stat(
            StatKey::new("damage"),
            Fact::new(
                StatValue::Quantity(Quantity {
                    value: f64::INFINITY,
                    unit: Unit::new("damage"),
                }),
                vec![evidence("data.heroes.test.ability1.damage")],
            ),
        )],
        vec![evidence("data.heroes.test")],
    );
    assert!(matches!(
        GameplayCatalog::new(identity(), vec![non_finite]),
        Err(GameplayDataError::InvalidQuantity { .. })
    ));
}

#[test]
fn every_embedded_ability_id_is_unique_per_hero() {
    let catalog = builtin().unwrap();
    for hero in catalog.heroes() {
        let ids: BTreeSet<_> = hero
            .abilities()
            .iter()
            .map(|ability| ability.id())
            .collect();
        assert_eq!(
            ids.len(),
            hero.abilities().len(),
            "ability IDs for {}",
            hero.id()
        );
    }
}
