//! Canonical Workshop program concepts.

use self::identity::{
    NodeIdentity, action_identity, condition_identity, declaration_identity, rule_identity,
    value_identity,
};
use crate::core::error::WorkshopError;
use crate::settings::Settings;
use crate::source::{FileId, SourceDocument, SourceFile, Span};

mod conversion;
mod identity;
pub(crate) mod shared;
mod source_map;
pub use shared::{EventTarget, EventTeam, ModifyOp, PlayerEventKind};
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
    /// The identity of the declaration this record was attached to.
    identity: NodeIdentity,
}

#[derive(Debug, Clone, Default)]
struct RuleProvenance {
    span: Option<crate::source::Span>,
    /// The recorded span of the rule's quoted name in `rule("name")`.
    name: Option<Span>,
    /// The recorded span of the subroutine name a `Subroutine` event binds.
    event_name: Option<Span>,
    /// The identity of the public rule this record was attached to.
    identity: NodeIdentity,
    conditions: Vec<ValueProvenance>,
    actions: Vec<ActionProvenance>,
}

impl RuleProvenance {
    /// Prepare the record for a setter write: attaching to a position whose
    /// node diverged from the record discards its own fields, while the
    /// condition and action tables keep their independently screened records.
    fn reseat(&mut self, identity: NodeIdentity) {
        if self.identity != identity {
            self.span = None;
            self.name = None;
            self.event_name = None;
        }
        self.identity = identity;
    }
}

#[derive(Debug, Clone, Default)]
struct ActionProvenance {
    span: Option<Span>,
    /// The recorded span of the variable or subroutine the action names: a
    /// set/modify/for target or a `Call Subroutine` callee.
    identifier: Option<Span>,
    /// The identity of the public action this record was attached to.
    identity: NodeIdentity,
    arguments: Vec<ValueProvenance>,
}

/// A provenance node carrying an authored span; shared by the child-slot
/// writer used by the condition/action span setters.
trait SpanSlot {
    fn span_slot(&mut self) -> &mut Option<Span>;
    /// Prepare the record for a setter write: attaching to a position whose
    /// node diverged from the record discards its own fields, while child
    /// tables keep their independently screened records.
    fn reseat(&mut self, identity: NodeIdentity);
}

impl SpanSlot for ValueProvenance {
    fn span_slot(&mut self) -> &mut Option<Span> {
        &mut self.span
    }
    fn reseat(&mut self, identity: NodeIdentity) {
        if self.identity != identity {
            self.span = None;
            self.identifier = None;
        }
        self.identity = identity;
    }
}

impl SpanSlot for ActionProvenance {
    fn span_slot(&mut self) -> &mut Option<Span> {
        &mut self.span
    }
    fn reseat(&mut self, identity: NodeIdentity) {
        if self.identity != identity {
            self.span = None;
            self.identifier = None;
        }
        self.identity = identity;
    }
}

/// Clear every recorded span in `provenance` that belongs to `file` and does
/// not resolve inside `document`. A span in another file is not stale: its
/// own source has not been attached yet, so it cannot be judged against this
/// document — `SourceDocument::byte_range` already rejects both a file
/// mismatch and an out-of-bounds extent, so the span's own file gate is what
/// keeps foreign-file spans alive until their source arrives.
fn prune_spans_outside(
    provenance: &mut ProgramProvenance,
    file: FileId,
    document: &SourceDocument,
) {
    fn prune_slot(slot: &mut Option<Span>, file: FileId, document: &SourceDocument) {
        if let Some(span) = slot {
            if span.file == file && document.byte_range(*span).is_none() {
                *slot = None;
            }
        }
    }
    fn prune_value(record: &mut ValueProvenance, file: FileId, document: &SourceDocument) {
        prune_slot(&mut record.span, file, document);
        prune_slot(&mut record.identifier, file, document);
        for child in &mut record.children {
            prune_value(child, file, document);
        }
    }
    for declaration in provenance
        .global_variables
        .iter_mut()
        .chain(&mut provenance.player_variables)
        .chain(&mut provenance.subroutines)
    {
        prune_slot(&mut declaration.span, file, document);
        prune_slot(&mut declaration.name_span, file, document);
    }
    for rule in &mut provenance.rules {
        prune_slot(&mut rule.span, file, document);
        prune_slot(&mut rule.name, file, document);
        prune_slot(&mut rule.event_name, file, document);
        for condition in &mut rule.conditions {
            prune_value(condition, file, document);
        }
        for action in &mut rule.actions {
            prune_slot(&mut action.span, file, document);
            prune_slot(&mut action.identifier, file, document);
            for argument in &mut action.arguments {
                prune_value(argument, file, document);
            }
        }
    }
}

