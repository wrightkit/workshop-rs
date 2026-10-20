use crate::core::error::WorkshopError;
use crate::core::suggest;

use super::member::{self, Member, rejected};
use super::table;
use super::{PathPart, Settings, SettingsNode};

/// Whether a settings diagnostic rejects the member or only advises about
/// it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    /// Emission rejects the member; [`crate::Program::validate`] fails.
    Error,
    /// The member is carried and emitted as written, but it is close to a
    /// declared spelling and is likely a misspelling. Warnings never fail
    /// validation.
    Warning,
}

/// One reported settings member: the [`WorkshopError`] a caller would see,
/// plus the canonical spelling it was close to when exactly one candidate
/// qualifies. The error's message already names the suggestion; the field
/// lets callers apply or render it without parsing text.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct SettingsDiagnostic {
    /// Whether the diagnostic rejects the member or only advises about it.
    pub severity: DiagnosticSeverity,
    /// The rejection or advisory, with its source span and any suggestion in
    /// the message. Warnings carry [`WorkshopError::Warning`].
    pub error: WorkshopError,
    /// The single canonical spelling the reported input was close to (a case
    /// or accent difference, or a small edit distance). `None` when no
    /// candidate is close or several are. The spelling is a Workshop display
    /// name (`enabled maps`, `Château Guillard`) — the form source text
    /// should carry — not a canonical table key.
    pub suggestion: Option<String>,
}

/// Check that emission accepts `settings`, returning the structured
/// diagnostics a caller can apply: one per offending member (rejected, or a
/// carried near-miss of a declared spelling), each carrying the node's
/// recorded source span when present.
pub fn check_emission_diagnostics(settings: &Settings) -> Vec<SettingsDiagnostic> {
    let mut diagnostics = Vec::new();
    walk(settings, &mut |visit| match visit {
        Visit::Rejected(diagnostic) => diagnostics.push(diagnostic),
        Visit::Member(node, path) => check_member(node, path, &mut diagnostics),
        Visit::OpaqueLeaf(node) => check_member(node, &[], &mut diagnostics),
    });
    diagnostics
}

/// Check that emission accepts `settings`, returning one error per offending
/// member. Warnings are excluded: they describe accepted members. Each error
/// carries the node's recorded source span when present.
pub fn check_emission(settings: &Settings) -> Vec<WorkshopError> {
    check_emission_diagnostics(settings)
        .into_iter()
        .filter(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error)
        .map(|diagnostic| diagnostic.error)
        .collect()
}

pub(crate) struct UncataloguedMember<'a> {
    pub(crate) name: &'a str,
    pub(crate) span: Option<crate::source::Span>,
    pub(crate) suggestion: Option<String>,
}

pub(crate) fn uncatalogued_members(settings: &Settings) -> Vec<UncataloguedMember<'_>> {
    let mut members = Vec::new();
    walk(settings, &mut |visit| match visit {
        Visit::Rejected(_) => {}
        Visit::Member(node, path) => {
            let mut full = path.to_vec();
            full.push(PathPart::Part(node.name()));
            let catalogued = table::lookup(&full);
            let member = match (node, catalogued) {
                (
                    SettingsNode::Raw { .. }
                    | SettingsNode::Group { .. }
                    | SettingsNode::RawGroup { .. },
                    _,
                ) => UncataloguedMember {
                    name: node.name(),
                    span: node.span(),
                    suggestion: carried_suggestion(node, path),
                },
                (SettingsNode::RawValue { value, .. }, Some(_)) => UncataloguedMember {
                    name: value,
                    span: node.span(),
                    suggestion: carried_suggestion(node, path),
                },
                (SettingsNode::RawValue { .. }, None) => UncataloguedMember {
                    name: node.name(),
                    span: node.span(),
                    suggestion: carried_suggestion(node, path),
                },
                _ => return,
            };
            members.push(member);
        }
        Visit::OpaqueLeaf(node @ (SettingsNode::Raw { .. } | SettingsNode::RawValue { .. })) => {
            members.push(UncataloguedMember {
                name: node.name(),
                span: node.span(),
                suggestion: None,
            });
        }
        Visit::OpaqueLeaf(_) => {}
    });
    members
}

