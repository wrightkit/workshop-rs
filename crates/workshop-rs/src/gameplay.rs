//! Canonical hero and ability gameplay data.
//!
//! This module owns the data contract used by gameplay-aware tooling. It is
//! deliberately independent from the Workshop [`crate::catalog`] identity
//! and from any source-language provider. Ability identity is the open
//! `hero + logical slot + optional hero-local variant` tuple; display names
//! are localized, evidence-backed metadata and are not semantic identity.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use serde::{Deserialize, Deserializer, Serialize};

/// A canonical hero identity. The value is stable within the gameplay data
/// contract and is not a closed Rust enum.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct HeroId(String);

impl HeroId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
    pub const fn from_static(value: &'static str) -> HeroIdRef {
        HeroIdRef(value)
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<HeroIdRef> for HeroId {
    fn from(value: HeroIdRef) -> Self {
        Self::new(value.0)
    }
}

impl From<&str> for HeroId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl std::fmt::Display for HeroId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A typed constant reference for a canonical hero identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HeroIdRef(&'static str);

impl HeroIdRef {
    pub const fn new(value: &'static str) -> Self {
        Self(value)
    }
    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

impl AsRef<str> for HeroIdRef {
    fn as_ref(&self) -> &str {
        self.0
    }
}

/// Canonical hero identity constants for the current roster. The identity
/// remains open; these symbols are ergonomic accessors, not a closed enum.
pub mod hero_ids {
    use super::HeroIdRef;
    pub const ANA: HeroIdRef = HeroIdRef::new("ana");
    pub const ANRAN: HeroIdRef = HeroIdRef::new("anran");
    pub const ASHE: HeroIdRef = HeroIdRef::new("ashe");
    pub const BAPTISTE: HeroIdRef = HeroIdRef::new("baptiste");
    pub const BASTION: HeroIdRef = HeroIdRef::new("bastion");
    pub const BRIGITTE: HeroIdRef = HeroIdRef::new("brigitte");
    pub const CASSIDY: HeroIdRef = HeroIdRef::new("cassidy");
    pub const DMON: HeroIdRef = HeroIdRef::new("dmon");
    pub const DOMINA: HeroIdRef = HeroIdRef::new("domina");
    pub const DOOMFIST: HeroIdRef = HeroIdRef::new("doomfist");
    pub const DVA: HeroIdRef = HeroIdRef::new("dva");
    pub const ECHO: HeroIdRef = HeroIdRef::new("echo");
    pub const EMRE: HeroIdRef = HeroIdRef::new("emre");
    pub const FREJA: HeroIdRef = HeroIdRef::new("freja");
    pub const GENJI: HeroIdRef = HeroIdRef::new("genji");
    pub const ILLARI: HeroIdRef = HeroIdRef::new("illari");
    pub const WRECKING_BALL: HeroIdRef = HeroIdRef::new("wreckingBall");
    pub const HANZO: HeroIdRef = HeroIdRef::new("hanzo");
    pub const JETPACK_CAT: HeroIdRef = HeroIdRef::new("jetpackCat");
    pub const JUNKER_QUEEN: HeroIdRef = HeroIdRef::new("junkerQueen");
    pub const JUNKRAT: HeroIdRef = HeroIdRef::new("junkrat");
    pub const KIRIKO: HeroIdRef = HeroIdRef::new("kiriko");
    pub const LUCIO: HeroIdRef = HeroIdRef::new("lucio");
    pub const MAUGA: HeroIdRef = HeroIdRef::new("mauga");
    pub const MEI: HeroIdRef = HeroIdRef::new("mei");
    pub const MERCY: HeroIdRef = HeroIdRef::new("mercy");
    pub const MIZUKI: HeroIdRef = HeroIdRef::new("mizuki");
    pub const MOIRA: HeroIdRef = HeroIdRef::new("moira");
    pub const ORISA: HeroIdRef = HeroIdRef::new("orisa");
    pub const PHARAH: HeroIdRef = HeroIdRef::new("pharah");
    pub const REAPER: HeroIdRef = HeroIdRef::new("reaper");
    pub const REINHARDT: HeroIdRef = HeroIdRef::new("reinhardt");
    pub const ROADHOG: HeroIdRef = HeroIdRef::new("roadhog");
    pub const SHION: HeroIdRef = HeroIdRef::new("shion");
    pub const SIERRA: HeroIdRef = HeroIdRef::new("sierra");
    pub const SIGMA: HeroIdRef = HeroIdRef::new("sigma");
    pub const SOJOURN: HeroIdRef = HeroIdRef::new("sojourn");
    pub const SOLDIER: HeroIdRef = HeroIdRef::new("soldier");
    pub const SOMBRA: HeroIdRef = HeroIdRef::new("sombra");
    pub const SYMMETRA: HeroIdRef = HeroIdRef::new("symmetra");
    pub const TORBJORN: HeroIdRef = HeroIdRef::new("torbjorn");
    pub const TRACER: HeroIdRef = HeroIdRef::new("tracer");
    pub const WIDOWMAKER: HeroIdRef = HeroIdRef::new("widowmaker");
    pub const WINSTON: HeroIdRef = HeroIdRef::new("winston");
    pub const ZARYA: HeroIdRef = HeroIdRef::new("zarya");
    pub const ZENYATTA: HeroIdRef = HeroIdRef::new("zenyatta");
    pub const RAMATTRA: HeroIdRef = HeroIdRef::new("ramattra");
    pub const LIFEWEAVER: HeroIdRef = HeroIdRef::new("lifeweaver");
    pub const VENTURE: HeroIdRef = HeroIdRef::new("venture");
    pub const JUNO: HeroIdRef = HeroIdRef::new("juno");
    pub const HAZARD: HeroIdRef = HeroIdRef::new("hazard");
    pub const WUYANG: HeroIdRef = HeroIdRef::new("wuyang");
    pub const VENDETTA: HeroIdRef = HeroIdRef::new("vendetta");
}

macro_rules! open_string_id {
    ($name:ident, $reference:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
        pub struct $name(String);
        impl $name {
            pub fn new(value: impl Into<String>) -> Self {
                Self(value.into())
            }
            pub const fn from_static(value: &'static str) -> $reference {
                $reference(value)
            }
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
        impl From<$reference> for $name {
            fn from(value: $reference) -> Self {
                Self::new(value.0)
            }
        }
        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self::new(value)
            }
        }
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(self.as_str())
            }
        }
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub struct $reference(&'static str);
        impl $reference {
            pub const fn new(value: &'static str) -> Self {
                Self(value)
            }
            pub const fn as_str(self) -> &'static str {
                self.0
            }
        }
        impl AsRef<str> for $reference {
            fn as_ref(&self) -> &str {
                self.0
            }
        }
    };
}

