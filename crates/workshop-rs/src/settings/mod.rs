//! The neutral settings carrier.
//!
//! A typed, non-serde tree for custom-game-settings blocks shared by
//! validation and emission. The tree is a carrier: settings are carried and
//! emitted, never interpreted by lowering/analysis. Source-backed path and
//! display data remain implementation details behind this module's APIs.
//!
//! Extracted from the Wright-authored `wright-ir` crate; see
//! [`docs/provenance.md`](https://github.com/wrightkit/workshop-rs/blob/main/docs/provenance.md).

pub(crate) mod check;
pub(crate) mod emitter;
mod member;
pub(crate) mod parser;

pub(crate) mod reconciliation;
pub(crate) mod schema;
pub(crate) mod table;

/// A segment of a path accepted by settings schema lookups.
#[derive(Debug, Clone, Copy, Hash)]
pub enum PathPart<'a> {
    /// A literal key (mode names under `gamemodes` are literal keys too:
    /// per-key subsets are exact-path entries).
    Part(&'a str),
    /// Any team slot (allTeams).
    Team,
    /// Any hero-config slot.
    Hero,
}

impl<'b> PartialEq<PathPart<'b>> for PathPart<'_> {
    fn eq(&self, other: &PathPart<'b>) -> bool {
        match (self, other) {
            (PathPart::Part(left), PathPart::Part(right)) => left == right,
            (PathPart::Team, PathPart::Team) => true,
            (PathPart::Hero, PathPart::Hero) => true,
            _ => false,
        }
    }
}

impl Eq for PathPart<'_> {}

pub use check::{
    DiagnosticSeverity, SettingsDiagnostic, check_emission, check_emission_diagnostics,
};
pub use schema::{
    Applicability, EffectiveNumber, NumericBounds, NumericBoundsError, SettingDefinition,
    SettingEnumMember, SettingId, SettingIdentity, SettingOccurrence, SettingOperationError,
    SettingPresentation, SettingScope, SettingSource, SettingSourceEdit, SettingSourceKind,
    SettingTarget, SettingTargetKind, SettingValue, SettingValueDomain, TeamId, definition,
    definitions, definitions_by_id, validate_catalog,
};

use crate::core::source::Span;

/// A settings block: `settings { ... }` with its typed children.
#[derive(Debug, Clone)]
pub struct Settings {
    pub span: Option<Span>,
    pub children: Vec<SettingsNode>,
}

/// One member of a settings group.
#[derive(Debug, Clone)]
pub enum SettingsNode {
    /// User-authored mode data under `settings.workshop`.
    Workshop {
        children: Vec<SettingsNode>,
        span: Option<Span>,
    },
    Group {
        name: String,
        children: Vec<SettingsNode>,
        span: Option<Span>,
    },
    Number {
        name: String,
        value: f64,
        span: Option<Span>,
    },
    Bool {
        name: String,
        value: bool,
        span: Option<Span>,
    },
    /// A presence-only Workshop extension setting (for example `Beam Effects`).
    Flag { name: String, span: Option<Span> },
    String {
        name: String,
        value: String,
        span: Option<Span>,
    },
    List {
        name: String,
        elements: Vec<SettingsListElement>,
        span: Option<Span>,
    },
    /// A syntactically valid settings member whose semantic catalog entry is
    /// not yet declared. The raw value is carried explicitly so parsing does
    /// not fabricate a type or silently discard project settings.
    Raw {
        name: String,
        value: String,
        span: Option<Span>,
    },
    /// A member written as `name: value` with the value as authored and never
    /// interpreted. At a catalogued path `name` is the canonical key and is
    /// emitted in the target locale; elsewhere it is written as is. Carries
    /// catalogued keys with values the catalog does not declare, and members
    /// whose empty value still needs its `:`.
    RawValue {
        name: String,
        value: String,
        span: Option<Span>,
    },
}

/// One element of a settings list (corpus lists are all strings).
#[derive(Debug, Clone)]
pub struct SettingsListElement {
    pub value: String,
    pub span: Option<Span>,
}

impl SettingsListElement {
    fn clear_span(&mut self) {
        self.span = None;
    }
}

impl Settings {
    /// Drop every span recorded on the block and its members.
    ///
    /// Called when the program's file table is rebound to a different
    /// coordinate space (`SourceMap::apply`): the map carries no settings
    /// entries, so the spans parsed against the previous text cannot be
    /// re-anchored and must not survive as stale positions.
    pub(crate) fn clear_spans(&mut self) {
        self.span = None;
        for node in &mut self.children {
            node.clear_spans();
        }
    }
}

impl SettingsNode {
    /// The source span of this node, if any.
    pub fn span(&self) -> Option<Span> {
        match self {
            SettingsNode::Workshop { span, .. }
            | SettingsNode::Group { span, .. }
            | SettingsNode::Number { span, .. }
            | SettingsNode::Bool { span, .. }
            | SettingsNode::Flag { span, .. }
            | SettingsNode::String { span, .. }
            | SettingsNode::List { span, .. }
            | SettingsNode::Raw { span, .. }
            | SettingsNode::RawValue { span, .. } => *span,
        }
    }

    /// Drop this node's span and, for containers, every descendant span.
    fn clear_spans(&mut self) {
        match self {
            SettingsNode::Workshop { children, span }
            | SettingsNode::Group { children, span, .. } => {
                *span = None;
                for node in children {
                    node.clear_spans();
                }
            }
            SettingsNode::Number { span, .. }
            | SettingsNode::Bool { span, .. }
            | SettingsNode::Flag { span, .. }
            | SettingsNode::String { span, .. }
            | SettingsNode::Raw { span, .. }
            | SettingsNode::RawValue { span, .. } => *span = None,
            SettingsNode::List { elements, span, .. } => {
                *span = None;
                for element in elements {
                    element.clear_span();
                }
            }
        }
    }

    /// The key name of this node.
    pub fn name(&self) -> &str {
        match self {
            SettingsNode::Workshop { .. } => "workshop",
            SettingsNode::Group { name, .. }
            | SettingsNode::Number { name, .. }
            | SettingsNode::Bool { name, .. }
            | SettingsNode::Flag { name, .. }
            | SettingsNode::String { name, .. }
            | SettingsNode::List { name, .. }
            | SettingsNode::Raw { name, .. }
            | SettingsNode::RawValue { name, .. } => name,
        }
    }
}
