use super::identity::{condition_identity, value_identity};
use super::{
    DeclarationProvenance, Program, ProgramProvenance, Value, ValueProvenance,
    action_argument_values, fit, value_children,
};
use crate::source::{SourceFile, Span};

mod wire;

use wire::{
    MappedNode, Shape, WireSpan, WireValue, push_declarations, wire_children, wire_value_unmapped,
};

/// Identifier of the canonical Workshop text artifact: the Workshop text alone.
pub const TEXT_V1: &str = "workshop-rs/text-v1";

/// Identifier of the canonical mapped Workshop artifact: Workshop text plus a
/// [`SourceMap`], serialized by [`MappedText::to_json`].
pub const MAPPED_TEXT_V1: &str = "workshop-rs/mapped-text-v1";

/// A source mapping detached from a [`Program`].
///
/// A source map records the file table, the program shape, and the
/// position-keyed spans of a span-bearing program. Extract it with
/// [`SourceMap::extract`], carry it beside the emitted Workshop text, and
/// [`apply`](Self::apply) it to a program parsed from that text. Spans use
/// [`Position`] units: 1-based lines and columns counted in Unicode scalar
/// values.
///
/// The mapping granularity is rule, condition, action, direct action argument,
/// and variable and subroutine declarations. Entries may additionally carry the
/// identifier span the node names — a rule's name, a rule's subroutine event
/// binding, an action's target or callee, a value's variable or subroutine
/// identifier — and condition and action-argument entries carry the provenance
/// of the value's children, keyed by position in the public [`Value`] tree.
/// Every field of an entry is optional: a node with only finer-grained
/// provenance, such as a member read that records just its identifier,
/// appears as an entry without `span`. Nodes without an authored origin have
/// no entry, so consumers report evidence on them as unmapped.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceMap {
    files: Vec<String>,
    shape: Shape,
    spans: Vec<MappedNode>,
}

/// Workshop text together with the [`SourceMap`] of its authored origin: the
/// `workshop-rs/mapped-text-v1` artifact.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct MappedText {
    /// The Workshop text, itself a `workshop-rs/text-v1` artifact.
    pub text: String,
    /// The mapping from the program parsed from [`text`](Self::text) to the
    /// authored source.
    pub map: SourceMap,
}

/// A failure while decoding or applying a [`SourceMap`].
///
/// A failed [`SourceMap::apply`] leaves the program unchanged.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceMapError {
    GlobalVariableCount {
        expected: usize,
        found: usize,
    },
    PlayerVariableCount {
        expected: usize,
        found: usize,
    },
    SubroutineCount {
        expected: usize,
        found: usize,
    },
    RuleCount {
        expected: usize,
        found: usize,
    },
    ConditionCount {
        rule: usize,
        expected: usize,
        found: usize,
    },
    ActionCount {
        rule: usize,
        expected: usize,
        found: usize,
    },
    /// An entry addresses a node outside the program shape.
    InvalidPosition,
    /// Two entries map the same node.
    DuplicateEntry,
    /// An entry carries no position at all.
    EmptyEntry,
    /// A span references a file outside the file table.
    UnknownFile(usize),
    /// A span is not a valid 1-based interval.
    InvalidSpan(Span),
    /// The artifact declares a format other than `workshop-rs/mapped-text-v1`.
    UnsupportedFormat(String),
    /// The artifact is not well-formed JSON of the expected structure.
    Malformed(String),
}

impl std::fmt::Display for SourceMapError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mismatch = |formatter: &mut std::fmt::Formatter<'_>, what, expected, found| {
            write!(
                formatter,
                "source map shape mismatch: expected {expected} {what}, found {found}"
            )
        };
        match self {
            Self::GlobalVariableCount { expected, found } => {
                mismatch(formatter, "global variables", expected, found)
            }
            Self::PlayerVariableCount { expected, found } => {
                mismatch(formatter, "player variables", expected, found)
            }
            Self::SubroutineCount { expected, found } => {
                mismatch(formatter, "subroutines", expected, found)
            }
            Self::RuleCount { expected, found } => mismatch(formatter, "rules", expected, found),
            Self::ConditionCount {
                rule,
                expected,
                found,
            } => mismatch(
                formatter,
                &format!("conditions in rule {rule}"),
                expected,
                found,
            ),
            Self::ActionCount {
                rule,
                expected,
                found,
            } => mismatch(
                formatter,
                &format!("actions in rule {rule}"),
                expected,
                found,
            ),
            Self::InvalidPosition => {
                write!(formatter, "source map entry is outside the program shape")
            }
            Self::DuplicateEntry => write!(formatter, "source map maps a node twice"),
            Self::EmptyEntry => write!(formatter, "source map entry has no position"),
            Self::UnknownFile(file) => {
                write!(formatter, "source map span references unknown file {file}")
            }
            Self::InvalidSpan(span) => write!(formatter, "invalid source map span {span:?}"),
            Self::UnsupportedFormat(format) => {
                write!(formatter, "unsupported mapped text format {format:?}")
            }
            Self::Malformed(message) => write!(formatter, "malformed mapped text: {message}"),
        }
    }
}

