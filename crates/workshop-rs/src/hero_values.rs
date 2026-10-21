//! Versioned hero base values: health pools, ability cooldowns, and ultimate
//! costs, pinned to one game client version per file (`docs/hero-values.md`).

use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

use serde::Deserialize;

use crate::gameplay::{AbilityVariant, GameplayDataError, HeroId, LogicalSlot};

const FORMAT: &str = "workshop-rs/hero-values-v1";

/// The client version of the newest checked-in hero-values file.
pub const CURRENT_VERSION: &str = "2.25.0.0 - 154461.1";

const VALUES_2_25_0_0_154461_1: &str = include_str!("gameplay/values/2.25.0.0 - 154461.1.json");

/// A health pool kind: plain health, shield, or armor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthPool {
    Health,
    Shield,
    Armor,
}

/// The validated hero-value set of one client version.
#[derive(Debug)]
pub struct HeroValues {
    version: String,
    health: HashMap<(HeroId, HealthPool), f64>,
    cooldowns: HashMap<(HeroId, LogicalSlot, Option<AbilityVariant>), f64>,
    ultimate_costs: HashMap<HeroId, f64>,
}

impl HeroValues {
    /// The `balanceVersion` of the file this set was loaded from.
    pub fn version(&self) -> &str {
        &self.version
    }
    /// Health points of one pool, or `None` when the file holds no such value.
    pub fn health(&self, hero: impl AsRef<str>, pool: HealthPool) -> Option<f64> {
        self.health
            .get(&(HeroId::new(hero.as_ref()), pool))
            .copied()
    }
    /// Cooldown seconds of the ability at `slot`/`variant`, or `None` when absent.
    pub fn cooldown(
        &self,
        hero: impl AsRef<str>,
        slot: impl AsRef<str>,
        variant: Option<&str>,
    ) -> Option<f64> {
        self.cooldowns
            .get(&(
                HeroId::new(hero.as_ref()),
                LogicalSlot::new(slot.as_ref()),
                variant.map(AbilityVariant::new),
            ))
            .copied()
    }
    /// Ultimate cost in points, or `None` when the file holds no such value.
    pub fn ultimate_cost(&self, hero: impl AsRef<str>) -> Option<f64> {
        self.ultimate_costs
            .get(&HeroId::new(hero.as_ref()))
            .copied()
    }
}

