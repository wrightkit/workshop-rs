use std::ops::Range;

use super::super::table::{self, KeyKind};
use super::*;
use crate::gameplay::{AbilityVariant, GameplayDataError, HeroId, LogicalSlot, data};

impl SettingSourceEdit {
    pub fn range(&self) -> Range<usize> {
        self.edit.range()
    }

    pub fn replacement(&self) -> &str {
        self.edit.replacement()
    }

    /// Apply this edit when the targeted source bytes are unchanged.
    pub fn apply(&self, source: &str) -> Result<String, SettingOperationError> {
        self.edit
            .apply(source)
            .map_err(|_| SettingOperationError::SourceMismatch)
    }
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

    /// The canonical path as typed segments: literal keys and the `<team>`
    /// / `<hero>` template slots a concrete path fills.
    pub(crate) fn path_parts(&self) -> &[PathPart<'static>] {
        self.path_parts
    }

    pub fn domain(&self) -> &SettingValueDomain {
        &self.domain
    }

    /// Enumerate the accepted members when this setting uses an enum value.
    /// Boolean settings backed by an enum token (such as `Enabled`) expose
    /// that token here as well.
    pub fn enum_members(&self) -> impl Iterator<Item = SettingEnumMember> + '_ {
        self.enum_domain
            .into_iter()
            .flat_map(table::enum_members)
            .map(|member| SettingEnumMember {
                domain: member.domain,
                id: member.member,
                english_name: member.name,
            })
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