open_string_id!(LogicalSlot, LogicalSlotRef);
open_string_id!(AbilityVariant, AbilityVariantRef);
open_string_id!(KeywordId, KeywordIdRef);
open_string_id!(StatKey, StatKeyRef);
open_string_id!(Unit, UnitRef);
open_string_id!(HeroRole, HeroRoleRef);

/// The canonical, serializable identity of an ability record.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct AbilityRef {
    hero: HeroId,
    slot: LogicalSlot,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    variant: Option<AbilityVariant>,
}

impl AbilityRef {
    pub fn new(hero: HeroId, slot: LogicalSlot, variant: Option<AbilityVariant>) -> Self {
        Self {
            hero,
            slot,
            variant,
        }
    }
    pub fn hero(&self) -> &HeroId {
        &self.hero
    }
    pub fn slot(&self) -> &LogicalSlot {
        &self.slot
    }
    pub fn variant(&self) -> Option<&AbilityVariant> {
        self.variant.as_ref()
    }
}

/// Typed constants for stable logical slot classifications.
pub mod slots {
    use super::LogicalSlotRef;
    pub const PRIMARY_FIRE: LogicalSlotRef = LogicalSlotRef::new("primaryFire");
    pub const SECONDARY_FIRE: LogicalSlotRef = LogicalSlotRef::new("secondaryFire");
    pub const ABILITY_1: LogicalSlotRef = LogicalSlotRef::new("ability1");
    pub const ABILITY_2: LogicalSlotRef = LogicalSlotRef::new("ability2");
    pub const ABILITY_3: LogicalSlotRef = LogicalSlotRef::new("ability3");
    pub const ULTIMATE: LogicalSlotRef = LogicalSlotRef::new("ultimate");
    pub const PASSIVE: LogicalSlotRef = LogicalSlotRef::new("passive");
}