#[derive(Debug, Clone, Default)]
struct ValueProvenance {
    span: Option<Span>,
    /// The recorded span of the variable or subroutine identifier the value
    /// names, when the value is such a reference.
    identifier: Option<Span>,
    /// The identity of the public node this record was attached to.
    identity: NodeIdentity,
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

    /// Attach authored source text to a registered file. Returns `false` when
    /// `file` is not a known file entry.
    ///
    /// Attaching text audits the provenance that belongs to this file: a
    /// recorded span in `file` that does not resolve inside the retained
    /// document is stale data — for a provider-mapped program it can describe
    /// the pre-expansion token stream or a coordinate space the map no longer
    /// owns — and is cleared so consumers see no span rather than a wrong
    /// one, matching how `record_identities` retires displaced records
    /// (wrightkit/wright#583). Spans recorded against other files are left
    /// untouched; they are audited when their own source is attached.
    pub fn set_file_source(&mut self, file: FileId, source: impl Into<String>) -> bool {
        let Some(entry) = self.files.get_mut(file.index()) else {
            return false;
        };
        entry.set_source(source);
        if let (Some(provenance), Some(document)) = (
            self.provenance.as_mut(),
            self.files.get(file.index()).and_then(SourceFile::source),
        ) {
            prune_spans_outside(provenance, file, document);
        }
        true
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
        self.set_child_span(
            rule,
            condition,
            span,
            |rule| rule.conditions.len(),
            |rule, index| condition_identity(&rule.conditions[index]),
            |rule, index| SourceMappingError::InvalidCondition {
                rule,
                condition: index,
            },
            |rule_data| &mut rule_data.conditions,
        )
    }

    /// Attach the authored span of a public action in its linear rule order.
    pub fn set_action_span(
        &mut self,
        rule: usize,
        action: usize,
        span: Option<Span>,
    ) -> std::result::Result<(), SourceMappingError> {
        self.set_child_span(
            rule,
            action,
            span,
            |rule| rule.actions.len(),
            |rule, index| action_identity(&rule.actions[index]),
            |rule, index| SourceMappingError::InvalidAction {
                rule,
                action: index,
            },
            |rule_data| &mut rule_data.actions,
        )
    }

    /// Shared span setter for condition/action provenance slots: validates the
    /// span, bounds-checks the child index against the public rule shape, then
    /// resizes the table and writes the slot with the node's current identity.
    /// The identity closure runs only after the bounds check because it
    /// indexes the public child list.
    #[allow(clippy::too_many_arguments)]
    fn set_child_span<T: SpanSlot + Default>(
        &mut self,
        rule: usize,
        index: usize,
        span: Option<Span>,
        count: impl Fn(&crate::Rule) -> usize,
        identity: impl Fn(&crate::Rule, usize) -> NodeIdentity,
        invalid: impl Fn(usize, usize) -> SourceMappingError,
        slots: impl Fn(&mut RuleProvenance) -> &mut Vec<T>,
    ) -> std::result::Result<(), SourceMappingError> {
        self.validate_span(span)?;
        let public = self
            .rules
            .get(rule)
            .ok_or(SourceMappingError::InvalidRule(rule))?;
        let child_count = count(public);
        if index >= child_count {
            return Err(invalid(rule, index));
        }
        let identity = identity(public, index);
        let rule_data = self.rule_provenance_mut(rule)?;
        fit(slots(rule_data), child_count);
        let record = &mut slots(rule_data)[index];
        record.reseat(identity);
        *record.span_slot() = span;
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
        let (identity, argument_count) = {
            let action_value = self
                .rules
                .get(rule)
                .ok_or(SourceMappingError::InvalidRule(rule))?
                .actions
                .get(action)
                .ok_or(SourceMappingError::InvalidAction { rule, action })?;
            let argument_values = action_argument_values(action_value);
            let Some(&argument_value) = argument_values.get(argument) else {
                return Err(SourceMappingError::InvalidActionArgument {
                    rule,
                    action,
                    argument,
                });
            };
            (value_identity(argument_value), argument_values.len())
        };
        let action_data = self.action_provenance_mut(rule, action)?;
        fit(&mut action_data.arguments, argument_count);
        let record = &mut action_data.arguments[argument];
        record.reseat(identity);
        record.span = span;
        Ok(())
    }