pub(crate) fn carried_suggestion(node: &SettingsNode, path: &[PathPart<'_>]) -> Option<String> {
    let mut full = path.to_vec();
    full.push(PathPart::Part(node.name()));
    match (node, table::lookup(&full)) {
        (SettingsNode::Raw { .. }, _)
        | (SettingsNode::Group { .. } | SettingsNode::RawGroup { .. }, None) => {
            suggest::suggest(node.name(), table::key_spellings(path).into_iter())
        }
        (SettingsNode::RawValue { value, .. }, Some(entry)) => match entry.kind {
            table::KeyKind::Enum(domain) => suggest::suggest(value, table::enum_spellings(domain)),
            _ => None,
        },
        (SettingsNode::RawValue { .. }, None) => {
            suggest::suggest(node.name(), table::key_spellings(path).into_iter())
        }
        _ => None,
    }
}

enum Visit<'a, 'p> {
    Rejected(SettingsDiagnostic),
    Member(&'a SettingsNode, &'p [PathPart<'a>]),
    OpaqueLeaf(&'a SettingsNode),
}

fn walk<'a>(settings: &'a Settings, visit: &mut dyn FnMut(Visit<'a, '_>)) {
    for child in &settings.children {
        match child {
            SettingsNode::Workshop { children, .. } => walk_workshop(children, visit),
            SettingsNode::Group { name, children, .. } => match name.as_str() {
                "main" | "lobby" | "extensions" => {
                    for member in children {
                        visit(Visit::Member(member, &[PathPart::Part(name)]));
                    }
                }
                "gamemodes" => walk_modes(children, visit),
                "heroes" => walk_heroes(children, visit),
                _ => walk_opaque(children, visit),
            },
            SettingsNode::RawGroup { children, .. } => walk_opaque(children, visit),
            other => reject_stray_member(other, visit),
        }
    }
}

fn reject_stray_member<'a>(node: &'a SettingsNode, visit: &mut dyn FnMut(Visit<'a, '_>)) {
    visit(Visit::Rejected(rejected(
        node.span(),
        "settings block children must be groups".to_string(),
        None,
    )));
    if matches!(
        node,
        SettingsNode::Raw { .. } | SettingsNode::RawValue { .. }
    ) {
        visit(Visit::OpaqueLeaf(node));
    }
}

fn walk_workshop<'a>(children: &'a [SettingsNode], visit: &mut dyn FnMut(Visit<'a, '_>)) {
    for child in children {
        match child {
            SettingsNode::Group { children, .. }
            | SettingsNode::Workshop { children, .. }
            | SettingsNode::RawGroup { children, .. } => {
                walk_workshop(children, visit);
            }
            SettingsNode::Raw { .. } | SettingsNode::RawValue { .. } => {}
            other => visit(Visit::Rejected(rejected(
                other.span(),
                "settings.workshop contains a typed builtin setting".to_string(),
                None,
            ))),
        }
    }
}

fn walk_modes<'a>(modes: &'a [SettingsNode], visit: &mut dyn FnMut(Visit<'a, '_>)) {
    for mode in modes {
        if let SettingsNode::RawGroup { children, .. } = mode {
            walk_opaque(children, visit);
            continue;
        }
        let SettingsNode::Group { name, children, .. } = mode else {
            visit(Visit::Rejected(rejected(
                mode.span(),
                "mode entries must be groups".to_string(),
                None,
            )));
            continue;
        };
        let path = [PathPart::Part("gamemodes"), PathPart::Part(name)];
        for member in children {
            // `enabled` bools are consumed by the mode header and never reach
            // the emission table.
            if matches!(member, SettingsNode::Bool { name, .. } if name == "enabled") {
                continue;
            }
            visit(Visit::Member(member, &path));
        }
    }
}

