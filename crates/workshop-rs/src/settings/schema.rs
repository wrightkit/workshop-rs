//! Canonical typed facts for Workshop custom-game settings.
//!
//! Definitions are a semantic projection of the reviewed settings table. The
//! table remains the parser/emitter lookup source, while [`Settings`] and
//! [`SettingsNode`] remain the source-preserving authored-value carrier.

use std::fmt;

use crate::gameplay::{AbilityVariant, HeroId, LogicalSlot};
use crate::{gameplay::GameplayDataError, gameplay_data};

use super::table::{self, KeyKind, PathPart, TableEntry};

/// A locale-independent Workshop setting concept identity.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SettingId(String);

impl SettingId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for SettingId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl fmt::Display for SettingId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Whether a definition has a reviewed canonical concept identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingIdentity {
    Known(SettingId),
    Unknown,
}

/// The Workshop-native section that owns a setting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SettingScope {
    Main,
    Lobby,
    GameModes,
    Heroes,
    Extensions,
    Workshop,
    Unknown,
}

/// An open team identity used by hero settings structure.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TeamId(String);

impl TeamId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// The semantic entity to which a setting applies.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SettingTarget {
    Global,
    Mode(String),
    Team(TeamId),
    Hero {
        team: Option<TeamId>,
        hero: HeroId,
    },
    TeamAbility {
        team: Option<TeamId>,
        slot: LogicalSlot,
        variant: Option<AbilityVariant>,
    },
    HeroAbility {
        team: Option<TeamId>,
        hero: HeroId,
        slot: LogicalSlot,
        variant: Option<AbilityVariant>,
    },
}

/// The target shape described by a definition. Concrete identities are
/// supplied separately when applicability is queried.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingTargetKind {
    Global,
    Mode,
    Team,
    TeamAbility {
        slot: LogicalSlot,
        variant: Option<AbilityVariant>,
    },
    Hero,
    HeroAbility {
        slot: LogicalSlot,
        variant: Option<AbilityVariant>,
    },
    Unknown,
}

/// The result of asking whether a definition applies to a target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Applicability {
    Applicable,
    NotApplicable,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumericBoundsError {
    NonFinite,
    Reversed,
}

/// Evidence-backed effective numeric bounds. `None` means the current
/// reviewed evidence does not establish that bound.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct NumericBounds {
    min: Option<f64>,
    max: Option<f64>,
}

impl NumericBounds {
    pub const fn unknown() -> Self {
        Self {
            min: None,
            max: None,
        }
    }

    pub fn new(min: Option<f64>, max: Option<f64>) -> Result<Self, NumericBoundsError> {
        if min.is_some_and(|value| !value.is_finite())
            || max.is_some_and(|value| !value.is_finite())
        {
            return Err(NumericBoundsError::NonFinite);
        }
        if min.zip(max).is_some_and(|(min, max)| min > max) {
            return Err(NumericBoundsError::Reversed);
        }
        Ok(Self { min, max })
    }

    pub fn min(&self) -> Option<f64> {
        self.min
    }

    pub fn max(&self) -> Option<f64> {
        self.max
    }

    pub fn effective(&self, authored: f64) -> Option<EffectiveNumber> {
        if !authored.is_finite() || self.min.is_none() && self.max.is_none() {
            return None;
        }
        match (self.min, self.max) {
            (Some(min), None) if authored >= min => return None,
            (None, Some(max)) if authored <= max => return None,
            _ => {}
        }
        let mut effective = authored;
        if let Some(min) = self.min {
            effective = effective.max(min);
        }
        if let Some(max) = self.max {
            effective = effective.min(max);
        }
        Some(EffectiveNumber {
            authored,
            effective,
        })
    }
}

/// An authored numeric value paired with its Workshop-effective value.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct EffectiveNumber {
    pub authored: f64,
    pub effective: f64,
}

/// The machine-readable value domain of a setting.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum SettingValueDomain {
    Boolean,
    Number(NumericBounds),
    Percent(NumericBounds),
    String,
    Enum { domain: String },
    HeroList,
    MapList,
    PresenceOnly,
}

impl SettingValueDomain {
    /// Apply evidenced effective clamping without changing the authored
    /// value held by [`super::SettingsNode`].
    pub fn effective_number(&self, authored: f64) -> Option<EffectiveNumber> {
        match self {
            Self::Number(bounds) | Self::Percent(bounds) => bounds.effective(authored),
            _ => None,
        }
    }
}

