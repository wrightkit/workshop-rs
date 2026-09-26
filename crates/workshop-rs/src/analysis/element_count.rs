//! Canonical Workshop element-count analysis.
//!
//! The calculator operates on the canonical public program, not source-language syntax or
//! emitted text. Every occurrence of a component costs its base amount at any
//! nesting depth, direct action and condition arguments cost one less, and
//! every pair of hero literals in one direct argument adds one. The base costs
//! and the block-closing rules are documented, with their evidence, in
//! `docs/element-count.md`.

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
            name_slot: false,
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
    /// Set while counting an argument that names a variable rather than reads it.
    name_slot: bool,
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
            children.push(self.condition(condition.value)?.node);
        }
        for (index, action) in rule.actions.iter().enumerate() {
            let rule_final = index + 1 == rule.actions.len();
            children.push(self.action(*action, rule_final)?.node);
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
                    heroes += counted.heroes / 2 * 2;
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

    fn action(&mut self, id: ActionId, rule_final: bool) -> Result<Counted, ElementCountError> {
        let node_id = self.next_node_id();
        if let Some(&active_id) = self.actions.get(&id.index()) {
            return Err(ElementCountError::Cycle {
                kind: ElementNodeKind::Action,
                id: active_id,
            });
        }
        self.actions.insert(id.index(), node_id);
        let mut action = self.program.actions.get(id);
        while let Some(Action::Disabled { action: inner, .. }) = action {
            action = self.program.actions.get(*inner);
        }
        let Some(action) = action else {
            return Err(ElementCountError::InvalidProgram {
                message: format!("dangling action {}", id.index()),
            });
        };
        let result = self.action_inner(action, node_id, rule_final);
        self.actions.remove(&id.index());
        result
    }

    fn action_inner(
        &mut self,
        action: &Action,
        node_id: usize,
        rule_final: bool,
    ) -> Result<Counted, ElementCountError> {
        let span = action.span();
        let mut children = Vec::new();
        let mut heroes = 0;
        let mut base = 1;
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
                // `Else If` and `Else` are actions of their own; so is `End`. The
                // canonical emitter closes the last action of a rule without its
                // `End`, and only that one: nested and loop blocks keep theirs.
                base += branches.len().saturating_sub(1)
                    + usize::from(else_body.is_some())
                    + usize::from(!rule_final);
                for branch in branches {
                    self.push_action_value(&mut children, &mut heroes, branch.condition)?;
                    for nested in &branch.body {
                        children.push(self.action(*nested, false)?.node);
                    }
                }
                if let Some(body) = else_body {
                    for nested in body {
                        children.push(self.action(*nested, false)?.node);
                    }
                }
            }
            Action::While {
                condition, body, ..
            } => {
                name = "while";
                base += 1;
                self.push_action_value(&mut children, &mut heroes, *condition)?;
                for nested in body {
                    children.push(self.action(*nested, false)?.node);
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
                base += 1;
                for value in [start, stop, step] {
                    self.push_action_value(&mut children, &mut heroes, *value)?;
                }
                for nested in body {
                    children.push(self.action(*nested, false)?.node);
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
                base += 1;
                for value in [player, start, stop, step] {
                    self.push_action_value(&mut children, &mut heroes, *value)?;
                }
                for nested in body {
                    children.push(self.action(*nested, false)?.node);
                }
            }
            Action::Disabled { .. } => {
                return Err(ElementCountError::InvalidProgram {
                    message: "a disabled action wraps no enabled action".to_string(),
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
                for (index, argument) in args.iter().enumerate() {
                    self.name_slot = variable_slot(self.catalog, Kind::Action, action_name, index)
                        || (index == 0 && takes_variable(self.catalog, Kind::Action, action_name));
                    self.push_action_value(&mut children, &mut heroes, *argument)?;
                }
                // An omitted argument is still a filled slot; its default is a
                // direct argument, so it costs its own value minus one.
                base = (base as isize
                    + omitted_default_cost(
                        self.catalog,
                        Kind::Action,
                        action_name,
                        args.len(),
                        true,
                    ))
                .max(0) as usize;
            }
        }
        Ok(Counted::finish(
            ElementNodeKind::Action,
            node_id,
            name,
            span,
            base,
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
        // Pairs are counted within each direct argument, not across them.
        *heroes += counted.heroes / 2 * 2;
        children.push(counted.node);
        Ok(())
    }

    fn value(&mut self, id: ValueId, top_level: bool) -> Result<Counted, ElementCountError> {
        let name_slot = std::mem::take(&mut self.name_slot);
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
            Value::Number { .. } => self.value_node(node_id, "number", span, 2, vec![], 0),
            Value::String(_) => self.value_node(node_id, "string", span, 1, vec![], 0),
            Value::LocalizedString(_) => {
                self.value_node(node_id, "localized string", span, 2, vec![], 0)
            }
            Value::Bool(_) => self.value_node(node_id, "boolean", span, 1, vec![], 0),
            Value::Null => self.value_node(node_id, "null", span, 1, vec![], 0),
            Value::Array(elements) => {
                self.value_children(node_id, "array", span, 2, elements, None)
            }
            Value::Vector { x, y, z } => {
                self.value_children(node_id, "vector", span, 1, &[*x, *y, *z], None)
            }
            Value::Enum { value_type, .. } => {
                let heroes = usize::from(value_type == "Hero");
                let base = literal_enum_cost(value_type);
                self.value_node(node_id, value_type, span, base, vec![], heroes)
            }
            Value::GlobalVariable(_) => {
                let base = if name_slot { 1 } else { 2 };
                self.value_node(node_id, "global variable", span, base, vec![], 0)
            }
            Value::PlayerVariable { player, .. } => {
                // A named player variable is two direct arguments (player, name).
                let base = if name_slot { 0 } else { 2 };
                self.value_children(node_id, "player variable", span, base, &[*player], None)
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
                    let base = if is_comparison(name) {
                        // The operator is a literal of its own.
                        2
                    } else if name == "array" || name == "evaluateOnce" {
                        2
                    } else {
                        1
                    };
                    let omitted =
                        omitted_default_cost(self.catalog, Kind::Value, name, args.len(), false);
                    let mut counted = self.value_children(
                        node_id,
                        name,
                        span,
                        (base as isize + omitted) as usize,
                        &child_ids,
                        Some((Kind::Value, name)),
                    )?;
                    let setting = workshop_setting_adjustment(name);
                    counted.node.adjustment += setting;
                    counted.node.count = (counted.node.count as isize + setting).max(0) as usize;
                    Ok(counted)
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
        owner: Option<(Kind, &str)>,
    ) -> Result<Counted, ElementCountError> {
        let mut children = Vec::with_capacity(ids.len());
        let mut heroes = 0;
        for (index, child) in ids.iter().enumerate() {
            self.name_slot =
                owner.is_some_and(|(kind, name)| variable_slot(self.catalog, kind, name, index));
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

/// The cost of the defaults filled in for arguments a call leaves out.
fn omitted_default_cost(
    catalog: &Catalog,
    kind: Kind,
    name: &str,
    given: usize,
    direct: bool,
) -> isize {
    let Some(entry) = catalog.entry(kind, name) else {
        return 0;
    };
    (given..entry.param_count())
        .filter_map(|index| entry.param_default(index))
        .map(|default| {
            let cost = if default.parse::<f64>().is_ok() {
                2
            } else {
                default.split('.').next().map_or(1, literal_enum_cost) as isize
            };
            cost - isize::from(direct)
        })
        .sum()
}

/// Enum constants the client spells as a value wrapping a literal (`Team(Team 1)`,
/// `Hero(Ana)`, `Color(White)`, `Button(Reload)`, `Map(...)`) cost both nodes.
fn literal_enum_cost(value_type: &str) -> usize {
    if matches!(value_type, "Team" | "Hero" | "Color" | "Button" | "Map") {
        2
    } else {
        1
    }
}

/// Whether argument `index` of a call names a variable instead of reading one.
fn variable_slot(catalog: &Catalog, kind: Kind, name: &str, index: usize) -> bool {
    catalog.entry(kind, name).is_some_and(|entry| {
        entry.param_type(index) == Some("Variable")
            || entry
                .params()
                .get(index)
                .is_some_and(|param| param == "Variable")
    })
}

/// Whether a call has a parameter that names a variable; the player and the
/// variable of a player variable are then one argument here.
fn takes_variable(catalog: &Catalog, kind: Kind, name: &str) -> bool {
    catalog.entry(kind, name).is_some_and(|entry| {
        (0..entry.param_count()).any(|index| variable_slot(catalog, kind, name, index))
    })
}

/// The fixed adjustment the client applies to a Workshop setting value, whose
/// category, name and bounds are literals that are not separate elements.
fn workshop_setting_adjustment(name: &str) -> isize {
    match name {
        "workshopSettingInteger"
        | "workshopSettingFloat"
        | "createWorkshopSettingInt"
        | "createWorkshopSettingFloat" => -3,
        "workshopSettingCombo" | "createWorkshopSettingEnum" => -2,
        _ => 0,
    }
}
