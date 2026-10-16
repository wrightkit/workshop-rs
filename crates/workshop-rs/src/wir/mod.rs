mod action;
mod dump;
mod event;
mod rule;
mod validate;
mod value;

pub(crate) mod error;

pub(crate) use crate::program::shared::ModifyOp;
pub(crate) use action::{Action, IfBranch};
pub(crate) use event::{Event, EventTarget, EventTeam, PlayerEventKind};
pub(crate) use rule::{Condition, Rule, WorkshopSubroutine, WorkshopVariable};
pub(crate) use value::{Value, ValueNode};

use crate::core::arena::Arena;
use crate::core::ids::Id;
use crate::core::source::SourceFile;

pub(crate) type GlobalVarId = Id<WorkshopVariable>;
pub(crate) type PlayerVarId = Id<WorkshopVariable>;
pub(crate) type SubroutineId = Id<WorkshopSubroutine>;
pub(crate) type RuleId = Id<Rule>;
pub(crate) type ActionId = Id<Action>;
pub(crate) type ValueId = Id<ValueNode>;

pub(crate) const AMBIGUOUS_ENUM_CALL: &str = "__ambiguous_enum";

pub(crate) fn is_canonical_helper(name: &str) -> bool {
    matches!(name, "memberAccess" | "+" | "-" | "*" | "/" | "%") || is_operation_helper(name)
}

pub(crate) fn is_canonical_helper_call(name: &str, argument_count: usize) -> bool {
    is_canonical_helper(name)
        && (argument_count == 0 || matches!(name, "memberAccess" | "+" | "-" | "*" | "/" | "%"))
}

pub(crate) fn is_operation_helper(name: &str) -> bool {
    matches!(
        name,
        "add"
            | "subtract"
            | "multiply"
            | "divide"
            | "modulo"
            | "min"
            | "max"
            | "raiseToPower"
            | "appendToArray"
            | "removeFromArray"
            | "removeFromArrayByValue"
            | "removeFromArrayByIndex"
    )
}

pub(crate) fn is_comparison_operator(name: &str) -> bool {
    matches!(name, "==" | "!=" | "<" | "<=" | ">" | ">=")
}

#[derive(Debug, Clone)]
pub(crate) struct Program {
    pub(crate) files: Arena<SourceFile>,
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

    pub(crate) fn validate(&self) -> Result<(), error::IrError> {
        validate::validate(self)
    }

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