/// Common stat identity constants. Long-tail stats remain open string IDs.
pub mod stat_keys {
    use super::StatKeyRef;
    pub const COOLDOWN: StatKeyRef = StatKeyRef::new("cooldown");
    pub const DAMAGE: StatKeyRef = StatKeyRef::new("damage");
    pub const HEALING: StatKeyRef = StatKeyRef::new("healing");
    pub const DURATION: StatKeyRef = StatKeyRef::new("duration");
    pub const CHARGES: StatKeyRef = StatKeyRef::new("charges");
    pub const RESOURCE_COST: StatKeyRef = StatKeyRef::new("resourceCost");
}

/// Common unit identity constants. New units can be represented without an enum change.
pub mod units {
    use super::UnitRef;
    pub const SECONDS: UnitRef = UnitRef::new("seconds");
    pub const PERCENT: UnitRef = UnitRef::new("percent");
    pub const HEALTH: UnitRef = UnitRef::new("health");
    pub const DAMAGE: UnitRef = UnitRef::new("damage");
    pub const HEALING: UnitRef = UnitRef::new("healing");
    pub const METERS: UnitRef = UnitRef::new("meters");
    pub const AMMO: UnitRef = UnitRef::new("ammo");
    pub const CHARGES: UnitRef = UnitRef::new("charges");
    pub const RESOURCE: UnitRef = UnitRef::new("resource");
}

/// A deterministic set of localized display strings.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct LocalizedText(BTreeMap<String, String>);

impl LocalizedText {
    pub fn new(values: impl IntoIterator<Item = (String, String)>) -> Self {
        Self(values.into_iter().collect())
    }
    pub fn get(&self, locale: &str) -> Option<&str> {
        self.0.get(locale).map(String::as_str).or_else(|| {
            self.0
                .iter()
                .find(|(known, _)| known.eq_ignore_ascii_case(locale))
                .map(|(_, text)| text.as_str())
        })
    }
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.0
            .iter()
            .map(|(locale, text)| (locale.as_str(), text.as_str()))
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// A machine-identifiable evidence reference for a gameplay fact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceRef {
    pub source: String,
    pub locator: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// Identity and provenance of a gameplay dataset. This is distinct from the Workshop parser/catalog dataset identity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameplayDatasetIdentity {
    pub dataset_id: String,
    pub version: String,
    pub digest: String,
    pub source: String,
    pub license: String,
    pub target: String,
    pub reviewed: bool,
}

/// A gameplay fact tied to evidence in the dataset version being consumed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Fact<T> {
    pub value: T,
    pub evidence: Vec<EvidenceRef>,
}

impl<T> Fact<T> {
    pub fn new(value: T, evidence: Vec<EvidenceRef>) -> Self {
        Self { value, evidence }
    }
    pub fn value(&self) -> &T {
        &self.value
    }
    pub fn evidence(&self) -> &[EvidenceRef] {
        &self.evidence
    }
}

/// A finite numeric quantity with an explicit unit.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Quantity {
    pub value: f64,
    pub unit: Unit,
}

impl Quantity {
    pub fn new(value: f64, unit: Unit) -> Result<Self, GameplayDataError> {
        if !value.is_finite() {
            return Err(GameplayDataError::InvalidQuantity { value });
        }
        Ok(Self { value, unit })
    }
}

impl<'de> Deserialize<'de> for Quantity {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawQuantity {
            value: f64,
            unit: Unit,
        }
        let raw = RawQuantity::deserialize(deserializer)?;
        Self::new(raw.value, raw.unit).map_err(serde::de::Error::custom)
    }
}

