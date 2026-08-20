//! Semantic-completeness inspection for permissive raw Workshop parsing.
//!
//! Structural WIR validation deliberately remains separate from this report:
//! a preserved node can be structurally valid while still being unsuitable
//! for definitive analysis.

use crate::catalog::{Catalog, Kind};
use crate::settings::SettingsNode;
use crate::source::Span;
use crate::wir::{Action, Program, Value};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IncompletenessKind {
    RawSetting,
    UnknownAction,
    UnknownValue,
    OpaqueAction,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticIssue {
    pub kind: IncompletenessKind,
    pub name: String,
    pub span: Option<Span>,
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
        inspect_value(value, catalog, &mut issues);
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
        }),
        SettingsNode::Number { .. }
        | SettingsNode::Bool { .. }
        | SettingsNode::String { .. }
        | SettingsNode::List { .. } => {}
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
                issues.push(SemanticIssue {
                    kind,
                    name: name.clone(),
                    span: *span,
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

fn inspect_value(node: &crate::wir::ValueNode, catalog: &Catalog, issues: &mut Vec<SemanticIssue>) {
    if let Value::Call { name, .. } = &node.value {
        if catalog.entry(Kind::Value, name).is_none()
            && catalog.entry(Kind::Operator, name).is_none()
        {
            issues.push(SemanticIssue {
                kind: IncompletenessKind::UnknownValue,
                name: name.clone(),
                span: node.span,
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
    }
}
