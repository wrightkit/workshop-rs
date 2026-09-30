//! Canonical typed facts for Workshop custom-game settings.
//!
//! Definitions are a semantic projection of the reviewed settings table. The
//! table remains the parser/emitter lookup source, while [`Settings`] and
//! [`SettingsNode`] remain the source-preserving authored-value carrier.

use std::fmt;

use crate::gameplay::{AbilityVariant, HeroId, LogicalSlot};

#[cfg(test)]
use super::reconciliation;
#[cfg(test)]
use super::table::TableEntry;
#[cfg(test)]
use super::table::{self, KeyKind};
use super::{PathPart, Settings, SettingsNode};

mod operations;
mod projection;
pub use projection::{definition, definitions, definitions_by_id, validate_catalog};
#[cfg(test)]
use projection::{validate_enum_projection, validate_raw_projection};

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
#[non_exhaustive]
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
#[non_exhaustive]
pub enum NumericBoundsError {
    NonFinite,
    Reversed,
}

/// Source-backed effective numeric bounds. `None` means the current reviewed
/// source does not establish that bound.
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
#[non_exhaustive]
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
    /// The machine-readable kind name of this domain.
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Boolean => "boolean",
            Self::Number(_) => "number",
            Self::Percent(_) => "percent",
            Self::String => "string",
            Self::Enum { .. } => "enum",
            Self::HeroList => "hero-list",
            Self::MapList => "map-list",
            Self::PresenceOnly => "presence-only",
        }
    }
}

/// One accepted spelling of a setting enum member.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SettingEnumMember {
    domain: &'static str,
    id: &'static str,
    english_name: &'static str,
}

impl SettingEnumMember {
    pub fn domain(&self) -> &str {
        self.domain
    }

    pub fn id(&self) -> &str {
        self.id
    }

    pub fn english_name(&self) -> &str {
        self.english_name
    }
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

impl SettingValue {
    /// The machine-readable kind name of this value, mirroring
    /// [`SettingValueDomain::kind`].
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Boolean(_) => "boolean",
            Self::Number(_) => "number",
            Self::Percent(_) => "percent",
            Self::String(_) => "string",
            Self::Enum(_) => "enum",
            Self::HeroList(_) => "hero-list",
            Self::MapList(_) => "map-list",
            Self::PresenceOnly => "presence-only",
        }
    }
}

/// A typed occurrence together with a source-backed effective numeric value.
#[derive(Debug, Clone, PartialEq)]
pub struct SettingOccurrence {
    pub authored: SettingValue,
    pub effective: Option<EffectiveNumber>,
}

/// One checked replacement in the original Workshop source text.
///
/// The edit changes only `range`; [`Self::apply`] refuses a source buffer whose
/// bytes at that range no longer equal `expected`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingSourceEdit {
    edit: crate::core::source::SourceEdit,
}

/// Failure from a typed settings query or source-preserving edit.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
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
        span: Option<crate::core::source::Span>,
    },
    InvalidValue {
        setting: SettingId,
        message: String,
        span: Option<crate::core::source::Span>,
    },
    SourceUnavailable {
        setting: SettingId,
    },
    SourceMismatch,
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
            Self::SourceUnavailable { setting } => {
                write!(formatter, "setting {setting} has no editable source")
            }
            Self::SourceMismatch => formatter.write_str("source no longer matches the edit"),
        }
    }
}

impl std::error::Error for SettingOperationError {}

impl SettingValueDomain {
    /// Apply source-backed effective clamping without changing the authored
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

/// Source metadata shared by the reviewed table projection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct SettingSource {
    pub kind: SettingSourceKind,
    pub source: &'static str,
    pub reviewed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingSourceKind {
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
    enum_domain: Option<&'static str>,
    presentation: SettingPresentation,
    source: SettingSource,
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
        member: "afterAGame",
        name: "After A Game",
    };
    static GENERATED_ENUM_MEMBER: table::EnumMember = table::EnumMember {
        domain: "mapRotation",
        member: "afterAGame",
        name: "After Game",
    };
    static DISPLAY_NAME_COLLISION: table::EnumMember = table::EnumMember {
        domain: "mapRotation",
        member: "afterMirrorMatch",
        name: "After A Game",
    };
    static EXPORT_ENUM_MEMBER: table::EnumMember = table::EnumMember {
        domain: "setting_lobby_mapRotation",
        member: "afterGame",
        name: "After A Game",
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
            enum_domain: None,
            presentation: SettingPresentation {
                english_name: "Value",
                locale_section: "labels",
            },
            source: SettingSource {
                kind: SettingSourceKind::RawWorkshopFixture,
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
        let errors =
            validate_enum_projection([&FIXTURE_ENUM_MEMBER], [&GENERATED_ENUM_MEMBER], &[]);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("mapRotation.afterAGame"));
    }

    #[test]
    fn enum_projection_rejects_display_name_to_identity_collisions() {
        let errors =
            validate_enum_projection([&FIXTURE_ENUM_MEMBER, &DISPLAY_NAME_COLLISION], [], &[]);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("conflicting settings enum display name"));
    }

    #[test]
    fn enum_projection_reconciles_export_members_to_canonical_identities() {
        let mappings = [reconciliation::EnumMemberMapping {
            source_domain: "setting_lobby_mapRotation".to_string(),
            source_member: "afterGame".to_string(),
            target_domain: "mapRotation".to_string(),
            target_member: "afterAGame".to_string(),
        }];
        let errors =
            validate_enum_projection([&FIXTURE_ENUM_MEMBER], [&EXPORT_ENUM_MEMBER], &mappings);
        assert!(errors.is_empty(), "{errors:?}");
    }
}