/// A typed common or extensible gameplay stat value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "kind", content = "value")]
pub enum StatValue {
    Quantity(Quantity),
    Text(String),
    Boolean(bool),
    Choice(String),
}

/// An ability record in a logical slot. The hero is supplied by its parent Hero record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ability {
    slot: LogicalSlot,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    variant: Option<AbilityVariant>,
    name: Fact<LocalizedText>,
    #[serde(default)]
    keywords: BTreeSet<KeywordId>,
    #[serde(default)]
    stats: BTreeMap<StatKey, Fact<StatValue>>,
    evidence: Vec<EvidenceRef>,
}

impl Ability {
    pub fn new(
        slot: LogicalSlot,
        variant: Option<AbilityVariant>,
        name: Fact<LocalizedText>,
        evidence: Vec<EvidenceRef>,
    ) -> Self {
        Self {
            slot,
            variant,
            name,
            keywords: BTreeSet::new(),
            stats: BTreeMap::new(),
            evidence,
        }
    }
    pub fn with_keyword(mut self, keyword: impl Into<KeywordId>) -> Self {
        self.keywords.insert(keyword.into());
        self
    }
    pub fn with_stat(mut self, key: StatKey, value: Fact<StatValue>) -> Self {
        self.stats.insert(key, value);
        self
    }
    pub fn reference(&self, hero: &HeroId) -> AbilityRef {
        AbilityRef::new(hero.clone(), self.slot.clone(), self.variant.clone())
    }
    pub fn slot(&self) -> &LogicalSlot {
        &self.slot
    }
    pub fn variant(&self) -> Option<&AbilityVariant> {
        self.variant.as_ref()
    }
    pub fn name(&self) -> &Fact<LocalizedText> {
        &self.name
    }
    pub fn keywords(&self) -> impl Iterator<Item = &KeywordId> {
        self.keywords.iter()
    }
    pub fn has_keyword(&self, keyword: &str) -> bool {
        self.keywords.iter().any(|known| known.as_str() == keyword)
    }
    pub fn stat(&self, key: &StatKey) -> Option<&Fact<StatValue>> {
        self.stats.get(key)
    }
    pub fn stats(&self) -> impl Iterator<Item = (&StatKey, &Fact<StatValue>)> {
        self.stats.iter()
    }
    pub fn evidence(&self) -> &[EvidenceRef] {
        &self.evidence
    }
}

/// A hero record with a non-uniform ability kit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Hero {
    id: HeroId,
    name: Fact<LocalizedText>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    role: Option<Fact<HeroRole>>,
    #[serde(default)]
    stats: BTreeMap<StatKey, Fact<StatValue>>,
    abilities: Vec<Ability>,
    evidence: Vec<EvidenceRef>,
}

