//! Locale-independent emission acceptance checks for settings trees.
//!
//! [`crate::Program::validate`] runs [`check_emission`] on every program, so
//! checking raw Workshop input reports the same settings errors emission
//! would. The raw parser leaves unresolvable leaf members as verbatim
//! [`SettingsNode::Raw`] payloads; [`check_emission`] reports every other
//! member the emitter would reject, without producing Workshop text.
//!
//! The checks mirror [`super::emitter`]'s member handling and must stay in
//! sync with it: an empty result means `emit` will not reject a settings
//! member for emission-table acceptance. Locale and hero display-name
//! resolution can still fail inside emission and are not checked here.

use crate::core::error::WorkshopError;
use crate::source::Span;

use super::table::{self, KeyKind};
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

fn rejected(span: Option<Span>, message: String, suggestion: Option<String>) -> SettingsDiagnostic {
    let message = suggest::with_suggestion_text(message, suggestion.as_deref());
    SettingsDiagnostic {
        error: WorkshopError::malformed(message, span),
        suggestion,
    }
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
    let Some(entry) = table::lookup(&full) else {
        errors.push(rejected(
            node.span(),
            format!(
                "settings key '{}' is outside the emission table",
                table::path_string(&full)
            ),
            suggest::suggest(name, table::key_spellings(path).into_iter()),
        ));
        return;
    };
    match (node, &entry.kind) {
        (SettingsNode::Flag { .. }, KeyKind::Flag)
        | (SettingsNode::String { .. }, KeyKind::String)
        | (SettingsNode::Number { .. }, KeyKind::Number | KeyKind::Percent)
        | (SettingsNode::Bool { .. }, KeyKind::Bool | KeyKind::YesNo) => {}
        (SettingsNode::String { value, .. }, KeyKind::Enum(domain)) => {
            if table::enum_name(domain, value).is_none() {
                errors.push(rejected(
                    node.span(),
                    format!("unknown value '{value}' for settings key '{name}'"),
                    suggest::suggest(value, table::enum_spellings(domain)),
                ));
            }
        }
        (SettingsNode::Bool { value, .. }, KeyKind::BoolEnum(domain)) => {
            if !*value {
                errors.push(rejected(
                    node.span(),
                    format!("unsupported false value for settings key '{name}'"),
                    None,
                ));
            } else if table::enum_name(domain, "enabled").is_none() {
                errors.push(rejected(
                    node.span(),
                    format!("unknown value 'enabled' for settings key '{name}'"),
                    None,
                ));
            }
        }
        (SettingsNode::List { elements, .. }, KeyKind::ListMap) => {
            for element in elements {
                if table::map_name(&element.value).is_none() {
                    errors.push(rejected(
                        element.span,
                        format!("unknown map '{}' in settings list '{name}'", element.value),
                        suggest::suggest(&element.value, table::map_spellings()),
                    ));
                }
            }
        }
        (SettingsNode::List { elements, .. }, KeyKind::ListHero) => {
            for element in elements {
                if table::hero_name(&element.value).is_none() {
                    errors.push(rejected(
                        element.span,
                        format!("unknown hero '{}' in settings list '{name}'", element.value),
                        suggest::suggest(&element.value, table::hero_spellings()),
                    ));
                }
            }
        }
        _ => errors.push(rejected(
            node.span(),
            format!("settings key '{name}' does not match its table kind"),
            None,
        )),
    }
}
