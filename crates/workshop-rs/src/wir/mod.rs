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

pub(crate) mod error;

pub(crate) use action::{Action, IfBranch, ModifyOp};
pub(crate) use event::{Event, EventTarget, EventTeam, PlayerEventKind};
pub(crate) use rule::{Condition, Rule, WorkshopSubroutine, WorkshopVariable};
pub(crate) use value::{Value, ValueNode};

use crate::core::arena::Arena;
use crate::core::ids::Id;
use crate::core::source::SourceFile;

/// A typed ID referencing a [`WorkshopVariable`] in the global table.
pub(crate) type GlobalVarId = Id<WorkshopVariable>;
/// A typed ID referencing a [`WorkshopVariable`] in the player table.
pub(crate) type PlayerVarId = Id<WorkshopVariable>;
/// A typed ID referencing a [`WorkshopSubroutine`].
pub(crate) type SubroutineId = Id<WorkshopSubroutine>;
/// A typed ID referencing a [`Rule`].
pub(crate) type RuleId = Id<Rule>;
/// A typed ID referencing an [`Action`] in the action arena.
pub(crate) type ActionId = Id<Action>;
/// A typed ID referencing a [`ValueNode`] in the value arena.
pub(crate) type ValueId = Id<ValueNode>;

pub(crate) const AMBIGUOUS_ENUM_CALL: &str = "__ambiguous_enum";

/// The Workshop IR program: tables and arenas produced by lowering.
#[derive(Debug, Clone)]
pub(crate) struct Program {
    /// The source-file registry, copied from the source HIR so spans remain
    /// resolvable for diagnostics.
    pub(crate) files: Arena<SourceFile>,
    /// The custom-game-settings carrier, copied inertly from the source HIR
    /// (emitted verbatim, never lowered, #86).
    pub(crate) settings: Option<crate::settings::Settings>,
    pub(crate) global_variables: Arena<WorkshopVariable>,
    pub(crate) player_variables: Arena<WorkshopVariable>,
    pub(crate) subroutines: Arena<WorkshopSubroutine>,
    pub(crate) rules: Arena<Rule>,
    pub(crate) values: Arena<ValueNode>,
    pub(crate) actions: Arena<Action>,
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
    pub(crate) fn add_file(&mut self, file: SourceFile) -> crate::source::FileId {
        let id = self.files.push(file);
        self.files.get_mut(id).unwrap().bind_file(id);
        id
    }

    #[cfg(test)]
    pub(crate) fn source(
        &self,
        file: crate::source::FileId,
    ) -> Option<&crate::source::SourceDocument> {
        self.files.get(file).and_then(SourceFile::source)
    }

    #[cfg(test)]
    pub(crate) fn semantic_issues(
        &self,
        catalog: &crate::catalog::Catalog,
    ) -> Vec<crate::rules::SemanticIssue> {
        crate::analysis::semantic::inspect_wir(self, catalog)
    }

    /// Validate structural invariants: every ID resolves and every span is
    /// valid. Returns the first violation as a structured [`IrError`].
    ///
    /// [`IrError`]: crate::wir::error::IrError
    pub(crate) fn validate(&self) -> Result<(), error::IrError> {
        validate::validate(self)
    }

    /// Render a deterministic debug dump of the workshop program.
    pub(crate) fn dump(&self) -> String {
        dump::dump(self)
    }
}

pub(crate) fn ambiguous_enum_parts(
    program: &Program,
    value_id: ValueId,
) -> Option<(&str, &[ValueId])> {
    let Value::Call { name, args } = &program.values.get(value_id)?.value else {
        return None;
    };
    if name != AMBIGUOUS_ENUM_CALL || args.len() != 2 {
        return None;
    }
    let Value::String(spelling) = &program.values.get(args[0])?.value else {
        return None;
    };
    let Value::Array(candidates) = &program.values.get(args[1])?.value else {
        return None;
    };
    Some((spelling, candidates))
}

pub(crate) fn ambiguous_enum_parts_by_args<'a>(
    program: &'a Program,
    args: &[ValueId],
) -> Option<(&'a str, Vec<(String, String)>)> {
    if args.len() != 2 {
        return None;
    }
    let Value::String(spelling) = &program.values.get(args[0])?.value else {
        return None;
    };
    let Value::Array(candidate_ids) = &program.values.get(args[1])?.value else {
        return None;
    };
    let candidates = candidate_ids
        .iter()
        .map(|candidate_id| {
            let Value::Enum { value_type, value } = &program.values.get(*candidate_id)?.value
            else {
                return None;
            };
            Some((value_type.clone(), value.clone()))
        })
        .collect::<Option<Vec<_>>>()?;
    Some((spelling, candidates))
}
