//! Canonical typed facts for Workshop custom-game settings.
//!
//! Definitions are a semantic projection of the reviewed settings table. The
//! table remains the parser/emitter lookup source, while [`Settings`] and
//! [`SettingsNode`] remain the source-preserving authored-value carrier.

use std::fmt;

use crate::gameplay::{AbilityVariant, HeroId, LogicalSlot};

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

/// The Workshop-native section that owns a setting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SettingScope {
    Main,
    Lobby,
    GameModes,
    Heroes,
    Extensions,
    Workshop,
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
    Hero,
    HeroAbility {
        slot: LogicalSlot,
        variant: Option<AbilityVariant>,
    },
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
    id: SettingId,
    scope: SettingScope,
    path: String,
    key: &'static str,
    target: TargetPattern,
    domain: SettingValueDomain,
    presentation: SettingPresentation,
    provenance: SettingProvenance,
}

impl SettingDefinition {
    pub fn id(&self) -> &SettingId {
        &self.id
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
            TargetPattern::Hero { .. } => SettingTargetKind::Hero,
            TargetPattern::HeroAbility { slot, variant, .. } => SettingTargetKind::HeroAbility {
                slot: slot.clone(),
                variant: variant.clone(),
            },
        }
    }

    pub fn presentation(&self) -> &SettingPresentation {
        &self.presentation
    }

    pub fn localized_name(&self, locale: &str, target: &SettingTarget) -> Option<&'static str> {
        match target {
            SettingTarget::Hero { hero, .. } | SettingTarget::HeroAbility { hero, .. } => {
                table::hero_setting_name(hero.as_str(), self.key, locale)
                    .or_else(|| self.presentation.localized_name(locale))
            }
            _ => self.presentation.localized_name(locale),
        }
    }

    pub fn provenance(&self) -> SettingProvenance {
        self.provenance
    }

    /// Query effective applicability without exposing table deduplication.
    pub fn applicability(&self, target: &SettingTarget) -> Applicability {
        match (&self.target, target) {
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
                } else if table::hero_name(actual_hero.as_str()).is_none() {
                    Applicability::Unknown
                } else {
                    match table::hero_setting_applicability(actual_hero.as_str(), self.key) {
                        Some(true) => Applicability::Applicable,
                        Some(false) | None => Applicability::NotApplicable,
                    }
                }
            }
            (
                TargetPattern::HeroAbility {
                    team, hero, slot, ..
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
                {
                    return Applicability::NotApplicable;
                }
                if table::hero_name(actual_hero.as_str()).is_none() {
                    Applicability::Unknown
                } else {
                    match table::hero_setting_applicability(actual_hero.as_str(), self.key) {
                        Some(true) => Applicability::Applicable,
                        Some(false) => Applicability::NotApplicable,
                        None => Applicability::NotApplicable,
                    }
                }
            }
            _ => Applicability::NotApplicable,
        }
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
}

fn team_matches(expected: Option<&str>, actual: Option<&TeamId>) -> bool {
    expected.is_none_or(|expected| actual.is_some_and(|actual| actual.as_str() == expected))
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
        let id = SettingId::new(canonical_id(scope, key, entry.path));
        Self {
            id,
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
        _ => SettingScope::Workshop,
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
        [PathPart::Part("heroes"), PathPart::Team, ..] => TargetPattern::Team(None),
        [PathPart::Part("heroes"), PathPart::Part(team), ..] => {
            TargetPattern::Team(Some((*team).to_string()))
        }
        [
            PathPart::Part("main" | "lobby" | "extensions" | "workshop"),
            ..,
        ] => TargetPattern::Global,
        _ => TargetPattern::Global,
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

fn canonical_id(scope: SettingScope, key: &str, path: &[PathPart<'_>]) -> String {
    let prefix = match scope {
        SettingScope::Main => "main",
        SettingScope::Lobby => "lobby",
        SettingScope::GameModes => "gameMode",
        SettingScope::Heroes => "hero",
        SettingScope::Extensions => "extension",
        SettingScope::Workshop => "workshop",
    };
    let concept = match (scope, key, semantic_ability_slot_for_path(path)) {
        (
            SettingScope::Heroes,
            "enablePrimaryFire"
            | "enableSecondaryFire"
            | "enableAbility1"
            | "enableAbility2"
            | "enableAbility3"
            | "enableUlt"
            | "enablePassive"
            | "enableAutomaticFire"
            | "enableScoping",
            Some(_),
        ) => "ability.enabled".to_string(),
        (SettingScope::Heroes, "enableGenericSecondaryFire", Some(_)) => {
            "ability.enabled".to_string()
        }
        (SettingScope::Heroes, "enablePassiveUnlimitedFuel", Some(_)) => {
            "ability.passiveUnlimitedFuel".to_string()
        }
        (SettingScope::Heroes, "enablePrimaryFireFreezeStack", Some(_)) => {
            "ability.primaryFireFreezeStack".to_string()
        }
        (SettingScope::Heroes, "passiveUltGen%", _) => {
            "ability.ultimateGeneration.passive".to_string()
        }
        (SettingScope::Heroes, "combatUltGen%", _) => {
            "ability.ultimateGeneration.combat".to_string()
        }
        (SettingScope::Heroes, "ultGen%", _) => "ability.ultimateGeneration".to_string(),
        (SettingScope::Heroes, key, Some(_)) => ability_concept(key),
        (SettingScope::Heroes, key, _) => key.trim_end_matches('%').to_string(),
        (_, key, _) => key.trim_end_matches('%').to_string(),
    };
    format!("setting.{prefix}.{concept}")
}

fn ability_concept(key: &str) -> String {
    let key = key.trim_end_matches('%');
    let suffix = ["ability1", "ability2", "ability3", "secondaryFire"]
        .iter()
        .find_map(|prefix| key.strip_prefix(prefix))
        .unwrap_or(key);
    let concept = match suffix {
        "Acceleration" => "acceleration",
        "ChargeRate" => "chargeRate",
        "Cooldown" => "cooldown",
        "Distance" => "distance",
        "Duration" => "duration",
        "EnemyKb" => "enemyKnockback",
        "FuseTime" => "fuseTime",
        "Healing" => "healing",
        "Health" => "health",
        "Heat" => "heat",
        "Height" => "height",
        "Kb" => "knockback",
        "MaxDamage" => "maximumDamage",
        "MaxHealing" => "maximumHealing",
        "MaxTime" => "maximumTime",
        "Quantity" => "quantity",
        "RechargeRate" => "rechargeRate",
        "RefuelScalar" => "refuelScalar",
        "SelfKb" => "selfKnockback",
        "Speed" => "speed",
        "AlternateForm" => "alternateForm",
        "Cost" => "resourceCost",
        "EnergyChargeRate" => "energyChargeRate",
        "MaximumTime" => "maximumTime",
        "MovementSpeedPenalty" => "movementSpeedPenalty",
        "RecallDelay" => "recallDelay",
        "Regen" => "regeneration",
        other => return format!("ability.custom.{other}"),
    };
    format!("ability.{concept}")
}