impl std::error::Error for SourceMapError {}

impl SourceMap {
    /// Extract the current mapping of a span-bearing program.
    ///
    /// Only spans that are still valid for the program's current shape and
    /// node content are extracted; see [`Program::rule_span`].
    pub fn extract(program: &Program) -> Self {
        let mut spans = Vec::new();
        push_declarations(
            program,
            &program.global_variables,
            |provenance| &provenance.global_variables,
            |index, span, name_span| MappedNode::GlobalVariable {
                index,
                span,
                name_span,
            },
            &mut spans,
        );
        push_declarations(
            program,
            &program.player_variables,
            |provenance| &provenance.player_variables,
            |index, span, name_span| MappedNode::PlayerVariable {
                index,
                span,
                name_span,
            },
            &mut spans,
        );
        push_declarations(
            program,
            &program.subroutines,
            |provenance| &provenance.subroutines,
            |index, span, name_span| MappedNode::Subroutine {
                index,
                span,
                name_span,
            },
            &mut spans,
        );
        for (rule, public) in program.rules.iter().enumerate() {
            if let Some(provenance) = program.rule_provenance(rule)
                && (provenance.span.is_some()
                    || provenance.name.is_some()
                    || provenance.event_name.is_some())
            {
                spans.push(MappedNode::Rule {
                    rule,
                    span: provenance.span.map(WireSpan::from),
                    name_span: provenance.name.map(WireSpan::from),
                    event_name_span: provenance.event_name.map(WireSpan::from),
                });
            }
            for (condition, public_condition) in public.conditions.iter().enumerate() {
                let Some(record) = program.condition_record(rule, condition) else {
                    continue;
                };
                let children =
                    wire_children(&record.children, &value_children(&public_condition.value));
                let (span, identifier) = if record.identity == condition_identity(public_condition)
                {
                    (record.span, record.identifier)
                } else {
                    Default::default()
                };
                if span.is_some() || identifier.is_some() || !children.is_empty() {
                    spans.push(MappedNode::Condition {
                        rule,
                        condition,
                        span: span.map(WireSpan::from),
                        identifier_span: identifier.map(WireSpan::from),
                        children,
                    });
                }
            }
            for (action, public_action) in public.actions.iter().enumerate() {
                if let Some(provenance) = program.action_provenance(rule, action)
                    && (provenance.span.is_some() || provenance.identifier.is_some())
                {
                    spans.push(MappedNode::Action {
                        rule,
                        action,
                        span: provenance.span.map(WireSpan::from),
                        identifier_span: provenance.identifier.map(WireSpan::from),
                    });
                }
                for (argument, value) in action_argument_values(public_action)
                    .iter()
                    .copied()
                    .enumerate()
                {
                    let Some(record) = program.argument_record(rule, action, argument) else {
                        continue;
                    };
                    let children = wire_children(&record.children, &value_children(value));
                    let (span, identifier) = if record.identity == value_identity(value) {
                        (record.span, record.identifier)
                    } else {
                        Default::default()
                    };
                    if span.is_some() || identifier.is_some() || !children.is_empty() {
                        spans.push(MappedNode::ActionArgument {
                            rule,
                            action,
                            argument,
                            span: span.map(WireSpan::from),
                            identifier_span: identifier.map(WireSpan::from),
                            children,
                        });
                    }
                }
            }
        }
        Self {
            files: program.files.iter().map(|file| file.path.clone()).collect(),
            shape: Shape::of(program),
            spans,
        }
    }

