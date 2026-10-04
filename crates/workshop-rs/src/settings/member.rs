use crate::core::error::WorkshopError;
use crate::source::Span;

use super::check::SettingsDiagnostic;
use super::table::{self, KeyKind, TableEntry};
use super::{PathPart, SettingsListElement, SettingsNode, suggest};

pub(super) enum Member<'a> {
    Flag,
    String(&'a str),
    Number(f64),
    Percent(f64),
    Bool(bool),
    YesNo(bool),
    Enum {
        domain: &'static str,
        value: &'a str,
        english: &'static str,
    },
    List {
        elements: &'a [SettingsListElement],
        kind: ListKind,
    },
}

#[derive(Clone, Copy)]
pub(super) enum ListKind {
    Map,
    Hero,
}

impl ListKind {
    pub(super) fn section(self) -> &'static str {
        match self {
            Self::Map => "maps",
            Self::Hero => "heroes",
        }
    }

    pub(super) fn label(self) -> &'static str {
        match self {
            Self::Map => "map",
            Self::Hero => "hero",
        }
    }

    pub(super) fn resolve(
        self,
        element: &SettingsListElement,
        name: &str,
    ) -> Result<&'static str, SettingsDiagnostic> {
        let english = match self {
            Self::Map => table::map_name(&element.value),
            Self::Hero => table::hero_name(&element.value),
        };
        english.ok_or_else(|| {
            let suggestion = match self {
                Self::Map => suggest::suggest(&element.value, table::map_spellings()),
                Self::Hero => suggest::suggest(&element.value, table::hero_spellings()),
            };
            rejected(
                element.span,
                format!(
                    "unknown {} '{}' in settings list '{name}'",
                    self.label(),
                    element.value
                ),
                suggestion,
            )
        })
    }
}

pub(super) fn lookup(
    node: &SettingsNode,
    full: &[PathPart<'_>],
) -> Result<&'static TableEntry, SettingsDiagnostic> {
    table::lookup(full).ok_or_else(|| {
        rejected(
            node.span(),
            format!(
                "settings key '{}' is outside the emission table",
                table::path_string(full)
            ),
            suggest::suggest(
                node.name(),
                table::key_spellings(&full[..full.len() - 1]).into_iter(),
            ),
        )
    })
}

pub(super) fn accept<'a>(
    node: &'a SettingsNode,
    entry: &TableEntry,
) -> Result<Member<'a>, SettingsDiagnostic> {
    let name = node.name();
    Ok(match (node, entry.kind) {
        (SettingsNode::Flag { .. }, KeyKind::Flag) => Member::Flag,
        (SettingsNode::String { value, .. }, KeyKind::String) => Member::String(value),
        (SettingsNode::Number { value, .. }, KeyKind::Number) => Member::Number(*value),
        (SettingsNode::Number { value, .. }, KeyKind::Percent) => Member::Percent(*value),
        (SettingsNode::Bool { value, .. }, KeyKind::Bool) => Member::Bool(*value),
        (SettingsNode::Bool { value, .. }, KeyKind::YesNo) => Member::YesNo(*value),
        (SettingsNode::String { value, .. }, KeyKind::Enum(domain)) => {
            let english = table::enum_name(domain, value).ok_or_else(|| {
                rejected(
                    node.span(),
                    format!("unknown value '{value}' for settings key '{name}'"),
                    suggest::suggest(value, table::enum_spellings(domain)),
                )
            })?;
            Member::Enum {
                domain,
                value,
                english,
            }
        }
        (SettingsNode::Bool { value, .. }, KeyKind::BoolEnum(domain)) => {
            if !value {
                return Err(rejected(
                    node.span(),
                    format!("unsupported false value for settings key '{name}'"),
                    None,
                ));
            }
            let english = table::enum_name(domain, "enabled").ok_or_else(|| {
                rejected(
                    node.span(),
                    format!("unknown value 'enabled' for settings key '{name}'"),
                    None,
                )
            })?;
            Member::Enum {
                domain,
                value: "enabled",
                english,
            }
        }
        (SettingsNode::List { elements, .. }, KeyKind::ListMap) => Member::List {
            elements,
            kind: ListKind::Map,
        },
        (SettingsNode::List { elements, .. }, KeyKind::ListHero) => Member::List {
            elements,
            kind: ListKind::Hero,
        },
        _ => {
            return Err(rejected(
                node.span(),
                format!("settings key '{name}' does not match its table kind"),
                None,
            ));
        }
    })
}

pub(super) fn rejected(
    span: Option<Span>,
    message: String,
    suggestion: Option<String>,
) -> SettingsDiagnostic {
    let message = suggest::with_suggestion_text(message, suggestion.as_deref());
    SettingsDiagnostic {
        error: WorkshopError::malformed(message, span),
        suggestion,
    }
}
