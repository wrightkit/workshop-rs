//! Canonical Workshop element-count analysis.
//!
//! The calculator operates on the canonical public program, not source-language syntax or
//! emitted text. Its rules are the documented Workshop.codes model: rules,
//! actions, conditions, and ordinary values cost one element; arrays and
//! evaluate-once values cost two; localized strings cost two; direct action or
//! condition arguments are reduced by one; and every pair of hero literals in
//! those arguments adds one. Custom game settings and rule parameters cost
//! zero.

use std::collections::HashMap;
use std::fmt;

use crate::catalog::{Catalog, Kind};
use crate::core::source::Span;
use crate::wir::{self, Action, ActionId, Program, Value, ValueId};

/// The Workshop node category represented in an element-count report.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElementNodeKind {
    Rule,
    Condition,
    Action,
    Value,
}

/// One node's contribution and its nested element-count analysis.
#[derive(Debug, Clone)]
pub struct ElementCountNode {
    pub kind: ElementNodeKind,
    /// An opaque identity unique within this report. It is not a WIR or
    /// storage arena index and has no meaning across reports.
    pub id: usize,
    /// The canonical Workshop or analysis name for this node.
    pub name: String,
    /// The authored source span, when the program retained one.
    pub span: Option<Span>,
    /// The node-local contribution before child counts and adjustments.
    pub base_count: usize,
    /// The signed node-local adjustment, such as a direct-argument reduction
    /// or hero-pair surcharge.
    pub adjustment: isize,
    /// The node's recursive count: `base_count + adjustment + children`.
    pub count: usize,
    /// Nested values, conditions, and actions in canonical source order.
    pub children: Vec<ElementCountNode>,
}

/// A structured element-count report for one canonical Workshop program.
#[derive(Debug, Clone)]
pub struct ElementCountReport {
    /// The sum of all rule counts.
    pub total: usize,
    /// Rule nodes in canonical source/WIR order.
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
    /// The input cannot be materialized or structurally validated as a
    /// canonical Workshop program.
    InvalidProgram { message: String },
    /// The input contains a construct for which this analyzer has no exact
    /// canonical count.
    Unsupported {
        kind: ElementNodeKind,
        name: String,
        span: Option<Span>,
        reason: String,
    },
    /// The internal graph contains a recursive value or action reference.
    Cycle {
        kind: ElementNodeKind,
        /// The opaque identity of the active node involved in the cycle.
        id: usize,
    },
}

impl fmt::Display for ElementCountError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidProgram { message } => write!(formatter, "invalid program: {message}"),
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
    /// report is produced. Native display actions are represented by their
    /// canonical catalog-backed action calls.
    pub(crate) fn element_count(
        &self,
        catalog: &Catalog,
    ) -> Result<ElementCountReport, ElementCountError> {
        self.validate()
            .map_err(|error| ElementCountError::InvalidProgram {
                message: error.to_string(),
            })?;
        crate::rules::validate::validate_wir(self, catalog).map_err(|error| {
            ElementCountError::InvalidProgram {
                message: error.to_string(),
            }
        })?;

        let mut counter = Counter {
            program: self,
            catalog,
            values: HashMap::new(),
            actions: HashMap::new(),
            next_node_id: 0,
        };
        let mut rules = Vec::with_capacity(self.rules.len());
        for rule in self.rules.iter() {
            rules.push(counter.rule(rule)?);
        }
        let total = rules.iter().map(|rule| rule.count).sum();
        Ok(ElementCountReport { total, rules })
    }
}

impl crate::program::Program {
    /// Count the canonical Workshop target represented by this program.
    pub fn element_count(
        &self,
        catalog: &Catalog,
    ) -> Result<ElementCountReport, ElementCountError> {
        let storage = self
            .to_wir()
            .map_err(|error| ElementCountError::InvalidProgram {
                message: error.to_string(),
            })?;
        storage.element_count(catalog)
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
    values: HashMap<usize, usize>,
    actions: HashMap<usize, usize>,
    next_node_id: usize,
}

impl Counter<'_> {
    fn next_node_id(&mut self) -> usize {
        let id = self.next_node_id;
        self.next_node_id += 1;
        id
    }