/// Locale-facing names associated with a canonical setting concept.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingPresentation {
    pub english_name: &'static str,
    pub locale_section: &'static str,
}

impl SettingPresentation {
    pub fn localized_name(&self, locale: &str) -> Option<&'static str> {
        if locale.eq_ignore_ascii_case("en-US") {
            Some(self.english_name)
        } else {
            table::localized_name(locale, self.locale_section, self.english_name)
        }
    }
}

/// Provenance shared by the reviewed table projection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SettingProvenance {
    pub kind: SettingEvidenceKind,
    pub source: &'static str,
    pub reviewed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingEvidenceKind {
    RawWorkshopFixture,
    WorkshopDataExport,
}

/// One canonical semantic definition projected from an existing table entry.
#[derive(Debug, Clone, PartialEq)]
pub struct SettingDefinition {
    identity: SettingIdentity,
    scope: SettingScope,
    path: String,
    key: &'static str,
    target: TargetPattern,
    domain: SettingValueDomain,
    presentation: SettingPresentation,
    provenance: SettingProvenance,
}

impl SettingDefinition {
    pub fn identity(&self) -> &SettingIdentity {
        &self.identity
    }

    pub fn id(&self) -> Option<&SettingId> {
        match &self.identity {
            SettingIdentity::Known(id) => Some(id),
            SettingIdentity::Unknown => None,
        }
    }

    pub fn scope(&self) -> SettingScope {
        self.scope
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn domain(&self) -> &SettingValueDomain {
        &self.domain
    }

    pub fn target_kind(&self) -> SettingTargetKind {
        match &self.target {
            TargetPattern::Global => SettingTargetKind::Global,
            TargetPattern::Mode(_) => SettingTargetKind::Mode,
            TargetPattern::Team(_) => SettingTargetKind::Team,
            TargetPattern::TeamAbility { slot, variant, .. } => SettingTargetKind::TeamAbility {
                slot: slot.clone(),
                variant: variant.clone(),
            },
            TargetPattern::Hero { .. } => SettingTargetKind::Hero,
            TargetPattern::HeroAbility { slot, variant, .. } => SettingTargetKind::HeroAbility {
                slot: slot.clone(),
                variant: variant.clone(),
            },
            TargetPattern::Unknown => SettingTargetKind::Unknown,
        }
    }

    pub fn presentation(&self) -> &SettingPresentation {
        &self.presentation
    }

    pub fn localized_name(
        &self,
        locale: &str,
        target: &SettingTarget,
    ) -> Result<Option<&'static str>, GameplayDataError> {
        match target {
            SettingTarget::Hero { hero, .. } | SettingTarget::HeroAbility { hero, .. } => {
                if self.applicability(target)? == Applicability::NotApplicable {
                    Ok(None)
                } else {
                    Ok(table::hero_setting_name(hero.as_str(), self.key, locale)
                        .or_else(|| self.presentation.localized_name(locale)))
                }
            }
            _ => Ok(self.presentation.localized_name(locale)),
        }
    }

    pub fn provenance(&self) -> SettingProvenance {
        self.provenance
    }

