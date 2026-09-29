//! Canonical Workshop program concepts.

use std::collections::HashMap;

use crate::core::error::{Result, WorkshopError};
use crate::settings::Settings;
use crate::source::{FileId, SourceDocument, SourceFile, Span};
use crate::wir;

mod source_map;
pub use source_map::{MAPPED_TEXT_V1, MappedText, SourceMap, SourceMapError, TEXT_V1};

/// A complete Workshop program built from Workshop concepts.
#[derive(Debug, Clone, Default)]
pub struct Program {
    pub settings: Option<Settings>,
    pub global_variables: Vec<Variable>,
    pub player_variables: Vec<Variable>,
    pub subroutines: Vec<Subroutine>,
    pub rules: Vec<Rule>,
    files: Vec<SourceFile>,
    provenance: Option<Box<ProgramProvenance>>,
}

#[derive(Debug, Clone, Default)]
struct ProgramProvenance {
    global_variables: Vec<DeclarationProvenance>,
    player_variables: Vec<DeclarationProvenance>,
    subroutines: Vec<DeclarationProvenance>,
    rules: Vec<RuleProvenance>,
}

#[derive(Debug, Clone, Copy, Default)]
struct DeclarationProvenance {
    span: Option<Span>,
    name_span: Option<Span>,
}

#[derive(Debug, Clone, Default)]
struct RuleProvenance {
    span: Option<crate::source::Span>,
    /// The recorded span of the rule's quoted name in `rule("name")`.
    name: Option<Span>,
    /// The recorded span of the subroutine name a `Subroutine` event binds.
    event_name: Option<Span>,
    conditions: Vec<ValueProvenance>,
    actions: Vec<ActionProvenance>,
}

#[derive(Debug, Clone, Default)]
struct ActionProvenance {
    span: Option<Span>,
    /// The recorded span of the variable or subroutine the action names: a
    /// set/modify/for target or a `Call Subroutine` callee.
    identifier: Option<Span>,
    arguments: Vec<ValueProvenance>,
}

/// The recorded span of one value node and its children, mirroring the
/// structure of the public [`Value`] tree.
#[derive(Debug, Clone, Default)]
struct ValueProvenance {
    span: Option<Span>,
    /// The recorded span of the variable or subroutine identifier the value
    /// names, when the value is such a reference.
    identifier: Option<Span>,
    children: Vec<ValueProvenance>,
}

/// A failure while attaching source mappings to a public [`Program`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceMappingError {
    UnknownFile(FileId),
    InvalidSpan(Span),
    InvalidRule(usize),
    InvalidCondition {
        rule: usize,
        condition: usize,
    },
    InvalidAction {
        rule: usize,
        action: usize,
    },
    InvalidActionArgument {
        rule: usize,
        action: usize,
        argument: usize,
    },
    InvalidGlobalVariable(usize),
    InvalidPlayerVariable(usize),
    InvalidSubroutine(usize),
}

impl std::fmt::Display for SourceMappingError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownFile(file) => {
                write!(formatter, "source span references unknown file {file}")
            }
            Self::InvalidSpan(span) => write!(formatter, "invalid source span {span:?}"),
            Self::InvalidRule(rule) => write!(formatter, "invalid rule index {rule}"),
            Self::InvalidCondition { rule, condition } => {
                write!(
                    formatter,
                    "invalid condition index {condition} in rule {rule}"
                )
            }
            Self::InvalidAction { rule, action } => {
                write!(formatter, "invalid action index {action} in rule {rule}")
            }
            Self::InvalidActionArgument {
                rule,
                action,
                argument,
            } => write!(
                formatter,
                "invalid argument index {argument} in action {action} of rule {rule}"
            ),
            Self::InvalidGlobalVariable(variable) => {
                write!(formatter, "invalid global variable index {variable}")
            }
            Self::InvalidPlayerVariable(variable) => {
                write!(formatter, "invalid player variable index {variable}")
            }
            Self::InvalidSubroutine(subroutine) => {
                write!(formatter, "invalid subroutine index {subroutine}")
            }
        }
    }
}

impl std::error::Error for SourceMappingError {}

impl Program {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a source file and return its public file identity.
    pub fn add_file(&mut self, mut file: SourceFile) -> FileId {
        let id = FileId::from_index(self.files.len());
        file.bind_file(id);
        self.files.push(file);
        id
    }

    pub fn global_variable(&mut self, variable: Variable) -> &mut Self {
        self.global_variables.push(variable);
        self
    }

    pub fn player_variable(&mut self, variable: Variable) -> &mut Self {
        self.player_variables.push(variable);
        self
    }

    pub fn subroutine(&mut self, subroutine: Subroutine) -> &mut Self {
        self.subroutines.push(subroutine);
        self
    }

    pub fn rule(&mut self, rule: Rule) -> &mut Self {
        self.rules.push(rule);
        self
    }

    /// Return the retained source document for a parsed file.
    pub fn source(&self, file: FileId) -> Option<&SourceDocument> {
        self.files.get(file.index()).and_then(SourceFile::source)
    }

    /// Attach the authored span of a public rule.
    pub fn set_rule_span(
        &mut self,
        rule: usize,
        span: Option<Span>,
    ) -> std::result::Result<(), SourceMappingError> {
        self.validate_span(span)?;
        self.rule_provenance_mut(rule)?.span = span;
        Ok(())
    }

    /// Attach the authored span of a public rule condition value.
    pub fn set_condition_span(
        &mut self,
        rule: usize,
        condition: usize,
        span: Option<Span>,
    ) -> std::result::Result<(), SourceMappingError> {
        self.validate_span(span)?;
        let condition_count = self
            .rules
            .get(rule)
            .ok_or(SourceMappingError::InvalidRule(rule))?
            .conditions
            .len();
        if condition >= condition_count {
            return Err(SourceMappingError::InvalidCondition { rule, condition });
        }
        let rule_data = self.rule_provenance_mut(rule)?;
        fit(&mut rule_data.conditions, condition_count);
        rule_data.conditions[condition].span = span;
        Ok(())
    }

    /// Attach the authored span of a public action in its linear rule order.
    pub fn set_action_span(
        &mut self,
        rule: usize,
        action: usize,
        span: Option<Span>,
    ) -> std::result::Result<(), SourceMappingError> {
        self.validate_span(span)?;
        let action_count = self
            .rules
            .get(rule)
            .ok_or(SourceMappingError::InvalidRule(rule))?
            .actions
            .len();
        if action >= action_count {
            return Err(SourceMappingError::InvalidAction { rule, action });
        }
        let rule_data = self.rule_provenance_mut(rule)?;
        fit(&mut rule_data.actions, action_count);
        rule_data.actions[action].span = span;
        Ok(())
    }

    /// Attach the authored span of a direct value argument of a public action.
    pub fn set_action_argument_span(
        &mut self,
        rule: usize,
        action: usize,
        argument: usize,
        span: Option<Span>,
    ) -> std::result::Result<(), SourceMappingError> {
        self.validate_span(span)?;
        let action_value = self
            .rules
            .get(rule)
            .ok_or(SourceMappingError::InvalidRule(rule))?
            .actions
            .get(action)
            .ok_or(SourceMappingError::InvalidAction { rule, action })?;
        let argument_count = action_argument_count(action_value);
        if argument >= argument_count {
            return Err(SourceMappingError::InvalidActionArgument {
                rule,
                action,
                argument,
            });
        }
        let action_data = self.action_provenance_mut(rule, action)?;
        action_data
            .arguments
            .resize_with(argument + 1, ValueProvenance::default);
        action_data.arguments[argument].span = span;
        Ok(())
    }

    /// Attach the authored and identifier spans of a global variable.
    pub fn set_global_variable_spans(
        &mut self,
        variable: usize,
        span: Option<Span>,
        name_span: Option<Span>,
    ) -> std::result::Result<(), SourceMappingError> {
        self.validate_span(span)?;
        self.validate_span(name_span)?;
        if variable >= self.global_variables.len() {
            return Err(SourceMappingError::InvalidGlobalVariable(variable));
        }
        let variable_count = self.global_variables.len();
        let provenance = self.provenance_mut();
        fit(&mut provenance.global_variables, variable_count);
        provenance.global_variables[variable] = DeclarationProvenance { span, name_span };
        Ok(())
    }