    pub fn source(&self) -> SettingSource {
        self.source
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
    /// and, when source-backed, its effective numeric value.
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

    /// Build a source-text edit for one existing scalar occurrence.
    ///
    /// The caller retains the original source and applies the returned edit
    /// while its targeted bytes are unchanged. Comments, whitespace, and every
    /// other byte remain outside the edit range and are therefore preserved
    /// without assigning them comment/trivia ownership semantics.
    pub fn source_edit(
        &self,
        source: &str,
        settings: &Settings,
        locale: &str,
        target: &SettingTarget,
        value: SettingValue,
    ) -> Result<SettingSourceEdit, SettingOperationError> {
        let id = self.operation_id()?;
        self.ensure_write_target(target)?;
        let path = self.concrete_path(target);
        let node = find_node(&settings.children, &path).ok_or_else(|| {
            SettingOperationError::NotFound {
                setting: id.clone(),
                target: target.clone(),
            }
        })?;
        validate_value(&self.domain, &id, &value, node.span())?;
        let span = node
            .span()
            .ok_or_else(|| SettingOperationError::SourceUnavailable {
                setting: id.clone(),
            })?;
        let range = source_value_range(source, span, &id)?;
        let kind = table::lookup(self.path_parts)
            .expect("settings definition must retain its table entry")
            .kind;
        let replacement = source_value_spelling(&self.domain, kind, locale, &id, value)?;
        let edit = crate::core::source::SourceEdit::from_source(source, range, replacement)
            .map_err(|_| SettingOperationError::SourceUnavailable {
                setting: id.clone(),
            })?;
        Ok(SettingSourceEdit { edit })
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

fn source_value_range(
    source: &str,
    span: crate::core::source::Span,
    setting: &SettingId,
) -> Result<Range<usize>, SettingOperationError> {
    let start = crate::core::source::byte_offset(source, span.start).ok_or_else(|| {
        SettingOperationError::SourceUnavailable {
            setting: setting.clone(),
        }
    })?;
    let end = crate::core::source::byte_offset(source, span.end).ok_or_else(|| {
        SettingOperationError::SourceUnavailable {
            setting: setting.clone(),
        }
    })?;
    let member =
        source
            .get(start..end)
            .ok_or_else(|| SettingOperationError::SourceUnavailable {
                setting: setting.clone(),
            })?;
    let Some(colon) = member.find(':') else {
        return Err(SettingOperationError::SourceUnavailable {
            setting: setting.clone(),
        });
    };
    let value_start = start + colon + 1;
    let leading = source[value_start..end].len()
        - source[value_start..end]
            .trim_start_matches(char::is_whitespace)
            .len();
    let range = value_start + leading..end;
    if range.is_empty() || source.get(range.clone()).is_none() {
        return Err(SettingOperationError::SourceUnavailable {
            setting: setting.clone(),
        });
    }
    Ok(range)
}

fn source_value_spelling(
    domain: &SettingValueDomain,
    kind: KeyKind,
    locale: &str,
    setting: &SettingId,
    value: SettingValue,
) -> Result<String, SettingOperationError> {
    let localized = |section: &str, english: &str| {
        if locale.eq_ignore_ascii_case("en-US") {
            Some(english)
        } else {
            table::localized_name(locale, section, english)
        }
        .map(str::to_string)
        .ok_or_else(|| SettingOperationError::InvalidValue {
            setting: setting.clone(),
            message: format!("missing {section} locale mapping for '{english}' in {locale}"),
            span: None,
        })
    };
    match (domain, kind, value) {
        (SettingValueDomain::Boolean, KeyKind::Bool, SettingValue::Boolean(value)) => {
            localized("tokens", if value { "On" } else { "Off" })
        }
        (SettingValueDomain::Boolean, KeyKind::YesNo, SettingValue::Boolean(value)) => {
            localized("tokens", if value { "Yes" } else { "No" })
        }
        (SettingValueDomain::Boolean, KeyKind::BoolEnum(domain), SettingValue::Boolean(true)) => {
            let english = table::enum_name(domain, "enabled").ok_or_else(|| {
                SettingOperationError::InvalidValue {
                    setting: setting.clone(),
                    message: format!("unknown enabled member for enum domain '{domain}'"),
                    span: None,
                }
            })?;
            localized("enums", english)
        }
        (SettingValueDomain::Boolean, KeyKind::BoolEnum(_), SettingValue::Boolean(false)) => {
            Err(SettingOperationError::InvalidValue {
                setting: setting.clone(),
                message: "false is unsupported by this Workshop boolean-enum setting".to_string(),
                span: None,
            })
        }
        (SettingValueDomain::Number(_), KeyKind::Number, SettingValue::Number(value)) => {
            Ok(crate::format::format_setting_number(value))
        }
        (SettingValueDomain::Percent(_), KeyKind::Percent, SettingValue::Percent(value)) => {
            Ok(format!("{}%", crate::format::format_setting_number(value)))
        }
        (SettingValueDomain::String, KeyKind::String, SettingValue::String(value)) => Ok(format!(
            "\"{}\"",
            crate::output::emitter::escape_settings_string(&value)
        )),
        (SettingValueDomain::Enum { domain }, KeyKind::Enum(_), SettingValue::Enum(member)) => {
            let english = table::enum_name(domain, &member).ok_or_else(|| {
                SettingOperationError::InvalidValue {
                    setting: setting.clone(),
                    message: format!("unknown member '{member}' for enum domain '{domain}'"),
                    span: None,
                }
            })?;
            localized("enums", english)
        }
        _ => Err(SettingOperationError::SourceUnavailable {
            setting: setting.clone(),
        }),
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

fn validate_value(
    domain: &SettingValueDomain,
    id: &SettingId,
    value: &SettingValue,
    span: Option<crate::core::source::Span>,
) -> Result<(), SettingOperationError> {
    let expected = domain.kind();
    if value.kind() != expected {
        return Err(SettingOperationError::WrongValueKind {
            setting: id.clone(),
            expected,
            actual: value.kind(),
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
                actual: value.kind(),
                span: node.span(),
            });
        }
    }
    Ok(())
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
    data::builtin_ref().map_err(Clone::clone).map(|catalog| {
        catalog.hero(hero).map(|hero| match variant {
            Some(variant) => hero.ability_variant(slot, variant).is_ok(),
            None => !hero.abilities_in_slot(slot).is_empty(),
        })
    })
}