    /// Query effective applicability without exposing table deduplication.
    pub fn applicability(
        &self,
        target: &SettingTarget,
    ) -> Result<Applicability, GameplayDataError> {
        Ok(match (&self.target, target) {
            (TargetPattern::Global, SettingTarget::Global) => Applicability::Applicable,
            (TargetPattern::Mode(expected), SettingTarget::Mode(actual)) => {
                if expected
                    .as_deref()
                    .is_none_or(|expected| expected == actual)
                {
                    Applicability::Applicable
                } else {
                    Applicability::NotApplicable
                }
            }
            (TargetPattern::Team(expected), SettingTarget::Team(actual)) => {
                if expected
                    .as_deref()
                    .is_none_or(|expected| expected == actual.as_str())
                {
                    Applicability::Applicable
                } else {
                    Applicability::NotApplicable
                }
            }
            (TargetPattern::Team(expected), SettingTarget::Hero { team, .. }) => {
                if team_matches(expected.as_deref(), team.as_ref()) {
                    Applicability::Unknown
                } else {
                    Applicability::NotApplicable
                }
            }
            (
                TargetPattern::TeamAbility {
                    team,
                    slot,
                    variant: expected_variant,
                },
                SettingTarget::TeamAbility {
                    team: actual_team,
                    slot: actual_slot,
                    variant: actual_variant,
                },
            ) => {
                if !team_matches(team.as_deref(), actual_team.as_ref())
                    || slot != actual_slot
                    || expected_variant
                        .as_ref()
                        .is_some_and(|expected| actual_variant.as_ref() != Some(expected))
                {
                    Applicability::NotApplicable
                } else {
                    Applicability::Applicable
                }
            }
            (
                TargetPattern::TeamAbility {
                    team,
                    slot,
                    variant: expected_variant,
                },
                SettingTarget::HeroAbility {
                    team: actual_team,
                    hero: actual_hero,
                    slot: actual_slot,
                    variant: actual_variant,
                },
            ) => {
                if !team_matches(team.as_deref(), actual_team.as_ref())
                    || slot != actual_slot
                    || expected_variant
                        .as_ref()
                        .is_some_and(|expected| actual_variant.as_ref() != Some(expected))
                {
                    Applicability::NotApplicable
                } else {
                    match hero_ability_exists(actual_hero, actual_slot, actual_variant.as_ref())? {
                        Some(true) => Applicability::Unknown,
                        Some(false) => Applicability::NotApplicable,
                        None => Applicability::Unknown,
                    }
                }
            }
            (
                TargetPattern::Hero { team, hero },
                SettingTarget::Hero {
                    team: actual_team,
                    hero: actual_hero,
                },
            ) => {
                if !team_matches(team.as_deref(), actual_team.as_ref())
                    || hero
                        .as_deref()
                        .is_some_and(|expected| expected != actual_hero.as_str())
                {
                    Applicability::NotApplicable
                } else {
                    Applicability::Unknown
                }
            }
            (
                TargetPattern::HeroAbility {
                    team,
                    hero,
                    slot,
                    variant: expected_variant,
                },
                SettingTarget::HeroAbility {
                    team: actual_team,
                    hero: actual_hero,
                    slot: actual_slot,
                    ..
                },
            ) => {
                if !team_matches(team.as_deref(), actual_team.as_ref())
                    || hero
                        .as_deref()
                        .is_some_and(|expected| expected != actual_hero.as_str())
                    || slot.as_str() != actual_slot.as_str()
                    || expected_variant
                        .as_ref()
                        .is_some_and(|expected| Some(expected) != target_variant(target))
                {
                    return Ok(Applicability::NotApplicable);
                }
                match hero_ability_exists(actual_hero, actual_slot, target_variant(target))? {
                    None => Applicability::Unknown,
                    Some(false) => Applicability::NotApplicable,
                    Some(true) => Applicability::Unknown,
                }
            }
            (TargetPattern::Unknown, _) => Applicability::Unknown,
            _ => Applicability::NotApplicable,
        })
    }

    pub fn effective_number(&self, authored: f64) -> Option<EffectiveNumber> {
        self.domain.effective_number(authored)
    }
}

#[derive(Debug, Clone, PartialEq)]
enum TargetPattern {
    Global,
    Mode(Option<String>),
    Team(Option<String>),
    TeamAbility {
        team: Option<String>,
        slot: LogicalSlot,
        variant: Option<AbilityVariant>,
    },
    Hero {
        team: Option<String>,
        hero: Option<String>,
    },
    HeroAbility {
        team: Option<String>,
        hero: Option<String>,
        slot: LogicalSlot,
        variant: Option<AbilityVariant>,
    },
    Unknown,
}

fn team_matches(expected: Option<&str>, actual: Option<&TeamId>) -> bool {
    expected.is_none_or(|expected| actual.is_some_and(|actual| actual.as_str() == expected))
}

fn target_variant(target: &SettingTarget) -> Option<&AbilityVariant> {
    match target {
        SettingTarget::HeroAbility { variant, .. } => variant.as_ref(),
        _ => None,
    }
}