fn walk_heroes<'a>(teams: &'a [SettingsNode], visit: &mut dyn FnMut(Visit<'a, '_>)) {
    const TEAM: [PathPart<'static>; 2] = [PathPart::Part("heroes"), PathPart::Team];
    const HERO: [PathPart<'static>; 3] = [PathPart::Part("heroes"), PathPart::Team, PathPart::Hero];
    for team in teams {
        if let SettingsNode::RawGroup { children, .. } = team {
            walk_opaque(children, visit);
            continue;
        }
        let SettingsNode::Group { name, children, .. } = team else {
            visit(Visit::Rejected(rejected(
                team.span(),
                "team entries must be groups".to_string(),
                None,
            )));
            continue;
        };
        if table::team_name(name).is_none() {
            visit(Visit::Rejected(rejected(
                team.span(),
                format!("unknown team '{name}'"),
                suggest::suggest(name, table::team_spellings()),
            )));
            continue;
        }
        for member in children {
            match member {
                SettingsNode::RawGroup { children, .. } => walk_opaque(children, visit),
                SettingsNode::Group { name, children, .. } => {
                    if table::hero_name(name).is_none() {
                        visit(Visit::Rejected(rejected(
                            member.span(),
                            format!("unknown hero '{name}'"),
                            suggest::suggest(name, table::hero_spellings()),
                        )));
                        continue;
                    }
                    for inner in children {
                        if let Some(diagnostic) = unemittable_hero_member(inner, name) {
                            visit(Visit::Rejected(diagnostic));
                        }
                        visit(Visit::Member(inner, &HERO));
                    }
                }
                other => visit(Visit::Member(other, &TEAM)),
            }
        }
    }
}

fn unemittable_hero_member(node: &SettingsNode, hero: &str) -> Option<SettingsDiagnostic> {
    if matches!(
        node,
        SettingsNode::Raw { .. } | SettingsNode::RawGroup { .. }
    ) {
        return None;
    }
    let key = node.name();
    let slot = table::ability_slot_for_path(&[
        PathPart::Part("heroes"),
        PathPart::Team,
        PathPart::Hero,
        PathPart::Part(key),
    ])?;
    if table::hero_setting_name(hero, key, "en-US").is_some() {
        return None;
    }
    hero_provably_lacks_slot(hero, slot).then(|| {
        rejected(
            node.span(),
            format!("hero '{hero}' has no '{slot}' ability for settings key '{key}'"),
            None,
        )
    })
}

fn hero_provably_lacks_slot(hero: &str, slot: &str) -> bool {
    crate::gameplay::data::builtin_ref()
        .ok()
        .and_then(|catalog| catalog.hero(&crate::gameplay::HeroId::new(hero)))
        .is_some_and(|hero| {
            hero.abilities_in_slot(&crate::gameplay::LogicalSlot::new(slot))
                .is_empty()
        })
}

fn warn_near_miss(
    node: &SettingsNode,
    path: &[PathPart<'_>],
    spelling: String,
) -> SettingsDiagnostic {
    let mut full = path.to_vec();
    full.push(PathPart::Part(node.name()));
    let message = match node {
        SettingsNode::RawValue { value, .. } if table::lookup(&full).is_some() => format!(
            "settings value '{value}' for key '{}' is not declared",
            table::path_string(&full)
        ),
        other => format!("settings key '{}' is not declared", other.name()),
    };
    let message = suggest::with_suggestion_text(message, Some(&spelling));
    SettingsDiagnostic {
        severity: DiagnosticSeverity::Warning,
        error: WorkshopError::warning(message, node.span()),
        suggestion: Some(spelling),
    }
}

fn walk_opaque<'a>(children: &'a [SettingsNode], visit: &mut dyn FnMut(Visit<'a, '_>)) {
    for child in children {
        match child {
            SettingsNode::Group { children, .. } | SettingsNode::RawGroup { children, .. } => {
                walk_opaque(children, visit)
            }
            other => visit(Visit::OpaqueLeaf(other)),
        }
    }
}

fn check_member(node: &SettingsNode, path: &[PathPart<'_>], errors: &mut Vec<SettingsDiagnostic>) {
    let name = node.name();
    let mut full = path.to_vec();
    full.push(PathPart::Part(name));
    if !path.is_empty()
        && let Some(spelling) = carried_suggestion(node, path)
    {
        errors.push(warn_near_miss(node, path, spelling));
    }
    match (node, table::lookup(&full)) {
        (SettingsNode::Raw { .. } | SettingsNode::RawValue { .. }, _) => return,
        // Mirrors `settings_member`: a block is carried as written whether
        // or not its key is catalogued; only its leaves must have a written
        // form (#412).
        (SettingsNode::Group { children, .. } | SettingsNode::RawGroup { children, .. }, _) => {
            for child in children {
                check_member(child, &[], errors);
            }
            return;
        }
        _ => {}
    }
    match member::lookup(node, &full).and_then(|entry| member::accept(node, entry)) {
        Ok(Member::List { elements, kind }) => {
            for element in elements {
                if let Err(diagnostic) = kind.resolve(element, name) {
                    errors.push(*diagnostic);
                }
            }
        }
        Ok(_) => {}
        Err(diagnostic) => errors.push(*diagnostic),
    }
}
