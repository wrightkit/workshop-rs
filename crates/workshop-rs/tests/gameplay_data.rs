use std::collections::BTreeSet;

use workshop_rs::gameplay::data::{GAMEPLAY_DATA, builtin, builtin_ref, content_digest, load};
use workshop_rs::gameplay::{
    Ability, AbilityVariant, Fact, GameplayCatalog, GameplayDataError, GameplayDatasetIdentity,
    Hero, HeroId, LocalizedText, LogicalSlot, Quantity, SourceReference, StatKey, StatValue, Unit,
};

const SOURCE: &str = "workshop-data/workshop-data.json@5a7d0e294b8cad73b9701987bb584d0551d7fa4d";
const OFFICIAL_HERO_SOURCE: &str = "Blizzard Entertainment official Overwatch hero detail";

fn source_reference(locator: &str) -> SourceReference {
    SourceReference::new(SOURCE, locator).with_note("commitDate=2026-10-06")
}

fn names(name: &str, locator: &str) -> Fact<LocalizedText> {
    Fact::new(
        LocalizedText::new([("en-US".to_string(), name.to_string())]),
        vec![source_reference(locator)],
    )
}

fn identity() -> GameplayDatasetIdentity {
    GameplayDatasetIdentity::new(
        "test-gameplay",
        "test",
        "sha256:test",
        SOURCE,
        "MIT-compatible test data",
        "test",
        true,
    )
}

fn assert_sources(sources: &[SourceReference], source: &str, locator: &str) {
    assert!(
        sources
            .iter()
            .any(|item| item.source == source && item.locator == locator),
        "missing source {source} at {locator}"
    );
}

fn ability(name: &str, slot: &str, variant: Option<&str>) -> Ability {
    Ability::new(
        LogicalSlot::new(slot),
        variant.map(AbilityVariant::new),
        names(name, "data.heroes.test.ability1"),
        vec![source_reference("data.heroes.test.ability1")],
    )
}

#[test]
fn builtin_ref_returns_the_same_cached_instance() {
    let first = builtin_ref().expect("built-in gameplay data loads");
    let second = builtin_ref().expect("built-in gameplay data loads");
    assert!(std::ptr::eq(first, second));
}

