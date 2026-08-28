//! Canonical typed facts for Workshop custom-game settings.
//!
//! Definitions are a semantic projection of the reviewed settings table. The
//! table remains the parser/emitter lookup source, while [`Settings`] and
//! [`SettingsNode`] remain the source-preserving authored-value carrier.

use std::fmt;

use crate::gameplay::{AbilityVariant, HeroId, LogicalSlot};
use crate::{gameplay::GameplayDataError, gameplay_data};

use super::table::{self, KeyKind, PathPart, TableEntry};
use super::{Settings, SettingsNode};

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
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

/// A typed authored value in the settings carrier.
#[derive(Debug, Clone, PartialEq)]
pub enum SettingValue {
    Boolean(bool),
    Number(f64),
    Percent(f64),
    String(String),
    Enum(String),
    HeroList(Vec<String>),
    MapList(Vec<String>),
    PresenceOnly,
}

/// A typed occurrence together with an evidenced effective numeric value.
#[derive(Debug, Clone, PartialEq)]
pub struct SettingOccurrence {
    pub authored: SettingValue,
    pub effective: Option<EffectiveNumber>,
}

/// Failure from a typed settings query or source-preserving edit.
#[derive(Debug, Clone, PartialEq)]
pub enum SettingOperationError {
    NotApplicable {
        setting: SettingId,
        target: SettingTarget,
    },
    NotFound {
        setting: SettingId,
        target: SettingTarget,
    },
    ApplicabilityUnknown {
        setting: SettingId,
        target: Box<SettingTarget>,
    },
    WrongValueKind {
        setting: SettingId,
        expected: &'static str,
        actual: &'static str,
        span: Option<crate::source::Span>,
    },
    InvalidValue {
        setting: SettingId,
        message: String,
        span: Option<crate::source::Span>,
    },
}

impl fmt::Display for SettingOperationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotApplicable { setting, target } => {
                write!(
                    formatter,
                    "setting {setting} does not apply to target {target:?}"
                )
            }
            Self::NotFound { setting, target } => {
                write!(
                    formatter,
                    "setting {setting} was not found for target {target:?}"
                )
            }
            Self::ApplicabilityUnknown { setting, target } => write!(
                formatter,
                "applicability of setting {setting} is unknown for target {target:?}"
            ),
            Self::WrongValueKind {
                setting,
                expected,
                actual,
                ..
            } => write!(
                formatter,
                "setting {setting} expects {expected} value, got {actual}"
            ),
            Self::InvalidValue {
                setting, message, ..
            } => write!(formatter, "invalid value for setting {setting}: {message}"),
        }
    }
}

impl std::error::Error for SettingOperationError {}

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
    path_parts: &'static [PathPart<'static>],
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

    /// Read an existing source-preserving occurrence with its authored value
    /// and, when evidenced, its effective numeric value.
    pub fn read(
        &self,
        settings: &Settings,
        target: &SettingTarget,
    ) -> Result<SettingOccurrence, SettingOperationError> {
        let id = self.operation_id()?;
        self.ensure_read_target(target)?;
        let path = self.concrete_path(target);
        let node = find_node(&settings.children, &path).ok_or_else(|| {
            SettingOperationError::NotFound {
                setting: id.clone(),
                target: target.clone(),
            }
        })?;
        let authored = value_from_node(node, &self.domain, &id)?;
        let effective = match authored {
            SettingValue::Number(value) | SettingValue::Percent(value) => {
                self.effective_number(value)
            }
            _ => None,
        };
        Ok(SettingOccurrence {
            authored,
            effective,
        })
    }

    /// Update one existing occurrence without rebuilding the surrounding
    /// settings tree. Unknown and unrelated source structure is untouched.
    pub fn write(
        &self,
        settings: &mut Settings,
        target: &SettingTarget,
        value: SettingValue,
    ) -> Result<(), SettingOperationError> {
        let id = self.operation_id()?;
        self.ensure_write_target(target)?;
        let path = self.concrete_path(target);
        let node = find_node_mut(&mut settings.children, &path).ok_or_else(|| {
            SettingOperationError::NotFound {
                setting: id.clone(),
                target: target.clone(),
            }
        })?;
        let span = node.span();
        validate_value(&self.domain, &id, &value, span)?;
        apply_value(node, &id, value)
    }

    fn ensure_read_target(&self, target: &SettingTarget) -> Result<(), SettingOperationError> {
        let id = self.operation_id()?;
        match self
            .applicability(target)
            .map_err(|error| SettingOperationError::InvalidValue {
                setting: id.clone(),
                message: error.to_string(),
                span: None,
            })? {
            Applicability::NotApplicable => Err(SettingOperationError::NotApplicable {
                setting: id,
                target: target.clone(),
            }),
            Applicability::Applicable | Applicability::Unknown => Ok(()),
        }
    }

    fn ensure_write_target(&self, target: &SettingTarget) -> Result<(), SettingOperationError> {
        let id = self.operation_id()?;
        match self
            .applicability(target)
            .map_err(|error| SettingOperationError::InvalidValue {
                setting: id.clone(),
                message: error.to_string(),
                span: None,
            })? {
            Applicability::NotApplicable => Err(SettingOperationError::NotApplicable {
                setting: id,
                target: target.clone(),
            }),
            Applicability::Unknown => Err(SettingOperationError::ApplicabilityUnknown {
                setting: id,
                target: Box::new(target.clone()),
            }),
            Applicability::Applicable => Ok(()),
        }
    }

    fn operation_id(&self) -> Result<SettingId, SettingOperationError> {
        self.id()
            .cloned()
            .ok_or_else(|| SettingOperationError::InvalidValue {
                setting: SettingId::new("unknown"),
                message: "setting has no reviewed canonical identity".to_string(),
                span: None,
            })
    }

    fn concrete_path(&self, target: &SettingTarget) -> Vec<String> {
        self.path_parts
            .iter()
            .map(|part| match part {
                PathPart::Part(name) => (*name).to_string(),
                PathPart::Team => target_team(target),
                PathPart::Hero => target_hero(target),
            })
            .collect()
    }
}