    /// Attach the authored and identifier spans of a player variable.
    pub fn set_player_variable_spans(
        &mut self,
        variable: usize,
        span: Option<Span>,
        name_span: Option<Span>,
    ) -> std::result::Result<(), SourceMappingError> {
        self.validate_span(span)?;
        self.validate_span(name_span)?;
        if variable >= self.player_variables.len() {
            return Err(SourceMappingError::InvalidPlayerVariable(variable));
        }
        let variable_count = self.player_variables.len();
        let provenance = self.provenance_mut();
        fit(&mut provenance.player_variables, variable_count);
        provenance.player_variables[variable] = DeclarationProvenance { span, name_span };
        Ok(())
    }

    /// Attach the authored and identifier spans of a subroutine.
    pub fn set_subroutine_spans(
        &mut self,
        subroutine: usize,
        span: Option<Span>,
        name_span: Option<Span>,
    ) -> std::result::Result<(), SourceMappingError> {
        self.validate_span(span)?;
        self.validate_span(name_span)?;
        if subroutine >= self.subroutines.len() {
            return Err(SourceMappingError::InvalidSubroutine(subroutine));
        }
        let subroutine_count = self.subroutines.len();
        let provenance = self.provenance_mut();
        fit(&mut provenance.subroutines, subroutine_count);
        provenance.subroutines[subroutine] = DeclarationProvenance { span, name_span };
        Ok(())
    }

    /// Return the authored span of a public rule, when source metadata exists.
    ///
    /// Attached mappings record the program shape they were attached to. Every
    /// span accessor returns `None` once the public rules, conditions, or
    /// actions have been inserted or removed since the mapping was attached.
    pub fn rule_span(&self, rule: usize) -> Option<crate::source::Span> {
        self.rule_provenance(rule)?.span
    }

    /// Return the authored span of a public rule condition value.
    pub fn condition_span(&self, rule: usize, condition: usize) -> Option<crate::source::Span> {
        self.condition_provenance(rule, condition)?.span
    }

    /// Return the authored span of a public action in its linear rule order.
    pub fn action_span(&self, rule: usize, action: usize) -> Option<crate::source::Span> {
        self.action_provenance(rule, action)?.span
    }

    /// Return the authored span of a direct value argument of a public action.
    pub fn action_argument_span(
        &self,
        rule: usize,
        action: usize,
        argument: usize,
    ) -> Option<crate::source::Span> {
        self.action_provenance(rule, action)?
            .arguments
            .get(argument)?
            .span
    }

    /// Return the authored span of the declared name of a global variable.
    ///
    /// Source-language providers attach an explicit identifier span through
    /// [`set_global_variable_spans`](Self::set_global_variable_spans). For raw
    /// Workshop parses the recorded declaration span already covers exactly
    /// the declared name and is returned as the identifier span.
    pub fn global_variable_name_span(&self, variable: usize) -> Option<Span> {
        self.declaration_name_span(
            |provenance| &provenance.global_variables,
            self.global_variables.len(),
            variable,
        )
    }

    /// Return the authored span of the declared name of a player variable.
    /// See [`global_variable_name_span`](Self::global_variable_name_span).
    pub fn player_variable_name_span(&self, variable: usize) -> Option<Span> {
        self.declaration_name_span(
            |provenance| &provenance.player_variables,
            self.player_variables.len(),
            variable,
        )
    }

    /// Return the authored span of the declared name of a subroutine.
    /// See [`global_variable_name_span`](Self::global_variable_name_span).
    pub fn subroutine_name_span(&self, subroutine: usize) -> Option<Span> {
        self.declaration_name_span(
            |provenance| &provenance.subroutines,
            self.subroutines.len(),
            subroutine,
        )
    }

    /// Return the span recorded for the variable or subroutine identifier a
    /// public action names: the target of set/modify and for-variable actions,
    /// or the callee of a [`Call Subroutine`](Action::CallSubroutine) action.
    ///
    /// Raw Workshop parses record the variable name for `Set`/`Modify`
    /// variable actions, `For` variable loops, and `Global.name`/`Event
    /// Player.name` infix assignments, and the callee name for `Call
    /// Subroutine`. Indexed writes lower to `... Variable At Index` calls and
    /// record the name on their variable argument — see
    /// [`action_argument_value_span`](Self::action_argument_value_span).
    /// Other action forms always return `None`.
    pub fn action_identifier_span(&self, rule: usize, action: usize) -> Option<Span> {
        self.action_provenance(rule, action)?.identifier
    }

    /// Return the span recorded for a rule's name inside its `rule("name")`
    /// string, or `None` when no provenance was recorded.
    pub fn rule_name_span(&self, rule: usize) -> Option<Span> {
        self.rule_provenance(rule)?.name
    }

    /// Return the span recorded for the subroutine name a rule's `Subroutine`
    /// event binding names, or `None` for other event kinds and when no
    /// provenance was recorded.
    pub fn rule_event_name_span(&self, rule: usize) -> Option<Span> {
        self.rule_provenance(rule)?.event_name
    }

    /// Return the authored span of a value nested inside a public rule
    /// condition.
    ///
    /// `path` walks the public [`Value`] tree: each element selects a child by
    /// position — `Value::Array` elements and `Value::Call` arguments by
    /// index, `Value::Vector` components as `0`/`1`/`2` for x/y/z, and a
    /// `Value::PlayerVariable` player at `0`. An empty path returns the
    /// condition value's own span, matching [`condition_span`](Self::condition_span).
    ///
    /// For a variable or subroutine reference the identifier span is returned
    /// when the parser recorded one: raw Workshop records the variable name
    /// for `Global.name`, `Global/Player Variable(name)`, `Event Player.name`,
    /// bare-name, and `... At Index` argument spellings. Other nodes return
    /// the span recorded for the node itself.
    pub fn condition_value_span(
        &self,
        rule: usize,
        condition: usize,
        path: &[usize],
    ) -> Option<crate::source::Span> {
        let mut value = self.condition_provenance(rule, condition)?;
        for &index in path {
            value = value.children.get(index)?;
        }
        value.identifier.or(value.span)
    }

    /// Return the authored span of a value nested inside a direct value
    /// argument of a public action.
    ///
    /// `argument` selects the same direct argument as
    /// [`action_argument_span`](Self::action_argument_span) and `path` walks
    /// into it the way [`condition_value_span`](Self::condition_value_span)
    /// describes; an empty path returns the argument's own span. Like
    /// `condition_value_span`, a variable or subroutine reference returns its
    /// recorded identifier span.
    pub fn action_argument_value_span(
        &self,
        rule: usize,
        action: usize,
        argument: usize,
        path: &[usize],
    ) -> Option<crate::source::Span> {
        let mut value = self
            .action_provenance(rule, action)?
            .arguments
            .get(argument)?;
        for &index in path {
            value = value.children.get(index)?;
        }
        value.identifier.or(value.span)
    }

    /// Create a checked source edit through the authored source attached to
    /// this canonical program.
    pub fn edit_source(
        &self,
        span: crate::source::Span,
        replacement: impl Into<String>,
    ) -> std::result::Result<crate::source::SourceEdit, crate::source::SourceEditError> {
        self.source(span.file)
            .ok_or(crate::source::SourceEditError::InvalidRange)?
            .edit_span(span, replacement)
    }

    /// Validate the structural invariants of the canonical program.
    pub fn validate(&self) -> std::result::Result<(), WorkshopError> {
        let storage = self.to_wir()?;
        storage
            .validate()
            .map_err(|error| WorkshopError::Malformed {
                message: error.to_string(),
                span: error.span(),
            })
    }

    /// Report constructs that are structurally preserved but not fully
    /// understood by the canonical catalog.
    pub fn semantic_issues(
        &self,
        catalog: &crate::catalog::Catalog,
    ) -> Vec<crate::rules::SemanticIssue> {
        crate::analysis::semantic::inspect(self, catalog)
    }

    /// Render the program through the canonical Workshop debug representation.
    pub fn dump(&self) -> String {
        self.to_wir().map_or_else(
            |error| format!("invalid program: {error}"),
            |program| program.dump(),
        )
    }

    fn rule_provenance(&self, rule: usize) -> Option<&RuleProvenance> {
        let recorded = &self.provenance.as_deref()?.rules;
        if recorded.len() != self.rules.len() {
            return None;
        }
        recorded.get(rule)
    }

    fn action_provenance(&self, rule: usize, action: usize) -> Option<&ActionProvenance> {
        let recorded = self.rule_provenance(rule)?;
        if recorded.actions.len() != self.rules[rule].actions.len() {
            return None;
        }
        recorded.actions.get(action)
    }

    fn condition_provenance(&self, rule: usize, condition: usize) -> Option<&ValueProvenance> {
        let recorded = self.rule_provenance(rule)?;
        if recorded.conditions.len() != self.rules[rule].conditions.len() {
            return None;
        }
        recorded.conditions.get(condition)
    }

