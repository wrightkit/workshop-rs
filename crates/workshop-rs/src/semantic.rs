//! Semantic-completeness inspection for permissive raw Workshop parsing.
//!
//! Structural WIR validation deliberately remains separate from this report:
//! a preserved node can be structurally valid while still being unsuitable
//! for definitive analysis.

use crate::catalog::{Catalog, Kind};
use crate::settings::SettingsNode;
use crate::settings::table;
use crate::source::Span;
use crate::wir::{Action, Program, Value};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IncompletenessKind {
    RawSetting,
    UnknownAction,
    UnknownValue,
    OpaqueAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResidualClassification {
    ProjectDefinedConstruct,
    SourceDeclaredVariable,
    ProducerExtension,
    LegacyOpaque,
    UnresolvedIdentifier,
}

impl ResidualClassification {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ProjectDefinedConstruct => "project-defined-construct",
            Self::SourceDeclaredVariable => "source-declared-variable",
            Self::ProducerExtension => "producer-extension",
            Self::LegacyOpaque => "legacy-opaque-construct",
            Self::UnresolvedIdentifier => "truly-unresolved-identifier",
        }
    }

    pub fn evidence(self) -> &'static str {
        match self {
            Self::ProjectDefinedConstruct => {
                "source settings or construct was preserved without a canonical catalog identity"
            }
            Self::SourceDeclaredVariable => {
                "the identifier matches a variable declaration in the parsed source program"
            }
            Self::ProducerExtension => {
                "the source uses an action-shaped identity outside the canonical catalog and no declaration resolves it"
            }
            Self::LegacyOpaque => {
                "the parser preserved a legacy raw construct without a canonical contract"
            }
            Self::UnresolvedIdentifier => {
                "the identifier matches neither a source declaration nor a canonical catalog identity"
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticIssue {
    pub kind: IncompletenessKind,
    pub name: String,
    pub span: Option<Span>,
    pub classification: ResidualClassification,
}

/// Report preserved or catalog-unknown constructs that must not be treated as
/// fully understood by downstream analysis.
pub fn inspect(program: &Program, catalog: &Catalog) -> Vec<SemanticIssue> {
    let mut issues = Vec::new();
    if let Some(settings) = &program.settings {
        for node in &settings.children {
            inspect_setting(node, &mut issues);
        }
    }
    for action in program.actions.iter() {
        inspect_action(action, program, catalog, &mut issues);
    }
    for value in program.values.iter() {
        inspect_value(value, program, catalog, &mut issues);
    }
    issues
}

fn inspect_setting(node: &SettingsNode, issues: &mut Vec<SemanticIssue>) {
    match node {
        SettingsNode::Group { children, .. } => {
            for child in children {
                inspect_setting(child, issues);
            }
        }
        SettingsNode::Raw { name, span, .. } => issues.push(SemanticIssue {
            kind: IncompletenessKind::RawSetting,
            name: name.clone(),
            span: *span,
            classification: ResidualClassification::ProjectDefinedConstruct,
        }),
        SettingsNode::List {
            name,
            elements,
            span,
        } => {
            let known = match name.as_str() {
                "enabledMaps" | "disabledMaps" => elements
                    .iter()
                    .all(|element| table::map_name(&element.value).is_some()),
                "enabledHeroes" | "disabledHeroes" => elements
                    .iter()
                    .all(|element| table::hero_name(&element.value).is_some()),
                _ => true,
            };
            if !known {
                issues.push(SemanticIssue {
                    kind: IncompletenessKind::RawSetting,
                    name: name.clone(),
                    span: *span,
                    classification: ResidualClassification::ProjectDefinedConstruct,
                });
            }
        }
        SettingsNode::Number { .. }
        | SettingsNode::Bool { .. }
        | SettingsNode::Flag { .. }
        | SettingsNode::String { .. } => {}
    }
}

fn inspect_action(
    action: &Action,
    program: &Program,
    catalog: &Catalog,
    issues: &mut Vec<SemanticIssue>,
) {
    match action {
        Action::Call { name, span, .. } => {
            let kind = if name == "rawWorkshopAction" {
                Some(IncompletenessKind::OpaqueAction)
            } else if catalog.entry(Kind::Action, name).is_none() {
                Some(IncompletenessKind::UnknownAction)
            } else {
                None
            };
            if let Some(kind) = kind {
                let classification = if kind == IncompletenessKind::OpaqueAction {
                    ResidualClassification::LegacyOpaque
                } else {
                    ResidualClassification::ProducerExtension
                };
                issues.push(SemanticIssue {
                    kind,
                    name: name.clone(),
                    span: *span,
                    classification,
                });
            }
        }
        Action::If {
            branches,
            else_body,
            ..
        } => {
            for branch in branches {
                inspect_action_id(branch.body.as_slice(), program, catalog, issues);
            }
            if let Some(body) = else_body {
                inspect_action_id(body.as_slice(), program, catalog, issues);
            }
        }
        Action::While { body, .. }
        | Action::ForGlobalVariable { body, .. }
        | Action::ForPlayerVariable { body, .. } => {
            inspect_action_id(body.as_slice(), program, catalog, issues);
        }
        Action::SetGlobalVariable { .. }
        | Action::ModifyGlobalVariable { .. }
        | Action::SetPlayerVariable { .. }
        | Action::ModifyPlayerVariable { .. }
        | Action::AssignMember { .. }
        | Action::CallSubroutine { .. }
        | Action::Debug { .. }
        | Action::Print { .. } => {}
    }
}

fn inspect_action_id(
    ids: &[crate::wir::ActionId],
    program: &Program,
    catalog: &Catalog,
    issues: &mut Vec<SemanticIssue>,
) {
    for id in ids {
        if let Some(action) = program.actions.get(*id) {
            inspect_action(action, program, catalog, issues);
        }
    }
}

fn inspect_value(
    node: &crate::wir::ValueNode,
    program: &Program,
    catalog: &Catalog,
    issues: &mut Vec<SemanticIssue>,
) {
    if let Value::Call { name, .. } = &node.value {
        // These names are canonical WIR helpers rather than Workshop
        // builtins: memberAccess preserves dynamic receiver properties, and
        // infix operators are lowered to their source spelling for emission.
        let canonical_helper =
            matches!(name.as_str(), "memberAccess" | "+" | "-" | "*" | "/" | "%");
        if canonical_helper {
            return;
        }
        if catalog.entry(Kind::Value, name).is_none()
            && catalog.entry(Kind::Operator, name).is_none()
        {
            issues.push(SemanticIssue {
                kind: IncompletenessKind::UnknownValue,
                name: name.clone(),
                span: node.span,
                classification: if program
                    .global_variables
                    .iter()
                    .any(|variable| variable.name == *name)
                    || program
                        .player_variables
                        .iter()
                        .any(|variable| variable.name == *name)
                {
                    ResidualClassification::SourceDeclaredVariable
                } else {
                    ResidualClassification::UnresolvedIdentifier
                },
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::{Settings, SettingsNode};

    #[test]
    fn reports_preserved_and_unknown_nodes() {
        let catalog = Catalog::builtin().expect("builtin catalog");
        let mut program = Program {
            settings: Some(Settings {
                span: None,
                children: vec![SettingsNode::Raw {
                    name: "Future Setting".to_string(),
                    value: "opaque".to_string(),
                    span: None,
                }],
            }),
            ..Program::default()
        };
        program.actions.push(Action::Call {
            name: "rawWorkshopAction".to_string(),
            args: Vec::new(),
            span: None,
        });
        program.actions.push(Action::Call {
            name: "futureAction".to_string(),
            args: Vec::new(),
            span: None,
        });
        program.values.push(crate::wir::ValueNode::new(
            Value::Call {
                name: "futureValue".to_string(),
                args: Vec::new(),
            },
            None,
        ));

        let issues = inspect(&program, &catalog);
        assert!(
            issues
                .iter()
                .any(|issue| issue.kind == IncompletenessKind::RawSetting)
        );
        assert!(
            issues
                .iter()
                .any(|issue| issue.kind == IncompletenessKind::OpaqueAction)
        );
        assert!(
            issues
                .iter()
                .any(|issue| issue.kind == IncompletenessKind::UnknownAction)
        );
        assert!(
            issues
                .iter()
                .any(|issue| issue.kind == IncompletenessKind::UnknownValue)
        );
        assert!(issues.iter().any(|issue| {
            issue.kind == IncompletenessKind::RawSetting
                && issue.classification == ResidualClassification::ProjectDefinedConstruct
        }));
        assert!(issues.iter().any(|issue| {
            issue.kind == IncompletenessKind::OpaqueAction
                && issue.classification == ResidualClassification::LegacyOpaque
        }));
        assert!(issues.iter().any(|issue| {
            issue.kind == IncompletenessKind::UnknownValue
                && issue.classification == ResidualClassification::UnresolvedIdentifier
        }));
    }
}
