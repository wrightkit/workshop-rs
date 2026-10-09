//! Locale-independent emission acceptance checks for settings trees.
//!
//! [`crate::Program::validate`] runs [`check_emission`] on every program, so
//! checking raw Workshop input reports the same settings errors emission
//! would. Members the catalog does not declare are carried as written
//! ([`SettingsNode::Raw`], [`SettingsNode::RawValue`], or a block of them) and
//! accepted; [`check_emission`] reports every other member the emitter would
//! reject, without producing Workshop text.
//!
//! Member acceptance is shared with emission. Locale and hero display-name
//! resolution can still fail inside emission and are not checked here.

use crate::core::error::WorkshopError;
use crate::core::suggest;

use super::member::{self, Member, rejected};
use super::table;
use super::{PathPart, Settings, SettingsNode};

/// One rejected settings member: the [`WorkshopError`] a caller would see,
/// plus the canonical spelling it was close to when exactly one candidate
/// qualifies. The error's message already names the suggestion; the field
/// lets callers apply or render it without parsing text.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct SettingsDiagnostic {
    /// The rejection, with its source span and any suggestion in the message.
    pub error: WorkshopError,
    /// The single canonical spelling the rejected input was close to (a case
    /// or accent difference, or a small edit distance). `None` when no
    /// candidate is close or several are. The spelling is a Workshop display
    /// name (`enabled maps`, `Château Guillard`) — the form source text
    /// should carry — not a canonical table key.
    pub suggestion: Option<String>,
}

/// Check that emission accepts `settings`, returning the structured
/// diagnostics a caller can apply: one per offending member, each carrying
/// the node's recorded source span when present.
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
/// member. Each error carries the node's recorded source span when present.
pub fn check_emission(settings: &Settings) -> Vec<WorkshopError> {
    check_emission_diagnostics(settings)
        .into_iter()
        .map(|diagnostic| diagnostic.error)
        .collect()
}

/// A settings member emitted as written rather than through the catalog.
pub(crate) struct UncataloguedMember<'a> {
    /// The uncatalogued key, or the undeclared value of a catalogued key.
    pub(crate) name: &'a str,
    pub(crate) span: Option<crate::source::Span>,
    /// The canonical spelling the name was close to, if exactly one was.
    pub(crate) suggestion: Option<String>,
}

/// The members emission writes as authored: uncatalogued members and blocks,
/// and catalogued keys carrying an undeclared value. Members inside an
/// uncatalogued top-level group are reported individually; a member block is
/// reported as one member.
pub(crate) fn uncatalogued_members(settings: &Settings) -> Vec<UncataloguedMember<'_>> {
    let mut members = Vec::new();
    walk(settings, &mut |visit| match visit {
        Visit::Rejected(_) => {}
        Visit::Member(node, path) => {
            let mut full = path.to_vec();
            full.push(PathPart::Part(node.name()));
            let catalogued = table::lookup(&full);
            let member = match (node, catalogued) {
                (SettingsNode::Raw { .. }, _) | (SettingsNode::Group { .. }, None) => {
                    UncataloguedMember {
                        name: node.name(),
                        span: node.span(),
                        suggestion: suggest::suggest(
                            node.name(),
                            table::key_spellings(path).into_iter(),
                        ),
                    }
                }
                (SettingsNode::RawValue { value, .. }, Some(entry)) => UncataloguedMember {
                    name: value,
                    span: node.span(),
                    suggestion: match entry.kind {
                        table::KeyKind::Enum(domain) => {
                            suggest::suggest(value, table::enum_spellings(domain))
                        }
                        _ => None,
                    },
                },
                (SettingsNode::RawValue { .. }, None) => UncataloguedMember {
                    name: node.name(),
                    span: node.span(),
                    suggestion: suggest::suggest(
                        node.name(),
                        table::key_spellings(path).into_iter(),
                    ),
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

enum Visit<'a, 'p> {
    /// A structural problem emission rejects before reaching any member.
    Rejected(SettingsDiagnostic),
    /// A member at a catalogued position, with its parent path.
    Member(&'a SettingsNode, &'p [PathPart<'a>]),
    /// A leaf inside an uncatalogued top-level group.
    OpaqueLeaf(&'a SettingsNode),
}

/// Visit every settings member the way emission reaches it.
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
            other => {
                visit(Visit::Rejected(rejected(
                    other.span(),
                    "settings block children must be groups".to_string(),
                    None,
                )));
                // A stray raw member is still a preserved residual.
                if matches!(
                    other,
                    SettingsNode::Raw { .. } | SettingsNode::RawValue { .. }
                ) {
                    visit(Visit::OpaqueLeaf(other));
                }
            }
        }
    }
}

fn walk_workshop<'a>(children: &'a [SettingsNode], visit: &mut dyn FnMut(Visit<'a, '_>)) {
    for child in children {
        match child {
            SettingsNode::Group { children, .. } | SettingsNode::Workshop { children, .. } => {
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
                        visit(Visit::Member(inner, &HERO));
                    }
                }
                other => visit(Visit::Member(other, &TEAM)),
            }
        }
    }
}

fn walk_opaque<'a>(children: &'a [SettingsNode], visit: &mut dyn FnMut(Visit<'a, '_>)) {
    for child in children {
        match child {
            // Mirrors `emit_opaque_group`: nested groups pass through
            // verbatim; only leaf members are table-checked.
            SettingsNode::Group { children, .. } => walk_opaque(children, visit),
            other => visit(Visit::OpaqueLeaf(other)),
        }
    }
}

fn check_member(node: &SettingsNode, path: &[PathPart<'_>], errors: &mut Vec<SettingsDiagnostic>) {
    let name = node.name();
    let mut full = path.to_vec();
    full.push(PathPart::Part(name));
    match (node, table::lookup(&full)) {
        (SettingsNode::Raw { .. } | SettingsNode::RawValue { .. }, _) => return,
        // Mirrors `settings_member`: an uncatalogued block is emitted as
        // written; only its leaves must have a written form.
        (SettingsNode::Group { children, .. }, None) => {
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