    fn declaration_name_span(
        &self,
        recorded: impl Fn(&ProgramProvenance) -> &[DeclarationProvenance],
        count: usize,
        position: usize,
    ) -> Option<Span> {
        let declaration = self.declaration_provenance(recorded, count, position);
        declaration.name_span.or(declaration.span)
    }

    fn declaration_provenance(
        &self,
        recorded: impl Fn(&ProgramProvenance) -> &[DeclarationProvenance],
        count: usize,
        position: usize,
    ) -> DeclarationProvenance {
        self.provenance
            .as_deref()
            .map(recorded)
            .filter(|recorded| recorded.len() == count)
            .and_then(|recorded| recorded.get(position))
            .copied()
            .unwrap_or_default()
    }

    fn validate_span(&self, span: Option<Span>) -> std::result::Result<(), SourceMappingError> {
        let Some(span) = span else {
            return Ok(());
        };
        if !span.is_valid() {
            return Err(SourceMappingError::InvalidSpan(span));
        }
        if self.files.get(span.file.index()).is_none() {
            return Err(SourceMappingError::UnknownFile(span.file));
        }
        Ok(())
    }

    fn provenance_mut(&mut self) -> &mut ProgramProvenance {
        self.provenance
            .get_or_insert_with(|| Box::new(ProgramProvenance::default()))
            .as_mut()
    }

    fn rule_provenance_mut(
        &mut self,
        rule: usize,
    ) -> std::result::Result<&mut RuleProvenance, SourceMappingError> {
        if rule >= self.rules.len() {
            return Err(SourceMappingError::InvalidRule(rule));
        }
        let rule_count = self.rules.len();
        let provenance = self.provenance_mut();
        fit(&mut provenance.rules, rule_count);
        Ok(&mut provenance.rules[rule])
    }

    fn action_provenance_mut(
        &mut self,
        rule: usize,
        action: usize,
    ) -> std::result::Result<&mut ActionProvenance, SourceMappingError> {
        let action_count = self
            .rules
            .get(rule)
            .ok_or(SourceMappingError::InvalidRule(rule))?
            .actions
            .len();
        if action >= action_count {
            return Err(SourceMappingError::InvalidAction { rule, action });
        }
        let rule_data = self.rule_provenance_mut(rule)?;
        fit(&mut rule_data.actions, action_count);
        Ok(&mut rule_data.actions[action])
    }

    pub(crate) fn from_wir(storage: wir::Program) -> Result<Self> {
        let mut program = Self {
            settings: storage.settings.clone(),
            global_variables: storage
                .global_variables
                .iter()
                .map(|variable| Variable::with_index(variable.name.clone(), variable.index))
                .collect(),
            player_variables: storage
                .player_variables
                .iter()
                .map(|variable| Variable::with_index(variable.name.clone(), variable.index))
                .collect(),
            subroutines: storage
                .subroutines
                .iter()
                .map(|subroutine| Subroutine::with_index(subroutine.name.clone(), subroutine.index))
                .collect(),
            rules: Vec::with_capacity(storage.rules.len()),
            files: storage.files.iter().cloned().collect(),
            provenance: Some(Box::new(ProgramProvenance {
                global_variables: storage
                    .global_variables
                    .iter()
                    .map(|variable| DeclarationProvenance {
                        span: variable.span,
                        name_span: variable.name_span,
                    })
                    .collect(),
                player_variables: storage
                    .player_variables
                    .iter()
                    .map(|variable| DeclarationProvenance {
                        span: variable.span,
                        name_span: variable.name_span,
                    })
                    .collect(),
                subroutines: storage
                    .subroutines
                    .iter()
                    .map(|subroutine| DeclarationProvenance {
                        span: subroutine.span,
                        name_span: subroutine.name_span,
                    })
                    .collect(),
                rules: Vec::with_capacity(storage.rules.len()),
            })),
        };
        for rule in storage.rules.iter() {
            let event = public_event(&storage, &rule.event)?;
            let conditions = rule
                .conditions
                .iter()
                .map(|condition| {
                    Ok(Condition {
                        value: public_value(&storage, condition.value)?,
                        disabled: condition.disabled,
                    })
                })
                .collect::<Result<Vec<_>>>()?;
            let mut actions = Vec::new();
            let mut action_provenance = Vec::new();
            for action in &rule.actions {
                public_actions(&storage, *action, &mut actions)?;
                public_action_provenance(&storage, *action, &mut action_provenance)?;
            }
            let event_name = match &rule.event {
                wir::Event::Subroutine { name_span, .. } => *name_span,
                _ => None,
            };
            program
                .provenance
                .as_mut()
                .expect("parsed programs retain provenance")
                .rules
                .push(RuleProvenance {
                    span: rule.span,
                    name: rule.name_span,
                    event_name,
                    conditions: rule
                        .conditions
                        .iter()
                        .map(|condition| value_provenance(&storage, condition.value))
                        .collect(),
                    actions: action_provenance,
                });
            program.rules.push(Rule {
                name: rule.name.clone(),
                disabled: rule.disabled,
                event,
                conditions,
                actions,
            });
        }
        Ok(program)
    }

    pub(crate) fn to_wir(&self) -> Result<wir::Program> {
        let mut storage = wir::Program {
            settings: self.settings.clone(),
            ..Default::default()
        };

        for file in &self.files {
            storage.add_file(file.clone());
        }

        let mut globals = HashMap::new();
        for (position, variable) in self.global_variables.iter().enumerate() {
            let declaration = self.declaration_provenance(
                |provenance| &provenance.global_variables,
                self.global_variables.len(),
                position,
            );
            let id = storage.global_variables.push(wir::WorkshopVariable {
                name: variable.name.clone(),
                index: variable.index.unwrap_or(position as u32),
                span: declaration.span,
                name_span: declaration.name_span,
            });
            globals.insert(variable.name.clone(), id);
        }
        let mut players = HashMap::new();
        for (position, variable) in self.player_variables.iter().enumerate() {
            let declaration = self.declaration_provenance(
                |provenance| &provenance.player_variables,
                self.player_variables.len(),
                position,
            );
            let id = storage.player_variables.push(wir::WorkshopVariable {
                name: variable.name.clone(),
                index: variable.index.unwrap_or(position as u32),
                span: declaration.span,
                name_span: declaration.name_span,
            });
            players.insert(variable.name.clone(), id);
        }
        let mut subroutines = HashMap::new();
        for (position, subroutine) in self.subroutines.iter().enumerate() {
            let declaration = self.declaration_provenance(
                |provenance| &provenance.subroutines,
                self.subroutines.len(),
                position,
            );
            let id = storage.subroutines.push(wir::WorkshopSubroutine {
                name: subroutine.name.clone(),
                index: subroutine.index.unwrap_or(position as u32),
                span: declaration.span,
                name_span: declaration.name_span,
            });
            subroutines.insert(subroutine.name.clone(), id);
        }

        for (rule_index, rule) in self.rules.iter().enumerate() {
            let mut event = wir_event(&rule.event, &subroutines)?;
            if let wir::Event::Subroutine { name_span, .. } = &mut event {
                *name_span = self
                    .rule_provenance(rule_index)
                    .and_then(|provenance| provenance.event_name);
            }
            let conditions = rule
                .conditions
                .iter()
                .enumerate()
                .map(|(condition_index, condition)| {
                    wir_value(
                        &condition.value,
                        &mut storage,
                        &globals,
                        &players,
                        &subroutines,
                    )
                    .inspect(|&value| {
                        if let Some(provenance) =
                            self.condition_provenance(rule_index, condition_index)
                        {
                            apply_value_provenance(&mut storage, value, provenance);
                        }
                    })
                    .map(|value| wir::Condition {
                        value,
                        disabled: condition.disabled,
                    })
                })
                .collect::<Result<Vec<_>>>()?;
            let mut actions = Vec::new();
            let mut position = 0;
            lower_actions(
                &rule.actions,
                &mut position,
                &mut actions,
                &mut storage,
                &globals,
                &players,
                &subroutines,
            )?;
            if let Some(provenance) = self
                .rule_provenance(rule_index)
                .filter(|provenance| provenance.actions.len() == rule.actions.len())
            {
                let mut public_position = 0;
                apply_action_provenance(
                    &mut storage,
                    &actions,
                    &provenance.actions,
                    &mut public_position,
                )?;
            }
            if position != rule.actions.len() {
                return Err(WorkshopError::Malformed {
                    message: "unexpected control-flow terminator in rule actions".to_string(),
                    span: None,
                });
            }
            storage.rules.push(wir::Rule {
                name: rule.name.clone(),
                span: self.rule_span(rule_index),
                name_span: self
                    .rule_provenance(rule_index)
                    .and_then(|provenance| provenance.name),
                disabled: rule.disabled,
                event,
                conditions,
                actions,
            });
        }
        Ok(storage)
    }
}