impl Hero {
    pub fn new(
        id: HeroId,
        name: Fact<LocalizedText>,
        abilities: Vec<Ability>,
        evidence: Vec<EvidenceRef>,
    ) -> Self {
        Self {
            id,
            name,
            role: None,
            stats: BTreeMap::new(),
            abilities,
            evidence,
        }
    }
    pub fn with_role(mut self, role: Fact<HeroRole>) -> Self {
        self.role = Some(role);
        self
    }
    pub fn with_stat(mut self, key: StatKey, value: Fact<StatValue>) -> Self {
        self.stats.insert(key, value);
        self
    }
    pub fn id(&self) -> &HeroId {
        &self.id
    }
    pub fn name(&self) -> &Fact<LocalizedText> {
        &self.name
    }
    pub fn role(&self) -> Option<&Fact<HeroRole>> {
        self.role.as_ref()
    }
    pub fn stat(&self, key: &StatKey) -> Option<&Fact<StatValue>> {
        self.stats.get(key)
    }
    pub fn stats(&self) -> impl Iterator<Item = (&StatKey, &Fact<StatValue>)> {
        self.stats.iter()
    }
    pub fn abilities(&self) -> &[Ability] {
        &self.abilities
    }
    pub fn abilities_in_slot(&self, slot: &LogicalSlot) -> Vec<&Ability> {
        self.abilities
            .iter()
            .filter(|ability| ability.slot() == slot)
            .collect()
    }
    pub fn ability(&self, slot: &LogicalSlot) -> Result<&Ability, AbilityLookupError> {
        let matches = self.abilities_in_slot(slot);
        match matches.as_slice() {
            [] => Err(AbilityLookupError::Missing {
                hero: self.id.clone(),
                slot: slot.clone(),
            }),
            [ability] => Ok(ability),
            _ => Err(AbilityLookupError::Ambiguous {
                hero: self.id.clone(),
                slot: slot.clone(),
                candidates: matches
                    .into_iter()
                    .map(|ability| ability.reference(&self.id))
                    .collect(),
            }),
        }
    }
    pub fn ability_ref(
        &self,
        slot: &LogicalSlot,
        variant: Option<&AbilityVariant>,
    ) -> Result<&Ability, AbilityLookupError> {
        match variant {
            Some(variant) => self.ability_variant(slot, variant),
            None => self.ability(slot),
        }
    }
    pub fn ability_variant(
        &self,
        slot: &LogicalSlot,
        variant: &AbilityVariant,
    ) -> Result<&Ability, AbilityLookupError> {
        self.abilities
            .iter()
            .find(|ability| ability.slot() == slot && ability.variant.as_ref() == Some(variant))
            .ok_or_else(|| AbilityLookupError::MissingVariant {
                hero: self.id.clone(),
                slot: slot.clone(),
                variant: variant.clone(),
            })
    }
    pub fn evidence(&self) -> &[EvidenceRef] {
        &self.evidence
    }
}

/// Explicit failure for a logical-slot lookup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AbilityLookupError {
    Missing {
        hero: HeroId,
        slot: LogicalSlot,
    },
    Ambiguous {
        hero: HeroId,
        slot: LogicalSlot,
        candidates: Vec<AbilityRef>,
    },
    MissingVariant {
        hero: HeroId,
        slot: LogicalSlot,
        variant: AbilityVariant,
    },
}

impl std::fmt::Display for AbilityLookupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Missing { hero, slot } => {
                write!(f, "hero '{hero}' has no ability in slot '{slot}'")
            }
            Self::Ambiguous {
                hero,
                slot,
                candidates,
            } => write!(
                f,
                "hero '{hero}' has multiple abilities in slot '{slot}': {candidates:?}"
            ),
            Self::MissingVariant {
                hero,
                slot,
                variant,
            } => write!(
                f,
                "hero '{hero}' has no ability in slot '{slot}' with variant '{variant}'"
            ),
        }
    }
}
impl std::error::Error for AbilityLookupError {}

/// Validation and construction errors for gameplay data.
#[derive(Debug, Clone, PartialEq)]
pub enum GameplayDataError {
    EmptyIdentity(&'static str),
    DuplicateHero(HeroId),
    DuplicateSlotVariant {
        hero: HeroId,
        slot: LogicalSlot,
        variant: Option<AbilityVariant>,
    },
    VariantRequired {
        hero: HeroId,
        slot: LogicalSlot,
    },
    MissingEvidence(String),
    EmptyId(&'static str),
    InvalidQuantity {
        value: f64,
    },
    MissingEvidence(String),
    EmptyId(&'static str),
    InvalidQuantity { value: f64 },
    Malformed(String),
    UnsupportedSchema(u32),
    DigestMismatch { declared: String, computed: String },
}

impl std::fmt::Display for GameplayDataError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyIdentity(field) => {
                write!(f, "gameplay dataset identity field '{field}' is empty")
            }
            Self::DuplicateHero(id) => write!(f, "duplicate hero identity '{id}'"),
            Self::DuplicateSlotVariant {
                hero,
                slot,
                variant,
            } => write!(
                f,
                "hero '{hero}' has duplicate slot/variant '{slot}'/'{variant:?}'"
            ),
            Self::VariantRequired { hero, slot } => write!(
                f,
                "hero '{hero}' has multiple abilities in slot '{slot}' but not every record has a variant"
            ),
            Self::MissingEvidence(path) => write!(f, "gameplay fact '{path}' has no evidence"),
            Self::EmptyId(field) => write!(f, "gameplay identity '{field}' is empty"),
            Self::InvalidQuantity { value } => write!(f, "quantity value '{value}' is not finite"),
            Self::Malformed(message) => write!(f, "malformed gameplay data: {message}"),
            Self::UnsupportedSchema(version) => {
                write!(f, "unsupported gameplay data schemaVersion {version}")
            }
            Self::DigestMismatch { declared, computed } => write!(
                f,
                "gameplay data digest mismatch: declared '{declared}', content '{computed}'"
            ),
        }
    }
}
impl std::error::Error for GameplayDataError {}