fn hero_ability_exists(
    hero: &HeroId,
    slot: &LogicalSlot,
    variant: Option<&AbilityVariant>,
) -> Result<Option<bool>, GameplayDataError> {
    gameplay_data::builtin_ref()
        .map_err(Clone::clone)
        .map(|catalog| {
            catalog.hero(hero).and_then(|hero| {
                if let Some(variant) = variant {
                    Some(hero.ability_variant(slot, variant).is_ok())
                } else {
                    (!hero.abilities_in_slot(slot).is_empty()).then_some(true)
                }
            })
        })
}

/// Project all currently reviewed table entries into the canonical semantic
/// model. This is intentionally a projection, not a second settings catalog.
pub fn definitions() -> impl Iterator<Item = SettingDefinition> {
    table::entries().map(SettingDefinition::from_entry)
}

/// Project one reviewed table entry into the canonical semantic definition.
pub fn definition(path: &[PathPart<'_>]) -> Option<SettingDefinition> {
    table::lookup(path).map(SettingDefinition::from_entry)
}

impl SettingDefinition {
    fn from_entry(entry: &TableEntry) -> Self {
        let scope = scope_for(entry.path);
        let key = entry
            .path
            .last()
            .and_then(|part| match part {
                PathPart::Part(key) => Some(*key),
                _ => None,
            })
            .unwrap_or("");
        let target = target_for(entry.path);
        let path = table::path_string(entry.path);
        let domain = domain_for(entry.kind);
        let identity = canonical_id(scope, key, entry.path)
            .map(SettingIdentity::Known)
            .unwrap_or(SettingIdentity::Unknown);
        Self {
            identity,
            scope,
            path,
            key,
            target,
            domain,
            presentation: SettingPresentation {
                english_name: entry.workshop_name,
                locale_section: "labels",
            },
            provenance: SettingProvenance {
                kind: if table::is_generated_entry(entry) {
                    SettingEvidenceKind::WorkshopDataExport
                } else {
                    SettingEvidenceKind::RawWorkshopFixture
                },
                source: if table::is_generated_entry(entry) {
                    "workshop-data/workshop-data.json"
                } else {
                    "pinned raw Workshop settings fixtures"
                },
                reviewed: true,
            },
        }
    }
}

fn scope_for(path: &[PathPart<'_>]) -> SettingScope {
    match path.first() {
        Some(PathPart::Part("main")) => SettingScope::Main,
        Some(PathPart::Part("lobby")) => SettingScope::Lobby,
        Some(PathPart::Part("gamemodes")) => SettingScope::GameModes,
        Some(PathPart::Part("heroes")) => SettingScope::Heroes,
        Some(PathPart::Part("extensions")) => SettingScope::Extensions,
        Some(PathPart::Part("workshop")) => SettingScope::Workshop,
        _ => SettingScope::Unknown,
    }
}

fn target_for(path: &[PathPart<'_>]) -> TargetPattern {
    match path {
        [PathPart::Part("gamemodes"), PathPart::Part("general"), ..] => TargetPattern::Global,
        [PathPart::Part("gamemodes"), PathPart::Part(mode), ..] => {
            TargetPattern::Mode(Some((*mode).to_string()))
        }
        [PathPart::Part("gamemodes"), ..] => TargetPattern::Mode(None),
        [PathPart::Part("heroes"), PathPart::Team, PathPart::Hero, ..] => {
            target_for_hero(path, None)
        }
        [
            PathPart::Part("heroes"),
            PathPart::Part(team),
            PathPart::Hero,
            ..,
        ] => target_for_hero(path, Some((*team).to_string())),
        [PathPart::Part("heroes"), PathPart::Team, ..] => target_for_team(path, None),
        [PathPart::Part("heroes"), PathPart::Part(team), ..] => {
            target_for_team(path, Some((*team).to_string()))
        }
        [
            PathPart::Part("main" | "lobby" | "extensions" | "workshop"),
            ..,
        ] => TargetPattern::Global,
        _ => TargetPattern::Unknown,
    }
}

fn target_for_team(path: &[PathPart<'_>], team: Option<String>) -> TargetPattern {
    match semantic_ability_slot_for_path(path) {
        Some(slot) => TargetPattern::TeamAbility {
            team,
            slot: LogicalSlot::new(slot),
            variant: None,
        },
        None => TargetPattern::Team(team),
    }
}

fn target_for_hero(path: &[PathPart<'_>], team: Option<String>) -> TargetPattern {
    let slot = semantic_ability_slot_for_path(path).map(str::to_string);
    match slot {
        Some(slot) => TargetPattern::HeroAbility {
            team,
            hero: None,
            slot: LogicalSlot::new(slot),
            variant: None,
        },
        None => TargetPattern::Hero { team, hero: None },
    }
}

fn semantic_ability_slot_for_path(path: &[PathPart<'_>]) -> Option<&'static str> {
    match path.last() {
        Some(PathPart::Part("enablePrimaryFire")) => Some("primaryFire"),
        Some(PathPart::Part("enableGenericSecondaryFire")) => Some("secondaryFire"),
        Some(PathPart::Part("enablePassiveUnlimitedFuel")) => Some("passive"),
        Some(PathPart::Part("enablePrimaryFireFreezeStack")) => Some("primaryFire"),
        Some(PathPart::Part(key)) if key.starts_with("ability1") => Some("ability1"),
        Some(PathPart::Part(key)) if key.starts_with("ability2") => Some("ability2"),
        Some(PathPart::Part(key)) if key.starts_with("ability3") => Some("ability3"),
        Some(PathPart::Part(key)) if key.starts_with("secondaryFire") => Some("secondaryFire"),
        _ => table::ability_slot_for_path(path),
    }
}

fn domain_for(kind: KeyKind) -> SettingValueDomain {
    match kind {
        KeyKind::Flag => SettingValueDomain::PresenceOnly,
        KeyKind::String => SettingValueDomain::String,
        KeyKind::Bool => SettingValueDomain::Boolean,
        KeyKind::Number => SettingValueDomain::Number(NumericBounds::unknown()),
        KeyKind::Percent => SettingValueDomain::Percent(NumericBounds::unknown()),
        KeyKind::Enum(domain) => SettingValueDomain::Enum {
            domain: domain.to_string(),
        },
        KeyKind::ListMap => SettingValueDomain::MapList,
        KeyKind::ListHero => SettingValueDomain::HeroList,
    }
}

fn canonical_id(scope: SettingScope, key: &str, path: &[PathPart<'_>]) -> Option<SettingId> {
    let prefix = match scope {
        SettingScope::Main => "main",
        SettingScope::Lobby => "lobby",
        SettingScope::GameModes => "gameMode",
        SettingScope::Heroes => "hero",
        SettingScope::Extensions => "extension",
        SettingScope::Workshop => "workshop",
        SettingScope::Unknown => "unknown",
    };
    if matches!(scope, SettingScope::Unknown)
        || matches!(scope, SettingScope::Heroes) && semantic_ability_slot_for_path(path).is_some()
    {
        return None;
    }
    Some(SettingId::new(format!(
        "setting.{prefix}.{}",
        key.trim_end_matches('%')
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn definition(target: TargetPattern) -> SettingDefinition {
        SettingDefinition {
            identity: SettingIdentity::Known(SettingId::new("setting.test.value")),
            scope: SettingScope::Heroes,
            path: "heroes.test.value".to_string(),
            key: "value",
            target,
            domain: SettingValueDomain::Boolean,
            presentation: SettingPresentation {
                english_name: "Value",
                locale_section: "labels",
            },
            provenance: SettingProvenance {
                kind: SettingEvidenceKind::RawWorkshopFixture,
                source: "test",
                reviewed: true,
            },
        }
    }

    #[test]
    fn common_target_narrowing_rejects_team_and_slot_mismatches() {
        let team = definition(TargetPattern::Team(Some("team1".to_string())));
        assert_eq!(
            team.applicability(&SettingTarget::Hero {
                team: Some(TeamId::new("team2")),
                hero: HeroId::from(crate::gameplay::hero_ids::ANA),
            })
            .expect("applicability"),
            Applicability::NotApplicable
        );

        let team_ability = definition(TargetPattern::TeamAbility {
            team: Some("team1".to_string()),
            slot: LogicalSlot::from(crate::gameplay::slots::PRIMARY_FIRE),
            variant: None,
        });
        let target = SettingTarget::HeroAbility {
            team: Some(TeamId::new("team2")),
            hero: HeroId::from(crate::gameplay::hero_ids::DVA),
            slot: LogicalSlot::from(crate::gameplay::slots::ABILITY_1),
            variant: Some(AbilityVariant::new("mech")),
        };
        assert_eq!(
            team_ability.applicability(&target).expect("applicability"),
            Applicability::NotApplicable
        );
    }
}