fn fit<T: Default>(items: &mut Vec<T>, len: usize) {
    items.truncate(len);
    items.resize_with(len, T::default);
}

fn public_event(storage: &wir::Program, event: &wir::Event) -> Result<Event> {
    Ok(match event {
        wir::Event::Global => Event::Global,
        wir::Event::EachPlayer => Event::EachPlayer,
        wir::Event::EachPlayerWithFilters { team, target } => Event::EachPlayerWithFilters {
            team: public_team(*team),
            target: public_target(target),
        },
        wir::Event::Player { kind, team, target } => Event::Player {
            kind: public_player_event(*kind),
            team: public_team(*team),
            target: public_target(target),
        },
        wir::Event::Subroutine { subroutine, .. } => Event::Subroutine(
            storage
                .subroutines
                .get(*subroutine)
                .ok_or_else(|| malformed_id("subroutine", subroutine.index()))?
                .name
                .clone(),
        ),
    })
}

fn public_team(team: wir::EventTeam) -> EventTeam {
    match team {
        wir::EventTeam::All => EventTeam::All,
        wir::EventTeam::Team1 => EventTeam::Team1,
        wir::EventTeam::Team2 => EventTeam::Team2,
    }
}

fn public_target(target: &wir::EventTarget) -> EventTarget {
    match target {
        wir::EventTarget::All => EventTarget::All,
        wir::EventTarget::Slot(slot) => EventTarget::Slot(*slot),
        wir::EventTarget::Hero(hero) => EventTarget::Hero(hero.clone()),
    }
}

fn public_player_event(kind: wir::PlayerEventKind) -> PlayerEventKind {
    match kind {
        wir::PlayerEventKind::DealtDamage => PlayerEventKind::DealtDamage,
        wir::PlayerEventKind::DealtFinalBlow => PlayerEventKind::DealtFinalBlow,
        wir::PlayerEventKind::DealtHealing => PlayerEventKind::DealtHealing,
        wir::PlayerEventKind::DealtKnockback => PlayerEventKind::DealtKnockback,
        wir::PlayerEventKind::Died => PlayerEventKind::Died,
        wir::PlayerEventKind::EarnedElimination => PlayerEventKind::EarnedElimination,
        wir::PlayerEventKind::Joined => PlayerEventKind::Joined,
        wir::PlayerEventKind::Left => PlayerEventKind::Left,
        wir::PlayerEventKind::ReceivedHealing => PlayerEventKind::ReceivedHealing,
        wir::PlayerEventKind::ReceivedKnockback => PlayerEventKind::ReceivedKnockback,
        wir::PlayerEventKind::TookDamage => PlayerEventKind::TookDamage,
    }
}

fn public_value(storage: &wir::Program, id: wir::ValueId) -> Result<Value> {
    let node = storage
        .values
        .get(id)
        .ok_or_else(|| malformed_id("value", id.index()))?;
    Ok(match &node.value {
        wir::Value::Number { value, .. } => Value::Number(*value),
        wir::Value::String(value) => Value::String(value.clone()),
        wir::Value::LocalizedString(value) => Value::LocalizedString(value.clone()),
        wir::Value::Bool(value) => Value::Bool(*value),
        wir::Value::Null => Value::Null,
        wir::Value::Array(values) => Value::Array(
            values
                .iter()
                .map(|value| public_value(storage, *value))
                .collect::<Result<Vec<_>>>()?,
        ),
        wir::Value::Vector { x, y, z } => Value::Vector {
            x: Box::new(public_value(storage, *x)?),
            y: Box::new(public_value(storage, *y)?),
            z: Box::new(public_value(storage, *z)?),
        },
        wir::Value::Enum { value_type, value } => Value::Enum {
            value_type: value_type.clone(),
            value: value.clone(),
        },
        wir::Value::GlobalVariable(id) => Value::GlobalVariable(
            storage
                .global_variables
                .get(*id)
                .ok_or_else(|| malformed_id("global variable", id.index()))?
                .name
                .clone(),
        ),
        wir::Value::PlayerVariable { player, variable } => Value::PlayerVariable {
            player: Box::new(public_value(storage, *player)?),
            variable: storage
                .player_variables
                .get(*variable)
                .ok_or_else(|| malformed_id("player variable", variable.index()))?
                .name
                .clone(),
        },
        wir::Value::Subroutine(id) => Value::Subroutine(
            storage
                .subroutines
                .get(*id)
                .ok_or_else(|| malformed_id("subroutine", id.index()))?
                .name
                .clone(),
        ),
        wir::Value::EventPlayer => Value::EventPlayer,
        wir::Value::Call { name, args } => Value::Call {
            name: name.clone(),
            args: args
                .iter()
                .map(|arg| public_value(storage, *arg))
                .collect::<Result<Vec<_>>>()?,
        },
    })
}

fn public_actions(
    storage: &wir::Program,
    id: wir::ActionId,
    output: &mut Vec<Action>,
) -> Result<()> {
    let action = storage
        .actions
        .get(id)
        .ok_or_else(|| malformed_id("action", id.index()))?;
    match action {
        wir::Action::SetGlobalVariable {
            variable, value, ..
        } => output.push(Action::SetGlobalVariable {
            variable: storage
                .global_variables
                .get(*variable)
                .ok_or_else(|| malformed_id("global variable", variable.index()))?
                .name
                .clone(),
            value: public_value(storage, *value)?,
        }),
        wir::Action::ModifyGlobalVariable {
            variable,
            op,
            value,
            ..
        } => output.push(Action::ModifyGlobalVariable {
            variable: storage
                .global_variables
                .get(*variable)
                .ok_or_else(|| malformed_id("global variable", variable.index()))?
                .name
                .clone(),
            op: public_modify(*op),
            value: public_value(storage, *value)?,
        }),
        wir::Action::SetPlayerVariable {
            player,
            variable,
            value,
            ..
        } => output.push(Action::SetPlayerVariable {
            player: public_value(storage, *player)?,
            variable: storage
                .player_variables
                .get(*variable)
                .ok_or_else(|| malformed_id("player variable", variable.index()))?
                .name
                .clone(),
            value: public_value(storage, *value)?,
        }),
        wir::Action::ModifyPlayerVariable {
            player,
            variable,
            op,
            value,
            ..
        } => output.push(Action::ModifyPlayerVariable {
            player: public_value(storage, *player)?,
            variable: storage
                .player_variables
                .get(*variable)
                .ok_or_else(|| malformed_id("player variable", variable.index()))?
                .name
                .clone(),
            op: public_modify(*op),
            value: public_value(storage, *value)?,
        }),
        wir::Action::AssignMember {
            target, op, value, ..
        } => output.push(Action::AssignMember {
            target: public_value(storage, *target)?,
            op: op.map(public_modify),
            value: public_value(storage, *value)?,
        }),
        wir::Action::CallSubroutine { subroutine, .. } => output.push(Action::CallSubroutine {
            subroutine: storage
                .subroutines
                .get(*subroutine)
                .ok_or_else(|| malformed_id("subroutine", subroutine.index()))?
                .name
                .clone(),
        }),
        wir::Action::If {
            branches,
            else_body,
            ..
        } => {
            for (index, branch) in branches.iter().enumerate() {
                output.push(if index == 0 {
                    Action::If {
                        condition: public_value(storage, branch.condition)?,
                    }
                } else {
                    Action::ElseIf {
                        condition: public_value(storage, branch.condition)?,
                    }
                });
                for action in &branch.body {
                    public_actions(storage, *action, output)?;
                }
            }
            if let Some(body) = else_body {
                output.push(Action::Else);
                for action in body {
                    public_actions(storage, *action, output)?;
                }
            }
            output.push(Action::End);
        }
        wir::Action::While {
            condition, body, ..
        } => {
            output.push(Action::While {
                condition: public_value(storage, *condition)?,
            });
            for action in body {
                public_actions(storage, *action, output)?;
            }
            output.push(Action::End);
        }
        wir::Action::ForGlobalVariable {
            variable,
            start,
            stop,
            step,
            body,
            ..
        } => {
            output.push(Action::ForGlobalVariable {
                variable: storage
                    .global_variables
                    .get(*variable)
                    .ok_or_else(|| malformed_id("global variable", variable.index()))?
                    .name
                    .clone(),
                start: public_value(storage, *start)?,
                stop: public_value(storage, *stop)?,
                step: public_value(storage, *step)?,
            });
            for action in body {
                public_actions(storage, *action, output)?;
            }
            output.push(Action::End);
        }
        wir::Action::ForPlayerVariable {
            player,
            variable,
            start,
            stop,
            step,
            body,
            ..
        } => {
            output.push(Action::ForPlayerVariable {
                player: public_value(storage, *player)?,
                variable: storage
                    .player_variables
                    .get(*variable)
                    .ok_or_else(|| malformed_id("player variable", variable.index()))?
                    .name
                    .clone(),
                start: public_value(storage, *start)?,
                stop: public_value(storage, *stop)?,
                step: public_value(storage, *step)?,
            });
            for action in body {
                public_actions(storage, *action, output)?;
            }
            output.push(Action::End);
        }
        wir::Action::Disabled { action, .. } => {
            let first = output.len();
            public_actions(storage, *action, output)?;
            let inner = output.remove(first);
            output.insert(first, Action::disabled(inner));
        }
        wir::Action::Call { name, args, .. } => output.push(Action::Call {
            name: name.clone(),
            args: args
                .iter()
                .map(|arg| public_value(storage, *arg))
                .collect::<Result<Vec<_>>>()?,
        }),
    }
    Ok(())
}