fn target_team(target: &SettingTarget) -> String {
    match target {
        SettingTarget::Team(team)
        | SettingTarget::Hero {
            team: Some(team), ..
        }
        | SettingTarget::TeamAbility {
            team: Some(team), ..
        }
        | SettingTarget::HeroAbility {
            team: Some(team), ..
        } => team.as_str().to_string(),
        _ => "allTeams".to_string(),
    }
}

fn target_hero(target: &SettingTarget) -> String {
    match target {
        SettingTarget::Hero { hero, .. } | SettingTarget::HeroAbility { hero, .. } => {
            hero.as_str().to_string()
        }
        _ => String::new(),
    }
}

fn find_node<'a>(children: &'a [SettingsNode], path: &[String]) -> Option<&'a SettingsNode> {
    let (name, rest) = path.split_first()?;
    let node = children.iter().find(|node| node.name() == name)?;
    if rest.is_empty() {
        Some(node)
    } else {
        match node {
            SettingsNode::Workshop { children, .. } | SettingsNode::Group { children, .. } => {
                find_node(children, rest)
            }
            _ => None,
        }
    }
}

fn find_node_mut<'a>(
    children: &'a mut [SettingsNode],
    path: &[String],
) -> Option<&'a mut SettingsNode> {
    let (name, rest) = path.split_first()?;
    let node = children.iter_mut().find(|node| node.name() == name)?;
    if rest.is_empty() {
        Some(node)
    } else {
        match node {
            SettingsNode::Workshop { children, .. } | SettingsNode::Group { children, .. } => {
                find_node_mut(children, rest)
            }
            _ => None,
        }
    }
}

fn value_kind(value: &SettingValue) -> &'static str {
    match value {
        SettingValue::Boolean(_) => "boolean",
        SettingValue::Number(_) => "number",
        SettingValue::Percent(_) => "percent",
        SettingValue::String(_) => "string",
        SettingValue::Enum(_) => "enum",
        SettingValue::HeroList(_) => "hero-list",
        SettingValue::MapList(_) => "map-list",
        SettingValue::PresenceOnly => "presence-only",
    }
}

fn domain_kind(domain: &SettingValueDomain) -> &'static str {
    match domain {
        SettingValueDomain::Boolean => "boolean",
        SettingValueDomain::Number(_) => "number",
        SettingValueDomain::Percent(_) => "percent",
        SettingValueDomain::String => "string",
        SettingValueDomain::Enum { .. } => "enum",
        SettingValueDomain::HeroList => "hero-list",
        SettingValueDomain::MapList => "map-list",
        SettingValueDomain::PresenceOnly => "presence-only",
    }
}