    /// Attach the authored and identifier spans of a global variable.
    pub fn set_global_variable_spans(
        &mut self,
        variable: usize,
        span: Option<Span>,
        name_span: Option<Span>,
    ) -> std::result::Result<(), SourceMappingError> {
        self.set_declaration_spans(DeclarationTable::GlobalVariables, variable, span, name_span)
    }

    /// Attach the authored and identifier spans of a player variable.
    pub fn set_player_variable_spans(
        &mut self,
        variable: usize,
        span: Option<Span>,
        name_span: Option<Span>,
    ) -> std::result::Result<(), SourceMappingError> {
        self.set_declaration_spans(DeclarationTable::PlayerVariables, variable, span, name_span)
    }

    /// Attach the authored and identifier spans of a subroutine.
    pub fn set_subroutine_spans(
        &mut self,
        subroutine: usize,
        span: Option<Span>,
        name_span: Option<Span>,
    ) -> std::result::Result<(), SourceMappingError> {
        self.set_declaration_spans(DeclarationTable::Subroutines, subroutine, span, name_span)
    }

    fn set_declaration_spans(
        &mut self,
        table: DeclarationTable,
        index: usize,
        span: Option<Span>,
        name_span: Option<Span>,
    ) -> std::result::Result<(), SourceMappingError> {
        self.validate_span(span)?;
        self.validate_span(name_span)?;
        let count = table.count(self);
        if index >= count {
            return Err(table.invalid(index));
        }
        let identity = table.identity(self, index);
        let slots = table.slots(self.provenance_mut());
        fit(slots, count);
        slots[index] = DeclarationProvenance {
            span,
            name_span,
            identity,
        };
        Ok(())
    }