fn public_action_provenance(
    storage: &wir::Program,
    id: wir::ActionId,
    output: &mut Vec<ActionProvenance>,
) -> Result<()> {
    let action = storage
        .actions
        .get(id)
        .ok_or_else(|| malformed_id("action", id.index()))?;
    let identifier = action_identifier(action);
    let push = |output: &mut Vec<ActionProvenance>, arguments: &[wir::ValueId]| {
        output.push(ActionProvenance {
            span: action.span(),
            identifier,
            arguments: arguments
                .iter()
                .map(|value| value_provenance(storage, *value))
                .collect(),
        });
    };
    let push_without_span = |output: &mut Vec<ActionProvenance>, arguments: &[wir::ValueId]| {
        output.push(ActionProvenance {
            span: None,
            identifier: None,
            arguments: arguments
                .iter()
                .map(|value| value_provenance(storage, *value))
                .collect(),
        });
    };
    match action {
        wir::Action::SetGlobalVariable { value, .. }
        | wir::Action::ModifyGlobalVariable { value, .. } => push(output, &[*value]),
        wir::Action::SetPlayerVariable { player, value, .. }
        | wir::Action::ModifyPlayerVariable { player, value, .. } => {
            push(output, &[*player, *value])
        }
        wir::Action::AssignMember { target, value, .. } => push(output, &[*target, *value]),
        wir::Action::CallSubroutine { .. } => push(output, &[]),
        wir::Action::If {
            branches,
            else_body,
            ..
        } => {
            for (index, branch) in branches.iter().enumerate() {
                if index == 0 {
                    push(output, &[branch.condition]);
                } else {
                    push_without_span(output, &[branch.condition]);
                }
                for action in &branch.body {
                    public_action_provenance(storage, *action, output)?;
                }
                if index + 1 == branches.len() && else_body.is_none() {
                    push_without_span(output, &[]);
                }
            }
            if let Some(body) = else_body {
                push_without_span(output, &[]);
                for action in body {
                    public_action_provenance(storage, *action, output)?;
                }
                push_without_span(output, &[]);
            }
        }
        wir::Action::While {
            condition, body, ..
        } => {
            push(output, &[*condition]);
            for action in body {
                public_action_provenance(storage, *action, output)?;
            }
            push_without_span(output, &[]);
        }
        wir::Action::ForGlobalVariable {
            start,
            stop,
            step,
            body,
            ..
        } => {
            push(output, &[*start, *stop, *step]);
            for action in body {
                public_action_provenance(storage, *action, output)?;
            }
            push_without_span(output, &[]);
        }
        wir::Action::ForPlayerVariable {
            player,
            start,
            stop,
            step,
            body,
            ..
        } => {
            push(output, &[*player, *start, *stop, *step]);
            for action in body {
                public_action_provenance(storage, *action, output)?;
            }
            push_without_span(output, &[]);
        }
        wir::Action::Disabled { action, .. } => {
            public_action_provenance(storage, *action, output)?;
        }
        wir::Action::Call { args, .. } => push(output, args),
    }
    Ok(())
}

fn lower_actions(
    actions: &[Action],
    position: &mut usize,
    output: &mut Vec<wir::ActionId>,
    storage: &mut wir::Program,
    globals: &HashMap<String, wir::GlobalVarId>,
    players: &HashMap<String, wir::PlayerVarId>,
    subroutines: &HashMap<String, wir::SubroutineId>,
) -> Result<()> {
    while *position < actions.len() {
        let (disabled, current) = match &actions[*position] {
            Action::Disabled { action } => (true, action.as_ref()),
            action => (false, action),
        };
        if disabled
            && matches!(
                current,
                Action::ElseIf { .. } | Action::Else | Action::End | Action::Disabled { .. }
            )
        {
            return Err(WorkshopError::Unsupported {
                message: "the disabled modifier applies to a single executable action".to_string(),
                span: None,
            });
        }
        match current {
            Action::ElseIf { .. } | Action::Else | Action::End => return Ok(()),
            Action::If { condition } => {
                *position += 1;
                let mut branches = vec![wir::IfBranch {
                    condition: wir_value(condition, storage, globals, players, subroutines)?,
                    body: Vec::new(),
                }];
                lower_actions(
                    actions,
                    position,
                    &mut branches[0].body,
                    storage,
                    globals,
                    players,
                    subroutines,
                )?;
                while let Some(Action::ElseIf { condition }) = actions.get(*position) {
                    *position += 1;
                    let mut body = Vec::new();
                    lower_actions(
                        actions,
                        position,
                        &mut body,
                        storage,
                        globals,
                        players,
                        subroutines,
                    )?;
                    branches.push(wir::IfBranch {
                        condition: wir_value(condition, storage, globals, players, subroutines)?,
                        body,
                    });
                }
                let else_body = if matches!(actions.get(*position), Some(Action::Else)) {
                    *position += 1;
                    let mut body = Vec::new();
                    lower_actions(
                        actions,
                        position,
                        &mut body,
                        storage,
                        globals,
                        players,
                        subroutines,
                    )?;
                    Some(body)
                } else {
                    None
                };
                if !matches!(actions.get(*position), Some(Action::End)) {
                    return Err(WorkshopError::Malformed {
                        message: "control-flow action is missing End".to_string(),
                        span: None,
                    });
                }
                *position += 1;
                output.push(storage.actions.push(wir::Action::If {
                    branches,
                    else_body,
                    span: None,
                }));
            }
            Action::While { condition } => {
                *position += 1;
                let mut body = Vec::new();
                lower_actions(
                    actions,
                    position,
                    &mut body,
                    storage,
                    globals,
                    players,
                    subroutines,
                )?;
                require_end(actions, position)?;
                let condition = wir_value(condition, storage, globals, players, subroutines)?;
                output.push(storage.actions.push(wir::Action::While {
                    condition,
                    body,
                    span: None,
                }));
            }
            Action::ForGlobalVariable {
                variable,
                start,
                stop,
                step,
            } => {
                *position += 1;
                let mut body = Vec::new();
                lower_actions(
                    actions,
                    position,
                    &mut body,
                    storage,
                    globals,
                    players,
                    subroutines,
                )?;
                require_end(actions, position)?;
                let variable = *globals
                    .get(variable)
                    .ok_or_else(|| unknown_name("global variable", variable))?;
                let start = wir_value(start, storage, globals, players, subroutines)?;
                let stop = wir_value(stop, storage, globals, players, subroutines)?;
                let step = wir_value(step, storage, globals, players, subroutines)?;
                output.push(storage.actions.push(wir::Action::ForGlobalVariable {
                    variable,
                    start,
                    stop,
                    step,
                    body,
                    span: None,
                    target_span: None,
                }));
            }
            Action::ForPlayerVariable {
                player,
                variable,
                start,
                stop,
                step,
            } => {
                *position += 1;
                let mut body = Vec::new();
                lower_actions(
                    actions,
                    position,
                    &mut body,
                    storage,
                    globals,
                    players,
                    subroutines,
                )?;
                require_end(actions, position)?;
                let player = wir_value(player, storage, globals, players, subroutines)?;
                let variable = *players
                    .get(variable)
                    .ok_or_else(|| unknown_name("player variable", variable))?;
                let start = wir_value(start, storage, globals, players, subroutines)?;
                let stop = wir_value(stop, storage, globals, players, subroutines)?;
                let step = wir_value(step, storage, globals, players, subroutines)?;
                output.push(storage.actions.push(wir::Action::ForPlayerVariable {
                    player,
                    variable,
                    start,
                    stop,
                    step,
                    body,
                    span: None,
                    target_span: None,
                }));
            }
            action => {
                *position += 1;
                let lowered = wir_action(action, storage, globals, players, subroutines)?;
                output.push(lowered);
            }
        }
        if disabled {
            let action = output.pop().expect("a lowered action was just pushed");
            output.push(
                storage
                    .actions
                    .push(wir::Action::Disabled { action, span: None }),
            );
        }
    }
    Ok(())
}