    /// The paths of the file table that mapped spans refer to by file index.
    pub fn files(&self) -> &[String] {
        &self.files
    }

    /// Replace the source mapping of `program` with this map.
    ///
    /// The program's file table becomes this map's file table, and nodes
    /// without an entry carry no span. The program must have exactly the shape
    /// the map was extracted from; otherwise the whole mapping is rejected and
    /// `program` is unchanged.
    pub fn apply(&self, program: &mut Program) -> Result<(), SourceMapError> {
        self.shape.check(program)?;

        let mut provenance = ProgramProvenance::default();
        fit(
            &mut provenance.global_variables,
            self.shape.global_variables,
        );
        fit(
            &mut provenance.player_variables,
            self.shape.player_variables,
        );
        fit(&mut provenance.subroutines, self.shape.subroutines);
        fit(&mut provenance.rules, self.shape.rules.len());
        for (rule, shape) in provenance.rules.iter_mut().zip(&self.shape.rules) {
            fit(&mut rule.conditions, shape.conditions);
            fit(&mut rule.actions, shape.actions);
        }

        for node in &self.spans {
            match node {
                MappedNode::GlobalVariable {
                    index,
                    span,
                    name_span,
                }
                | MappedNode::PlayerVariable {
                    index,
                    span,
                    name_span,
                }
                | MappedNode::Subroutine {
                    index,
                    span,
                    name_span,
                } => {
                    let declarations = match node {
                        MappedNode::GlobalVariable { .. } => &mut provenance.global_variables,
                        MappedNode::PlayerVariable { .. } => &mut provenance.player_variables,
                        MappedNode::Subroutine { .. } => &mut provenance.subroutines,
                        _ => unreachable!(),
                    };
                    let declaration = declarations
                        .get_mut(*index)
                        .ok_or(SourceMapError::InvalidPosition)?;
                    let mapped = self.declaration(*span, *name_span)?;
                    if declaration.span.is_some() || declaration.name_span.is_some() {
                        return Err(SourceMapError::DuplicateEntry);
                    }
                    *declaration = mapped;
                }
                MappedNode::Rule {
                    rule,
                    span,
                    name_span,
                    event_name_span,
                } => {
                    let span = span.map(|span| self.span(span)).transpose()?;
                    let name = name_span.map(|span| self.span(span)).transpose()?;
                    let event_name = event_name_span.map(|span| self.span(span)).transpose()?;
                    let slot = provenance
                        .rules
                        .get_mut(*rule)
                        .ok_or(SourceMapError::InvalidPosition)?;
                    if slot.span.is_some() || slot.name.is_some() || slot.event_name.is_some() {
                        return Err(SourceMapError::DuplicateEntry);
                    }
                    if span.is_none() && name.is_none() && event_name.is_none() {
                        return Err(SourceMapError::EmptyEntry);
                    }
                    slot.span = span;
                    slot.name = name;
                    slot.event_name = event_name;
                }
                MappedNode::Condition {
                    rule,
                    condition,
                    span,
                    identifier_span,
                    children,
                } => {
                    let span = span.map(|span| self.span(span)).transpose()?;
                    let identifier = identifier_span.map(|span| self.span(span)).transpose()?;
                    let has_children = children.iter().any(|child| !wire_value_unmapped(child));
                    let children = self.mapped_children(
                        children,
                        &program
                            .rules
                            .get(*rule)
                            .and_then(|rule| rule.conditions.get(*condition))
                            .ok_or(SourceMapError::InvalidPosition)?
                            .value,
                    )?;
                    let slot = provenance
                        .rules
                        .get_mut(*rule)
                        .and_then(|rule| rule.conditions.get_mut(*condition))
                        .ok_or(SourceMapError::InvalidPosition)?;
                    if slot.span.is_some() || slot.identifier.is_some() || !slot.children.is_empty()
                    {
                        return Err(SourceMapError::DuplicateEntry);
                    }
                    if span.is_none() && identifier.is_none() && !has_children {
                        return Err(SourceMapError::EmptyEntry);
                    }
                    slot.span = span;
                    slot.identifier = identifier;
                    slot.children = children;
                }
                MappedNode::Action {
                    rule,
                    action,
                    span,
                    identifier_span,
                } => {
                    let span = span.map(|span| self.span(span)).transpose()?;
                    let identifier = identifier_span.map(|span| self.span(span)).transpose()?;
                    let slot = provenance
                        .rules
                        .get_mut(*rule)
                        .and_then(|rule| rule.actions.get_mut(*action))
                        .ok_or(SourceMapError::InvalidPosition)?;
                    if slot.span.is_some() || slot.identifier.is_some() {
                        return Err(SourceMapError::DuplicateEntry);
                    }
                    if span.is_none() && identifier.is_none() {
                        return Err(SourceMapError::EmptyEntry);
                    }
                    slot.span = span;
                    slot.identifier = identifier;
                }
                MappedNode::ActionArgument {
                    rule,
                    action,
                    argument,
                    span,
                    identifier_span,
                    children,
                } => {
                    let span = span.map(|span| self.span(span)).transpose()?;
                    let identifier = identifier_span.map(|span| self.span(span)).transpose()?;
                    let argument_values = program
                        .rules
                        .get(*rule)
                        .and_then(|rule| rule.actions.get(*action))
                        .map(action_argument_values)
                        .ok_or(SourceMapError::InvalidPosition)?;
                    let Some(&value) = argument_values.get(*argument) else {
                        return Err(SourceMapError::InvalidPosition);
                    };
                    let count = argument_values.len();
                    let has_children = children.iter().any(|child| !wire_value_unmapped(child));
                    let children = self.mapped_children(children, value)?;
                    let arguments = &mut provenance
                        .rules
                        .get_mut(*rule)
                        .and_then(|rule| rule.actions.get_mut(*action))
                        .ok_or(SourceMapError::InvalidPosition)?
                        .arguments;
                    fit(arguments, count);
                    let slot = &mut arguments[*argument];
                    if slot.span.is_some() || slot.identifier.is_some() || !slot.children.is_empty()
                    {
                        return Err(SourceMapError::DuplicateEntry);
                    }
                    if span.is_none() && identifier.is_none() && !has_children {
                        return Err(SourceMapError::EmptyEntry);
                    }
                    slot.span = span;
                    slot.identifier = identifier;
                    slot.children = children;
                }
            }
        }

        program.files.clear();
        for path in &self.files {
            program.add_file(SourceFile::new(path));
        }
        if let Some(settings) = &mut program.settings {
            settings.drop_unmapped_spans();
        }
        program.provenance = Some(Box::new(provenance));
        program.record_identities();
        Ok(())
    }

