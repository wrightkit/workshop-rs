//! Deterministic queries, locale resolution, and small calculations over
//! [`crate::gameplay`]. This layer consumes validated records and never
//! invents missing facts or display-name aliases.

use std::cmp::Ordering;

use crate::gameplay::{
    Ability, AbilityRef, AbilityVariant, Fact, GameplayCatalog, Hero, HeroId, LogicalSlot,
    Quantity, StatKey, StatValue, Unit, units,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AbilityMatch<'a> {
    pub hero: &'a Hero,
    pub ability: &'a Ability,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatOwner {
    Hero(HeroId),
    Ability { reference: AbilityRef },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameplayQueryError {
    MissingHero {
        hero: HeroId,
    },
    MissingAbility {
        reference: AbilityRef,
    },
    MissingSlot {
        hero: HeroId,
        slot: LogicalSlot,
    },
    AmbiguousSlot {
        hero: HeroId,
        slot: LogicalSlot,
        candidates: Vec<AbilityRef>,
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
            Self::MissingAbility { reference } => {
                write!(f, "no ability for canonical reference {reference:?}")
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
            Self::MissingStat { owner, stat } => write!(f, "{owner:?} has no stat '{stat}'"),
            Self::WrongStatType {
                owner,
                stat,
                expected,
            } => write!(f, "{owner:?} stat '{stat}' is not a {expected}"),
        }
    }
}
impl std::error::Error for GameplayQueryError {}

/// Explicit failures from locale-aware display-name resolution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AbilityNameResolutionError {
    MissingHero {
        hero: HeroId,
    },
    MissingSlot {
        hero: HeroId,
        slot: LogicalSlot,
    },
    MissingVariant {
        reference: AbilityRef,
    },
    AmbiguousSlot {
        hero: HeroId,
        slot: LogicalSlot,
        candidates: Vec<AbilityRef>,
    },
    UnsupportedLocale {
        locale: String,
    },
    MissingName {
        reference: AbilityRef,
        locale: String,
    },
    MissingDisplayName {
        hero: HeroId,
        locale: String,
        name: String,
    },
    AmbiguousName {
        hero: HeroId,
        locale: String,
        name: String,
        candidates: Vec<AbilityRef>,
    },
}

impl std::fmt::Display for AbilityNameResolutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingHero { hero } => write!(f, "gameplay catalog has no hero '{hero}'"),
            Self::MissingSlot { hero, slot } => {
                write!(f, "hero '{hero}' has no ability in slot '{slot}'")
            }
            Self::MissingVariant { reference } => {
                write!(f, "no ability for canonical reference {reference:?}")
            }
            Self::AmbiguousSlot {
                hero,
                slot,
                candidates,
            } => write!(
                f,
                "hero '{hero}' has multiple abilities in slot '{slot}': {candidates:?}"
            ),
            Self::UnsupportedLocale { locale } => write!(
                f,
                "locale '{locale}' is unsupported by the gameplay name data"
            ),
            Self::MissingName { reference, locale } => write!(
                f,
                "ability {reference:?} has no evidenced name for locale '{locale}'"
            ),
            Self::MissingDisplayName { hero, locale, name } => write!(
                f,
                "hero '{hero}' has no ability named '{name}' for locale '{locale}'"
            ),
            Self::AmbiguousName {
                hero,
                locale,
                name,
                candidates,
            } => write!(
                f,
                "hero '{hero}' has multiple abilities named '{name}' for locale '{locale}': {candidates:?}"
            ),
        }
    }
}
impl std::error::Error for AbilityNameResolutionError {}

pub const MIN_CUSTOM_GAME_COOLDOWN_PERCENTAGE: f64 = 0.0;
pub const MAX_CUSTOM_GAME_COOLDOWN_PERCENTAGE: f64 = 500.0;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CooldownNonApplicability {
    WrongValueType,
    WrongUnit { actual: Unit },
    NonPositiveBase,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CooldownError {
    Missing {
        ability: AbilityRef,
    },
    NonApplicable {
        ability: AbilityRef,
        reason: CooldownNonApplicability,
    },
    InvalidBase {
        ability: AbilityRef,
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
        ability: AbilityRef,
    },
    MissingAbility {
        reference: AbilityRef,
    },
}
impl std::fmt::Display for CooldownError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Missing { ability } => write!(f, "ability {ability:?} has no cooldown stat"),
            Self::NonApplicable { ability, reason } => write!(
                f,
                "ability {ability:?} cooldown is not applicable: {reason:?}"
            ),
            Self::InvalidBase { ability, value } => write!(
                f,
                "ability {ability:?} cooldown base '{value}' is not finite"
            ),
            Self::InvalidTarget { value } => write!(
                f,
                "target cooldown '{value}' must be finite and non-negative"
            ),
            Self::TargetWrongUnit { actual } => {
                write!(f, "target cooldown has unit '{actual}', expected 'seconds'")
            }
            Self::InvalidPercentage(error) => error.fmt(f),
            Self::CalculationOverflow { ability } => write!(
                f,
                "cooldown calculation for ability {ability:?} is not finite"
            ),
            Self::MissingAbility { reference } => {
                write!(f, "no ability for canonical reference {reference:?}")
            }
        }
    }
}
impl std::error::Error for CooldownError {}