fn apply_action_provenance(
    storage: &mut wir::Program,
    actions: &[wir::ActionId],
    provenance: &[ActionProvenance],
    position: &mut usize,
) -> Result<()> {
    for id in actions {
        let action = storage
            .actions
            .get(*id)
            .cloned()
            .ok_or_else(|| malformed_id("action", id.index()))?;
        match action {
            wir::Action::If {
                branches,
                else_body,
                ..
            } => {
                let source = provenance.get(*position).cloned().unwrap_or_default();
                *position += 1;
                apply_action_source(storage, *id, &source);
                for (branch_index, branch) in branches.iter().enumerate() {
                    if branch_index > 0 {
                        let source = provenance.get(*position).cloned().unwrap_or_default();
                        *position += 1;
                        if let Some(provenance) = source.arguments.first() {
                            apply_value_provenance(storage, branch.condition, provenance);
                        }
                    }
                    apply_action_provenance(storage, &branch.body, provenance, position)?;
                }
                if let Some(body) = else_body {
                    *position += 1;
                    apply_action_provenance(storage, &body, provenance, position)?;
                }
                *position += 1;
            }
            wir::Action::While { body, .. } => {
                let source = provenance.get(*position).cloned().unwrap_or_default();
                *position += 1;
                apply_action_source(storage, *id, &source);
                apply_action_provenance(storage, &body, provenance, position)?;
                *position += 1;
            }
            wir::Action::ForGlobalVariable { body, .. }
            | wir::Action::ForPlayerVariable { body, .. } => {
                let source = provenance.get(*position).cloned().unwrap_or_default();
                *position += 1;
                apply_action_source(storage, *id, &source);
                apply_action_provenance(storage, &body, provenance, position)?;
                *position += 1;
            }
            wir::Action::Disabled { action, .. } => {
                let start = *position;
                apply_action_provenance(storage, &[action], provenance, position)?;
                let source = provenance.get(start).cloned().unwrap_or_default();
                if let Some(wir::Action::Disabled { span, .. }) = storage.actions.get_mut(*id) {
                    *span = source.span;
                }
            }
            _ => {
                let source = provenance.get(*position).cloned().unwrap_or_default();
                *position += 1;
                apply_action_source(storage, *id, &source);
            }
        }
    }
    Ok(())
}

fn apply_action_source(storage: &mut wir::Program, id: wir::ActionId, source: &ActionProvenance) {
    if let Some(action) = storage.actions.get_mut(id) {
        match action {
            wir::Action::SetGlobalVariable {
                span, target_span, ..
            }
            | wir::Action::ModifyGlobalVariable {
                span, target_span, ..
            }
            | wir::Action::SetPlayerVariable {
                span, target_span, ..
            }
            | wir::Action::ModifyPlayerVariable {
                span, target_span, ..
            }
            | wir::Action::ForGlobalVariable {
                span, target_span, ..
            }
            | wir::Action::ForPlayerVariable {
                span, target_span, ..
            } => {
                *span = source.span;
                *target_span = source.identifier;
            }
            wir::Action::CallSubroutine {
                span, callee_span, ..
            } => {
                *span = source.span;
                *callee_span = source.identifier;
            }
            wir::Action::AssignMember { span, .. }
            | wir::Action::If { span, .. }
            | wir::Action::While { span, .. }
            | wir::Action::Disabled { span, .. }
            | wir::Action::Call { span, .. } => *span = source.span,
        }
    }
    let value_ids = storage
        .actions
        .get(id)
        .map(action_value_ids)
        .unwrap_or_default();
    for (value, provenance) in value_ids.into_iter().zip(&source.arguments) {
        apply_value_provenance(storage, value, provenance);
    }
}

fn action_value_ids(action: &wir::Action) -> Vec<wir::ValueId> {
    match action {
        wir::Action::SetGlobalVariable { value, .. }
        | wir::Action::ModifyGlobalVariable { value, .. } => vec![*value],
        wir::Action::SetPlayerVariable { player, value, .. }
        | wir::Action::ModifyPlayerVariable { player, value, .. } => vec![*player, *value],
        wir::Action::AssignMember { target, value, .. } => vec![*target, *value],
        wir::Action::If { branches, .. } => {
            branches.iter().map(|branch| branch.condition).collect()
        }
        wir::Action::While { condition, .. } => vec![*condition],
        wir::Action::ForGlobalVariable {
            start, stop, step, ..
        } => vec![*start, *stop, *step],
        wir::Action::ForPlayerVariable {
            player,
            start,
            stop,
            step,
            ..
        } => vec![*player, *start, *stop, *step],
        wir::Action::Call { args, .. } => args.clone(),
        wir::Action::CallSubroutine { .. } | wir::Action::Disabled { .. } => Vec::new(),
    }
}

/// The recorded span of the variable or subroutine a WIR action names, when
/// the parse produced one.
fn action_identifier(action: &wir::Action) -> Option<Span> {
    match action {
        wir::Action::SetGlobalVariable { target_span, .. }
        | wir::Action::ModifyGlobalVariable { target_span, .. }
        | wir::Action::SetPlayerVariable { target_span, .. }
        | wir::Action::ModifyPlayerVariable { target_span, .. }
        | wir::Action::ForGlobalVariable { target_span, .. }
        | wir::Action::ForPlayerVariable { target_span, .. } => *target_span,
        wir::Action::CallSubroutine { callee_span, .. } => *callee_span,
        _ => None,
    }
}

/// The recorded provenance of a WIR value node and its children, mirroring
/// the public [`Value`] tree.
fn value_provenance(storage: &wir::Program, id: wir::ValueId) -> ValueProvenance {
    let Some(node) = storage.values.get(id) else {
        return ValueProvenance::default();
    };
    ValueProvenance {
        span: node.span,
        identifier: node.identifier,
        children: wir_value_children(&node.value)
            .into_iter()
            .map(|child| value_provenance(storage, child))
            .collect(),
    }
}

/// The child value ids of a WIR value, in the order the public [`Value`]
/// exposes them.
fn wir_value_children(value: &wir::Value) -> Vec<wir::ValueId> {
    match value {
        wir::Value::Array(values) => values.clone(),
        wir::Value::Vector { x, y, z } => vec![*x, *y, *z],
        wir::Value::PlayerVariable { player, .. } => vec![*player],
        wir::Value::Call { args, .. } => args.clone(),
        _ => Vec::new(),
    }
}

/// Write a recorded value-provenance tree back onto a WIR value subtree.
fn apply_value_provenance(
    storage: &mut wir::Program,
    value: wir::ValueId,
    source: &ValueProvenance,
) {
    let Some(node) = storage.values.get(value) else {
        return;
    };
    let children = wir_value_children(&node.value);
    if let Some(node) = storage.values.get_mut(value) {
        node.span = source.span;
        node.identifier = source.identifier;
    }
    for (child, source) in children.into_iter().zip(&source.children) {
        apply_value_provenance(storage, child, source);
    }
}

fn require_end(actions: &[Action], position: &mut usize) -> Result<()> {
    if !matches!(actions.get(*position), Some(Action::End)) {
        return Err(WorkshopError::Malformed {
            message: "control-flow action is missing End".to_string(),
            span: None,
        });
    }
    *position += 1;
    Ok(())
}

