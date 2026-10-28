use crate::catalog::{Catalog, Kind};
use crate::core::source::Span;
use crate::program::{action_argument_values, value_children};
use crate::settings::SettingsNode;
use crate::settings::table;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    /// A carried settings member whose spelling is close to exactly one
    /// declared key or enum member — a likely misspelling, not a new
    /// project-defined construct.
    CatalogSpellingNearMiss,
}

impl ResidualClassification {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ProjectDefinedConstruct => "project-defined-construct",
            Self::SourceDeclaredVariable => "source-declared-variable",
            Self::ProducerExtension => "producer-extension",
            Self::LegacyOpaque => "legacy-opaque-construct",
            Self::UnresolvedIdentifier => "truly-unresolved-identifier",
            Self::CatalogSpellingNearMiss => "catalog-spelling-near-miss",
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
            Self::CatalogSpellingNearMiss => {
                "the spelling is within a small edit distance of exactly one declared settings member"
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct SemanticIssue {
    pub kind: IncompletenessKind,
    pub name: String,
    pub span: Option<Span>,
    pub classification: ResidualClassification,
    /// The canonical spelling `name` was close to, when exactly one was.
    pub suggestion: Option<String>,
}

/// Report preserved or catalog-unknown constructs that must not be treated as
/// fully understood by downstream analysis.
///
/// Residual inspection is defined over the public canonical
/// [`crate::Program`] model and is independent from structural validation: it
/// may be used on `Program` values that [`Program::validate`] rejects, and an
/// internal WIR materialization failure never projects to an empty inventory.
/// Callers that also require structural validity call
/// [`Program::validate`](crate::Program::validate) separately; inspection
/// does not report validation errors.
///
/// [`Program::validate`]: crate::Program::validate
pub fn inspect(program: &crate::Program, catalog: &Catalog) -> Vec<SemanticIssue> {
    let mut issues = Vec::new();
    if let Some(settings) = &program.settings {
        for member in crate::settings::check::uncatalogued_members(settings) {
            issues.push(SemanticIssue {
                kind: IncompletenessKind::RawSetting,
                name: member.name.to_string(),
                span: member.span,
                classification: match member.suggestion {
                    Some(_) => ResidualClassification::CatalogSpellingNearMiss,
                    None => ResidualClassification::ProjectDefinedConstruct,
                },
                suggestion: member.suggestion,
            });
        }
        for node in &settings.children {
            inspect_setting(node, &mut issues);
        }
    }
    for (rule, rule_data) in program.rules.iter().enumerate() {
        for (position, action) in rule_data.actions.iter().enumerate() {
            inspect_action(action, program, rule, position, catalog, &mut issues);
        }
    }
    for (rule, rule_data) in program.rules.iter().enumerate() {
        for (condition, condition_data) in rule_data.conditions.iter().enumerate() {
            inspect_value_tree(
                &condition_data.value,
                &mut Vec::new(),
                &|path| program.condition_value_node_span(rule, condition, path),
                program,
                catalog,
                &mut issues,
            );
        }
        let mut position = 0;
        inspect_action_values(
            &rule_data.actions,
            &mut position,
            false,
            rule,
            program,
            catalog,
            &mut issues,
        );
    }
    issues
}

#[cfg(test)]
pub(crate) fn inspect_wir(program: &crate::wir::Program, catalog: &Catalog) -> Vec<SemanticIssue> {
    let program = crate::Program::from_wir(program.clone())
        .expect("test WIR materializes as a public program");
    inspect(&program, catalog)
}

fn inspect_setting(node: &SettingsNode, issues: &mut Vec<SemanticIssue>) {
    match node {
        SettingsNode::Workshop { .. } => {}
        SettingsNode::Group { children, .. } | SettingsNode::RawGroup { children, .. } => {
            for child in children {
                inspect_setting(child, issues);
            }
        }

        SettingsNode::List { name, elements, .. } => {
            let element_known: fn(&crate::settings::SettingsListElement) -> bool =
                match name.as_str() {
                    "enabledMaps" | "disabledMaps" => {
                        |element: &crate::settings::SettingsListElement| {
                            table::map_name(&element.value).is_some()
                        }
                    }
                    "enabledHeroes" | "disabledHeroes" => {
                        |element: &crate::settings::SettingsListElement| {
                            table::hero_name(&element.value).is_some()
                        }
                    }
                    _ => |_| true,
                };
            for element in elements {
                if !element_known(element) {
                    issues.push(SemanticIssue {
                        kind: IncompletenessKind::RawSetting,
                        name: element.value.clone(),
                        span: element.span,
                        classification: ResidualClassification::ProjectDefinedConstruct,
                        suggestion: None,
                    });
                }
            }
        }
        SettingsNode::Number { .. }
        | SettingsNode::Bool { .. }
        | SettingsNode::Flag { .. }
        | SettingsNode::String { .. }
        | SettingsNode::Raw { .. }
        | SettingsNode::RawValue { .. } => {}
    }
}

fn inspect_action(
    action: &crate::Action,
    program: &crate::Program,
    rule: usize,
    position: usize,
    catalog: &Catalog,
    issues: &mut Vec<SemanticIssue>,
) {
    match action {
        crate::Action::Call { name, .. } => {
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
                    span: program.action_span(rule, position),
                    classification,
                    suggestion: None,
                });
            }
        }
        crate::Action::Disabled { action } => {
            inspect_action(action, program, rule, position, catalog, issues);
        }
        _ => {}
    }
}

