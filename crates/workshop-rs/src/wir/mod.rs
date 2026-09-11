//! The Workshop IR model.
//!
//! Workshop IR models the lower-level workshop program structure: variables
//! with indexes, subroutines with indexes, and rules with events, conditions,
//! actions, and values. It is locale-independent (canonical catalog ids only,
//! never localized spellings) and protocol-agnostic.
//!
//! Name policy: call/value `name` fields keep the canonical catalog ids
//! (`countOf`, `wait`, `createBeamEffect`); mapping those to localized
//! Workshop presentation spellings is an emission concern.
//!
//! Extracted from the Wright-authored `wright-ir` crate (the `wir`,
//! `settings`, and `source` modules); see
//! [`docs/provenance.md`](https://github.com/wrightkit/workshop-rs/blob/main/docs/provenance.md).

mod action;
mod dump;
mod event;
mod rule;
mod validate;
mod value;

pub mod error;

pub use action::{Action, IfBranch, ModifyOp};
pub use event::{Event, EventTarget, EventTeam, PlayerEventKind};
pub use rule::{Rule, WorkshopSubroutine, WorkshopVariable};
pub use value::{Value, ValueNode};

/// The WIR-owned capability surface used by the canonical census. Providers
/// do not contribute source-language inventories to this registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CensusCapabilityKind {
    Variable,
    PlayerVariable,
    Subroutine,
    ControlFlow,
    String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CensusCapability {
    pub kind: CensusCapabilityKind,
    pub name: &'static str,
}

pub const CENSUS_CAPABILITIES: &[CensusCapability] = &[
    CensusCapability {
        kind: CensusCapabilityKind::Variable,
        name: "global",
    },
    CensusCapability {
        kind: CensusCapabilityKind::PlayerVariable,
        name: "player",
    },
    CensusCapability {
        kind: CensusCapabilityKind::Subroutine,
        name: "declaration-and-call",
    },
    CensusCapability {
        kind: CensusCapabilityKind::ControlFlow,
        name: "if",
    },
    CensusCapability {
        kind: CensusCapabilityKind::ControlFlow,
        name: "else-if",
    },
    CensusCapability {
        kind: CensusCapabilityKind::ControlFlow,
        name: "else",
    },
    CensusCapability {
        kind: CensusCapabilityKind::ControlFlow,
        name: "while",
    },
    CensusCapability {
        kind: CensusCapabilityKind::ControlFlow,
        name: "for-global-variable",
    },
    CensusCapability {
        kind: CensusCapabilityKind::String,
        name: "custom-string",
    },
];

use crate::core::arena::Arena;
use crate::core::ids::Id;
use crate::core::source::SourceFile;

/// A typed ID referencing a [`WorkshopVariable`] in the global table.
pub type GlobalVarId = Id<WorkshopVariable>;
/// A typed ID referencing a [`WorkshopVariable`] in the player table.
pub type PlayerVarId = Id<WorkshopVariable>;
/// A typed ID referencing a [`WorkshopSubroutine`].
pub type SubroutineId = Id<WorkshopSubroutine>;
/// A typed ID referencing a [`Rule`].
pub type RuleId = Id<Rule>;
/// A typed ID referencing an [`Action`] in the action arena.
pub type ActionId = Id<Action>;
/// A typed ID referencing a [`ValueNode`] in the value arena.
pub type ValueId = Id<ValueNode>;

/// The Workshop IR program: tables and arenas produced by lowering.
#[derive(Debug, Clone)]
pub struct Program {
    /// The source-file registry, copied from the source HIR so spans remain
    /// resolvable for diagnostics.
    pub files: Arena<SourceFile>,
    /// The custom-game-settings carrier, copied inertly from the source HIR
    /// (emitted verbatim, never lowered, #86).
    pub settings: Option<crate::settings::Settings>,
    pub global_variables: Arena<WorkshopVariable>,
    pub player_variables: Arena<WorkshopVariable>,
    pub subroutines: Arena<WorkshopSubroutine>,
    pub rules: Arena<Rule>,
    pub values: Arena<ValueNode>,
    pub actions: Arena<Action>,
}

impl Default for Program {
    fn default() -> Self {
        Program {
            files: Arena::new(),
            settings: None,
            global_variables: Arena::new(),
            player_variables: Arena::new(),
            subroutines: Arena::new(),
            rules: Arena::new(),
            values: Arena::new(),
            actions: Arena::new(),
        }
    }
}

impl Program {
    /// Add a source file and bind its optional source metadata to the returned
    /// file ID.
    pub fn add_file(&mut self, file: SourceFile) -> crate::source::FileId {
        let id = self.files.push(file);
        self.files.get_mut(id).unwrap().bind_file(id);
        id
    }

    /// Return retained authored source for a registered file, when available.
    pub fn source(&self, file: crate::source::FileId) -> Option<&crate::source::SourceDocument> {
        self.files.get(file).and_then(SourceFile::source)
    }

    /// Validate structural invariants: every ID resolves and every span is
    /// valid. Returns the first violation as a structured [`IrError`].
    ///
    /// [`IrError`]: crate::wir::error::IrError
    pub fn validate(&self) -> Result<(), error::IrError> {
        validate::validate(self)
    }

    /// Report preserved or unknown constructs separately from structural
    /// validation so consumers cannot present analysis as definitive.
    pub fn semantic_issues(
        &self,
        catalog: &crate::catalog::Catalog,
    ) -> Vec<crate::analysis::semantic::SemanticIssue> {
        crate::analysis::semantic::inspect_wir(self, catalog)
    }

    /// Render a deterministic debug dump of the workshop program.
    pub fn dump(&self) -> String {
        dump::dump(self)
    }
}