#[derive(Debug, Clone, Copy)]
pub struct GameplayQuery<'a> {
    catalog: &'a GameplayCatalog,
}
impl GameplayCatalog {
    pub fn query(&self) -> GameplayQuery<'_> {
        GameplayQuery { catalog: self }
    }
}

impl<'a> GameplayQuery<'a> {
    pub fn heroes(&self) -> &'a [Hero] {
        self.catalog.heroes()
    }

    pub fn hero(&self, hero: impl AsRef<str>) -> Result<&'a Hero, GameplayQueryError> {
        let id = HeroId::new(hero.as_ref());
        self.catalog
            .hero(&id)
            .ok_or(GameplayQueryError::MissingHero { hero: id })
    }

    pub fn kit(&self, hero: impl AsRef<str>) -> Result<Vec<&'a Ability>, GameplayQueryError> {
        let hero = self.hero(hero)?;
        let mut abilities = hero.abilities().iter().collect::<Vec<_>>();
        abilities.sort_by(|left, right| ability_order(left, right));
        Ok(abilities)
    }

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
        abilities.sort_by(|left, right| ability_order(left, right));
        Ok(abilities)
    }

    pub fn ability(
        &self,
        hero: impl AsRef<str>,
        slot: impl AsRef<str>,
    ) -> Result<&'a Ability, GameplayQueryError> {
        let hero = self.hero(hero)?;
        self.unique_slot(hero, LogicalSlot::new(slot.as_ref()))
    }

    pub fn ability_ref(&self, reference: &AbilityRef) -> Result<&'a Ability, GameplayQueryError> {
        self.catalog
            .ability(reference)
            .map_err(|error| match error {
                crate::gameplay::AbilityLookupError::Missing { .. }
                | crate::gameplay::AbilityLookupError::MissingVariant { .. } => {
                    GameplayQueryError::MissingAbility {
                        reference: reference.clone(),
                    }
                }
                crate::gameplay::AbilityLookupError::Ambiguous {
                    hero,
                    slot,
                    candidates,
                } => GameplayQueryError::AmbiguousSlot {
                    hero,
                    slot,
                    candidates,
                },
            })
    }

    pub fn slot_ability(
        &self,
        hero: impl AsRef<str>,
        slot: impl AsRef<str>,
    ) -> Result<&'a Ability, GameplayQueryError> {
        self.ability(hero, slot)
    }

    pub fn variant(
        &self,
        hero: impl AsRef<str>,
        slot: impl AsRef<str>,
        variant: impl AsRef<str>,
    ) -> Result<&'a Ability, GameplayQueryError> {
        let hero = self.hero(hero)?;
        let slot = LogicalSlot::new(slot.as_ref());
        let variant = AbilityVariant::new(variant.as_ref());
        hero.ability_variant(&slot, &variant)
            .map_err(|_| GameplayQueryError::MissingVariant {
                hero: hero.id().clone(),
                slot,
                variant,
            })
    }

    pub fn keyword(&self, keyword: impl AsRef<str>) -> Vec<AbilityMatch<'a>> {
        let mut matches = self
            .catalog
            .heroes()
            .iter()
            .flat_map(|hero| {
                hero.abilities()
                    .iter()
                    .filter(|ability| ability.has_keyword(keyword.as_ref()))
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
        slot: impl AsRef<str>,
        variant: Option<impl AsRef<str>>,
        stat: impl AsRef<str>,
    ) -> Result<&'a Fact<StatValue>, GameplayQueryError> {
        let reference = self.reference(hero, slot, variant)?;
        let ability = self.ability_ref(&reference)?;
        let stat = StatKey::new(stat.as_ref());
        ability
            .stat(&stat)
            .ok_or_else(|| GameplayQueryError::MissingStat {
                owner: StatOwner::Ability { reference },
                stat,
            })
    }

    pub fn quantity_stat(
        &self,
        hero: impl AsRef<str>,
        slot: impl AsRef<str>,
        variant: Option<impl AsRef<str>>,
        stat: impl AsRef<str>,
    ) -> Result<&'a Quantity, GameplayQueryError> {
        let reference = self.reference(hero, slot, variant)?;
        let stat_key = StatKey::new(stat.as_ref());
        let fact = self.stat(
            reference.hero().as_str(),
            reference.slot().as_str(),
            reference.variant().map(|v| v.as_str()),
            stat_key.as_str(),
        )?;
        match fact.value() {
            StatValue::Quantity(quantity) => Ok(quantity),
            _ => Err(GameplayQueryError::WrongStatType {
                owner: StatOwner::Ability { reference },
                stat: stat_key,
                expected: "quantity",
            }),
        }
    }

    pub fn ability_name(
        &self,
        hero: impl AsRef<str>,
        slot: impl AsRef<str>,
        variant: Option<impl AsRef<str>>,
        locale: impl AsRef<str>,
    ) -> Result<&'a str, AbilityNameResolutionError> {
        let reference = self.reference_for_names(hero, slot, variant)?;
        let locale = locale.as_ref();
        let ability = self.catalog.ability(&reference).map_err(|_| {
            AbilityNameResolutionError::MissingVariant {
                reference: reference.clone(),
            }
        })?;
        if let Some(name) = ability.name().value().get(locale) {
            return Ok(name);
        }
        if self
            .catalog
            .heroes()
            .iter()
            .flat_map(|hero| hero.abilities())
            .any(|ability| ability.name().value().get(locale).is_some())
        {
            return Err(AbilityNameResolutionError::MissingName {
                reference,
                locale: locale.to_string(),
            });
        }
        Err(AbilityNameResolutionError::UnsupportedLocale {
            locale: locale.to_string(),
        })
    }

    pub fn resolve_ability_name(
        &self,
        hero: impl AsRef<str>,
        locale: impl AsRef<str>,
        name: impl AsRef<str>,
    ) -> Result<AbilityRef, AbilityNameResolutionError> {
        let hero = self.hero_for_names(hero)?;
        let locale = locale.as_ref();
        let name = name.as_ref();
        let supported = hero
            .abilities()
            .iter()
            .any(|ability| ability.name().value().get(locale).is_some());
        if !supported {
            return Err(AbilityNameResolutionError::UnsupportedLocale {
                locale: locale.to_string(),
            });
        }
        let matches = hero
            .abilities()
            .iter()
            .filter(|ability| ability.name().value().get(locale) == Some(name))
            .map(|ability| ability.reference(hero.id()))
            .collect::<Vec<_>>();
        match matches.as_slice() {
            [] => Err(AbilityNameResolutionError::MissingDisplayName {
                hero: hero.id().clone(),
                locale: locale.to_string(),
                name: name.to_string(),
            }),
            [reference] => Ok(reference.clone()),
            _ => Err(AbilityNameResolutionError::AmbiguousName {
                hero: hero.id().clone(),
                locale: locale.to_string(),
                name: name.to_string(),
                candidates: matches,
            }),
        }
    }

    pub fn cooldown(&self, reference: &AbilityRef) -> Result<&'a Quantity, CooldownError> {
        let ability = self
            .ability_ref(reference)
            .map_err(|_| CooldownError::MissingAbility {
                reference: reference.clone(),
            })?;
        self.cooldown_value(reference, ability)
    }

    pub fn effective_cooldown(
        &self,
        reference: &AbilityRef,
        percentage: CooldownPercentage,
    ) -> Result<Quantity, CooldownError> {
        let base = self.cooldown(reference)?;
        let value = base.value * percentage.value() / 100.0;
        if !value.is_finite() {
            return Err(CooldownError::CalculationOverflow {
                ability: reference.clone(),
            });
        }
        Quantity::new(value, Unit::from(units::SECONDS)).map_err(|_| {
            CooldownError::CalculationOverflow {
                ability: reference.clone(),
            }
        })
    }

    pub fn required_cooldown_percentage(
        &self,
        reference: &AbilityRef,
        target: &Quantity,
    ) -> Result<CooldownPercentage, CooldownError> {
        let base = self.cooldown(reference)?;
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
        CooldownPercentage::new(target.value / base.value * 100.0)
            .map_err(CooldownError::InvalidPercentage)
    }

    fn unique_slot(
        &self,
        hero: &'a Hero,
        slot: LogicalSlot,
    ) -> Result<&'a Ability, GameplayQueryError> {
        let mut matches = hero.abilities_in_slot(&slot);
        match matches.len() {
            0 => Err(GameplayQueryError::MissingSlot {
                hero: hero.id().clone(),
                slot,
            }),
            1 => Ok(matches.pop().expect("length checked")),
            _ => {
                matches.sort_by(|left, right| ability_order(left, right));
                Err(GameplayQueryError::AmbiguousSlot {
                    hero: hero.id().clone(),
                    slot,
                    candidates: matches
                        .into_iter()
                        .map(|ability| ability.reference(hero.id()))
                        .collect(),
                })
            }
        }
    }

    fn reference(
        &self,
        hero: impl AsRef<str>,
        slot: impl AsRef<str>,
        variant: Option<impl AsRef<str>>,
    ) -> Result<AbilityRef, GameplayQueryError> {
        let hero = self.hero(hero)?;
        let slot = LogicalSlot::new(slot.as_ref());
        match variant {
            Some(variant) => Ok(AbilityRef::new(
                hero.id().clone(),
                slot,
                Some(AbilityVariant::new(variant.as_ref())),
            )),
            None => Ok(self.unique_slot(hero, slot)?.reference(hero.id())),
        }
    }

    fn hero_for_names(
        &self,
        hero: impl AsRef<str>,
    ) -> Result<&'a Hero, AbilityNameResolutionError> {
        let id = HeroId::new(hero.as_ref());
        self.catalog
            .hero(&id)
            .ok_or(AbilityNameResolutionError::MissingHero { hero: id })
    }

    fn reference_for_names(
        &self,
        hero: impl AsRef<str>,
        slot: impl AsRef<str>,
        variant: Option<impl AsRef<str>>,
    ) -> Result<AbilityRef, AbilityNameResolutionError> {
        let hero = self.hero_for_names(hero)?;
        let slot = LogicalSlot::new(slot.as_ref());
        match variant {
            Some(variant) => Ok(AbilityRef::new(
                hero.id().clone(),
                slot,
                Some(AbilityVariant::new(variant.as_ref())),
            )),
            None => {
                let mut matches = hero.abilities_in_slot(&slot);
                match matches.len() {
                    0 => Err(AbilityNameResolutionError::MissingSlot {
                        hero: hero.id().clone(),
                        slot,
                    }),
                    1 => Ok(matches.pop().expect("length checked").reference(hero.id())),
                    _ => Err(AbilityNameResolutionError::AmbiguousSlot {
                        hero: hero.id().clone(),
                        slot,
                        candidates: matches
                            .into_iter()
                            .map(|ability| ability.reference(hero.id()))
                            .collect(),
                    }),
                }
            }
        }
    }

    fn cooldown_value(
        &self,
        reference: &AbilityRef,
        ability: &'a Ability,
    ) -> Result<&'a Quantity, CooldownError> {
        let key = StatKey::from(crate::gameplay::stat_keys::COOLDOWN);
        let Some(fact) = ability.stat(&key) else {
            return Err(CooldownError::Missing {
                ability: reference.clone(),
            });
        };
        let StatValue::Quantity(quantity) = fact.value() else {
            return Err(CooldownError::NonApplicable {
                ability: reference.clone(),
                reason: CooldownNonApplicability::WrongValueType,
            });
        };
        let seconds = Unit::from(units::SECONDS);
        if quantity.unit != seconds {
            return Err(CooldownError::NonApplicable {
                ability: reference.clone(),
                reason: CooldownNonApplicability::WrongUnit {
                    actual: quantity.unit.clone(),
                },
            });
        }
        if !quantity.value.is_finite() {
            return Err(CooldownError::InvalidBase {
                ability: reference.clone(),
                value: quantity.value,
            });
        }
        if quantity.value <= 0.0 {
            return Err(CooldownError::NonApplicable {
                ability: reference.clone(),
                reason: CooldownNonApplicability::NonPositiveBase,
            });
        }
        Ok(quantity)
    }
}

fn ability_order(left: &Ability, right: &Ability) -> Ordering {
    left.slot()
        .cmp(right.slot())
        .then_with(|| left.variant().cmp(&right.variant()))
}
