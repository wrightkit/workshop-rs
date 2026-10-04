//! Locale-independent emission acceptance checks for settings trees.
//!
//! [`crate::Program::validate`] runs [`check_emission`] on every program, so
//! checking raw Workshop input reports the same settings errors emission
//! would. The raw parser leaves unresolvable leaf members as verbatim
//! [`SettingsNode::Raw`] payloads; [`check_emission`] reports every other
//! member the emitter would reject, without producing Workshop text.
//!
//! Member acceptance is shared with emission. Locale and hero display-name
//! resolution can still fail inside emission and are not checked here.

use crate::core::error::WorkshopError;

use super::member::{self, Member, rejected};
use super::table;
use super::{PathPart, Settings, SettingsNode, suggest};

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
    for child in &settings.children {
        match child {
            SettingsNode::Workshop { children, .. } => {
                check_workshop_children(children, &mut diagnostics)
            }
            SettingsNode::Group { name, children, .. } => match name.as_str() {
                "main" | "lobby" => {
                    for member in children {
                        check_member(member, &[PathPart::Part(name)], &mut diagnostics);
                    }
                }
                "gamemodes" => check_modes(children, &mut diagnostics),
                "heroes" => check_heroes(children, &mut diagnostics),
                "extensions" => {
                    for member in children {
                        check_member(member, &[PathPart::Part("extensions")], &mut diagnostics);
                    }
                }
                _ => check_opaque_children(children, &mut diagnostics),
            },
            other => diagnostics.push(rejected(
                other.span(),
                "settings block children must be groups".to_string(),
                None,
            )),
        }
    }
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

fn check_workshop_children(children: &[SettingsNode], errors: &mut Vec<SettingsDiagnostic>) {
    for child in children {
        match child {
            SettingsNode::Group { children, .. } | SettingsNode::Workshop { children, .. } => {
                check_workshop_children(children, errors);
            }
            SettingsNode::Raw { .. } => {}
            other => errors.push(rejected(
                other.span(),
                "settings.workshop contains a typed builtin setting".to_string(),
                None,
            )),
        }
    }
}

fn check_modes(modes: &[SettingsNode], errors: &mut Vec<SettingsDiagnostic>) {
    for mode in modes {
        let SettingsNode::Group { name, children, .. } = mode else {
            errors.push(rejected(
                mode.span(),
                "mode entries must be groups".to_string(),
                None,
            ));
            continue;
        };
        for member in children {
            // `enabled` bools are consumed by the mode header and never reach
            // the emission table.
            if matches!(member, SettingsNode::Bool { name, .. } if name == "enabled") {
                continue;
            }
            check_member(
                member,
                &[PathPart::Part("gamemodes"), PathPart::Part(name)],
                errors,
            );
        }
    }
}

fn check_heroes(teams: &[SettingsNode], errors: &mut Vec<SettingsDiagnostic>) {
    for team in teams {
        let SettingsNode::Group { name, children, .. } = team else {
            errors.push(rejected(
                team.span(),
                "team entries must be groups".to_string(),
                None,
            ));
            continue;
        };
        if table::team_name(name).is_none() {
            errors.push(rejected(
                team.span(),
                format!("unknown team '{name}'"),
                suggest::suggest(name, table::team_spellings()),
            ));
            continue;
        }
        for member in children {
            match member {
                SettingsNode::Group { name, children, .. } => {
                    if table::hero_name(name).is_none() {
                        errors.push(rejected(
                            member.span(),
                            format!("unknown hero '{name}'"),
                            suggest::suggest(name, table::hero_spellings()),
                        ));
                        continue;
                    }
                    for inner in children {
                        check_member(
                            inner,
                            &[PathPart::Part("heroes"), PathPart::Team, PathPart::Hero],
                            errors,
                        );
                    }
                }
                other => check_member(other, &[PathPart::Part("heroes"), PathPart::Team], errors),
            }
        }
    }
}

fn check_opaque_children(children: &[SettingsNode], errors: &mut Vec<SettingsDiagnostic>) {
    for child in children {
        match child {
            // Mirrors `emit_opaque_group`: nested groups pass through
            // verbatim; only leaf members are table-checked.
            SettingsNode::Group { children, .. } => check_opaque_children(children, errors),
            other => check_member(other, &[], errors),
        }
    }
}

fn check_member(node: &SettingsNode, path: &[PathPart<'_>], errors: &mut Vec<SettingsDiagnostic>) {
    if matches!(node, SettingsNode::Raw { .. }) {
        return;
    }
    let name = node.name();
    let mut full = path.to_vec();
    full.push(PathPart::Part(name));
    match member::lookup(node, &full).and_then(|entry| member::accept(node, entry)) {
        Ok(Member::List { elements, kind }) => {
            for element in elements {
                if let Err(diagnostic) = kind.resolve(element, name) {
                    errors.push(diagnostic);
                }
            }
        }
        Ok(_) => {}
        Err(diagnostic) => errors.push(diagnostic),
    }
}
