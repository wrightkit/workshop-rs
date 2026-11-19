//! The `workshop-rs/mapped-text-v1` wire format: [`MappedText`] encoding
//! and decoding, and the serialized shapes a [`SourceMap`] carries.

use serde::{Deserialize, Serialize};

use super::super::identity::value_identity;
use super::super::{
    DeclarationProvenance, Program, ProgramProvenance, Value, ValueProvenance, value_children,
};
use super::{MAPPED_TEXT_V1, MappedText, SourceMap, SourceMapError};
use crate::source::{FileId, Position, Span};

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
        let artifact = match serde_json::from_str::<Artifact>(json) {
            Ok(artifact) if artifact.format == MAPPED_TEXT_V1 => artifact,
            Ok(artifact) => return Err(SourceMapError::UnsupportedFormat(artifact.format)),
            Err(error) => {
                return Err(
                    match serde_json::from_str::<FormatProbe>(json).map(|probe| probe.format) {
                        Ok(Some(format)) if format != MAPPED_TEXT_V1 => {
                            SourceMapError::UnsupportedFormat(format)
                        }
                        Ok(None) => SourceMapError::Malformed("missing format".to_string()),
                        _ => SourceMapError::Malformed(error.to_string()),
                    },
                );
            }
        };
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

pub(super) fn wire_children(children: &[ValueProvenance], values: &[&Value]) -> Vec<WireValue> {
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

pub(super) fn wire_value_unmapped(value: &WireValue) -> bool {
    value.span.is_none()
        && value.identifier_span.is_none()
        && value.children.iter().all(wire_value_unmapped)
}

pub(super) fn push_declarations<T: std::fmt::Debug>(
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

/// The `format` field alone, read to classify a document that failed to decode.
#[derive(Deserialize)]
struct FormatProbe {
    format: Option<String>,
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
pub(super) struct Shape {
    pub(super) global_variables: usize,
    pub(super) player_variables: usize,
    pub(super) subroutines: usize,
    pub(super) rules: Vec<RuleShape>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct RuleShape {
    pub(super) conditions: usize,
    pub(super) actions: usize,
}

impl Shape {
    pub(super) fn of(program: &Program) -> Self {
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

    pub(super) fn check(&self, program: &Program) -> Result<(), SourceMapError> {
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
pub(super) enum MappedNode {
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

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct WireValue {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) span: Option<WireSpan>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) identifier_span: Option<WireSpan>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(super) children: Vec<WireValue>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct WireSpan {
    pub(super) file: usize,
    pub(super) start: WirePosition,
    pub(super) end: WirePosition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct WirePosition {
    pub(super) line: u32,
    pub(super) column: u32,
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
