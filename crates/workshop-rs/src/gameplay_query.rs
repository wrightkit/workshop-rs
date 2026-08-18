//! Deterministic queries and small calculations over [`crate::gameplay`].
//!
//! This module is a semantic access layer over validated gameplay records. It
//! does not load data, infer missing facts, or mutate the underlying catalog.

use std::cmp::Ordering;

use crate::gameplay::{
    Ability, AbilityId, AbilityVariant, Fact, GameplayCatalog, Hero, HeroId, LogicalSlot, Quantity,
    StatKey, StatValue, Unit, units,
};

/// A deterministic match returned by a keyword query.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AbilityMatch<'a> {
    pub hero: &'a Hero,
    pub ability: &'a Ability,
}

/// The owner of a stat lookup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatOwner {
    Hero(HeroId),
    Ability { hero: HeroId, ability: AbilityId },
}

/// Explicit failures from a gameplay query.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameplayQueryError {
    MissingHero {
        hero: HeroId,
    },
    MissingAbility {
        hero: HeroId,
        ability: AbilityId,
    },
    MissingSlot {
        hero: HeroId,
        slot: LogicalSlot,
    },
    AmbiguousSlot {
        hero: HeroId,
        slot: LogicalSlot,
        candidates: Vec<AbilityId>,
    },
    MissingVariant {
        hero: HeroId,
        slot: LogicalSlot,
        variant: AbilityVariant,
    },
    MissingStat {
        owner: StatOwner,
        stat: StatKey,
    },
    WrongStatType {
        owner: StatOwner,
        stat: StatKey,
        expected: &'static str,
    },
}

impl std::fmt::Display for GameplayQueryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingHero { hero } => write!(f, "gameplay catalog has no hero '{hero}'"),
            Self::MissingAbility { hero, ability } => {
                write!(f, "hero '{hero}' has no ability '{ability}'")
            }
            Self::MissingSlot { hero, slot } => {
                write!(f, "hero '{hero}' has no ability in slot '{slot}'")
            }
            Self::AmbiguousSlot {
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
            Self::MissingStat { owner, stat } => {
                write!(f, "{owner:?} has no stat '{stat}'")
            }
            Self::WrongStatType {
                owner,
                stat,
                expected,
            } => write!(f, "{owner:?} stat '{stat}' is not a {expected}"),
        }
    }
}

impl std::error::Error for GameplayQueryError {}

/// The Custom Game cooldown percentage range accepted by this API.
pub const MIN_CUSTOM_GAME_COOLDOWN_PERCENTAGE: f64 = 0.0;
pub const MAX_CUSTOM_GAME_COOLDOWN_PERCENTAGE: f64 = 500.0;

/// A validated Custom Game cooldown percentage.
///
/// `100%` preserves the base cooldown. `0%` disables the cooldown duration,
/// and `500%` is the maximum supported setting in this API.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct CooldownPercentage(f64);

impl CooldownPercentage {
    pub fn new(value: f64) -> Result<Self, CooldownPercentageError> {
        if !value.is_finite() {
            return Err(CooldownPercentageError::NotFinite { value });
        }
        if !(MIN_CUSTOM_GAME_COOLDOWN_PERCENTAGE..=MAX_CUSTOM_GAME_COOLDOWN_PERCENTAGE)
            .contains(&value)
        {
            return Err(CooldownPercentageError::OutOfRange { value });
        }
        Ok(Self(value))
    }

    pub fn value(self) -> f64 {
        self.0
    }
}

impl TryFrom<f64> for CooldownPercentage {
    type Error = CooldownPercentageError;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

/// Invalid Custom Game cooldown percentage input.
#[derive(Debug, Clone, PartialEq)]
pub enum CooldownPercentageError {
    NotFinite { value: f64 },
    OutOfRange { value: f64 },
}

impl std::fmt::Display for CooldownPercentageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFinite { value } => write!(f, "cooldown percentage '{value}' is not finite"),
            Self::OutOfRange { value } => write!(
                f,
                "cooldown percentage '{value}' is outside {MIN_CUSTOM_GAME_COOLDOWN_PERCENTAGE}%..={MAX_CUSTOM_GAME_COOLDOWN_PERCENTAGE}%"
            ),
        }
    }
}