#[test]
fn embedded_catalog_covers_the_pinned_roster_and_named_slots() {
    let catalog = builtin().expect("embedded gameplay data should load");
    assert!(!catalog.heroes().is_empty());

    // Representative slot topologies: basic kit, named fire slots, ability3,
    // passive, and the primary+secondary combination. The exact embedded
    // inventory is pinned by the dataset digest, not by this test.
    for (hero_id, expected_slots) in [
        ("ana", &["ability1", "ability2", "ultimate"][..]),
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
        (
            "hanzo",
            &["ability1", "ability2", "ability3", "ultimate"][..],
        ),
        (
            "mercy",
            &["ability1", "ability2", "ultimate", "passive"][..],
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
    ] {
        let hero = catalog.hero_by_id(hero_id).expect("expected hero identity");
        let actual_slots: BTreeSet<_> = hero
            .abilities()
            .iter()
            .map(|ability| ability.slot().as_str())
            .collect();
        let expected_slots: BTreeSet<_> = expected_slots.iter().copied().collect();
        assert_eq!(actual_slots, expected_slots, "slot topology for {hero_id}");
    }

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
    assert!(keyword_count > 0);
    assert!(variant_count > 0);

    let mut ids = BTreeSet::new();
    for hero in catalog.heroes() {
        assert!(
            ids.insert(hero.id().as_str()),
            "duplicate hero id {}",
            hero.id().as_str()
        );
        let role = hero.role().expect("official role sources for every hero");
        assert!(matches!(
            role.value().as_str(),
            "tank" | "damage" | "support"
        ));
        assert_eq!(role.sources().len(), 1);
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
                    .map(|ability| ability.slot().as_str().to_string())
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
                    .map(|ability| ability.slot().as_str().to_string())
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
            .slot()
            .as_str(),
        "ability1"
    );
    assert_eq!(
        first
            .hero_by_id("brigitte")
            .unwrap()
            .ability(&LogicalSlot::new("ability3"))
            .unwrap()
            .slot()
            .as_str(),
        "ability3"
    );
    assert!(
        first
            .hero_by_id("ramattra")
            .unwrap()
            .ability_variant(
                &LogicalSlot::new("primaryFire"),
                &AbilityVariant::new("omnic")
            )
            .is_ok()
    );
    assert!(
        first
            .hero_by_id("dva")
            .unwrap()
            .ability(&LogicalSlot::new("ability1"))
            .is_ok()
    );
    assert!(
        first
            .hero_by_id("bastion")
            .unwrap()
            .ability(&LogicalSlot::new("ability1"))
            .is_ok()
    );
}

#[test]
fn embedded_facts_have_representative_names_values_and_official_sources() {
    let catalog = builtin().unwrap();

    let ana = catalog.hero_by_id("ana").unwrap();
    assert_eq!(ana.role().unwrap().value().as_str(), "support");
    for (slot, name, keyword) in [
        ("ability1", "Sleep Dart", "crowdControl"),
        ("ability2", "Biotic Grenade", "healing"),
        ("ultimate", "Nano Boost", "buff"),
    ] {
        let ability = ana.ability(&LogicalSlot::new(slot)).unwrap();
        assert_eq!(ability.name().value().get("en-US"), Some(name));
        assert!(ability.has_keyword(keyword));
        assert_sources(
            ability.sources(),
            OFFICIAL_HERO_SOURCE,
            "https://overwatch.blizzard.com/en-us/heroes/ana/",
        );
    }

    let brigitte = catalog.hero_by_id("brigitte").unwrap();
    assert_eq!(brigitte.role().unwrap().value().as_str(), "support");
    for (slot, name, keyword) in [
        ("ability1", "Whip Shot", "knockback"),
        ("ability3", "Shield Bash", "mobility"),
    ] {
        let ability = brigitte.ability(&LogicalSlot::new(slot)).unwrap();
        assert_eq!(ability.name().value().get("en-US"), Some(name));
        assert!(ability.has_keyword(keyword));
        assert_sources(
            ability.sources(),
            OFFICIAL_HERO_SOURCE,
            "https://overwatch.blizzard.com/en-us/heroes/brigitte/",
        );
    }

    let ramattra = catalog.hero_by_id("ramattra").unwrap();
    assert_eq!(ramattra.role().unwrap().value().as_str(), "tank");
    let vortex = ramattra.ability(&LogicalSlot::new("ability2")).unwrap();
    assert_eq!(vortex.name().value().get("en-US"), Some("Ravenous Vortex"));
    assert!(vortex.has_keyword("crowdControl"));
    assert_sources(
        vortex.sources(),
        OFFICIAL_HERO_SOURCE,
        "https://overwatch.blizzard.com/en-us/heroes/ramattra/",
    );
    for (variant, name) in [("omnic", "Void Accelerator"), ("nemesis", "Pummel")] {
        let ability = ramattra
            .ability_variant(
                &LogicalSlot::new("primaryFire"),
                &AbilityVariant::new(variant),
            )
            .unwrap();
        assert_eq!(ability.variant().unwrap().as_str(), variant);
        assert_eq!(ability.name().value().get("en-US"), Some(name));
        assert_sources(
            ability.sources(),
            OFFICIAL_HERO_SOURCE,
            "https://overwatch.blizzard.com/en-us/heroes/ramattra/",
        );
    }

    let dva = catalog.hero_by_id("dva").unwrap();
    assert_eq!(dva.role().unwrap().value().as_str(), "tank");
    let matrix = dva.ability(&LogicalSlot::new("secondaryFire")).unwrap();
    assert!(matrix.has_keyword("barrier"));
    assert!(matrix.has_keyword("resource"));
    assert_sources(
        matrix.sources(),
        OFFICIAL_HERO_SOURCE,
        "https://overwatch.blizzard.com/en-us/heroes/dva/",
    );
    for (variant, name) in [("mech", "Fusion Cannons"), ("pilot", "Light Gun")] {
        let ability = dva
            .ability_variant(
                &LogicalSlot::new("primaryFire"),
                &AbilityVariant::new(variant),
            )
            .unwrap();
        assert_eq!(ability.variant().unwrap().as_str(), variant);
        assert_eq!(ability.name().value().get("en-US"), Some(name));
        assert_sources(
            ability.sources(),
            OFFICIAL_HERO_SOURCE,
            "https://overwatch.blizzard.com/en-us/heroes/dva/",
        );
    }

    let bastion = catalog.hero_by_id("bastion").unwrap();
    assert_eq!(bastion.role().unwrap().value().as_str(), "damage");
    let reconfigure = bastion.ability(&LogicalSlot::new("ability1")).unwrap();
    assert!(reconfigure.has_keyword("form"));
    assert_sources(
        reconfigure.sources(),
        OFFICIAL_HERO_SOURCE,
        "https://overwatch.blizzard.com/en-us/heroes/bastion/",
    );
    for (variant, name) in [
        ("assault", "Configuration: Assault"),
        ("recon", "Configuration: Recon"),
    ] {
        let ability = bastion
            .ability_variant(
                &LogicalSlot::new("primaryFire"),
                &AbilityVariant::new(variant),
            )
            .unwrap();
        assert_eq!(ability.variant().unwrap().as_str(), variant);
        assert_eq!(ability.name().value().get("en-US"), Some(name));
        assert_sources(
            ability.sources(),
            OFFICIAL_HERO_SOURCE,
            "https://overwatch.blizzard.com/en-us/heroes/bastion/",
        );
    }

    let venture = catalog.hero_by_id("venture").unwrap();
    assert_eq!(venture.role().unwrap().value().as_str(), "damage");
    assert!(venture.stat(&StatKey::new("health")).is_none());
    let drill_dash = venture.ability(&LogicalSlot::new("secondaryFire")).unwrap();
    assert!(drill_dash.has_keyword("mobility"));
    assert_sources(
        drill_dash.sources(),
        OFFICIAL_HERO_SOURCE,
        "https://overwatch.blizzard.com/en-us/heroes/venture/",
    );
    assert!(drill_dash.stat(&StatKey::new("cooldown")).is_none());
    assert!(drill_dash.stat(&StatKey::new("damage")).is_none());
}

#[test]
fn embedded_dmon_abilities_carry_the_verified_locale_names() {
    let catalog = builtin().unwrap();
    let dmon = catalog.hero_by_id("dmon").unwrap();
    for (slot, en, zh_cn, zh_tw) in [
        ("secondaryFire", "Power Barrier", "强力屏障", "能量屏障"),
        ("ability1", "Propulsors", "助推器", "推進器"),
        ("ability2", "Fusion Repeater", "聚变连发枪", "熱核連射砲"),
        ("ultimate", "Limit Break", "无极斩", "突破極限"),
    ] {
        let value = dmon
            .ability(&LogicalSlot::new(slot))
            .unwrap()
            .name()
            .value();
        assert_eq!(value.get("en-US"), Some(en));
        assert_eq!(value.get("zh-TW"), Some(zh_tw));
        assert_eq!(
            value.get("zh-CN"),
            Some(zh_cn),
            "official zh-CN name for dmon {slot}"
        );
    }
    assert_eq!(
        dmon.name().value().get("zh-CN"),
        Some("D.Mon"),
        "official zh-CN hero name for dmon"
    );
}

#[test]
fn embedded_doctrine_is_an_official_source_only_hero() {
    let catalog = builtin().unwrap();
    let doctrine = catalog.hero_by_id("doctrine").unwrap();
    assert_eq!(doctrine.role().unwrap().value().as_str(), "support");
    for (locale, name) in [("en-US", "Doctrine"), ("zh-CN", "血律"), ("zh-TW", "血律")] {
        assert_eq!(doctrine.name().value().get(locale), Some(name));
    }
    for (slot, en, zh_cn) in [
        ("primaryFire", "Eternal Scepter", "永生权杖"),
        ("secondaryFire", "Infuse", "灌注"),
        ("ability1", "Invigorating Drones", "焕生无人机"),
        ("ability2", "Shrouded Momentum", "迅影疾行"),
        ("ultimate", "Deliverance", "救赎恩典"),
    ] {
        let ability = doctrine.ability(&LogicalSlot::new(slot)).unwrap();
        assert_eq!(ability.name().value().get("en-US"), Some(en));
        assert_eq!(ability.name().value().get("zh-CN"), Some(zh_cn));
    }
    for source_record in doctrine.sources().iter().chain(doctrine.name().sources()) {
        assert_eq!(source_record.source, OFFICIAL_HERO_SOURCE);
    }
}

#[test]
fn official_zh_cn_names_fill_export_gaps() {
    let catalog = builtin().unwrap();
    for (hero_id, slot, variant, zh_cn) in [
        ("dva", "primaryFire", "mech", "聚变机炮"),
        ("dva", "primaryFire", "pilot", "光枪"),
        ("bastion", "primaryFire", "assault", "强攻模式"),
        ("bastion", "primaryFire", "recon", "侦察模式"),
        ("ramattra", "primaryFire", "omnic", "虚空加速器（智械形态）"),
        ("ramattra", "primaryFire", "nemesis", "猛拳（天罚形态）"),
    ] {
        let hero = catalog.hero_by_id(hero_id).unwrap();
        let ability = hero
            .ability_variant(&LogicalSlot::new(slot), &AbilityVariant::new(variant))
            .unwrap();
        assert_eq!(ability.name().value().get("zh-CN"), Some(zh_cn));
    }
    let sojourn = catalog.hero_by_id("sojourn").unwrap();
    assert_eq!(
        sojourn
            .ability(&LogicalSlot::new("secondaryFire"))
            .unwrap()
            .name()
            .value()
            .get("zh-CN"),
        Some("充能射击")
    );
}

#[test]
fn embedded_records_carry_well_formed_provenance_on_every_fact() {
    fn assert_well_formed(record: &SourceReference) {
        assert!(
            !record.source.is_empty(),
            "a provenance record names a source"
        );
        assert!(
            record.locator.starts_with("data.heroes.") || record.locator.starts_with("https://"),
            "unrecognized provenance locator: {}",
            record.locator
        );
    }
    let catalog = builtin().unwrap();
    for hero in catalog.heroes() {
        assert!(!hero.sources().is_empty(), "{} has no sources", hero.id());
        for record in hero.sources().iter().chain(hero.name().sources()) {
            assert_well_formed(record);
        }
        let role = hero.role().expect("official role sources for every hero");
        for record in role.sources() {
            assert_well_formed(record);
        }
        for ability in hero.abilities() {
            for record in ability.sources().iter().chain(ability.name().sources()) {
                assert_well_formed(record);
            }
        }
    }
}

#[test]
fn representative_hero_and_ability_records_round_trip_through_json() {
    let catalog = builtin().unwrap();
    let original = catalog.hero_by_id("ramattra").unwrap();
    let encoded = serde_json::to_string(original).unwrap();
    let decoded: Hero = serde_json::from_str(&encoded).unwrap();

    assert_eq!(original, &decoded);
    assert_eq!(
        decoded
            .ability_variant(
                &LogicalSlot::new("primaryFire"),
                &AbilityVariant::new("nemesis")
            )
            .unwrap()
            .variant()
            .unwrap()
            .as_str(),
        "nemesis"
    );
}

#[test]
fn loader_rejects_stale_digest_and_unsupported_schema() {
    let stale = GAMEPLAY_DATA.replacen(
        "0f80dcfbf4014ffe68d05ffba471d26e221ca6789407af6dc4c7eb601e284a87",
        "e15bf17d413e7057bc7ef25e90a6e33df1a79e279a9dbff41e643a30fb9f7635",
        1,
    );
    assert!(matches!(
        load(&stale),
        Err(GameplayDataError::DigestMismatch { .. })
    ));

    let unsupported = GAMEPLAY_DATA.replacen("\"schemaVersion\": 2", "\"schemaVersion\": 3", 1);
    assert!(matches!(
        load(&unsupported),
        Err(GameplayDataError::UnsupportedSchema(3))
    ));

    let legacy = GAMEPLAY_DATA
        .replace("\"sources\"", "\"evidence\"")
        .replacen("\"schemaVersion\": 2", "\"schemaVersion\": 1", 1);
    assert!(matches!(
        load(&legacy),
        Err(GameplayDataError::UnsupportedSchema(1))
    ));
}

#[test]
fn catalog_rejects_invalid_slots_unqualified_variants_and_non_finite_values() {
    let empty_slot = Hero::new(
        HeroId::new("test"),
        names("Test", "data.heroes.test"),
        vec![ability("testAbility", " ", None)],
        vec![source_reference("data.heroes.test")],
    );
    assert!(matches!(
        GameplayCatalog::new(identity(), vec![empty_slot]),
        Err(GameplayDataError::EmptyId("ability slot"))
    ));

    let future_slot = Hero::new(
        HeroId::new("test"),
        names("Test", "data.heroes.test"),
        vec![ability("testAbility", "futureSlot", None)],
        vec![source_reference("data.heroes.test")],
    );
    assert!(GameplayCatalog::new(identity(), vec![future_slot]).is_ok());

    let unqualified = Hero::new(
        HeroId::new("test"),
        names("Test", "data.heroes.test"),
        vec![
            ability("one", "ability1", None),
            ability("two", "ability1", Some("alternate")),
        ],
        vec![source_reference("data.heroes.test")],
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
                vec![source_reference("data.heroes.test.ability1.damage")],
            ),
        )],
        vec![source_reference("data.heroes.test")],
    );
    assert!(matches!(
        GameplayCatalog::new(identity(), vec![non_finite]),
        Err(GameplayDataError::InvalidQuantity { .. })
    ));
}

#[test]
fn every_embedded_ability_reference_is_unique_per_hero() {
    let catalog = builtin().unwrap();
    for hero in catalog.heroes() {
        let refs: BTreeSet<_> = hero
            .abilities()
            .iter()
            .map(|ability| (ability.slot().clone(), ability.variant().cloned()))
            .collect();
        assert_eq!(
            refs.len(),
            hero.abilities().len(),
            "ability references for {}",
            hero.id()
        );
    }
}
