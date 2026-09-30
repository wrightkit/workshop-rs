//! Locale-independent emission acceptance checks for settings trees.
//!
//! The raw Workshop parser leaves unresolvable members as verbatim
//! [`SettingsNode::Raw`] payloads, so only producers that construct typed
//! trees can reach emission-table errors. [`check_emission`] reports every
//! member the emitter would reject, without producing Workshop text.
//!
//! The checks mirror [`super::emitter`]'s member handling and must stay in
//! sync with it: an empty result means `emit` will not fail on a settings
//! key. Locale-dependent name resolution is not part of emission acceptance
//! and is not checked here.

use crate::core::error::WorkshopError;

use super::table::{self, KeyKind};
use super::{PathPart, Settings, SettingsNode};

/// Check that emission accepts `settings`, returning one error per offending
/// member. Each error carries the node's recorded source span when present.
pub fn check_emission(settings: &Settings) -> Vec<WorkshopError> {
    let mut errors = Vec::new();
    for child in &settings.children {
        match child {
            SettingsNode::Workshop { children, .. } => {
                check_workshop_children(children, &mut errors)
            }
            SettingsNode::Group { name, children, .. } => match name.as_str() {
                "main" | "lobby" => {
                    for member in children {
                        check_member(member, &[PathPart::Part(name)], &mut errors);
                    }
                }
                "gamemodes" => check_modes(children, &mut errors),
                "heroes" => check_heroes(children, &mut errors),
                "extensions" => {
                    for member in children {
                        check_member(member, &[PathPart::Part("extensions")], &mut errors);
                    }
                }
                _ => check_opaque_children(children, &mut errors),
            },
            other => errors.push(malformed(
                other,
                "settings block children must be groups".to_string(),
            )),
        }
    }
    errors
}

fn malformed(node: &SettingsNode, message: String) -> WorkshopError {
    WorkshopError::Malformed {
        message,
        span: node.span(),
    }
}

fn check_workshop_children(children: &[SettingsNode], errors: &mut Vec<WorkshopError>) {
    for child in children {
        match child {
            SettingsNode::Group { children, .. } | SettingsNode::Workshop { children, .. } => {
                check_workshop_children(children, errors);
            }
            SettingsNode::Raw { .. } => {}
            other => errors.push(malformed(
                other,
                "settings.workshop contains a typed builtin setting".to_string(),
            )),
        }
    }
}

fn check_modes(modes: &[SettingsNode], errors: &mut Vec<WorkshopError>) {
    for mode in modes {
        let SettingsNode::Group { name, children, .. } = mode else {
            errors.push(malformed(mode, "mode entries must be groups".to_string()));
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

fn check_heroes(teams: &[SettingsNode], errors: &mut Vec<WorkshopError>) {
    for team in teams {
        let SettingsNode::Group { name, children, .. } = team else {
            errors.push(malformed(team, "team entries must be groups".to_string()));
            continue;
        };
        if table::team_name(name).is_none() {
            errors.push(malformed(team, format!("unknown team '{name}'")));
            continue;
        }
        for member in children {
            match member {
                SettingsNode::Group { name, children, .. } => {
                    if table::hero_name(name).is_none() {
                        errors.push(malformed(member, format!("unknown hero '{name}'")));
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

fn check_opaque_children(children: &[SettingsNode], errors: &mut Vec<WorkshopError>) {
    for child in children {
        match child {
            SettingsNode::Group { children, .. } => check_opaque_children(children, errors),
            other => check_member(other, &[], errors),
        }
    }
}

fn check_member(node: &SettingsNode, path: &[PathPart<'_>], errors: &mut Vec<WorkshopError>) {
    if matches!(node, SettingsNode::Raw { .. }) {
        return;
    }
    let name = node.name();
    let mut full = path.to_vec();
    full.push(PathPart::Part(name));
    let Some(entry) = table::lookup(&full) else {
        errors.push(malformed(
            node,
            format!(
                "settings key '{}' is outside the emission table",
                table::path_string(&full)
            ),
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
                errors.push(malformed(
                    node,
                    format!("unknown value '{value}' for settings key '{name}'"),
                ));
            }
        }
        (SettingsNode::Bool { value, .. }, KeyKind::BoolEnum(domain)) => {
            if !*value {
                errors.push(malformed(
                    node,
                    format!("unsupported false value for settings key '{name}'"),
                ));
            } else if table::enum_name(domain, "enabled").is_none() {
                errors.push(malformed(
                    node,
                    format!("unknown value 'enabled' for settings key '{name}'"),
                ));
            }
        }
        (SettingsNode::List { elements, .. }, KeyKind::ListMap) => {
            for element in elements {
                if table::map_name(&element.value).is_none() {
                    errors.push(WorkshopError::Malformed {
                        message: format!(
                            "unknown map '{}' in settings list '{name}'",
                            element.value
                        ),
                        span: element.span,
                    });
                }
            }
        }
        (SettingsNode::List { elements, .. }, KeyKind::ListHero) => {
            for element in elements {
                if table::hero_name(&element.value).is_none() {
                    errors.push(WorkshopError::Malformed {
                        message: format!(
                            "unknown hero '{}' in settings list '{name}'",
                            element.value
                        ),
                        span: element.span,
                    });
                }
            }
        }
        _ => errors.push(malformed(
            node,
            format!("settings key '{name}' does not match its table kind"),
        )),
    }
}