fn wir_action(
    action: &Action,
    storage: &mut wir::Program,
    globals: &HashMap<String, wir::GlobalVarId>,
    players: &HashMap<String, wir::PlayerVarId>,
    subroutines: &HashMap<String, wir::SubroutineId>,
) -> Result<wir::ActionId> {
    let action = match action {
        Action::SetGlobalVariable { variable, value } => wir::Action::SetGlobalVariable {
            variable: *globals
                .get(variable)
                .ok_or_else(|| unknown_name("global variable", variable))?,
            value: wir_value(value, storage, globals, players, subroutines)?,
            span: None,
            target_span: None,
        },
        Action::ModifyGlobalVariable {
            variable,
            op,
            value,
        } => wir::Action::ModifyGlobalVariable {
            variable: *globals
                .get(variable)
                .ok_or_else(|| unknown_name("global variable", variable))?,
            op: wir_modify(*op),
            value: wir_value(value, storage, globals, players, subroutines)?,
            span: None,
            target_span: None,
        },
        Action::SetPlayerVariable {
            player,
            variable,
            value,
        } => wir::Action::SetPlayerVariable {
            player: wir_value(player, storage, globals, players, subroutines)?,
            variable: *players
                .get(variable)
                .ok_or_else(|| unknown_name("player variable", variable))?,
            value: wir_value(value, storage, globals, players, subroutines)?,
            span: None,
            target_span: None,
        },
        Action::ModifyPlayerVariable {
            player,
            variable,
            op,
            value,
        } => wir::Action::ModifyPlayerVariable {
            player: wir_value(player, storage, globals, players, subroutines)?,
            variable: *players
                .get(variable)
                .ok_or_else(|| unknown_name("player variable", variable))?,
            op: wir_modify(*op),
            value: wir_value(value, storage, globals, players, subroutines)?,
            span: None,
            target_span: None,
        },
        Action::AssignMember { target, op, value } => wir::Action::AssignMember {
            target: wir_value(target, storage, globals, players, subroutines)?,
            op: op.map(wir_modify),
            value: wir_value(value, storage, globals, players, subroutines)?,
            span: None,
        },
        Action::CallSubroutine { subroutine } => wir::Action::CallSubroutine {
            subroutine: *subroutines
                .get(subroutine)
                .ok_or_else(|| unknown_name("subroutine", subroutine))?,
            span: None,
            callee_span: None,
        },
        Action::Disabled { .. } => {
            unreachable!("disabled actions are lowered by lower_actions")
        }
        Action::Call { name, args } => wir::Action::Call {
            name: name.clone(),
            args: args
                .iter()
                .map(|arg| wir_value(arg, storage, globals, players, subroutines))
                .collect::<Result<Vec<_>>>()?,
            span: None,
        },
        Action::ElseIf { .. }
        | Action::Else
        | Action::End
        | Action::If { .. }
        | Action::While { .. }
        | Action::ForGlobalVariable { .. }
        | Action::ForPlayerVariable { .. } => {
            unreachable!("structured actions are lowered by lower_actions")
        }
    };
    Ok(storage.actions.push(action))
}

fn wir_value(
    value: &Value,
    storage: &mut wir::Program,
    globals: &HashMap<String, wir::GlobalVarId>,
    players: &HashMap<String, wir::PlayerVarId>,
    subroutines: &HashMap<String, wir::SubroutineId>,
) -> Result<wir::ValueId> {
    let value = match value {
        Value::Number(value) => wir::Value::Number {
            value: *value,
            text: crate::core::format::format_number(*value),
        },
        Value::String(value) => wir::Value::String(value.clone()),
        Value::LocalizedString(value) => wir::Value::LocalizedString(value.clone()),
        Value::Bool(value) => wir::Value::Bool(*value),
        Value::Null => wir::Value::Null,
        Value::Array(values) => wir::Value::Array(
            values
                .iter()
                .map(|value| wir_value(value, storage, globals, players, subroutines))
                .collect::<Result<Vec<_>>>()?,
        ),
        Value::Vector { x, y, z } => wir::Value::Vector {
            x: wir_value(x, storage, globals, players, subroutines)?,
            y: wir_value(y, storage, globals, players, subroutines)?,
            z: wir_value(z, storage, globals, players, subroutines)?,
        },
        Value::Enum { value_type, value } => wir::Value::Enum {
            value_type: value_type.clone(),
            value: value.clone(),
        },
        Value::GlobalVariable(name) => wir::Value::GlobalVariable(
            *globals
                .get(name)
                .ok_or_else(|| unknown_name("global variable", name))?,
        ),
        Value::PlayerVariable { player, variable } => wir::Value::PlayerVariable {
            player: wir_value(player, storage, globals, players, subroutines)?,
            variable: *players
                .get(variable)
                .ok_or_else(|| unknown_name("player variable", variable))?,
        },
        Value::Subroutine(name) => wir::Value::Subroutine(
            *subroutines
                .get(name)
                .ok_or_else(|| unknown_name("subroutine", name))?,
        ),
        Value::EventPlayer => wir::Value::EventPlayer,
        Value::Call { name, args } => wir::Value::Call {
            name: name.clone(),
            args: args
                .iter()
                .map(|arg| wir_value(arg, storage, globals, players, subroutines))
                .collect::<Result<Vec<_>>>()?,
        },
    };
    Ok(storage.values.push(wir::ValueNode::new(value, None)))
}

fn wir_event(
    event: &Event,
    subroutines: &HashMap<String, wir::SubroutineId>,
) -> Result<wir::Event> {
    Ok(match event {
        Event::Global => wir::Event::Global,
        Event::EachPlayer => wir::Event::EachPlayer,
        Event::EachPlayerWithFilters { team, target } => wir::Event::EachPlayerWithFilters {
            team: wir_team(*team),
            target: wir_target(target),
        },
        Event::Player { kind, team, target } => wir::Event::Player {
            kind: wir_player_event(*kind),
            team: wir_team(*team),
            target: wir_target(target),
        },
        Event::Subroutine(name) => wir::Event::Subroutine {
            subroutine: *subroutines
                .get(name)
                .ok_or_else(|| unknown_name("subroutine", name))?,
            name_span: None,
        },
    })
}

fn wir_team(team: EventTeam) -> wir::EventTeam {
    match team {
        EventTeam::All => wir::EventTeam::All,
        EventTeam::Team1 => wir::EventTeam::Team1,
        EventTeam::Team2 => wir::EventTeam::Team2,
    }
}

fn wir_target(target: &EventTarget) -> wir::EventTarget {
    match target {
        EventTarget::All => wir::EventTarget::All,
        EventTarget::Slot(slot) => wir::EventTarget::Slot(*slot),
        EventTarget::Hero(hero) => wir::EventTarget::Hero(hero.clone()),
    }
}

fn wir_player_event(kind: PlayerEventKind) -> wir::PlayerEventKind {
    match kind {
        PlayerEventKind::DealtDamage => wir::PlayerEventKind::DealtDamage,
        PlayerEventKind::DealtFinalBlow => wir::PlayerEventKind::DealtFinalBlow,
        PlayerEventKind::DealtHealing => wir::PlayerEventKind::DealtHealing,
        PlayerEventKind::DealtKnockback => wir::PlayerEventKind::DealtKnockback,
        PlayerEventKind::Died => wir::PlayerEventKind::Died,
        PlayerEventKind::EarnedElimination => wir::PlayerEventKind::EarnedElimination,
        PlayerEventKind::Joined => wir::PlayerEventKind::Joined,
        PlayerEventKind::Left => wir::PlayerEventKind::Left,
        PlayerEventKind::ReceivedHealing => wir::PlayerEventKind::ReceivedHealing,
        PlayerEventKind::ReceivedKnockback => wir::PlayerEventKind::ReceivedKnockback,
        PlayerEventKind::TookDamage => wir::PlayerEventKind::TookDamage,
    }
}

fn public_modify(op: wir::ModifyOp) -> ModifyOp {
    match op {
        wir::ModifyOp::Add => ModifyOp::Add,
        wir::ModifyOp::Subtract => ModifyOp::Subtract,
        wir::ModifyOp::Multiply => ModifyOp::Multiply,
        wir::ModifyOp::Divide => ModifyOp::Divide,
        wir::ModifyOp::Modulo => ModifyOp::Modulo,
        wir::ModifyOp::Min => ModifyOp::Min,
        wir::ModifyOp::Max => ModifyOp::Max,
        wir::ModifyOp::RaiseToPower => ModifyOp::RaiseToPower,
        wir::ModifyOp::AppendToArray => ModifyOp::AppendToArray,
        wir::ModifyOp::RemoveFromArrayByValue => ModifyOp::RemoveFromArrayByValue,
        wir::ModifyOp::RemoveFromArrayByIndex => ModifyOp::RemoveFromArrayByIndex,
    }
}