fn validate_value(
    domain: &SettingValueDomain,
    id: &SettingId,
    value: &SettingValue,
    span: Option<crate::source::Span>,
) -> Result<(), SettingOperationError> {
    let expected = domain_kind(domain);
    if value_kind(value) != expected {
        return Err(SettingOperationError::WrongValueKind {
            setting: id.clone(),
            expected,
            actual: value_kind(value),
            span,
        });
    }
    match (domain, value) {
        (
            SettingValueDomain::Number(_) | SettingValueDomain::Percent(_),
            SettingValue::Number(value) | SettingValue::Percent(value),
        ) if !value.is_finite() => Err(SettingOperationError::InvalidValue {
            setting: id.clone(),
            message: "numeric settings values must be finite".to_string(),
            span,
        }),
        (SettingValueDomain::Enum { domain }, SettingValue::Enum(member))
            if table::enum_name(domain, member).is_none() =>
        {
            Err(SettingOperationError::InvalidValue {
                setting: id.clone(),
                message: format!("unknown member '{member}' for enum domain '{domain}'"),
                span,
            })
        }
        (SettingValueDomain::HeroList, SettingValue::HeroList(values))
            if values.iter().any(|value| table::hero_name(value).is_none()) =>
        {
            Err(SettingOperationError::InvalidValue {
                setting: id.clone(),
                message: "hero list contains an unknown hero".to_string(),
                span,
            })
        }
        (SettingValueDomain::MapList, SettingValue::MapList(values))
            if values.iter().any(|value| table::map_name(value).is_none()) =>
        {
            Err(SettingOperationError::InvalidValue {
                setting: id.clone(),
                message: "map list contains an unknown map".to_string(),
                span,
            })
        }
        _ => Ok(()),
    }
}

fn value_from_node(
    node: &SettingsNode,
    domain: &SettingValueDomain,
    id: &SettingId,
) -> Result<SettingValue, SettingOperationError> {
    let value = match node {
        SettingsNode::Bool { value, .. } => SettingValue::Boolean(*value),
        SettingsNode::Number { value, .. } => match domain {
            SettingValueDomain::Percent(_) => SettingValue::Percent(*value),
            _ => SettingValue::Number(*value),
        },
        SettingsNode::String { value, .. } => match domain {
            SettingValueDomain::Enum { .. } => SettingValue::Enum(value.clone()),
            _ => SettingValue::String(value.clone()),
        },
        SettingsNode::Flag { .. } => SettingValue::PresenceOnly,
        SettingsNode::List { elements, .. } => {
            let values = elements
                .iter()
                .map(|element| element.value.clone())
                .collect();
            match domain {
                SettingValueDomain::HeroList => SettingValue::HeroList(values),
                _ => SettingValue::MapList(values),
            }
        }
        _ => {
            return Err(SettingOperationError::InvalidValue {
                setting: id.clone(),
                message: "settings occurrence is not a typed leaf".to_string(),
                span: node.span(),
            });
        }
    };
    validate_value(domain, id, &value, node.span())?;
    Ok(value)
}

fn apply_value(
    node: &mut SettingsNode,
    id: &SettingId,
    value: SettingValue,
) -> Result<(), SettingOperationError> {
    match (node, value) {
        (SettingsNode::Bool { value: current, .. }, SettingValue::Boolean(value)) => {
            *current = value
        }
        (
            SettingsNode::Number { value: current, .. },
            SettingValue::Number(value) | SettingValue::Percent(value),
        ) => *current = value,
        (
            SettingsNode::String { value: current, .. },
            SettingValue::String(value) | SettingValue::Enum(value),
        ) => *current = value,
        (
            SettingsNode::List { elements, span, .. },
            SettingValue::HeroList(values) | SettingValue::MapList(values),
        ) => {
            if elements.len() != values.len() {
                return Err(SettingOperationError::InvalidValue {
                    setting: id.clone(),
                    message: "source-preserving list edits cannot change list length".to_string(),
                    span: *span,
                });
            }
            elements
                .iter_mut()
                .zip(values)
                .for_each(|(element, value)| element.value = value);
        }
        (SettingsNode::Flag { .. }, SettingValue::PresenceOnly) => {}
        (node, value) => {
            return Err(SettingOperationError::WrongValueKind {
                setting: id.clone(),
                expected: "existing typed value",
                actual: value_kind(&value),
                span: node.span(),
            });
        }
    }
    Ok(())
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
            catalog.hero(hero).map(|hero| match variant {
                Some(variant) => hero.ability_variant(slot, variant).is_ok(),
                None => !hero.abilities_in_slot(slot).is_empty(),
            })
        })
}