    fn span(&self, wire: WireSpan) -> Result<Span, SourceMapError> {
        if wire.file >= self.files.len() {
            return Err(SourceMapError::UnknownFile(wire.file));
        }
        let span = Span::from(wire);
        if !span.is_valid() {
            return Err(SourceMapError::InvalidSpan(span));
        }
        Ok(span)
    }

    fn declaration(
        &self,
        span: Option<WireSpan>,
        name_span: Option<WireSpan>,
    ) -> Result<DeclarationProvenance, SourceMapError> {
        if span.is_none() && name_span.is_none() {
            return Err(SourceMapError::EmptyEntry);
        }
        Ok(DeclarationProvenance {
            span: span.map(|span| self.span(span)).transpose()?,
            name_span: name_span.map(|span| self.span(span)).transpose()?,
            ..DeclarationProvenance::default()
        })
    }

    fn mapped_children(
        &self,
        wire: &[WireValue],
        value: &Value,
    ) -> Result<Vec<ValueProvenance>, SourceMapError> {
        let children = value_children(value);
        let mut mapped = wire
            .iter()
            .zip(children.iter().copied())
            .map(|(wire, value)| self.mapped_value(wire, value))
            .collect::<Result<Vec<_>, _>>()?;
        mapped.resize_with(children.len(), ValueProvenance::default);
        Ok(mapped)
    }

    fn mapped_value(
        &self,
        wire: &WireValue,
        value: &Value,
    ) -> Result<ValueProvenance, SourceMapError> {
        Ok(ValueProvenance {
            span: wire.span.map(|span| self.span(span)).transpose()?,
            identifier: wire
                .identifier_span
                .map(|span| self.span(span))
                .transpose()?,
            children: self.mapped_children(&wire.children, value)?,
            ..ValueProvenance::default()
        })
    }
}
