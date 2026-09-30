use super::super::reconciliation;
use super::super::table::{self, KeyKind, TableEntry};
use super::*;

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
            enum_domain: match entry.kind {
                KeyKind::BoolEnum(domain) | KeyKind::Enum(domain) => Some(domain),
                _ => None,
            },
            presentation: SettingPresentation {
                english_name: entry.workshop_name,
                locale_section: "labels",
            },
            source: SettingSource {
                kind: if table::is_generated_entry(entry) {
                    SettingSourceKind::WorkshopDataExport
                } else {
                    SettingSourceKind::RawWorkshopFixture
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
        KeyKind::Bool | KeyKind::YesNo | KeyKind::BoolEnum(_) => SettingValueDomain::Boolean,
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
    errors.extend(reconciliation::validate());
    errors.extend(validate_raw_projection(table::raw_entries()));
    errors.extend(validate_enum_projection(
        table::ENUM_MEMBERS.iter(),
        table::GENERATED_ENUM_MEMBERS.iter(),
        &reconciliation::data().enum_member_mappings,
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
        if !definition.source.reviewed {
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
pub(super) fn validate_raw_projection(
    entries: impl IntoIterator<Item = table::ProjectedEntry>,
) -> Vec<String> {
    use std::collections::HashMap;

    let mut errors = Vec::new();
    let mut paths = HashMap::new();
    for projected in entries {
        let entry = projected.entry;
        if let Some(previous) = paths.insert(entry.path, projected) {
            if previous.entry != entry
                && !reconciled_entry_override(
                    table::path_string(entry.path).as_str(),
                    previous,
                    projected,
                )
            {
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

fn reconciled_entry_override(
    path: &str,
    fixture: table::ProjectedEntry,
    generated: table::ProjectedEntry,
) -> bool {
    use table::ProjectionSource::{FixtureTable, WorkshopDataExport};

    let (fixture, generated) = match (fixture.source, generated.source) {
        (FixtureTable, WorkshopDataExport) => (fixture.entry, generated.entry),
        (WorkshopDataExport, FixtureTable) => (generated.entry, fixture.entry),
        _ => return false,
    };
    reconciliation::data()
        .entry_overrides
        .iter()
        .find(|override_| override_.path == path)
        .is_some_and(|override_| {
            entry_contract_matches(fixture, &override_.fixture)
                && entry_contract_matches(generated, &override_.generated)
        })
}

fn entry_contract_matches(entry: &TableEntry, expected: &reconciliation::EntryContract) -> bool {
    entry.workshop_name == expected.name && key_kind_matches(entry.kind, expected)
}

fn key_kind_matches(kind: KeyKind, expected: &reconciliation::EntryContract) -> bool {
    match (kind, expected.kind.as_str(), expected.domain.as_deref()) {
        (KeyKind::Flag, "flag", None)
        | (KeyKind::String, "string", None)
        | (KeyKind::Bool, "bool", None)
        | (KeyKind::YesNo, "yesNo", None)
        | (KeyKind::Number, "number", None)
        | (KeyKind::Percent, "percent", None)
        | (KeyKind::ListMap, "mapList", None)
        | (KeyKind::ListHero, "heroList", None) => true,
        (KeyKind::BoolEnum(actual), "boolEnum", Some(expected)) => actual == expected,
        (KeyKind::Enum(actual), "enum", Some(expected)) => actual == expected,
        _ => false,
    }
}

/// Validate enum members independently of entry lookup order. This catches
/// both stale enum projections and conflicting duplicate spellings that the
/// lookup helper would otherwise hide.
pub(super) fn validate_enum_projection(
    fixture_entries: impl IntoIterator<Item = &'static table::EnumMember>,
    generated_entries: impl IntoIterator<Item = &'static table::EnumMember>,
    mappings: &[reconciliation::EnumMemberMapping],
) -> Vec<String> {
    use std::collections::{HashMap, HashSet};

    let domains: HashSet<_> = table::entries()
        .filter_map(|entry| match entry.kind {
            KeyKind::Enum(domain) | KeyKind::BoolEnum(domain) => Some(domain),
            _ => None,
        })
        .collect();
    let mut errors = Vec::new();
    let mut members = HashMap::new();
    let mut names = HashMap::new();
    for member in fixture_entries {
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
        if let Some(previous) = names.insert((member.domain, member.name), member.member) {
            if previous != member.member {
                errors.push(format!(
                    "conflicting settings enum display name {}.{:?}: {previous} vs {}",
                    member.domain, member.name, member.member
                ));
            }
        }
    }
    let fixture_members: HashMap<_, _> = table::ENUM_MEMBERS
        .iter()
        .map(|member| ((member.domain, member.member), member))
        .collect();
    let mut mapped_sources = HashSet::new();
    for member in generated_entries {
        let key = (member.domain, member.member);
        if let Some(previous) = members.insert(key, member.name) {
            if previous != member.name {
                errors.push(format!(
                    "conflicting settings enum member {}.{}: {previous:?} vs {:?}",
                    member.domain, member.member, member.name
                ));
            }
        }
        let mapping = mappings.iter().find(|mapping| {
            mapping.source_domain == member.domain && mapping.source_member == member.member
        });
        if mapping.is_none() && !domains.contains(member.domain) {
            errors.push(format!("orphaned settings enum domain: {}", member.domain));
        }
        let (domain, canonical_member, name) = match mapping {
            Some(mapping) => {
                if !mapped_sources.insert((
                    mapping.source_domain.as_str(),
                    mapping.source_member.as_str(),
                )) {
                    errors.push(format!(
                        "duplicate settings enum reconciliation for {}.{}",
                        mapping.source_domain, mapping.source_member
                    ));
                }
                match fixture_members.get(&(
                    mapping.target_domain.as_str(),
                    mapping.target_member.as_str(),
                )) {
                    Some(target) if target.name == member.name => {
                        (target.domain, target.member, target.name)
                    }
                    Some(target) => {
                        errors.push(format!(
                            "settings enum reconciliation name mismatch {}.{} -> {}.{}: {:?} vs {:?}",
                            mapping.source_domain, mapping.source_member,
                            mapping.target_domain, mapping.target_member, member.name, target.name
                        ));
                        continue;
                    }
                    None => {
                        errors.push(format!(
                            "settings enum reconciliation target is missing: {}.{} -> {}.{}",
                            mapping.source_domain,
                            mapping.source_member,
                            mapping.target_domain,
                            mapping.target_member
                        ));
                        continue;
                    }
                }
            }
            None => (member.domain, member.member, member.name),
        };
        if let Some(previous) = names.insert((domain, name), canonical_member) {
            if previous != canonical_member {
                errors.push(format!(
                    "conflicting settings enum display name {}.{name:?}: {previous} vs {canonical_member}",
                    domain
                ));
            }
        }
    }
    for mapping in mappings {
        if !mapped_sources.contains(&(
            mapping.source_domain.as_str(),
            mapping.source_member.as_str(),
        )) {
            errors.push(format!(
                "orphaned settings enum reconciliation: {}.{}",
                mapping.source_domain, mapping.source_member
            ));
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
