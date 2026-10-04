//! Canonical Workshop artifact formats and the public [`SourceMap`].

use serde::{Deserialize, Serialize};

use super::identity::{condition_identity, value_identity};
use super::{
    DeclarationProvenance, Program, ProgramProvenance, Value, ValueProvenance,
    action_argument_values, fit, value_children,
};
use crate::source::{FileId, Position, SourceFile, Span};

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
            if let Some(provenance) = program.rule_provenance(rule) {
                if provenance.span.is_some()
                    || provenance.name.is_some()
                    || provenance.event_name.is_some()
                {
                    spans.push(MappedNode::Rule {
                        rule,
                        span: provenance.span.map(WireSpan::from),
                        name_span: provenance.name.map(WireSpan::from),
                        event_name_span: provenance.event_name.map(WireSpan::from),
                    });
                }
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
                if let Some(provenance) = program.action_provenance(rule, action) {
                    if provenance.span.is_some() || provenance.identifier.is_some() {
                        spans.push(MappedNode::Action {
                            rule,
                            action,
                            span: provenance.span.map(WireSpan::from),
                            identifier_span: provenance.identifier.map(WireSpan::from),
                        });
                    }
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
                } => {
                    let declaration = provenance
                        .global_variables
                        .get_mut(*index)
                        .ok_or(SourceMapError::InvalidPosition)?;
                    let mapped = self.declaration(*span, *name_span)?;
                    if declaration.span.is_some() || declaration.name_span.is_some() {
                        return Err(SourceMapError::DuplicateEntry);
                    }
                    *declaration = mapped;
                }
                MappedNode::PlayerVariable {
                    index,
                    span,
                    name_span,
                } => {
                    let declaration = provenance
                        .player_variables
                        .get_mut(*index)
                        .ok_or(SourceMapError::InvalidPosition)?;
                    let mapped = self.declaration(*span, *name_span)?;
                    if declaration.span.is_some() || declaration.name_span.is_some() {
                        return Err(SourceMapError::DuplicateEntry);
                    }
                    *declaration = mapped;
                }
                MappedNode::Subroutine {
                    index,
                    span,
                    name_span,
                } => {
                    let declaration = provenance
                        .subroutines
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
            program.add_file(SourceFile::new(path.clone()));
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

    /// Map serialized children onto a public value's children by position.
    ///
    /// Emission can canonicalize a value into a form with a different public
    /// arity — `(expr).member` reparse as a variable read drops the member-name
    /// child, for example — so children are matched positionally: entries
    /// beyond the value's children are dropped and positions with no entry are
    /// left unmapped.
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

impl MappedText {
    pub fn new(text: impl Into<String>, map: SourceMap) -> Self {
        Self {
            text: text.into(),
            map,
        }
    }

    /// Serialize as a `workshop-rs/mapped-text-v1` JSON document.
    pub fn to_json(&self) -> String {
        let artifact = Artifact {
            format: MAPPED_TEXT_V1.to_string(),
            text: self.text.clone(),
            files: self
                .map
                .files
                .iter()
                .map(|path| WireFile { path: path.clone() })
                .collect(),
            shape: self.map.shape.clone(),
            spans: self.map.spans.clone(),
        };
        serde_json::to_string(&artifact).expect("mapped text serializes to JSON")
    }

    /// Decode a `workshop-rs/mapped-text-v1` JSON document.
    ///
    /// Decoding checks the format and structure only; [`SourceMap::apply`]
    /// validates the mapping against the program it is applied to.
    pub fn from_json(json: &str) -> Result<Self, SourceMapError> {
        let value: serde_json::Value = serde_json::from_str(json)
            .map_err(|error| SourceMapError::Malformed(error.to_string()))?;
        match value.get("format").and_then(serde_json::Value::as_str) {
            Some(MAPPED_TEXT_V1) => {}
            Some(other) => return Err(SourceMapError::UnsupportedFormat(other.to_string())),
            None => return Err(SourceMapError::Malformed("missing format".to_string())),
        }
        let artifact: Artifact = serde_json::from_value(value)
            .map_err(|error| SourceMapError::Malformed(error.to_string()))?;
        Ok(Self {
            text: artifact.text,
            map: SourceMap {
                files: artifact.files.into_iter().map(|file| file.path).collect(),
                shape: artifact.shape,
                spans: artifact.spans,
            },
        })
    }
}

/// The wire form of recorded value-provenance children, paired with the
/// public children they describe: children keep their position up to the
/// last mapped one, a record whose value no longer has the recorded identity
/// serializes as unmapped, a level whose recorded count no longer matches is
/// dropped entirely, and a level with no recorded provenance at all is
/// omitted.
fn wire_children(children: &[ValueProvenance], values: &[&Value]) -> Vec<WireValue> {
    if children.len() != values.len() {
        return Vec::new();
    }
    let mut wire: Vec<WireValue> = children
        .iter()
        .zip(values.iter().copied())
        .map(|(child, value)| wire_value(child, value))
        .collect();
    let last = wire
        .iter()
        .rposition(|child| !wire_value_unmapped(child))
        .map(|index| index + 1)
        .unwrap_or(0);
    wire.truncate(last);
    wire
}

fn wire_value(provenance: &ValueProvenance, value: &Value) -> WireValue {
    let (span, identifier) = if provenance.identity == value_identity(value) {
        (provenance.span, provenance.identifier)
    } else {
        Default::default()
    };
    WireValue {
        span: span.map(WireSpan::from),
        identifier_span: identifier.map(WireSpan::from),
        children: wire_children(&provenance.children, &value_children(value)),
    }
}

/// Whether a serialized value entry contributes any position.
fn wire_value_unmapped(value: &WireValue) -> bool {
    value.span.is_none()
        && value.identifier_span.is_none()
        && value.children.iter().all(wire_value_unmapped)
}

fn push_declarations<T: std::fmt::Debug>(
    program: &Program,
    nodes: &[T],
    recorded: impl Fn(&ProgramProvenance) -> &[DeclarationProvenance],
    node: impl Fn(usize, Option<WireSpan>, Option<WireSpan>) -> MappedNode,
    output: &mut Vec<MappedNode>,
) {
    for index in 0..nodes.len() {
        let declaration = program.declaration_provenance(&recorded, nodes, index);
        if declaration.span.is_some() || declaration.name_span.is_some() {
            output.push(node(
                index,
                declaration.span.map(WireSpan::from),
                declaration.name_span.map(WireSpan::from),
            ));
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Artifact {
    format: String,
    text: String,
    files: Vec<WireFile>,
    shape: Shape,
    spans: Vec<MappedNode>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct WireFile {
    path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Shape {
    global_variables: usize,
    player_variables: usize,
    subroutines: usize,
    rules: Vec<RuleShape>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct RuleShape {
    conditions: usize,
    actions: usize,
}

impl Shape {
    fn of(program: &Program) -> Self {
        Self {
            global_variables: program.global_variables.len(),
            player_variables: program.player_variables.len(),
            subroutines: program.subroutines.len(),
            rules: program
                .rules
                .iter()
                .map(|rule| RuleShape {
                    conditions: rule.conditions.len(),
                    actions: rule.actions.len(),
                })
                .collect(),
        }
    }

    fn check(&self, program: &Program) -> Result<(), SourceMapError> {
        let found = Self::of(program);
        if self.global_variables != found.global_variables {
            return Err(SourceMapError::GlobalVariableCount {
                expected: self.global_variables,
                found: found.global_variables,
            });
        }
        if self.player_variables != found.player_variables {
            return Err(SourceMapError::PlayerVariableCount {
                expected: self.player_variables,
                found: found.player_variables,
            });
        }
        if self.subroutines != found.subroutines {
            return Err(SourceMapError::SubroutineCount {
                expected: self.subroutines,
                found: found.subroutines,
            });
        }
        if self.rules.len() != found.rules.len() {
            return Err(SourceMapError::RuleCount {
                expected: self.rules.len(),
                found: found.rules.len(),
            });
        }
        for (rule, (expected, found)) in self.rules.iter().zip(&found.rules).enumerate() {
            if expected.conditions != found.conditions {
                return Err(SourceMapError::ConditionCount {
                    rule,
                    expected: expected.conditions,
                    found: found.conditions,
                });
            }
            if expected.actions != found.actions {
                return Err(SourceMapError::ActionCount {
                    rule,
                    expected: expected.actions,
                    found: found.actions,
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "node", rename_all = "snake_case")]
enum MappedNode {
    Rule {
        rule: usize,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        span: Option<WireSpan>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        name_span: Option<WireSpan>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        event_name_span: Option<WireSpan>,
    },
    Condition {
        rule: usize,
        condition: usize,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        span: Option<WireSpan>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        identifier_span: Option<WireSpan>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        children: Vec<WireValue>,
    },
    Action {
        rule: usize,
        action: usize,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        span: Option<WireSpan>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        identifier_span: Option<WireSpan>,
    },
    ActionArgument {
        rule: usize,
        action: usize,
        argument: usize,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        span: Option<WireSpan>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        identifier_span: Option<WireSpan>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        children: Vec<WireValue>,
    },
    GlobalVariable {
        index: usize,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        span: Option<WireSpan>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        name_span: Option<WireSpan>,
    },
    PlayerVariable {
        index: usize,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        span: Option<WireSpan>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        name_span: Option<WireSpan>,
    },
    Subroutine {
        index: usize,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        span: Option<WireSpan>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        name_span: Option<WireSpan>,
    },
}

/// The mapped provenance of one value node, mirroring the structure of the
/// public [`Value`] tree: `children` addresses child values by position.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
struct WireValue {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    span: Option<WireSpan>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    identifier_span: Option<WireSpan>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    children: Vec<WireValue>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
struct WireSpan {
    file: usize,
    start: WirePosition,
    end: WirePosition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
struct WirePosition {
    line: u32,
    column: u32,
}

impl From<Span> for WireSpan {
    fn from(span: Span) -> Self {
        Self {
            file: span.file.index(),
            start: span.start.into(),
            end: span.end.into(),
        }
    }
}

impl From<WireSpan> for Span {
    fn from(wire: WireSpan) -> Self {
        Span::new(
            FileId::from_index(wire.file),
            wire.start.into(),
            wire.end.into(),
        )
    }
}

impl From<Position> for WirePosition {
    fn from(position: Position) -> Self {
        Self {
            line: position.line,
            column: position.col,
        }
    }
}

impl From<WirePosition> for Position {
    fn from(position: WirePosition) -> Self {
        Position::new(position.line, position.column)
    }
}