    /// Return the authored span of a public rule, when source metadata exists.
    ///
    /// Attached mappings record the program shape and node content they were
    /// attached to. Every span accessor returns `None` once the corresponding
    /// public list has been inserted into or removed from, or once the node
    /// at that position no longer has the content the mapping was attached
    /// to — see "Shape and content guard" in `docs/source-preservation.md`.
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
        self.action_argument_provenance(rule, action, argument)?
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
            &self.global_variables,
            variable,
        )
    }

    /// Return the authored span of the declared name of a player variable.
    /// See [`global_variable_name_span`](Self::global_variable_name_span).
    pub fn player_variable_name_span(&self, variable: usize) -> Option<Span> {
        self.declaration_name_span(
            |provenance| &provenance.player_variables,
            &self.player_variables,
            variable,
        )
    }

    /// Return the authored span of the declared name of a subroutine.
    /// See [`global_variable_name_span`](Self::global_variable_name_span).
    pub fn subroutine_name_span(&self, subroutine: usize) -> Option<Span> {
        self.declaration_name_span(
            |provenance| &provenance.subroutines,
            &self.subroutines,
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
        let record = self.condition_value_record(rule, condition, path)?;
        record.identifier.or(record.span)
    }

    /// The provenance record of a value nested inside a public rule
    /// condition, while the node at `path` still has the recorded identity.
    /// Unlike [`condition_value_span`](Self::condition_value_span) callers can
    /// read the node's own span independently of its identifier span.
    fn condition_value_record(
        &self,
        rule: usize,
        condition: usize,
        path: &[usize],
    ) -> Option<&ValueProvenance> {
        let record = self.condition_record(rule, condition)?;
        let value = &self.rules[rule].conditions[condition].value;
        let identity = condition_identity(&self.rules[rule].conditions[condition]);
        value_provenance_at(record, value, path, identity)
    }

    /// The authored node span of a value nested inside a public rule
    /// condition, without the identifier-span substitution
    /// [`condition_value_span`](Self::condition_value_span) performs for
    /// variable and subroutine references.
    pub(crate) fn condition_value_node_span(
        &self,
        rule: usize,
        condition: usize,
        path: &[usize],
    ) -> Option<crate::source::Span> {
        self.condition_value_record(rule, condition, path)?.span
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
        let record = self.action_argument_value_record(rule, action, argument, path)?;
        record.identifier.or(record.span)
    }

    /// The provenance record of a value nested inside a direct value argument
    /// of a public action, while the node at `path` still has the recorded
    /// identity.
    fn action_argument_value_record(
        &self,
        rule: usize,
        action: usize,
        argument: usize,
        path: &[usize],
    ) -> Option<&ValueProvenance> {
        let record = self.argument_record(rule, action, argument)?;
        let value = action_argument_values(&self.rules[rule].actions[action])[argument];
        value_provenance_at(record, value, path, value_identity(value))
    }

    /// The authored node span of a value nested inside a direct value
    /// argument of a public action, without the identifier-span substitution
    /// [`action_argument_value_span`](Self::action_argument_value_span)
    /// performs for variable and subroutine references.
    pub(crate) fn action_argument_value_node_span(
        &self,
        rule: usize,
        action: usize,
        argument: usize,
        path: &[usize],
    ) -> Option<crate::source::Span> {
        self.action_argument_value_record(rule, action, argument, path)?
            .span
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
    ///
    /// Settings are emission-checked too, so a program that would fail to
    /// emit fails validation the same way.
    pub fn validate(&self) -> std::result::Result<(), WorkshopError> {
        let storage = self.to_wir()?;
        storage
            .validate()
            .map_err(|error| WorkshopError::malformed(error.to_string(), error.span()))?;
        if let Some(settings) = &self.settings {
            if let Some(error) = crate::settings::check_emission(settings).into_iter().next() {
                return Err(error);
            }
        }
        Ok(())
    }

    /// Report every settings member the emission table rejects — plus, as
    /// [`crate::settings::DiagnosticSeverity::Warning`] entries, carried
    /// members close enough to a declared spelling to look like a
    /// misspelling — each with its source span and, when exactly one
    /// canonical spelling is close enough, the structured suggestion a
    /// caller can apply. [`validate`] already fails on the first error;
    /// warnings describe accepted members and never fail validation.
    ///
    /// [`validate`]: Program::validate
    pub fn settings_diagnostics(&self) -> Vec<crate::settings::SettingsDiagnostic> {
        self.settings
            .as_ref()
            .map(crate::settings::check_emission_diagnostics)
            .unwrap_or_default()
    }

    /// Report constructs that are structurally preserved but not fully
    /// understood by the canonical catalog.
    ///
    /// Residual inspection is defined over this public canonical model and is
    /// independent from [`validate`](Self::validate): it may be used on
    /// programs `validate` rejects, and an internal materialization failure
    /// does not suppress observable residuals. Callers that also require
    /// structural validity call `validate` separately; this inventory does
    /// not report validation errors.
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

    /// The recorded provenance row for a rule, while the rule table still
    /// matches the public rule count. Used both for the record's own fields
    /// — [`rule_provenance`] adds the identity check — and for descending
    /// into its condition and action tables, which carry records of their
    /// own.
    fn rule_record(&self, rule: usize) -> Option<&RuleProvenance> {
        let recorded = &self.provenance.as_deref()?.rules;
        if recorded.len() != self.rules.len() {
            return None;
        }
        recorded.get(rule)
    }

    /// The recorded provenance of a rule's own fields: the record is
    /// returned only while the rule at that position still has the recorded
    /// identity.
    fn rule_provenance(&self, rule: usize) -> Option<&RuleProvenance> {
        self.rule_record(rule)
            .filter(|record| record.identity == rule_identity(&self.rules[rule]))
    }

    fn action_record(&self, rule: usize, action: usize) -> Option<&ActionProvenance> {
        let recorded = self.rule_record(rule)?;
        if recorded.actions.len() != self.rules[rule].actions.len() {
            return None;
        }
        recorded.actions.get(action)
    }

    /// The recorded provenance of an action's own fields: the record is
    /// returned only while the action at that position still has the
    /// recorded identity.
    fn action_provenance(&self, rule: usize, action: usize) -> Option<&ActionProvenance> {
        self.action_record(rule, action)
            .filter(|record| record.identity == action_identity(&self.rules[rule].actions[action]))
    }

    fn argument_record(
        &self,
        rule: usize,
        action: usize,
        argument: usize,
    ) -> Option<&ValueProvenance> {
        let recorded = self.action_record(rule, action)?;
        let values = action_argument_values(&self.rules[rule].actions[action]);
        if recorded.arguments.len() != values.len() {
            return None;
        }
        recorded.arguments.get(argument)
    }

    /// The recorded provenance of an action argument's own fields: the
    /// record is returned only while the argument value at that position
    /// still has the recorded identity.
    fn action_argument_provenance(
        &self,
        rule: usize,
        action: usize,
        argument: usize,
    ) -> Option<&ValueProvenance> {
        let record = self.argument_record(rule, action, argument)?;
        let values = action_argument_values(&self.rules[rule].actions[action]);
        (record.identity == value_identity(values[argument])).then_some(record)
    }

    fn condition_record(&self, rule: usize, condition: usize) -> Option<&ValueProvenance> {
        let recorded = self.rule_record(rule)?;
        if recorded.conditions.len() != self.rules[rule].conditions.len() {
            return None;
        }
        recorded.conditions.get(condition)
    }

    /// The recorded provenance of a condition's own fields: the record is
    /// returned only while the condition at that position still has the
    /// recorded identity.
    fn condition_provenance(&self, rule: usize, condition: usize) -> Option<&ValueProvenance> {
        self.condition_record(rule, condition).filter(|record| {
            record.identity == condition_identity(&self.rules[rule].conditions[condition])
        })
    }

    fn declaration_name_span<T: std::fmt::Debug>(
        &self,
        recorded: impl Fn(&ProgramProvenance) -> &[DeclarationProvenance],
        nodes: &[T],
        position: usize,
    ) -> Option<Span> {
        let declaration = self.declaration_provenance(recorded, nodes, position);
        declaration.name_span.or(declaration.span)
    }

    fn declaration_provenance<T: std::fmt::Debug>(
        &self,
        recorded: impl Fn(&ProgramProvenance) -> &[DeclarationProvenance],
        nodes: &[T],
        position: usize,
    ) -> DeclarationProvenance {
        self.provenance
            .as_deref()
            .map(recorded)
            .filter(|recorded| recorded.len() == nodes.len())
            .and_then(|recorded| recorded.get(position).zip(nodes.get(position)))
            .filter(|(record, node)| record.identity == declaration_identity(node))
            .map(|(record, _)| *record)
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

    /// Mutable provenance access for the span setters: the addressed record
    /// is reseated for the current node so attaching a span never lets a
    /// record displaced by mutation resurface on a different node.
    fn rule_provenance_mut(
        &mut self,
        rule: usize,
    ) -> std::result::Result<&mut RuleProvenance, SourceMappingError> {
        if rule >= self.rules.len() {
            return Err(SourceMappingError::InvalidRule(rule));
        }
        let identity = rule_identity(&self.rules[rule]);
        let rule_count = self.rules.len();
        let provenance = self.provenance_mut();
        fit(&mut provenance.rules, rule_count);
        let record = &mut provenance.rules[rule];
        record.reseat(identity);
        Ok(record)
    }

    fn action_provenance_mut(
        &mut self,
        rule: usize,
        action: usize,
    ) -> std::result::Result<&mut ActionProvenance, SourceMappingError> {
        let public = self
            .rules
            .get(rule)
            .ok_or(SourceMappingError::InvalidRule(rule))?;
        let action_count = public.actions.len();
        if action >= action_count {
            return Err(SourceMappingError::InvalidAction { rule, action });
        }
        let identity = action_identity(&public.actions[action]);
        let rule_data = self.rule_provenance_mut(rule)?;
        fit(&mut rule_data.actions, action_count);
        let record = &mut rule_data.actions[action];
        record.reseat(identity);
        Ok(record)
    }

    /// Record the identity of the public node every attached provenance record
    /// describes, so records are only returned for the content they were
    /// attached to. Called once after parsing or applying a [`SourceMap`].
    fn record_identities(&mut self) {
        let Some(mut provenance) = self.provenance.take() else {
            return;
        };
        identity::record_identities(&mut provenance, self);
        self.provenance = Some(provenance);
    }
}

/// Walk the provenance tree of `value` along `path`. Descending only
/// requires each level's child table to match the public child count, so a
/// node that went stale does not hide unaffected siblings; the record at the
/// path's end is returned only while the node there still has the recorded
/// identity — `root_identity` for the starting record, its own
/// [`value_identity`] for every record reached by path.
fn value_provenance_at<'a>(
    record: &'a ValueProvenance,
    value: &'a Value,
    path: &[usize],
    root_identity: NodeIdentity,
) -> Option<&'a ValueProvenance> {
    let mut record = record;
    let mut value = value;
    let mut identity = root_identity;
    for &index in path {
        let children = value_children(value);
        if record.children.len() != children.len() {
            return None;
        }
        record = record.children.get(index)?;
        value = children[index];
        identity = value_identity(value);
    }
    (record.identity == identity).then_some(record)
}

