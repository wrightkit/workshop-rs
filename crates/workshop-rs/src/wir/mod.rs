//! The Workshop IR model.
//!
//! Workshop IR models the lower-level workshop program structure: variables
//! with indexes, subroutines with indexes, and rules with events, conditions,
//! actions, and values. It is locale-independent (canonical catalog ids only,
//! never localized spellings) and protocol-agnostic.
//!
//! Name policy: call/value `name` fields keep the canonical catalog ids
//! (`countOf`, `wait`, `createBeamEffect`); mapping those to localized
//! Workshop presentation spellings is an emission concern. `debug` and
//! `print` are represented as dedicated [`Action::Debug`]/[`Action::Print`]
//! nodes.
//!
//! Extracted from the Wright-authored `wright-ir` crate (the `wir`,
//! `settings`, and `source` modules); see
//! [`docs/provenance.md`](https://github.com/wrightkit/workshop-rs/blob/main/docs/provenance.md).

mod dump;
mod validate;

pub mod error;

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

use crate::arena::Arena;
use crate::ids::Id;
use crate::source::{SourceFile, Span};

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
    /// Validate structural invariants: every ID resolves and every span is
    /// valid. Returns the first violation as a structured [`IrError`].
    ///
    /// [`IrError`]: crate::wir::error::IrError
    pub fn validate(&self) -> Result<(), error::IrError> {
        validate::validate(self)
    }

    /// Render a deterministic debug dump of the workshop program.
    pub fn dump(&self) -> String {
        dump::dump(self)
    }
}

/// A workshop variable (global or player) with its assigned index.
///
/// Declaration initializers are lowered into synthetic "Initialize global
/// variables" / "Initialize player variables" rules during HIR → WIR lowering
/// (#112); the variable tables carry no initializer field, so the Initialize
/// rules are the single source of truth.
#[derive(Debug, Clone)]
pub struct WorkshopVariable {
    pub name: String,
    /// The workshop variable index assigned during lowering.
    pub index: u32,
    pub span: Option<Span>,
    /// The exact span of the declared identifier token.
    pub name_span: Option<Span>,
}

/// A workshop subroutine with its assigned index.
#[derive(Debug, Clone)]
pub struct WorkshopSubroutine {
    pub name: String,
    pub index: u32,
    pub span: Option<Span>,
    /// The exact span of the declared identifier token.
    pub name_span: Option<Span>,
}

/// A workshop rule.
#[derive(Debug, Clone)]
pub struct Rule {
    pub name: String,
    pub span: Option<Span>,
    /// The exact span of the rule name inside its string literal.
    pub name_span: Option<Span>,
    pub disabled: bool,
    pub event: Event,
    pub conditions: Vec<ValueId>,
    pub actions: Vec<ActionId>,
}

/// The team filter attached to a player-scoped Workshop event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventTeam {
    All,
    Team1,
    Team2,
}

/// The player filter attached to a player-scoped Workshop event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventTarget {
    All,
    Slot(u8),
    Hero(String),
}

/// A non-ongoing player event identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerEventKind {
    DealtDamage,
    DealtFinalBlow,
    DealtHealing,
    Died,
    EarnedElimination,
    Joined,
    Left,
    ReceivedHealing,
    TookDamage,
}

impl PlayerEventKind {
    /// The locale-independent catalog identity for this event.
    pub fn catalog_id(self) -> &'static str {
        match self {
            PlayerEventKind::DealtDamage => "playerDealtDamage",
            PlayerEventKind::DealtFinalBlow => "playerDealtFinalBlow",
            PlayerEventKind::DealtHealing => "playerDealtHealing",
            PlayerEventKind::Died => "playerDied",
            PlayerEventKind::EarnedElimination => "playerEarnedElimination",
            PlayerEventKind::Joined => "playerJoined",
            PlayerEventKind::Left => "playerLeft",
            PlayerEventKind::ReceivedHealing => "playerReceivedHealing",
            PlayerEventKind::TookDamage => "playerTookDamage",
        }
    }
}

/// A workshop event.
#[derive(Debug, Clone)]
pub enum Event {
    /// `Ongoing - Global` (from `@Event global`).
    Global,
    /// `Ongoing - Each Player` (from `@Event eachPlayer`).
    EachPlayer,
    /// `Ongoing - Each Player` with its canonical team/player filters.
    EachPlayerWithFilters {
        team: EventTeam,
        target: EventTarget,
    },
    /// A player-scoped Workshop event with canonical filters.
    Player {
        kind: PlayerEventKind,
        team: EventTeam,
        target: EventTarget,
    },
    /// A subroutine body (`def name():`), referencing the subroutine.
    Subroutine(SubroutineId),
}

/// A workshop value (expression) node with its source span.
#[derive(Debug, Clone)]
pub struct ValueNode {
    pub value: Value,
    pub span: Option<Span>,
}

