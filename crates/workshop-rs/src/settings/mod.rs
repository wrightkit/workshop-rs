//! The neutral settings carrier.
//!
//! A typed, non-serde tree for custom-game-settings blocks shared by
//! validation and emission. The tree is a carrier: settings are carried and
//! emitted, never interpreted by lowering/analysis. The fixture-evidenced
//! emission table lives in [`table`].
//!
//! Extracted from the Wright-authored `wright-ir` crate; see
//! [`docs/provenance.md`](https://github.com/wrightkit/workshop-rs/blob/main/docs/provenance.md).

pub mod schema;
pub mod table;

pub use schema::{
    Applicability, EffectiveNumber, NumericBounds, NumericBoundsError, SettingDefinition,
    SettingEvidenceKind, SettingId, SettingIdentity, SettingOccurrence, SettingOperationError,
    SettingPresentation, SettingProvenance, SettingScope, SettingTarget, SettingTargetKind,
    SettingValue, SettingValueDomain, TeamId, definition, definitions, definitions_by_id,
};

use crate::source::Span;

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
}

/// One element of a settings list (corpus lists are all strings).
#[derive(Debug, Clone)]
pub struct SettingsListElement {
    pub value: String,
    pub span: Option<Span>,
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
            | SettingsNode::Raw { span, .. } => *span,
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
            | SettingsNode::Raw { name, .. } => name,
        }
    }
}