    fn rule(&mut self, rule: &wir::Rule) -> Result<ElementCountNode, ElementCountError> {
        let node_id = self.next_node_id();
        let mut children = Vec::with_capacity(rule.conditions.len() + rule.actions.len());
        for condition in &rule.conditions {
            if condition.disabled {
                return Err(ElementCountError::Unsupported {
                    kind: ElementNodeKind::Condition,
                    name: "disabled condition".to_string(),
                    span: self
                        .program
                        .values
                        .get(condition.value)
                        .and_then(|v| v.span),
                    reason: "the element cost of a disabled condition is not established"
                        .to_string(),
                });
            }
            children.push(self.condition(condition.value)?.node);
        }
        for action in &rule.actions {
            children.push(self.action(*action)?.node);
        }
        Ok(Counted::finish(
            ElementNodeKind::Rule,
            node_id,
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
        let node_id = self.next_node_id();
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
            node_id,
            "condition",
            value.span,
            1,
            pair_surcharge(heroes),
            children,
            heroes,
        ))
    }

    fn action(&mut self, id: ActionId) -> Result<Counted, ElementCountError> {
        let node_id = self.next_node_id();
        if let Some(&active_id) = self.actions.get(&id.index()) {
            return Err(ElementCountError::Cycle {
                kind: ElementNodeKind::Action,
                id: active_id,
            });
        }
        self.actions.insert(id.index(), node_id);
        let Some(action) = self.program.actions.get(id) else {
            return Err(ElementCountError::InvalidProgram {
                message: format!("dangling action {}", id.index()),
            });
        };
        let result = self.action_inner(action, node_id);
        self.actions.remove(&id.index());
        result
    }

    fn action_inner(
        &mut self,
        action: &Action,
        node_id: usize,
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
            Action::Disabled { .. } => {
                return Err(ElementCountError::Unsupported {
                    kind: ElementNodeKind::Action,
                    name: "disabled action".to_string(),
                    span,
                    reason: "the element cost of a disabled action is not established".to_string(),
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
            node_id,
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
        let node_id = self.next_node_id();
        if let Some(&active_id) = self.values.get(&id.index()) {
            return Err(ElementCountError::Cycle {
                kind: ElementNodeKind::Value,
                id: active_id,
            });
        }
        self.values.insert(id.index(), node_id);
        let Some(value) = self.program.values.get(id) else {
            return Err(ElementCountError::InvalidProgram {
                message: format!("dangling value {}", id.index()),
            });
        };
        let span = value.span;
        let result = match &value.value {
            Value::Number { .. } => self.value_node(node_id, "number", span, 1, vec![], 0),
            Value::String(_) => self.value_node(node_id, "string", span, 1, vec![], 0),
            Value::LocalizedString(_) => {
                self.value_node(node_id, "localized string", span, 2, vec![], 0)
            }
            Value::Bool(_) => self.value_node(node_id, "boolean", span, 1, vec![], 0),
            Value::Null => self.value_node(node_id, "null", span, 1, vec![], 0),
            Value::Array(elements) => self.value_children(node_id, "array", span, 2, elements),
            Value::Vector { x, y, z } => {
                self.value_children(node_id, "vector", span, 1, &[*x, *y, *z])
            }
            Value::Enum { value_type, .. } => {
                let heroes = usize::from(value_type == "Hero");
                self.value_node(node_id, value_type, span, 1, vec![], heroes)
            }
            Value::GlobalVariable(_) => {
                self.value_node(node_id, "global variable", span, 1, vec![], 0)
            }
            Value::PlayerVariable { player, .. } => {
                self.value_children(node_id, "player variable", span, 1, &[*player])
            }
            Value::Subroutine(_) => self.value_node(node_id, "subroutine", span, 1, vec![], 0),
            Value::EventPlayer => self.value_node(node_id, "event player", span, 1, vec![], 0),
            Value::Call { name, args } => {
                if name == crate::wir::AMBIGUOUS_ENUM_CALL
                    && crate::wir::ambiguous_enum_parts(self.program, id).is_some()
                {
                    self.value_node(node_id, "ambiguous enum", span, 1, vec![], 0)
                } else {
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
                    self.value_children(node_id, name, span, base, &child_ids)
                }
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
        id: usize,
        name: impl Into<String>,
        span: Option<Span>,
        base: usize,
        children: Vec<ElementCountNode>,
        heroes: usize,
    ) -> Result<Counted, ElementCountError> {
        Ok(Counted::finish(
            ElementNodeKind::Value,
            id,
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
        id: usize,
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
            | "min"
            | "max"
            | "raiseToPower"
            | "appendToArray"
            | "removeFromArray"
            | "removeFromArrayByValue"
            | "removeFromArrayByIndex"
    )
}