impl std::error::Error for CooldownPercentageError {}

/// Why an ability's stat cannot be used as a cooldown.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CooldownNonApplicability {
    WrongValueType,
    WrongUnit { actual: Unit },
    NonPositiveBase,
}

/// Explicit failures from cooldown access and calculations.
#[derive(Debug, Clone, PartialEq)]
pub enum CooldownError {
    Missing {
        ability: AbilityId,
    },
    NonApplicable {
        ability: AbilityId,
        reason: CooldownNonApplicability,
    },
    InvalidBase {
        ability: AbilityId,
        value: f64,
    },
    InvalidTarget {
        value: f64,
    },
    TargetWrongUnit {
        actual: Unit,
    },
    InvalidPercentage(CooldownPercentageError),
    CalculationOverflow {
        ability: AbilityId,
    },
}

impl std::fmt::Display for CooldownError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Missing { ability } => {
                write!(f, "ability '{ability}' has no cooldown stat")
            }
            Self::NonApplicable { ability, reason } => {
                write!(
                    f,
                    "ability '{ability}' cooldown is not applicable: {reason:?}"
                )
            }
            Self::InvalidBase { ability, value } => {
                write!(
                    f,
                    "ability '{ability}' cooldown base '{value}' is not finite"
                )
            }
            Self::InvalidTarget { value } => {
                write!(
                    f,
                    "target cooldown '{value}' must be finite and non-negative"
                )
            }
            Self::TargetWrongUnit { actual } => {
                write!(f, "target cooldown has unit '{actual}', expected 'seconds'")
            }
            Self::InvalidPercentage(error) => error.fmt(f),
            Self::CalculationOverflow { ability } => {
                write!(
                    f,
                    "cooldown calculation for ability '{ability}' is not finite"
                )
            }
        }
    }
}

impl std::error::Error for CooldownError {}

/// A reusable, read-only query view over a validated [`GameplayCatalog`].
#[derive(Debug, Clone, Copy)]
pub struct GameplayQuery<'a> {
    catalog: &'a GameplayCatalog,
}

impl GameplayCatalog {
    /// Creates a deterministic query view without copying or changing data.
    pub fn query(&self) -> GameplayQuery<'_> {
        GameplayQuery { catalog: self }
    }
}

