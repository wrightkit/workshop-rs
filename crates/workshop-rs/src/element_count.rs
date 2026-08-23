//! Canonical Workshop element-count analysis.
//!
//! The calculator operates on validated WIR, not source-language syntax or
//! emitted text. Its rules are the documented Workshop.codes model: rules,
//! actions, conditions, and ordinary values cost one element; arrays and
//! evaluate-once values cost two; localized strings cost two; direct action or
//! condition arguments are reduced by one; and every pair of hero literals in
//! those arguments adds one. Custom game settings and rule parameters cost
//! zero.

use std::collections::HashSet;
use std::fmt;

use crate::catalog::{Catalog, Kind};
use crate::source::Span;
use crate::wir::{self, Action, ActionId, Program, Value, ValueId};

/// The WIR node category represented in an element-count report.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElementNodeKind {
    Rule,
    Condition,
    Action,
    Value,
}

/// One node's contribution, including its nested WIR provenance.
#[derive(Debug, Clone)]
pub struct ElementCountNode {
    pub kind: ElementNodeKind,
    /// The arena index of the represented WIR node. Rule nodes use the rule
    /// arena, action nodes use the action arena, and value nodes use the value
    /// arena. Synthetic condition nodes use the condition's value index.
    pub id: usize,
    pub name: String,
    pub span: Option<Span>,
    /// The contribution before children and adjustment rules.
    pub base_count: usize,
    /// The local adjustment from the canonical model, such as a top-level
    /// argument reduction or hero-pair surcharge.
    pub adjustment: isize,
    /// The complete contribution of this node and its children.
    pub count: usize,
    pub children: Vec<ElementCountNode>,
}

/// A structured element-count report for one canonical Workshop program.
#[derive(Debug, Clone)]
pub struct ElementCountReport {
    pub total: usize,
    pub rules: Vec<ElementCountNode>,
}

impl ElementCountReport {
    /// Return the per-rule total in source/WIR order.
    pub fn rule_counts(&self) -> impl Iterator<Item = (&str, usize)> {
        self.rules
            .iter()
            .map(|rule| (rule.name.as_str(), rule.count))
    }
}

/// A construct for which an exact canonical element count cannot be produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ElementCountError {
    InvalidProgram {
        message: String,
    },
    Unsupported {
        kind: ElementNodeKind,
        name: String,
        span: Option<Span>,
        reason: String,
    },
    Cycle {
        kind: ElementNodeKind,
        id: usize,
    },
}

impl fmt::Display for ElementCountError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidProgram { message } => write!(formatter, "invalid WIR: {message}"),
            Self::Unsupported {
                kind,
                name,
                span,
                reason,
            } => write!(
                formatter,
                "unsupported {kind:?} '{name}'{}: {reason}",
                span.map_or_else(String::new, |span| format!(" at {span:?}"))
            ),
            Self::Cycle { kind, id } => write!(formatter, "cyclic {kind:?} reference at {id}"),
        }
    }
}

impl std::error::Error for ElementCountError {}

impl Program {
    /// Count the canonical Workshop target represented by this WIR program.
    ///
    /// The catalog is used to reject unknown action/value identities before a
    /// report is produced. Presentation-only `Debug` and `Print` WIR nodes
    /// are intentionally rejected because their emitted HUD expansion is not
    /// yet represented as canonical WIR actions.
    pub fn element_count(
        &self,
        catalog: &Catalog,
    ) -> Result<ElementCountReport, ElementCountError> {
        self.validate()
            .map_err(|error| ElementCountError::InvalidProgram {
                message: error.to_string(),
            })?;
        crate::validate::validate_canonical_ids(self, catalog).map_err(|error| {
            ElementCountError::InvalidProgram {
                message: error.to_string(),
            }
        })?;

        let mut counter = Counter {
            program: self,
            catalog,
            values: HashSet::new(),
            actions: HashSet::new(),
        };
        let mut rules = Vec::with_capacity(self.rules.len());
        for (index, rule) in self.rules.iter().enumerate() {
            rules.push(counter.rule(index, rule)?);
        }
        let total = rules.iter().map(|rule| rule.count).sum();
        Ok(ElementCountReport { total, rules })
    }
}