fn wir_modify(op: ModifyOp) -> wir::ModifyOp {
    match op {
        ModifyOp::Add => wir::ModifyOp::Add,
        ModifyOp::Subtract => wir::ModifyOp::Subtract,
        ModifyOp::Multiply => wir::ModifyOp::Multiply,
        ModifyOp::Divide => wir::ModifyOp::Divide,
        ModifyOp::Modulo => wir::ModifyOp::Modulo,
        ModifyOp::Min => wir::ModifyOp::Min,
        ModifyOp::Max => wir::ModifyOp::Max,
        ModifyOp::RaiseToPower => wir::ModifyOp::RaiseToPower,
        ModifyOp::AppendToArray => wir::ModifyOp::AppendToArray,
        ModifyOp::RemoveFromArrayByValue => wir::ModifyOp::RemoveFromArrayByValue,
        ModifyOp::RemoveFromArrayByIndex => wir::ModifyOp::RemoveFromArrayByIndex,
    }
}

fn malformed_id(kind: &str, index: usize) -> WorkshopError {
    WorkshopError::Malformed {
        message: format!("dangling {kind} {index}"),
        span: None,
    }
}

fn unknown_name(kind: &str, name: &str) -> WorkshopError {
    WorkshopError::Malformed {
        message: format!("unknown {kind} '{name}'"),
        span: None,
    }
}

/// A Workshop global or player variable declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Variable {
    pub name: String,
    /// The raw Workshop declaration index, when the declaration has one.
    pub index: Option<u32>,
}

impl Variable {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            index: None,
        }
    }

    pub fn with_index(name: impl Into<String>, index: u32) -> Self {
        Self {
            name: name.into(),
            index: Some(index),
        }
    }
}

/// A Workshop subroutine declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Subroutine {
    pub name: String,
    /// The raw Workshop declaration index, when the declaration has one.
    pub index: Option<u32>,
}

impl Subroutine {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            index: None,
        }
    }

    pub fn with_index(name: impl Into<String>, index: u32) -> Self {
        Self {
            name: name.into(),
            index: Some(index),
        }
    }
}

/// A Workshop rule with explicit conditions and a linear action stream.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Rule {
    pub name: String,
    pub disabled: bool,
    pub event: Event,
    pub conditions: Vec<Condition>,
    pub actions: Vec<Action>,
}

impl Rule {
    pub fn new(name: impl Into<String>, event: Event) -> Self {
        Self {
            name: name.into(),
            disabled: false,
            event,
            conditions: Vec::new(),
            actions: Vec::new(),
        }
    }

    pub fn condition(mut self, condition: impl Into<Condition>) -> Self {
        self.conditions.push(condition.into());
        self
    }

    pub fn action(mut self, action: Action) -> Self {
        self.actions.push(action);
        self
    }
}

/// A rule condition. Conditions remain distinct from general value expressions.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Condition {
    pub value: Value,
    pub disabled: bool,
}

impl Condition {
    pub fn new(value: Value) -> Self {
        Self {
            value,
            disabled: false,
        }
    }

    pub fn disabled(value: Value) -> Self {
        Self {
            value,
            disabled: true,
        }
    }
}

impl From<Value> for Condition {
    fn from(value: Value) -> Self {
        Self::new(value)
    }
}

fn action_argument_count(action: &Action) -> usize {
    match action {
        Action::SetGlobalVariable { .. }
        | Action::ModifyGlobalVariable { .. }
        | Action::If { .. }
        | Action::ElseIf { .. }
        | Action::While { .. } => 1,
        Action::SetPlayerVariable { .. } | Action::ModifyPlayerVariable { .. } => 2,
        Action::AssignMember { .. } => 2,
        Action::ForGlobalVariable { .. } => 3,
        Action::ForPlayerVariable { .. } => 4,
        Action::Call { args, .. } => args.len(),
        Action::CallSubroutine { .. } | Action::Else | Action::End => 0,
        Action::Disabled { action } => action_argument_count(action),
    }
}

/// A Workshop event identity and its native filters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    Global,
    EachPlayer,
    EachPlayerWithFilters {
        team: EventTeam,
        target: EventTarget,
    },
    Player {
        kind: PlayerEventKind,
        team: EventTeam,
        target: EventTarget,
    },
    Subroutine(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventTeam {
    All,
    Team1,
    Team2,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventTarget {
    All,
    Slot(u8),
    Hero(String),
}

/// A non-ongoing player event identity.
///
/// Workshop can add player-scoped event identities independently of this
/// crate. Consumers should use a wildcard arm when matching this type.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerEventKind {
    DealtDamage,
    DealtFinalBlow,
    DealtHealing,
    DealtKnockback,
    Died,
    EarnedElimination,
    Joined,
    Left,
    ReceivedHealing,
    ReceivedKnockback,
    TookDamage,
}

/// A Workshop action line. Control flow is represented in the same order as
/// the Workshop source, including its explicit `End` lines.
#[derive(Debug, Clone)]
pub enum Action {
    SetGlobalVariable {
        variable: String,
        value: Value,
    },
    ModifyGlobalVariable {
        variable: String,
        op: ModifyOp,
        value: Value,
    },
    SetPlayerVariable {
        player: Value,
        variable: String,
        value: Value,
    },
    ModifyPlayerVariable {
        player: Value,
        variable: String,
        op: ModifyOp,
        value: Value,
    },
    AssignMember {
        target: Value,
        op: Option<ModifyOp>,
        value: Value,
    },
    CallSubroutine {
        subroutine: String,
    },
    If {
        condition: Value,
    },
    ElseIf {
        condition: Value,
    },
    Else,
    While {
        condition: Value,
    },
    ForGlobalVariable {
        variable: String,
        start: Value,
        stop: Value,
        step: Value,
    },
    ForPlayerVariable {
        player: Value,
        variable: String,
        start: Value,
        stop: Value,
        step: Value,
    },
    End,
    Disabled {
        action: Box<Action>,
    },
    Call {
        name: String,
        args: Vec<Value>,
    },
}

impl Action {
    /// Mark an action as disabled.
    pub fn disabled(action: Action) -> Self {
        Self::Disabled {
            action: Box::new(action),
        }
    }

    /// Construct a dynamic action call by canonical Workshop id.
    pub fn call(name: impl Into<String>, args: impl IntoIterator<Item = Value>) -> Self {
        Self::Call {
            name: name.into(),
            args: args.into_iter().collect(),
        }
    }
}

/// The operation used by a Workshop variable modification action.
///
/// Workshop can add modification operations independently of this crate.
/// Consumers should use a wildcard arm when matching this type.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModifyOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Min,
    Max,
    RaiseToPower,
    AppendToArray,
    RemoveFromArrayByValue,
    RemoveFromArrayByIndex,
}

/// A composable Workshop value expression.
#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    String(String),
    LocalizedString(String),
    Bool(bool),
    Null,
    Array(Vec<Value>),
    Vector {
        x: Box<Value>,
        y: Box<Value>,
        z: Box<Value>,
    },
    Enum {
        value_type: String,
        value: String,
    },
    GlobalVariable(String),
    PlayerVariable {
        player: Box<Value>,
        variable: String,
    },
    Subroutine(String),
    EventPlayer,
    Call {
        name: String,
        args: Vec<Value>,
    },
}

impl Value {
    /// Construct a numeric Workshop literal.
    pub fn number(value: f64) -> Self {
        Self::Number(value)
    }

    /// Construct a custom Workshop string literal.
    pub fn string(value: impl Into<String>) -> Self {
        Self::String(value.into())
    }

    pub fn global_variable(name: impl Into<String>) -> Self {
        Self::GlobalVariable(name.into())
    }

    pub fn player_variable(player: Value, name: impl Into<String>) -> Self {
        Self::PlayerVariable {
            player: Box::new(player),
            variable: name.into(),
        }
    }

    /// Construct a dynamic value call by canonical Workshop id.
    pub fn call(name: impl Into<String>, args: impl IntoIterator<Item = Value>) -> Self {
        Self::Call {
            name: name.into(),
            args: args.into_iter().collect(),
        }
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Self::Number(value)
    }
}

impl From<f32> for Value {
    fn from(value: f32) -> Self {
        Self::Number(f64::from(value))
    }
}

macro_rules! impl_integer_value {
    ($($type:ty),+ $(,)?) => {
        $(
            impl From<$type> for Value {
                fn from(value: $type) -> Self {
                    Self::Number(value as f64)
                }
            }
        )+
    };
}

impl_integer_value!(i8, i16, i32, i64, isize, u8, u16, u32, u64, usize);

impl From<String> for Value {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Self::String(value.to_string())
    }
}

impl<T: Into<Value>> From<Vec<T>> for Value {
    fn from(values: Vec<T>) -> Self {
        Self::Array(values.into_iter().map(Into::into).collect())
    }
}

impl<T: Into<Value>, const N: usize> From<[T; N]> for Value {
    fn from(values: [T; N]) -> Self {
        Self::Array(values.into_iter().map(Into::into).collect())
    }
}