fn fit<T: Default>(items: &mut Vec<T>, len: usize) {
    items.truncate(len);
    items.resize_with(len, T::default);
}

/// Selects one declaration-provenance table so the three span setters share
/// their validate-bounds-fit-assign skeleton.
#[derive(Copy, Clone)]
enum DeclarationTable {
    GlobalVariables,
    PlayerVariables,
    Subroutines,
}

impl DeclarationTable {
    fn count(self, program: &Program) -> usize {
        match self {
            Self::GlobalVariables => program.global_variables.len(),
            Self::PlayerVariables => program.player_variables.len(),
            Self::Subroutines => program.subroutines.len(),
        }
    }

    fn invalid(self, index: usize) -> SourceMappingError {
        match self {
            Self::GlobalVariables => SourceMappingError::InvalidGlobalVariable(index),
            Self::PlayerVariables => SourceMappingError::InvalidPlayerVariable(index),
            Self::Subroutines => SourceMappingError::InvalidSubroutine(index),
        }
    }

    fn identity(self, program: &Program, index: usize) -> NodeIdentity {
        match self {
            Self::GlobalVariables => declaration_identity(&program.global_variables[index]),
            Self::PlayerVariables => declaration_identity(&program.player_variables[index]),
            Self::Subroutines => declaration_identity(&program.subroutines[index]),
        }
    }