fn inspect_action_values(
    actions: &[crate::Action],
    position: &mut usize,
    stop_at_terminator: bool,
    rule: usize,
    program: &crate::Program,
    catalog: &Catalog,
    issues: &mut Vec<SemanticIssue>,
) {
    while *position < actions.len() {
        let index = *position;
        let current = match &actions[index] {
            crate::Action::Disabled { action } => action.as_ref(),
            action => action,
        };
        match current {
            crate::Action::ElseIf { .. } | crate::Action::Else | crate::Action::End
                if stop_at_terminator =>
            {
                return;
            }
            crate::Action::If { .. } => {
                inspect_action_arguments(rule, index, &actions[index], program, catalog, issues);
                *position += 1;
                inspect_action_values(actions, position, true, rule, program, catalog, issues);
                while matches!(actions.get(*position), Some(crate::Action::ElseIf { .. })) {
                    let elseif = *position;
                    *position += 1;
                    inspect_action_values(actions, position, true, rule, program, catalog, issues);
                    inspect_action_arguments(
                        rule,
                        elseif,
                        &actions[elseif],
                        program,
                        catalog,
                        issues,
                    );
                }
                if matches!(actions.get(*position), Some(crate::Action::Else)) {
                    *position += 1;
                    inspect_action_values(actions, position, true, rule, program, catalog, issues);
                }
                if matches!(actions.get(*position), Some(crate::Action::End)) {
                    *position += 1;
                }
            }
            crate::Action::While { .. }
            | crate::Action::ForGlobalVariable { .. }
            | crate::Action::ForPlayerVariable { .. } => {
                *position += 1;
                inspect_action_values(actions, position, true, rule, program, catalog, issues);
                if matches!(actions.get(*position), Some(crate::Action::End)) {
                    *position += 1;
                }
                inspect_action_arguments(rule, index, &actions[index], program, catalog, issues);
            }
            _ => {
                inspect_action_arguments(rule, index, &actions[index], program, catalog, issues);
                *position += 1;
            }
        }
    }
}

fn inspect_action_arguments(
    rule: usize,
    action: usize,
    action_data: &crate::Action,
    program: &crate::Program,
    catalog: &Catalog,
    issues: &mut Vec<SemanticIssue>,
) {
    for (argument, value) in action_argument_values(action_data).iter().enumerate() {
        inspect_value_tree(
            value,
            &mut Vec::new(),
            &|path| program.action_argument_value_node_span(rule, action, argument, path),
            program,
            catalog,
            issues,
        );
    }
}

fn inspect_value_tree(
    value: &crate::Value,
    path: &mut Vec<usize>,
    span_at: &impl Fn(&[usize]) -> Option<Span>,
    program: &crate::Program,
    catalog: &Catalog,
    issues: &mut Vec<SemanticIssue>,
) {
    for (index, child) in value_children(value).into_iter().enumerate() {
        path.push(index);
        inspect_value_tree(child, path, span_at, program, catalog, issues);
        path.pop();
    }
    if let crate::Value::Call { name, args } = value {
        let canonical_helper = name == crate::wir::AMBIGUOUS_ENUM_CALL
            || crate::wir::is_canonical_helper_call(name, args.len());
        if canonical_helper {
            return;
        }
        if catalog.entry(Kind::Value, name).is_none()
            && catalog.entry(Kind::Operator, name).is_none()
        {
            issues.push(SemanticIssue {
                kind: IncompletenessKind::UnknownValue,
                name: name.clone(),
                span: span_at(path),
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
                suggestion: None,
            });
        }
    }
}
