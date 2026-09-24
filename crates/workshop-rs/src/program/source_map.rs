//! Canonical Workshop artifact formats and the public [`SourceMap`].

use serde::{Deserialize, Serialize};

use super::{DeclarationProvenance, Program, ProgramProvenance, action_argument_count, fit};
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
/// and variable and subroutine declarations. Nodes without an authored origin
/// have no entry, so consumers report evidence on them as unmapped.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceMap {
    files: Vec<String>,
    shape: Shape,
    spans: Vec<MappedNode>,
}

/// Workshop text together with the [`SourceMap`] of its authored origin: the
/// `workshop-rs/mapped-text-v1` artifact.
#[derive(Debug, Clone, PartialEq, Eq)]
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
    /// Only spans that are still valid for the program's current shape are
    /// extracted; see [`Program::rule_span`].
    pub fn extract(program: &Program) -> Self {
        let mut spans = Vec::new();
        push_declarations(
            program,
            program.global_variables.len(),
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
            program.player_variables.len(),
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
            program.subroutines.len(),
            |provenance| &provenance.subroutines,
            |index, span, name_span| MappedNode::Subroutine {
                index,
                span,
                name_span,
            },
            &mut spans,
        );
        for (rule, public) in program.rules.iter().enumerate() {
            if let Some(span) = program.rule_span(rule) {
                spans.push(MappedNode::Rule {
                    rule,
                    span: span.into(),
                });
            }
            for condition in 0..public.conditions.len() {
                if let Some(span) = program.condition_span(rule, condition) {
                    spans.push(MappedNode::Condition {
                        rule,
                        condition,
                        span: span.into(),
                    });
                }
            }
            for (action, public_action) in public.actions.iter().enumerate() {
                if let Some(span) = program.action_span(rule, action) {
                    spans.push(MappedNode::Action {
                        rule,
                        action,
                        span: span.into(),
                    });
                }
                for argument in 0..action_argument_count(public_action) {
                    if let Some(span) = program.action_argument_span(rule, action, argument) {
                        spans.push(MappedNode::ActionArgument {
                            rule,
                            action,
                            argument,
                            span: span.into(),
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
                    *declaration = self.declaration(*span, *name_span)?;
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
                    *declaration = self.declaration(*span, *name_span)?;
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
                    *declaration = self.declaration(*span, *name_span)?;
                }
                MappedNode::Rule { rule, span } => {
                    let span = self.span(*span)?;
                    provenance
                        .rules
                        .get_mut(*rule)
                        .ok_or(SourceMapError::InvalidPosition)?
                        .span = Some(span);
                }
                MappedNode::Condition {
                    rule,
                    condition,
                    span,
                } => {
                    let span = self.span(*span)?;
                    *provenance
                        .rules
                        .get_mut(*rule)
                        .and_then(|rule| rule.conditions.get_mut(*condition))
                        .ok_or(SourceMapError::InvalidPosition)? = Some(span);
                }
                MappedNode::Action { rule, action, span } => {
                    let span = self.span(*span)?;
                    provenance
                        .rules
                        .get_mut(*rule)
                        .and_then(|rule| rule.actions.get_mut(*action))
                        .ok_or(SourceMapError::InvalidPosition)?
                        .span = Some(span);
                }
                MappedNode::ActionArgument {
                    rule,
                    action,
                    argument,
                    span,
                } => {
                    let span = self.span(*span)?;
                    let count = program
                        .rules
                        .get(*rule)
                        .and_then(|rule| rule.actions.get(*action))
                        .map(action_argument_count)
                        .ok_or(SourceMapError::InvalidPosition)?;
                    if *argument >= count {
                        return Err(SourceMapError::InvalidPosition);
                    }
                    let arguments = &mut provenance
                        .rules
                        .get_mut(*rule)
                        .and_then(|rule| rule.actions.get_mut(*action))
                        .ok_or(SourceMapError::InvalidPosition)?
                        .arguments;
                    fit(arguments, count);
                    arguments[*argument] = Some(span);
                }
            }
        }

        program.files.clear();
        for path in &self.files {
            program.add_file(SourceFile::new(path.clone()));
        }
        program.provenance = Some(Box::new(provenance));
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
        Ok(DeclarationProvenance {
            span: span.map(|span| self.span(span)).transpose()?,
            name_span: name_span.map(|span| self.span(span)).transpose()?,
        })
    }
}

impl MappedText {
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

fn push_declarations(
    program: &Program,
    count: usize,
    recorded: impl Fn(&ProgramProvenance) -> &[DeclarationProvenance],
    node: impl Fn(usize, Option<WireSpan>, Option<WireSpan>) -> MappedNode,
    output: &mut Vec<MappedNode>,
) {
    for index in 0..count {
        let declaration = program.declaration_provenance(&recorded, count, index);
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
        span: WireSpan,
    },
    Condition {
        rule: usize,
        condition: usize,
        span: WireSpan,
    },
    Action {
        rule: usize,
        action: usize,
        span: WireSpan,
    },
    ActionArgument {
        rule: usize,
        action: usize,
        argument: usize,
        span: WireSpan,
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