/// A workshop value (expression).
#[derive(Debug, Clone)]
pub enum Value {
    /// A numeric literal with its source spelling (`5`, `0.0`, `-22.05`);
    /// computed values (constant folding) carry the formatted spelling.
    Number {
        value: f64,
        text: String,
    },
    String(String),
    Bool(bool),
    Null,
    Array(Vec<ValueId>),
    Vector {
        x: ValueId,
        y: ValueId,
        z: ValueId,
    },
    /// A built-in enumerated value, e.g. `Team.ALL`.
    Enum {
        value_type: String,
        value: String,
    },
    GlobalVariable(GlobalVarId),
    PlayerVariable {
        player: ValueId,
        variable: PlayerVarId,
    },
    EventPlayer,
    /// A function call over workshop values.
    Call {
        name: String,
        args: Vec<ValueId>,
    },
}

impl ValueNode {
    /// Build a value node with a source span.
    pub fn new(value: Value, span: Option<Span>) -> Self {
        ValueNode { value, span }
    }
}

/// A workshop action.
#[derive(Debug, Clone)]
pub enum Action {
    SetGlobalVariable {
        variable: GlobalVarId,
        value: ValueId,
        span: Option<Span>,
        /// The exact span of the assigned variable identifier.
        target_span: Option<Span>,
    },
    ModifyGlobalVariable {
        variable: GlobalVarId,
        op: ModifyOp,
        value: ValueId,
        span: Option<Span>,
        /// The exact span of the modified variable identifier.
        target_span: Option<Span>,
    },
    SetPlayerVariable {
        player: ValueId,
        variable: PlayerVarId,
        value: ValueId,
        span: Option<Span>,
        /// The exact span of the assigned variable identifier.
        target_span: Option<Span>,
    },
    ModifyPlayerVariable {
        player: ValueId,
        variable: PlayerVarId,
        op: ModifyOp,
        value: ValueId,
        span: Option<Span>,
        /// The exact span of the modified variable identifier.
        target_span: Option<Span>,
    },
    CallSubroutine {
        subroutine: SubroutineId,
        span: Option<Span>,
        /// The exact span of the callee identifier occurrence.
        callee_span: Option<Span>,
    },
    If {
        branches: Vec<IfBranch>,
        else_body: Option<Vec<ActionId>>,
        span: Option<Span>,
    },
    While {
        condition: ValueId,
        body: Vec<ActionId>,
        span: Option<Span>,
    },
    ForGlobalVariable {
        variable: GlobalVarId,
        start: ValueId,
        stop: ValueId,
        step: ValueId,
        body: Vec<ActionId>,
        span: Option<Span>,
        /// The exact span of the loop variable identifier.
        target_span: Option<Span>,
    },
    /// `For Player Variable(player, name, start, stop, step)`: the
    /// per-player loop form (frontend-neutral; parsed from reference
    /// evidence, not emitted by Wright's own lowering, which models
    /// foreach counters as globals under the declared #119 contract).
    ForPlayerVariable {
        player: ValueId,
        variable: PlayerVarId,
        start: ValueId,
        stop: ValueId,
        step: ValueId,
        body: Vec<ActionId>,
        span: Option<Span>,
    },
    /// The `debug(value)` HUD debug effect.
    Debug { value: ValueId, span: Option<Span> },
    /// The `print(message)` HUD message effect.
    Print {
        message: ValueId,
        span: Option<Span>,
    },
    /// Any other action call with side effects.
    Call {
        name: String,
        args: Vec<ValueId>,
        span: Option<Span>,
    },
}

impl Action {
    /// The source span of this action, if any.
    pub fn span(&self) -> Option<Span> {
        match self {
            Action::SetGlobalVariable { span, .. }
            | Action::ModifyGlobalVariable { span, .. }
            | Action::SetPlayerVariable { span, .. }
            | Action::ModifyPlayerVariable { span, .. }
            | Action::CallSubroutine { span, .. }
            | Action::If { span, .. }
            | Action::While { span, .. }
            | Action::ForGlobalVariable { span, .. }
            | Action::ForPlayerVariable { span, .. }
            | Action::Debug { span, .. }
            | Action::Print { span, .. }
            | Action::Call { span, .. } => *span,
        }
    }
}

/// One condition/body pair of an `If` action.
#[derive(Debug, Clone)]
pub struct IfBranch {
    pub condition: ValueId,
    pub body: Vec<ActionId>,
}

/// The modify operators of the v0.1 surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModifyOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    RaiseToPower,
    AppendToArray,
    RemoveFromArray,
}

impl ModifyOp {
    /// A short canonical name for dumps and diagnostics.
    pub fn as_str(self) -> &'static str {
        match self {
            ModifyOp::Add => "Add",
            ModifyOp::Subtract => "Subtract",
            ModifyOp::Multiply => "Multiply",
            ModifyOp::Divide => "Divide",
            ModifyOp::Modulo => "Modulo",
            ModifyOp::RaiseToPower => "RaiseToPower",
            ModifyOp::AppendToArray => "AppendToArray",
            ModifyOp::RemoveFromArray => "RemoveFromArray",
        }
    }
}