/// The validated gameplay dataset and its lookup indexes.
#[derive(Debug, Clone)]
pub struct GameplayCatalog {
    identity: GameplayDatasetIdentity,
    heroes: Vec<Hero>,
    by_id: HashMap<HeroId, usize>,
}

impl GameplayCatalog {
    pub fn new(
        identity: GameplayDatasetIdentity,
        mut heroes: Vec<Hero>,
    ) -> Result<Self, GameplayDataError> {
        for (field, value) in [
            ("datasetId", identity.dataset_id.as_str()),
            ("version", identity.version.as_str()),
            ("digest", identity.digest.as_str()),
            ("source", identity.source.as_str()),
            ("license", identity.license.as_str()),
            ("target", identity.target.as_str()),
        ] {
            if value.is_empty() {
                return Err(GameplayDataError::EmptyIdentity(field));
            }
        }
        heroes.sort_by(|left, right| left.id.cmp(&right.id));
        for hero in &mut heroes {
            hero.abilities.sort_by(|left, right| {
                (&left.slot, &left.variant).cmp(&(&right.slot, &right.variant))
            });
        }
        let mut by_id = HashMap::with_capacity(heroes.len());
        for (index, hero) in heroes.iter().enumerate() {
            if by_id.insert(hero.id.clone(), index).is_some() {
                return Err(GameplayDataError::DuplicateHero(hero.id.clone()));
            }
            validate_hero(hero)?;
        }
        Ok(Self {
            identity,
            heroes,
            by_id,
        })
    }
    pub fn identity(&self) -> &GameplayDatasetIdentity {
        &self.identity
    }
    pub fn heroes(&self) -> &[Hero] {
        &self.heroes
    }
    pub fn hero(&self, id: &HeroId) -> Option<&Hero> {
        self.by_id.get(id).map(|index| &self.heroes[*index])
    }
    pub fn hero_by_id(&self, id: impl AsRef<str>) -> Option<&Hero> {
        self.hero(&HeroId::new(id.as_ref()))
    }
    pub fn ability(&self, reference: &AbilityRef) -> Result<&Ability, AbilityLookupError> {
        self.hero(reference.hero())
            .ok_or_else(|| AbilityLookupError::Missing {
                hero: reference.hero().clone(),
                slot: reference.slot().clone(),
            })?
            .ability_ref(reference.slot(), reference.variant())
    }
    pub fn find_abilities_by_keyword(&self, keyword: &str) -> Vec<(&Hero, &Ability)> {
        self.heroes
            .iter()
            .flat_map(|hero| {
                hero.abilities()
                    .iter()
                    .filter(move |ability| ability.has_keyword(keyword))
                    .map(move |ability| (hero, ability))
            })
            .collect()
    }
}