struct Counted {
    node: ElementCountNode,
    heroes: usize,
}

impl Counted {
    #[allow(clippy::too_many_arguments)]
    fn finish(
        kind: ElementNodeKind,
        id: usize,
        name: impl Into<String>,
        span: Option<Span>,
        base_count: usize,
        adjustment: isize,
        children: Vec<ElementCountNode>,
        heroes: usize,
    ) -> Self {
        let children_count: usize = children.iter().map(|child| child.count).sum();
        let count = (base_count as isize + children_count as isize + adjustment).max(0) as usize;
        Self {
            node: ElementCountNode {
                kind,
                id,
                name: name.into(),
                span,
                base_count,
                adjustment,
                count,
                children,
            },
            heroes,
        }
    }
}

struct Counter<'a> {
    program: &'a Program,
    catalog: &'a Catalog,
    values: HashSet<usize>,
    actions: HashSet<usize>,
}

impl Counter<'_> {
    fn rule(
        &mut self,
        index: usize,
        rule: &wir::Rule,
    ) -> Result<ElementCountNode, ElementCountError> {
        let mut children = Vec::with_capacity(rule.conditions.len() + rule.actions.len());
        for condition in &rule.conditions {
            children.push(self.condition(*condition)?.node);
        }
        for action in &rule.actions {
            children.push(self.action(*action)?.node);
        }
        Ok(Counted::finish(
            ElementNodeKind::Rule,
            index,
            &rule.name,
            rule.span,
            1,
            0,
            children,
            0,
        )
        .node)
    }

    fn condition(&mut self, id: ValueId) -> Result<Counted, ElementCountError> {
        let Some(value) = self.program.values.get(id) else {
            return Err(ElementCountError::InvalidProgram {
                message: format!("dangling condition value {}", id.index()),
            });
        };
        let (children, heroes) = match &value.value {
            Value::Call { name, args } if is_comparison(name) => {
                let mut children = Vec::with_capacity(args.len());
                let mut heroes = 0;
                for argument in args {
                    let counted = self.value(*argument, true)?;
                    heroes += counted.heroes;
                    children.push(counted.node);
                }
                (children, heroes)
            }
            _ => {
                let counted = self.value(id, true)?;
                (vec![counted.node], counted.heroes)
            }
        };
        Ok(Counted::finish(
            ElementNodeKind::Condition,
            id.index(),
            "condition",
            value.span,
            1,
            pair_surcharge(heroes),
            children,
            heroes,
        ))
    }

    fn action(&mut self, id: ActionId) -> Result<Counted, ElementCountError> {
        if !self.actions.insert(id.index()) {
            return Err(ElementCountError::Cycle {
                kind: ElementNodeKind::Action,
                id: id.index(),
            });
        }
        let Some(action) = self.program.actions.get(id) else {
            return Err(ElementCountError::InvalidProgram {
                message: format!("dangling action {}", id.index()),
            });
        };
        let result = self.action_inner(id, action);
        self.actions.remove(&id.index());
        result
    }

    fn action_inner(
        &mut self,
        id: ActionId,
        action: &Action,
    ) -> Result<Counted, ElementCountError> {
        let span = action.span();
        let mut children = Vec::new();
        let mut heroes = 0;
        let name;
        match action {
            Action::SetGlobalVariable { value, .. }
            | Action::ModifyGlobalVariable { value, .. } => {
                name = "variable action";
                self.push_action_value(&mut children, &mut heroes, *value)?;
            }
            Action::SetPlayerVariable { player, value, .. }
            | Action::ModifyPlayerVariable { player, value, .. } => {
                name = "player variable action";
                self.push_action_value(&mut children, &mut heroes, *player)?;
                self.push_action_value(&mut children, &mut heroes, *value)?;
            }
            Action::AssignMember { target, value, .. } => {
                name = "member assignment";
                self.push_action_value(&mut children, &mut heroes, *target)?;
                self.push_action_value(&mut children, &mut heroes, *value)?;
            }
            Action::CallSubroutine { .. } => {
                name = "call subroutine";
            }
            Action::If {
                branches,
                else_body,
                ..
            } => {
                name = "if";
                for branch in branches {
                    self.push_action_value(&mut children, &mut heroes, branch.condition)?;
                    for nested in &branch.body {
                        children.push(self.action(*nested)?.node);
                    }
                }
                if let Some(body) = else_body {
                    for nested in body {
                        children.push(self.action(*nested)?.node);
                    }
                }
            }
            Action::While {
                condition, body, ..
            } => {
                name = "while";
                self.push_action_value(&mut children, &mut heroes, *condition)?;
                for nested in body {
                    children.push(self.action(*nested)?.node);
                }
            }
            Action::ForGlobalVariable {
                start,
                stop,
                step,
                body,
                ..
            } => {
                name = "for global variable";
                for value in [start, stop, step] {
                    self.push_action_value(&mut children, &mut heroes, *value)?;
                }
                for nested in body {
                    children.push(self.action(*nested)?.node);
                }
            }
            Action::ForPlayerVariable {
                player,
                start,
                stop,
                step,
                body,
                ..
            } => {
                name = "for player variable";
                for value in [player, start, stop, step] {
                    self.push_action_value(&mut children, &mut heroes, *value)?;
                }
                for nested in body {
                    children.push(self.action(*nested)?.node);
                }
            }
            Action::Debug { .. } => {
                return Err(ElementCountError::Unsupported {
                    kind: ElementNodeKind::Action,
                    name: "debug".to_string(),
                    span,
                    reason: "the emitter expands Debug into a HUD action; count the canonical HUD action instead".to_string(),
                });
            }
            Action::Print { .. } => {
                return Err(ElementCountError::Unsupported {
                    kind: ElementNodeKind::Action,
                    name: "print".to_string(),
                    span,
                    reason: "the emitter expands Print into a HUD action; count the canonical HUD action instead".to_string(),
                });
            }
            Action::Call {
                name: action_name,
                args,
                ..
            } => {
                if self.catalog.entry(Kind::Action, action_name).is_none() {
                    return Err(ElementCountError::Unsupported {
                        kind: ElementNodeKind::Action,
                        name: action_name.clone(),
                        span,
                        reason: "the action is not a catalog identity".to_string(),
                    });
                }
                name = action_name.as_str();
                for argument in args {
                    self.push_action_value(&mut children, &mut heroes, *argument)?;
                }
            }
        }
        Ok(Counted::finish(
            ElementNodeKind::Action,
            id.index(),
            name,
            span,
            1,
            pair_surcharge(heroes),
            children,
            heroes,
        ))
    }

    fn push_action_value(
        &mut self,
        children: &mut Vec<ElementCountNode>,
        heroes: &mut usize,
        id: ValueId,
    ) -> Result<(), ElementCountError> {
        let counted = self.value(id, true)?;
        *heroes += counted.heroes;
        children.push(counted.node);
        Ok(())
    }

    fn value(&mut self, id: ValueId, top_level: bool) -> Result<Counted, ElementCountError> {
        if !self.values.insert(id.index()) {
            return Err(ElementCountError::Cycle {
                kind: ElementNodeKind::Value,
                id: id.index(),
            });
        }
        let Some(value) = self.program.values.get(id) else {
            return Err(ElementCountError::InvalidProgram {
                message: format!("dangling value {}", id.index()),
            });
        };
        let span = value.span;
        let result = match &value.value {
            Value::Number { .. } => self.value_node(id, "number", span, 1, vec![], 0),
            Value::String(_) => self.value_node(id, "string", span, 1, vec![], 0),
            Value::LocalizedString(_) => {
                self.value_node(id, "localized string", span, 2, vec![], 0)
            }
            Value::Bool(_) => self.value_node(id, "boolean", span, 1, vec![], 0),
            Value::Null => self.value_node(id, "null", span, 1, vec![], 0),
            Value::Array(elements) => self.value_children(id, "array", span, 2, elements),
            Value::Vector { x, y, z } => self.value_children(id, "vector", span, 1, &[*x, *y, *z]),
            Value::Enum { value_type, .. } => {
                let heroes = usize::from(value_type == "Hero");
                self.value_node(id, value_type, span, 1, vec![], heroes)
            }
            Value::GlobalVariable(_) => self.value_node(id, "global variable", span, 1, vec![], 0),
            Value::PlayerVariable { player, .. } => {
                self.value_children(id, "player variable", span, 1, &[*player])
            }
            Value::Subroutine(_) => self.value_node(id, "subroutine", span, 1, vec![], 0),
            Value::EventPlayer => self.value_node(id, "event player", span, 1, vec![], 0),
            Value::Call { name, args } => {
                if name != "memberAccess"
                    && self.catalog.entry(Kind::Value, name).is_none()
                    && self.catalog.entry(Kind::Operator, name).is_none()
                    && !is_canonical_helper(name)
                {
                    return Err(ElementCountError::Unsupported {
                        kind: ElementNodeKind::Value,
                        name: name.clone(),
                        span,
                        reason: "the value is not a catalog identity".to_string(),
                    });
                }
                let child_ids: Vec<ValueId> = if name == "memberAccess" {
                    args.first()
                        .copied()
                        .into_iter()
                        .chain(args.iter().copied().skip(2))
                        .collect()
                } else {
                    args.clone()
                };
                let base = if name == "array"
                    || name == "evalOnce"
                    || name.starts_with("workshopSetting")
                    || name.starts_with("createWorkshopSetting")
                {
                    2
                } else {
                    1
                };
                self.value_children(id, name, span, base, &child_ids)
            }
        }?;
        self.values.remove(&id.index());
        let mut result = result;
        if top_level {
            result.node.adjustment -= 1;
            result.node.count = (result.node.count as isize - 1).max(0) as usize;
        }
        Ok(result)
    }

    fn value_node(
        &self,
        id: ValueId,
        name: impl Into<String>,
        span: Option<Span>,
        base: usize,
        children: Vec<ElementCountNode>,
        heroes: usize,
    ) -> Result<Counted, ElementCountError> {
        Ok(Counted::finish(
            ElementNodeKind::Value,
            id.index(),
            name,
            span,
            base,
            0,
            children,
            heroes,
        ))
    }

    fn value_children(
        &mut self,
        id: ValueId,
        name: impl Into<String>,
        span: Option<Span>,
        base: usize,
        ids: &[ValueId],
    ) -> Result<Counted, ElementCountError> {
        let mut children = Vec::with_capacity(ids.len());
        let mut heroes = 0;
        for child in ids {
            let counted = self.value(*child, false)?;
            heroes += counted.heroes;
            children.push(counted.node);
        }
        self.value_node(id, name, span, base, children, heroes)
    }
}

fn pair_surcharge(heroes: usize) -> isize {
    (heroes / 2) as isize
}

fn is_comparison(name: &str) -> bool {
    matches!(name, "==" | "!=" | "<" | "<=" | ">" | ">=")
}

fn is_canonical_helper(name: &str) -> bool {
    matches!(
        name,
        "memberAccess"
            | "+"
            | "-"
            | "*"
            | "/"
            | "%"
            | "add"
            | "subtract"
            | "multiply"
            | "divide"
            | "modulo"
            | "raiseToPower"
            | "appendToArray"
            | "removeFromArray"
            | "removeFromArrayByIndex"
    )
}