/// The checked-in hero-values file for `version`, loaded and validated once.
pub fn builtin(version: &str) -> Result<&'static HeroValues, HeroValuesError> {
    match version {
        CURRENT_VERSION => {
            static DATA: OnceLock<Result<HeroValues, HeroValuesError>> = OnceLock::new();
            DATA.get_or_init(|| load(VALUES_2_25_0_0_154461_1))
                .as_ref()
                .map_err(Clone::clone)
        }
        _ => Err(HeroValuesError::UnknownVersion {
            version: version.to_string(),
        }),
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RawFile {
    format: String,
    balance_version: String,
    heroes: Vec<RawHero>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawHero {
    id: HeroId,
    values: Vec<serde_json::Value>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawHealth {
    #[serde(rename = "kind")]
    _kind: serde::de::IgnoredAny,
    pool: HealthPool,
    amount: f64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCooldown {
    #[serde(rename = "kind")]
    _kind: serde::de::IgnoredAny,
    slot: LogicalSlot,
    variant: Option<AbilityVariant>,
    seconds: f64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawUltimateCost {
    #[serde(rename = "kind")]
    _kind: serde::de::IgnoredAny,
    points: f64,
}

enum RawValue {
    Health(HealthPool, f64),
    Cooldown(LogicalSlot, Option<AbilityVariant>, f64),
    UltimateCost(f64),
}

fn positive(value: f64, field: &'static str) -> Result<f64, HeroValuesError> {
    if !value.is_finite() || value <= 0.0 {
        return Err(HeroValuesError::NonPositiveAmount { field });
    }
    Ok(value)
}

fn parse_value(value: serde_json::Value) -> Result<RawValue, HeroValuesError> {
    let kind = value
        .get("kind")
        .and_then(|kind| kind.as_str())
        .map(str::to_string)
        .ok_or_else(|| HeroValuesError::Malformed("value has no string 'kind'".to_string()))?;
    match kind.as_str() {
        "health" => {
            let raw: RawHealth = serde_json::from_value(value)
                .map_err(|error| HeroValuesError::Malformed(format!("health value: {error}")))?;
            Ok(RawValue::Health(raw.pool, positive(raw.amount, "amount")?))
        }
        "cooldown" => {
            let raw: RawCooldown = serde_json::from_value(value)
                .map_err(|error| HeroValuesError::Malformed(format!("cooldown value: {error}")))?;
            if raw.slot.as_str().is_empty() {
                return Err(HeroValuesError::EmptyId("slot"));
            }
            if raw
                .variant
                .as_ref()
                .is_some_and(|variant| variant.as_str().is_empty())
            {
                return Err(HeroValuesError::EmptyId("variant"));
            }
            Ok(RawValue::Cooldown(
                raw.slot,
                raw.variant,
                positive(raw.seconds, "seconds")?,
            ))
        }
        "ultimateCost" => {
            let raw: RawUltimateCost = serde_json::from_value(value).map_err(|error| {
                HeroValuesError::Malformed(format!("ultimateCost value: {error}"))
            })?;
            Ok(RawValue::UltimateCost(positive(raw.points, "points")?))
        }
        _ => Err(HeroValuesError::UnknownKind { kind }),
    }
}

/// Load and validate a hero-values file. Hero, slot, and variant identities are
/// checked against the checked-in gameplay dataset.
pub fn load(json: &str) -> Result<HeroValues, HeroValuesError> {
    let file: RawFile = serde_json::from_str(json)
        .map_err(|error| HeroValuesError::Malformed(error.to_string()))?;
    if file.format != FORMAT {
        return Err(HeroValuesError::InvalidFormat { found: file.format });
    }
    if file.balance_version.is_empty() {
        return Err(HeroValuesError::EmptyId("balanceVersion"));
    }
    let catalog = crate::gameplay::data::builtin_ref()
        .map_err(|error| HeroValuesError::GameplayDataUnavailable(error.clone()))?;

    let mut values = HeroValues {
        version: file.balance_version,
        health: HashMap::new(),
        cooldowns: HashMap::new(),
        ultimate_costs: HashMap::new(),
    };
    let mut seen_heroes = HashSet::new();
    for hero in file.heroes {
        if hero.id.as_str().is_empty() {
            return Err(HeroValuesError::EmptyId("hero"));
        }
        if !seen_heroes.insert(hero.id.clone()) {
            return Err(HeroValuesError::DuplicateHero { hero: hero.id });
        }
        let record = catalog
            .hero(&hero.id)
            .ok_or_else(|| HeroValuesError::UnknownHero {
                hero: hero.id.clone(),
            })?;
        for value in hero.values {
            match parse_value(value)? {
                RawValue::Health(pool, amount) => {
                    let key = (hero.id.clone(), pool);
                    if values.health.contains_key(&key) {
                        return Err(HeroValuesError::DuplicateValue {
                            hero: hero.id.clone(),
                            value: format!("health pool '{pool:?}'"),
                        });
                    }
                    values.health.insert(key, amount);
                }
                RawValue::Cooldown(slot, variant, seconds) => {
                    check_cooldown_target(record, &slot, variant.as_ref())?;
                    let key = (hero.id.clone(), slot.clone(), variant.clone());
                    if values.cooldowns.contains_key(&key) {
                        let target = match &variant {
                            Some(variant) => format!("cooldown '{slot}/{variant}'"),
                            None => format!("cooldown '{slot}'"),
                        };
                        return Err(HeroValuesError::DuplicateValue {
                            hero: hero.id.clone(),
                            value: target,
                        });
                    }
                    values.cooldowns.insert(key, seconds);
                }
                RawValue::UltimateCost(points) => {
                    if values.ultimate_costs.contains_key(&hero.id) {
                        return Err(HeroValuesError::DuplicateValue {
                            hero: hero.id.clone(),
                            value: "ultimate cost".to_string(),
                        });
                    }
                    values.ultimate_costs.insert(hero.id.clone(), points);
                }
            }
        }
    }
    Ok(values)
}

fn check_cooldown_target(
    hero: &crate::gameplay::Hero,
    slot: &LogicalSlot,
    variant: Option<&AbilityVariant>,
) -> Result<(), HeroValuesError> {
    match variant {
        Some(variant) => hero
            .ability_variant(slot, variant)
            .map(|_| ())
            .map_err(|_| HeroValuesError::UnknownVariant {
                hero: hero.id().clone(),
                slot: slot.clone(),
                variant: variant.clone(),
            }),
        None => match hero.abilities_in_slot(slot).len() {
            0 => Err(HeroValuesError::UnknownSlot {
                hero: hero.id().clone(),
                slot: slot.clone(),
            }),
            1 => Ok(()),
            _ => Err(HeroValuesError::AmbiguousSlot {
                hero: hero.id().clone(),
                slot: slot.clone(),
            }),
        },
    }
}

/// Validation and lookup errors for hero-values files.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum HeroValuesError {
    UnknownVersion {
        version: String,
    },
    Malformed(String),
    InvalidFormat {
        found: String,
    },
    UnknownKind {
        kind: String,
    },
    EmptyId(&'static str),
    NonPositiveAmount {
        field: &'static str,
    },
    UnknownHero {
        hero: HeroId,
    },
    UnknownSlot {
        hero: HeroId,
        slot: LogicalSlot,
    },
    AmbiguousSlot {
        hero: HeroId,
        slot: LogicalSlot,
    },
    UnknownVariant {
        hero: HeroId,
        slot: LogicalSlot,
        variant: AbilityVariant,
    },
    DuplicateHero {
        hero: HeroId,
    },
    DuplicateValue {
        hero: HeroId,
        value: String,
    },
    GameplayDataUnavailable(GameplayDataError),
}

impl std::fmt::Display for HeroValuesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownVersion { version } => {
                write!(
                    f,
                    "no hero-values file is registered for version '{version}'"
                )
            }
            Self::Malformed(message) => write!(f, "malformed hero values: {message}"),
            Self::InvalidFormat { found } => {
                write!(f, "hero-values format '{found}' is not '{FORMAT}'")
            }
            Self::UnknownKind { kind } => write!(f, "unknown hero value kind '{kind}'"),
            Self::EmptyId(field) => write!(f, "hero-values field '{field}' is empty"),
            Self::NonPositiveAmount { field } => {
                write!(
                    f,
                    "hero-values field '{field}' must be a finite positive number"
                )
            }
            Self::UnknownHero { hero } => {
                write!(f, "gameplay dataset has no hero '{hero}'")
            }
            Self::UnknownSlot { hero, slot } => {
                write!(f, "hero '{hero}' has no ability in slot '{slot}'")
            }
            Self::AmbiguousSlot { hero, slot } => write!(
                f,
                "hero '{hero}' has multiple abilities in slot '{slot}'; a variant is required"
            ),
            Self::UnknownVariant {
                hero,
                slot,
                variant,
            } => write!(
                f,
                "hero '{hero}' has no ability in slot '{slot}' with variant '{variant}'"
            ),
            Self::DuplicateHero { hero } => write!(f, "duplicate hero entry '{hero}'"),
            Self::DuplicateValue { hero, value } => {
                write!(f, "hero '{hero}' has a duplicate {value}")
            }
            Self::GameplayDataUnavailable(error) => {
                write!(f, "gameplay dataset is unavailable: {error}")
            }
        }
    }
}
impl std::error::Error for HeroValuesError {}