fn validate_hero(hero: &Hero) -> Result<(), GameplayDataError> {
    if hero.id.as_str().is_empty() {
        return Err(GameplayDataError::EmptyId("hero"));
    }
    if hero.evidence.is_empty() {
        return Err(GameplayDataError::MissingEvidence(format!(
            "hero {}",
            hero.id
        )));
    }
    if hero.name.evidence.is_empty() {
        return Err(GameplayDataError::MissingEvidence(format!(
            "hero {} name",
            hero.id
        )));
    }
    validate_evidence(&format!("hero {}", hero.id), &hero.evidence)?;
    validate_evidence(&format!("hero {} name", hero.id), &hero.name.evidence)?;
    if let Some(role) = &hero.role {
        if role.value.as_str().is_empty() {
            return Err(GameplayDataError::EmptyId("hero role"));
        }
        validate_fact(&format!("hero {} role", hero.id), role)?;
    }
    for (key, fact) in &hero.stats {
        if key.as_str().is_empty() {
            return Err(GameplayDataError::EmptyId("hero stat"));
        }
        validate_fact(&format!("hero {} stat {}", hero.id, key), fact)?;
        validate_stat_value(&format!("hero {} stat {}", hero.id, key), &fact.value)?;
    }
    let mut slot_variants = BTreeSet::new();
    let mut slot_counts: BTreeMap<LogicalSlot, usize> = BTreeMap::new();
    for ability in &hero.abilities {
        if ability.slot.as_str().trim().is_empty() {
            return Err(GameplayDataError::EmptyId("ability slot"));
        }
        if ability
            .variant
            .as_ref()
            .is_some_and(|variant| variant.as_str().is_empty())
        {
            return Err(GameplayDataError::EmptyId("ability variant"));
        }
        if ability.evidence.is_empty() {
            return Err(GameplayDataError::MissingEvidence(format!(
                "hero {} ability {}",
                hero.id, ability.slot
            )));
        }
        validate_evidence(
            &format!("hero {} ability {}", hero.id, ability.slot),
            &ability.evidence,
        )?;
        if ability.name.evidence.is_empty() {
            return Err(GameplayDataError::MissingEvidence(format!(
                "hero {} ability {} name",
                hero.id, ability.slot
            )));
        }
        validate_evidence(
            &format!("hero {} ability {} name", hero.id, ability.slot),
            &ability.name.evidence,
        )?;
        let slot_variant = (ability.slot.clone(), ability.variant.clone());
        if !slot_variants.insert(slot_variant) {
            return Err(GameplayDataError::DuplicateSlotVariant {
                hero: hero.id.clone(),
                slot: ability.slot.clone(),
                variant: ability.variant.clone(),
            });
        }
        *slot_counts.entry(ability.slot.clone()).or_default() += 1;
        for (key, fact) in &ability.stats {
            if key.as_str().is_empty() {
                return Err(GameplayDataError::EmptyId("ability stat"));
            }
            validate_fact(
                &format!("hero {} ability {} stat {}", hero.id, ability.slot, key),
                fact,
            )?;
            validate_stat_value(
                &format!("hero {} ability {} stat {}", hero.id, ability.slot, key),
                &fact.value,
            )?;
        }
    }
    for (slot, count) in slot_counts {
        if count > 1
            && hero
                .abilities
                .iter()
                .filter(|ability| ability.slot == slot)
                .any(|ability| ability.variant.is_none())
        {
            return Err(GameplayDataError::VariantRequired {
                hero: hero.id.clone(),
                slot,
            });
        }
    }
    Ok(())
}

fn validate_stat_value(_path: &str, value: &StatValue) -> Result<(), GameplayDataError> {
    if let StatValue::Quantity(quantity) = value {
        if !quantity.value.is_finite() {
            return Err(GameplayDataError::InvalidQuantity {
                value: quantity.value,
            });
        }
        if quantity.unit.as_str().is_empty() {
            return Err(GameplayDataError::EmptyId("quantity unit"));
        }
    }
    Ok(())
}

fn validate_fact<T>(path: &str, fact: &Fact<T>) -> Result<(), GameplayDataError> {
    if fact.evidence.is_empty() {
        return Err(GameplayDataError::MissingEvidence(path.to_string()));
    }
    validate_evidence(path, &fact.evidence)
}

fn validate_evidence(path: &str, evidence: &[EvidenceRef]) -> Result<(), GameplayDataError> {
    for item in evidence {
        if item.source.is_empty() || item.locator.is_empty() {
            return Err(GameplayDataError::MissingEvidence(path.to_string()));
        }
    }
    Ok(())
}