/// Project all currently reviewed table entries into the canonical semantic
/// catalog. The table remains the single parser/emitter source; this
/// projection supplies the stable semantic identity and typed facts consumed
/// by callers.
pub fn definitions() -> impl Iterator<Item = SettingDefinition> {
    table::entries().map(SettingDefinition::from_entry)
}

/// Project one reviewed table entry into the canonical semantic definition.
pub fn definition(path: &[PathPart<'_>]) -> Option<SettingDefinition> {
    table::lookup(path).map(SettingDefinition::from_entry)
}

/// Find all definitions for a canonical concept identity.
///
/// A concept can intentionally have more than one target shape, so the
/// result is an iterator rather than a single definition. This keeps normal
/// consumers independent of the private table paths while retaining the
/// target-specific schema facts.
pub fn definitions_by_id(id: &SettingId) -> impl Iterator<Item = SettingDefinition> {
    definitions().filter(move |definition| definition.id() == Some(id))
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
            path_parts: entry.path,
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
    if matches!(scope, SettingScope::Unknown) {
        return None;
    }
    let concept = canonical_concept(key, path)?;
    Some(SettingId::new(format!("setting.{prefix}.{concept}")))
}

/// Map a Workshop leaf to a locale-independent setting concept. These names
/// intentionally describe the setting's meaning, while hero and logical slot
/// topology stays in `SettingTarget`.
fn canonical_concept(key: &str, path: &[PathPart<'_>]) -> Option<String> {
    let key = key.trim_end_matches('%');
    Some(match key {
        "health" => "health".to_string(),
        "damageDealt" | "damageReceived" | "healingDealt" | "healingReceived" => key.to_string(),
        "passiveUltGen" => "ultimateGeneration.passive".to_string(),
        "combatUltGen" => "ultimateGeneration.combat".to_string(),
        "ultGen" => "ultimateGeneration".to_string(),
        "enableUlt" => "ability.enabled".to_string(),
        "enablePrimaryFire"
        | "enableSecondaryFire"
        | "enableGenericSecondaryFire"
        | "enableAbility1"
        | "enableAbility2"
        | "enableAbility3" => "ability.enabled".to_string(),
        "enableAutomaticFire" => "primaryFire.automaticFireEnabled".to_string(),
        "enableScoping" => "primaryFire.scopingEnabled".to_string(),
        "enablePassiveUnlimitedFuel" => "passive.unlimitedFuelEnabled".to_string(),
        "enablePrimaryFireFreezeStack" => "primaryFire.freezeStackEnabled".to_string(),
        "setValidControlPoints" | "firstActiveControlPoint" => path
            .iter()
            .filter_map(|part| match part {
                PathPart::Part(name) if *name != "gamemodes" && *name != key => Some(*name),
                _ => None,
            })
            .next()
            .map(|mode| format!("{key}.{mode}"))?,
        _ => key.to_string(),
    })
}

/// Validate the effective settings catalog and reject stale or conflicting
/// semantic projections before parser/emitter data is shipped.
pub fn validate_catalog() -> Result<(), Vec<String>> {
    use std::collections::{HashMap, HashSet};

    let mut errors = Vec::new();
    errors.extend(validate_raw_projection(table::raw_entries()));
    errors.extend(validate_enum_projection(
        table::ENUM_MEMBERS
            .iter()
            .chain(table::GENERATED_ENUM_MEMBERS.iter()),
    ));
    let mut paths = HashSet::new();
    let mut concepts: HashMap<(String, SettingTargetKind, String), SettingValueDomain> =
        HashMap::new();
    let mut concept_keys: HashMap<(String, SettingTargetKind), String> = HashMap::new();

    for definition in definitions() {
        if !paths.insert(definition.path.clone()) {
            errors.push(format!("duplicate settings path: {}", definition.path));
        }
        if definition.scope == SettingScope::Unknown {
            errors.push(format!("unknown settings scope: {}", definition.path));
        }
        let Some(id) = definition.id() else {
            errors.push(format!(
                "missing canonical settings identity: {}",
                definition.path
            ));
            continue;
        };
        if !definition.provenance.reviewed {
            errors.push(format!(
                "unreviewed settings definition: {}",
                definition.path
            ));
        }
        if definition.presentation.english_name.is_empty() {
            errors.push(format!(
                "missing settings presentation: {}",
                definition.path
            ));
        }
        let target_kind = definition.target_kind();
        let semantic_key = semantic_identity_key(definition.key);
        let collision_key = (id.as_str().to_string(), target_kind.clone());
        if let Some(previous_key) = concept_keys.insert(collision_key, semantic_key.clone()) {
            if previous_key != semantic_key {
                errors.push(format!(
                    "conflicting settings concepts for {id}: {previous_key} vs {semantic_key}"
                ));
            }
        }
        let key = (id.as_str().to_string(), target_kind, semantic_key);
        if let Some(previous) = concepts.insert(key, definition.domain.clone()) {
            if previous != definition.domain {
                errors.push(format!("conflicting settings domains for {id}"));
            }
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Reject raw table overlaps unless their complete parser/emitter contract is
/// identical. Effective lookup may deduplicate exact repeats, but must never
/// make a divergent generated or fixture projection silently win.
fn validate_raw_projection(
    entries: impl IntoIterator<Item = table::ProjectedEntry>,
) -> Vec<String> {
    use std::collections::HashMap;

    let mut errors = Vec::new();
    let mut paths = HashMap::new();
    for projected in entries {
        let entry = projected.entry;
        if let Some(previous) = paths.insert(entry.path, projected) {
            if previous.entry != entry {
                errors.push(format!(
                    "conflicting duplicate settings path between {} and {}: {}",
                    previous.source.label(),
                    projected.source.label(),
                    table::path_string(entry.path),
                ));
            }
        }
    }
    errors
}

/// Validate enum members independently of entry lookup order. This catches
/// both stale enum projections and conflicting duplicate spellings that the
/// lookup helper would otherwise hide.
fn validate_enum_projection(
    entries: impl IntoIterator<Item = &'static table::EnumMember>,
) -> Vec<String> {
    use std::collections::{HashMap, HashSet};

    let domains: HashSet<_> = table::entries()
        .filter_map(|entry| match entry.kind {
            KeyKind::Enum(domain) => Some(domain),
            _ => None,
        })
        .collect();
    let mut errors = Vec::new();
    let mut members = HashMap::new();
    for member in entries {
        if !domains.contains(member.domain) {
            errors.push(format!("orphaned settings enum domain: {}", member.domain));
        }
        let key = (member.domain, member.member);
        if let Some(previous) = members.insert(key, member.name) {
            if previous != member.name {
                errors.push(format!(
                    "conflicting settings enum member {}.{}: {previous:?} vs {:?}",
                    member.domain, member.member, member.name
                ));
            }
        }
    }
    errors
}

fn semantic_identity_key(key: &str) -> String {
    match key {
        "enableSecondaryFire" | "enableGenericSecondaryFire" => "enableSecondaryFire".to_string(),
        _ => key.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static DUPLICATE_PATH: [PathPart<'static>; 2] =
        [PathPart::Part("test"), PathPart::Part("value")];
    static FIXTURE_ENTRY: TableEntry = TableEntry {
        path: &DUPLICATE_PATH,
        workshop_name: "Fixture Value",
        kind: KeyKind::Bool,
    };
    static GENERATED_ENTRY: TableEntry = TableEntry {
        path: &DUPLICATE_PATH,
        workshop_name: "Generated Value",
        kind: KeyKind::Bool,
    };
    static FIXTURE_ENUM_MEMBER: table::EnumMember = table::EnumMember {
        domain: "mapRotation",
        member: "afterGame",
        name: "After A Game",
    };
    static GENERATED_ENUM_MEMBER: table::EnumMember = table::EnumMember {
        domain: "mapRotation",
        member: "afterGame",
        name: "After Game",
    };

    fn definition(target: TargetPattern) -> SettingDefinition {
        SettingDefinition {
            identity: SettingIdentity::Known(SettingId::new("setting.test.value")),
            scope: SettingScope::Heroes,
            path: "heroes.test.value".to_string(),
            path_parts: &[],
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

    #[test]
    fn raw_projection_conflicts_include_presentation_contract() {
        let errors = validate_raw_projection([
            table::ProjectedEntry {
                source: table::ProjectionSource::FixtureTable,
                entry: &FIXTURE_ENTRY,
            },
            table::ProjectedEntry {
                source: table::ProjectionSource::WorkshopDataExport,
                entry: &GENERATED_ENTRY,
            },
        ]);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("fixture table"));
        assert!(errors[0].contains("Workshop-data export"));
    }

    #[test]
    fn enum_projection_conflicts_are_not_hidden_by_lookup_order() {
        let errors = validate_enum_projection([&FIXTURE_ENUM_MEMBER, &GENERATED_ENUM_MEMBER]);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("mapRotation.afterGame"));
    }
}