impl<'a> GameplayQuery<'a> {
    /// Returns all heroes in the catalog's canonical deterministic order.
    pub fn heroes(&self) -> &'a [Hero] {
        self.catalog.heroes()
    }

    pub fn hero(&self, hero: impl AsRef<str>) -> Result<&'a Hero, GameplayQueryError> {
        let hero_id = HeroId::new(hero.as_ref());
        self.catalog
            .hero(&hero_id)
            .ok_or(GameplayQueryError::MissingHero { hero: hero_id })
    }

    /// Returns a complete kit sorted by logical slot, variant, then ability ID.
    pub fn kit(&self, hero: impl AsRef<str>) -> Result<Vec<&'a Ability>, GameplayQueryError> {
        let hero = self.hero(hero)?;
        let mut abilities = hero.abilities().iter().collect::<Vec<_>>();
        abilities.sort_by(|left, right| ability_order(left, right));
        Ok(abilities)
    }

    /// Returns every ability in a slot, sorted by ability ID.
    ///
    /// An absent slot is an explicit error. Multiple entries are returned as a
    /// collection; use [`Self::slot_ability`] when one result is required.
    pub fn slot(
        &self,
        hero: impl AsRef<str>,
        slot: impl AsRef<str>,
    ) -> Result<Vec<&'a Ability>, GameplayQueryError> {
        let hero = self.hero(hero)?;
        let slot = LogicalSlot::new(slot.as_ref());
        let mut abilities = hero.abilities_in_slot(&slot);
        if abilities.is_empty() {
            return Err(GameplayQueryError::MissingSlot {
                hero: hero.id().clone(),
                slot,
            });
        }
        abilities.sort_by(|left, right| left.id().cmp(right.id()));
        Ok(abilities)
    }

    /// Returns one ability by its canonical ability ID.
    pub fn ability(
        &self,
        hero: impl AsRef<str>,
        ability: impl AsRef<str>,
    ) -> Result<&'a Ability, GameplayQueryError> {
        let hero = self.hero(hero)?;
        let ability_id = AbilityId::new(ability.as_ref());
        hero.ability_by_id(&ability_id)
            .ok_or_else(|| GameplayQueryError::MissingAbility {
                hero: hero.id().clone(),
                ability: ability_id,
            })
    }

    /// Returns the only ability in a slot, rejecting missing and ambiguous slots.
    pub fn slot_ability(
        &self,
        hero: impl AsRef<str>,
        slot: impl AsRef<str>,
    ) -> Result<&'a Ability, GameplayQueryError> {
        let hero = self.hero(hero)?;
        let slot = LogicalSlot::new(slot.as_ref());
        let mut matches = hero.abilities_in_slot(&slot);
        match matches.len() {
            0 => Err(GameplayQueryError::MissingSlot {
                hero: hero.id().clone(),
                slot,
            }),
            1 => Ok(matches.pop().expect("length checked")),
            _ => {
                matches.sort_by(|left, right| left.id().cmp(right.id()));
                Err(GameplayQueryError::AmbiguousSlot {
                    hero: hero.id().clone(),
                    slot,
                    candidates: matches
                        .into_iter()
                        .map(|ability| ability.id().clone())
                        .collect(),
                })
            }
        }
    }

    /// Returns an explicitly selected ability variant in a slot.
    pub fn variant(
        &self,
        hero: impl AsRef<str>,
        slot: impl AsRef<str>,
        variant: impl AsRef<str>,
    ) -> Result<&'a Ability, GameplayQueryError> {
        let hero = self.hero(hero)?;
        let slot = LogicalSlot::new(slot.as_ref());
        let variant = AbilityVariant::new(variant.as_ref());
        hero.abilities()
            .iter()
            .find(|ability| ability.slot() == &slot && ability.variant() == Some(&variant))
            .ok_or(GameplayQueryError::MissingVariant {
                hero: hero.id().clone(),
                slot,
                variant,
            })
    }

    /// Finds keyword matches in hero ID, slot, variant, then ability ID order.
    /// No matches are represented by an empty collection.
    pub fn keyword(&self, keyword: impl AsRef<str>) -> Vec<AbilityMatch<'a>> {
        let keyword = keyword.as_ref();
        let mut matches = self
            .catalog
            .heroes()
            .iter()
            .flat_map(|hero| {
                hero.abilities()
                    .iter()
                    .filter(|ability| ability.has_keyword(keyword))
                    .map(move |ability| AbilityMatch { hero, ability })
            })
            .collect::<Vec<_>>();
        matches.sort_by(|left, right| {
            left.hero
                .id()
                .cmp(right.hero.id())
                .then_with(|| ability_order(left.ability, right.ability))
        });
        matches
    }

    pub fn hero_stat(
        &self,
        hero: impl AsRef<str>,
        stat: impl AsRef<str>,
    ) -> Result<&'a Fact<StatValue>, GameplayQueryError> {
        let hero = self.hero(hero)?;
        let stat = StatKey::new(stat.as_ref());
        hero.stat(&stat)
            .ok_or_else(|| GameplayQueryError::MissingStat {
                owner: StatOwner::Hero(hero.id().clone()),
                stat,
            })
    }

    pub fn stat(
        &self,
        hero: impl AsRef<str>,
        ability: impl AsRef<str>,
        stat: impl AsRef<str>,
    ) -> Result<&'a Fact<StatValue>, GameplayQueryError> {
        let hero = self.hero(hero)?;
        let ability_id = AbilityId::new(ability.as_ref());
        let ability =
            hero.ability_by_id(&ability_id)
                .ok_or_else(|| GameplayQueryError::MissingAbility {
                    hero: hero.id().clone(),
                    ability: ability_id.clone(),
                })?;
        let stat = StatKey::new(stat.as_ref());
        ability
            .stat(&stat)
            .ok_or_else(|| GameplayQueryError::MissingStat {
                owner: StatOwner::Ability {
                    hero: hero.id().clone(),
                    ability: ability_id,
                },
                stat,
            })
    }

    pub fn quantity_stat(
        &self,
        hero: impl AsRef<str>,
        ability: impl AsRef<str>,
        stat: impl AsRef<str>,
    ) -> Result<&'a Quantity, GameplayQueryError> {
        let hero_id = HeroId::new(hero.as_ref());
        let ability_id = AbilityId::new(ability.as_ref());
        let stat_key = StatKey::new(stat.as_ref());
        let fact = self.stat(hero_id.as_str(), ability_id.as_str(), stat_key.as_str())?;
        match fact.value() {
            StatValue::Quantity(quantity) => Ok(quantity),
            _ => Err(GameplayQueryError::WrongStatType {
                owner: StatOwner::Ability {
                    hero: hero_id,
                    ability: ability_id,
                },
                stat: stat_key,
                expected: "quantity",
            }),
        }
    }

    /// Reads an ability's positive, finite cooldown quantity in seconds.
    pub fn cooldown<'b>(&self, ability: &'b Ability) -> Result<&'b Quantity, CooldownError> {
        let seconds_unit = Unit::from(units::SECONDS);
        let cooldown_key = StatKey::from(crate::gameplay::stat_keys::COOLDOWN);
        let Some(fact) = ability.stat(&cooldown_key) else {
            return Err(CooldownError::Missing {
                ability: ability.id().clone(),
            });
        };
        let StatValue::Quantity(quantity) = fact.value() else {
            return Err(CooldownError::NonApplicable {
                ability: ability.id().clone(),
                reason: CooldownNonApplicability::WrongValueType,
            });
        };
        if quantity.unit != seconds_unit {
            return Err(CooldownError::NonApplicable {
                ability: ability.id().clone(),
                reason: CooldownNonApplicability::WrongUnit {
                    actual: quantity.unit.clone(),
                },
            });
        }
        if !quantity.value.is_finite() {
            return Err(CooldownError::InvalidBase {
                ability: ability.id().clone(),
                value: quantity.value,
            });
        }
        if quantity.value <= 0.0 {
            return Err(CooldownError::NonApplicable {
                ability: ability.id().clone(),
                reason: CooldownNonApplicability::NonPositiveBase,
            });
        }
        Ok(quantity)
    }

    /// Calculates `base cooldown * custom-game percentage / 100`.
    pub fn effective_cooldown(
        &self,
        ability: &Ability,
        percentage: CooldownPercentage,
    ) -> Result<Quantity, CooldownError> {
        let base = self.cooldown(ability)?;
        let value = base.value * percentage.value() / 100.0;
        if !value.is_finite() {
            return Err(CooldownError::CalculationOverflow {
                ability: ability.id().clone(),
            });
        }
        Quantity::new(value, Unit::from(units::SECONDS)).map_err(|_| {
            CooldownError::CalculationOverflow {
                ability: ability.id().clone(),
            }
        })
    }

    /// Calculates the Custom Game percentage needed for a target cooldown.
    pub fn required_cooldown_percentage(
        &self,
        ability: &Ability,
        target: &Quantity,
    ) -> Result<CooldownPercentage, CooldownError> {
        let base = self.cooldown(ability)?;
        if target.unit != Unit::from(units::SECONDS) {
            return Err(CooldownError::TargetWrongUnit {
                actual: target.unit.clone(),
            });
        }
        if !target.value.is_finite() || target.value < 0.0 {
            return Err(CooldownError::InvalidTarget {
                value: target.value,
            });
        }
        let percentage = target.value / base.value * 100.0;
        CooldownPercentage::new(percentage).map_err(CooldownError::InvalidPercentage)
    }
}

fn ability_order(left: &Ability, right: &Ability) -> Ordering {
    left.slot()
        .cmp(right.slot())
        .then_with(|| left.variant().cmp(&right.variant()))
        .then_with(|| left.id().cmp(right.id()))
}