    fn slots(self, provenance: &mut ProgramProvenance) -> &mut Vec<DeclarationProvenance> {
        match self {
            Self::GlobalVariables => &mut provenance.global_variables,
            Self::PlayerVariables => &mut provenance.player_variables,
            Self::Subroutines => &mut provenance.subroutines,
        }
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

/// The direct value arguments of a public action, in their mapped order.
/// Provenance rows pair these positionally with `wir::Action::value_args`.
pub(crate) fn action_argument_values(action: &Action) -> Vec<&Value> {
    match action {
        Action::SetGlobalVariable { value, .. } | Action::ModifyGlobalVariable { value, .. } => {
            vec![value]
        }
        Action::SetPlayerVariable { player, value, .. }
        | Action::ModifyPlayerVariable { player, value, .. } => vec![player, value],
        Action::AssignMember { target, value, .. } => vec![target, value],
        Action::If { condition } | Action::ElseIf { condition } | Action::While { condition } => {
            vec![condition]
        }
        Action::ForGlobalVariable {
            start, stop, step, ..
        } => vec![start, stop, step],
        Action::ForPlayerVariable {
            player,
            start,
            stop,
            step,
            ..
        } => vec![player, start, stop, step],
        Action::Call { args, .. } => args.iter().collect(),
        Action::CallSubroutine { .. } | Action::Else | Action::End => Vec::new(),
        Action::Disabled { action } => action_argument_values(action),
    }
}

/// The children a public value exposes to provenance addressing, in the order
/// [`Program::condition_value_span`] documents.
pub(crate) fn value_children(value: &Value) -> Vec<&Value> {
    match value {
        Value::Array(values) => values.iter().collect(),
        Value::Vector { x, y, z } => vec![x.as_ref(), y.as_ref(), z.as_ref()],
        Value::PlayerVariable { player, .. } => vec![player.as_ref()],
        Value::Call { args, .. } => args.iter().collect(),
        _ => Vec::new(),
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
