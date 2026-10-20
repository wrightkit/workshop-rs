//! Statement input for the `workshop-rs/rule-content-v1` format.
//!
//! [`statement_from_content`] is the read direction of the format's input
//! side: it converts one condition or action list item — `{"value": node}` or
//! `{"call": id, "args": [...]}`, with optional `"disabled": true` — into the
//! public model. A statement read from [`crate::Rule::content`] is valid input
//! unchanged, so rendering the converted statement again gives the same
//! document. The format id and schema are unchanged.
//!
//! The conversion refuses, with a distinct [`StatementContentError`], every
//! node it cannot turn into exactly the construct the node names: an unknown
//! id, a wrong argument count, an internal sentinel, a node that forms no
//! statement, and a node carrying spans. Nothing is approximated. A preserved
//! ambiguous enum member is refused too: it carries a locale-dependent
//! spelling, and the model value would not be the one a client wrote.

use std::fmt;

use serde_json::{Map, Value as JsonValue};

use crate::catalog::{Catalog, CatalogEntry, Kind};
use crate::program::{Action, Condition, ModifyOp, Value};
use crate::rules::content::{
    ASSIGN_MEMBER_CALL, GLOBAL_VARIABLE_CALL, MEMBER_ACCESS_CALL, MODIFY_MEMBER_CALL,
    PLAYER_VARIABLE_CALL, RAW_WORKSHOP_ACTION_CALL,
};
use crate::wir::{self, AMBIGUOUS_ENUM_CALL};

/// One rule statement in the public model: a condition or an action.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum Statement {
    /// A condition list item, `{"value": node}`.
    Condition(Condition),
    /// An action list item, `{"call": id, "args": [...]}`. A `"disabled"`
    /// modifier becomes [`Action::Disabled`].
    Action(Action),
}

/// Why [`statement_from_content`] refused a node. Every variant names the
/// offending node by its JSON pointer from the statement (`""` is the
/// statement itself) and a copy of the node.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum StatementContentError {
    /// A call id the catalog (and the format's reserved ids) does not name.
    UnknownId {
        id: String,
        path: String,
        node: JsonValue,
    },
    /// A call whose argument count the catalog signature does not accept.
    /// `max` is `None` for a variadic signature.
    ArgumentCount {
        id: String,
        min: usize,
        max: Option<usize>,
        supplied: usize,
        path: String,
        node: JsonValue,
    },
    /// An internal name (the ambiguous-enum helper) the format never exposes.
    InternalSentinel {
        id: String,
        path: String,
        node: JsonValue,
    },
    /// A node that forms no statement or value of the format.
    Unrepresentable {
        reason: String,
        path: String,
        node: JsonValue,
    },
    /// A node carrying spans, which are output-only.
    Spans {
        key: String,
        path: String,
        node: JsonValue,
    },
}

impl StatementContentError {
    /// The JSON pointer of the offending node, relative to the statement.
    pub fn path(&self) -> &str {
        match self {
            Self::UnknownId { path, .. }
            | Self::ArgumentCount { path, .. }
            | Self::InternalSentinel { path, .. }
            | Self::Unrepresentable { path, .. }
            | Self::Spans { path, .. } => path,
        }
    }

    /// The offending node.
    pub fn node(&self) -> &JsonValue {
        match self {
            Self::UnknownId { node, .. }
            | Self::ArgumentCount { node, .. }
            | Self::InternalSentinel { node, .. }
            | Self::Unrepresentable { node, .. }
            | Self::Spans { node, .. } => node,
        }
    }
}

impl fmt::Display for StatementContentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownId { id, path, .. } => {
                write!(f, "unknown rule-content-v1 id '{id}' at '{path}'")
            }
            Self::ArgumentCount {
                id,
                min,
                max,
                supplied,
                path,
                ..
            } => {
                let max = max.map_or_else(|| "+".to_string(), |max| format!("..{max}"));
                write!(
                    f,
                    "'{id}' at '{path}' expects {min}{max} argument(s), got {supplied}"
                )
            }
            Self::InternalSentinel { id, path, .. } => {
                write!(f, "internal name '{id}' at '{path}' is not rule-content-v1")
            }
            Self::Unrepresentable { reason, path, .. } => {
                write!(f, "node at '{path}' is not representable: {reason}")
            }
            Self::Spans { key, path, .. } => {
                write!(f, "'{key}' at '{path}': spans are output-only")
            }
        }
    }
}

impl std::error::Error for StatementContentError {}

type Result<T> = std::result::Result<T, StatementContentError>;

/// Convert one `rule-content-v1` statement into the public model.
///
/// `catalog` decides which ids exist and how many arguments they take. The
/// result is not validated as a program: whether the statement fits its
/// rule (an `If` without its `End`, a value of the wrong type) is for
/// canonical validation of the edited program.
///
/// ```
/// use serde_json::json;
/// use workshop_rs::catalog::Catalog;
/// use workshop_rs::rules::statement::{Statement, statement_from_content};
///
/// let catalog = Catalog::builtin().unwrap();
/// let statement = statement_from_content(
///     &catalog,
///     &json!({"call": "wait", "args": [0.5, {"enum": "Wait", "member": "IGNORE_CONDITION"}]}),
/// )
/// .unwrap();
/// assert!(matches!(statement, Statement::Action(_)));
/// ```
pub fn statement_from_content(catalog: &Catalog, node: &JsonValue) -> Result<Statement> {
    let converter = Converter { catalog };
    let object = converter.object(node, "")?;
    if object.contains_key("value") {
        converter.condition(object, node)
    } else if object.contains_key("call") {
        converter.action(object, node)
    } else {
        Err(unrepresentable(
            "a statement has a 'value' (condition) or a 'call' (action)",
            "",
            node,
        ))
    }
}

struct Converter<'a> {
    catalog: &'a Catalog,
}

fn unrepresentable(
    reason: impl Into<String>,
    path: &str,
    node: &JsonValue,
) -> StatementContentError {
    StatementContentError::Unrepresentable {
        reason: reason.into(),
        path: path.to_string(),
        node: node.clone(),
    }
}

fn child(path: &str, segment: impl fmt::Display) -> String {
    format!("{path}/{segment}")
}

impl Converter<'_> {
    /// The node as an object, after rejecting span keys.
    fn object<'n>(&self, node: &'n JsonValue, path: &str) -> Result<&'n Map<String, JsonValue>> {
        let Some(object) = node.as_object() else {
            return Err(unrepresentable("expected a JSON object", path, node));
        };
        for key in ["span", "spans"] {
            if object.contains_key(key) {
                return Err(StatementContentError::Spans {
                    key: key.to_string(),
                    path: path.to_string(),
                    node: node.clone(),
                });
            }
        }
        Ok(object)
    }

    fn only_keys(
        &self,
        object: &Map<String, JsonValue>,
        allowed: &[&str],
        path: &str,
        node: &JsonValue,
    ) -> Result<()> {
        match object.keys().find(|key| !allowed.contains(&key.as_str())) {
            Some(key) => Err(unrepresentable(
                format!("unexpected key '{key}'"),
                path,
                node,
            )),
            None => Ok(()),
        }
    }

    fn disabled(&self, object: &Map<String, JsonValue>, node: &JsonValue) -> Result<bool> {
        match object.get("disabled") {
            None => Ok(false),
            Some(JsonValue::Bool(true)) => Ok(true),
            Some(_) => Err(unrepresentable(
                "'disabled' is present only as true",
                "",
                node,
            )),
        }
    }

    fn condition(&self, object: &Map<String, JsonValue>, node: &JsonValue) -> Result<Statement> {
        self.only_keys(object, &["value", "disabled"], "", node)?;
        let disabled = self.disabled(object, node)?;
        let value = self.value(&object["value"], "/value")?;
        Ok(Statement::Condition(if disabled {
            Condition::disabled(value)
        } else {
            Condition::new(value)
        }))
    }

    fn action(&self, object: &Map<String, JsonValue>, node: &JsonValue) -> Result<Statement> {
        self.only_keys(object, &["call", "args", "disabled"], "", node)?;
        let disabled = self.disabled(object, node)?;
        let action = self.action_call(object, node, "")?;
        Ok(Statement::Action(if disabled {
            Action::disabled(action)
        } else {
            action
        }))
    }

    /// The `call` id and `args` list of a call node.
    fn call_parts<'n>(
        &self,
        object: &'n Map<String, JsonValue>,
        node: &JsonValue,
        path: &str,
    ) -> Result<(&'n str, &'n [JsonValue])> {
        let id = object
            .get("call")
            .and_then(JsonValue::as_str)
            .ok_or_else(|| unrepresentable("'call' must be a string id", path, node))?;
        let args = object
            .get("args")
            .and_then(JsonValue::as_array)
            .ok_or_else(|| unrepresentable("a call node needs an 'args' array", path, node))?;
        if id == AMBIGUOUS_ENUM_CALL {
            return Err(StatementContentError::InternalSentinel {
                id: id.to_string(),
                path: path.to_string(),
                node: node.clone(),
            });
        }
        Ok((id, args))
    }

    fn action_call(
        &self,
        object: &Map<String, JsonValue>,
        node: &JsonValue,
        path: &str,
    ) -> Result<Action> {
        let (id, args) = self.call_parts(object, node, path)?;
        let arg_path = |index: usize| child(&child(path, "args"), index);
        let unknown = || StatementContentError::UnknownId {
            id: id.to_string(),
            path: path.to_string(),
            node: node.clone(),
        };
        let count = |min: usize, max: usize| -> Result<()> {
            if (min..=max).contains(&args.len()) {
                Ok(())
            } else {
                Err(StatementContentError::ArgumentCount {
                    id: id.to_string(),
                    min,
                    max: Some(max),
                    supplied: args.len(),
                    path: path.to_string(),
                    node: node.clone(),
                })
            }
        };
        let value = |index: usize| self.value(&args[index], &arg_path(index));
        let variable = |index: usize| self.variable(&args[index], &arg_path(index));
        let op = |index: usize| self.modify_op(&args[index], &arg_path(index));

        let action = match id {
            RAW_WORKSHOP_ACTION_CALL => {
                return Err(unrepresentable(
                    "an opaque raw Workshop action has no canonical content",
                    path,
                    node,
                ));
            }
            "setGlobalVariable" => {
                count(2, 2)?;
                Action::SetGlobalVariable {
                    variable: variable(0)?,
                    value: value(1)?,
                }
            }
            "modifyGlobalVariable" => {
                count(3, 3)?;
                Action::ModifyGlobalVariable {
                    variable: variable(0)?,
                    op: op(1)?,
                    value: value(2)?,
                }
            }
            "setPlayerVariable" => {
                count(3, 3)?;
                Action::SetPlayerVariable {
                    player: value(0)?,
                    variable: variable(1)?,
                    value: value(2)?,
                }
            }
            "modifyPlayerVariable" => {
                count(4, 4)?;
                Action::ModifyPlayerVariable {
                    player: value(0)?,
                    variable: variable(1)?,
                    op: op(2)?,
                    value: value(3)?,
                }
            }
            ASSIGN_MEMBER_CALL => {
                count(2, 2)?;
                Action::AssignMember {
                    target: value(0)?,
                    op: None,
                    value: value(1)?,
                }
            }
            MODIFY_MEMBER_CALL => {
                count(3, 3)?;
                Action::AssignMember {
                    target: value(0)?,
                    op: Some(op(1)?),
                    value: value(2)?,
                }
            }
            "callSubroutine" => {
                count(1, 1)?;
                let JsonValue::Object(subroutine) = &args[0] else {
                    return Err(unrepresentable(
                        "expected a {\"subroutine\"} node",
                        &arg_path(0),
                        &args[0],
                    ));
                };
                self.object(&args[0], &arg_path(0))?;
                let name = subroutine.get("subroutine").and_then(JsonValue::as_str);
                match (name, subroutine.len()) {
                    (Some(name), 1) => Action::CallSubroutine {
                        subroutine: name.to_string(),
                    },
                    _ => {
                        return Err(unrepresentable(
                            "expected a {\"subroutine\"} node",
                            &arg_path(0),
                            &args[0],
                        ));
                    }
                }
            }
            "if" => {
                count(1, 1)?;
                Action::If {
                    condition: value(0)?,
                }
            }
            "elseIf" => {
                count(1, 1)?;
                Action::ElseIf {
                    condition: value(0)?,
                }
            }
            "while" => {
                count(1, 1)?;
                Action::While {
                    condition: value(0)?,
                }
            }
            "else" => {
                count(0, 0)?;
                Action::Else
            }
            "end" => {
                count(0, 0)?;
                Action::End
            }
            "forGlobalVariable" => {
                count(4, 4)?;
                Action::ForGlobalVariable {
                    variable: variable(0)?,
                    start: value(1)?,
                    stop: value(2)?,
                    step: value(3)?,
                }
            }
            "forPlayerVariable" => {
                count(5, 5)?;
                Action::ForPlayerVariable {
                    player: value(0)?,
                    variable: variable(1)?,
                    start: value(2)?,
                    stop: value(3)?,
                    step: value(4)?,
                }
            }
            _ => {
                let entry = self.catalog.entry(Kind::Action, id).ok_or_else(unknown)?;
                let values = self.values(args, path)?;
                self.check_arity(entry, &values, node, path)?;
                // The model keeps a player-variable chase under the global
                // chase's name; the document names it by its own id.
                let name = match (id, values.first()) {
                    ("chasePlayerVariableAtRate", Some(Value::PlayerVariable { .. })) => {
                        "chaseAtRate"
                    }
                    ("chasePlayerVariableOverTime", Some(Value::PlayerVariable { .. })) => {
                        "chaseOverTime"
                    }
                    ("chasePlayerVariableAtRate" | "chasePlayerVariableOverTime", _) => {
                        return Err(unrepresentable(
                            "a player-variable chase takes a playerVariable target",
                            &arg_path(0),
                            &args[0],
                        ));
                    }
                    ("chaseAtRate" | "chaseOverTime", Some(Value::PlayerVariable { .. })) => {
                        return Err(unrepresentable(
                            "a playerVariable chase is named chasePlayerVariable…",
                            path,
                            node,
                        ));
                    }
                    _ => id,
                };
                Action::call(name, values)
            }
        };
        Ok(action)
    }

    /// Check the argument count against a catalog signature, with the rule
    /// canonical validation applies.
    fn check_arity(
        &self,
        entry: &CatalogEntry,
        values: &[Value],
        node: &JsonValue,
        path: &str,
    ) -> Result<()> {
        let folded = entry.folds_player_variable_pair()
            && matches!(values.first(), Some(Value::PlayerVariable { .. }));
        let supplied = values.len() + usize::from(folded);
        if let Some((min, max)) = entry.rejected_arity(supplied) {
            return Err(StatementContentError::ArgumentCount {
                id: entry.id.clone(),
                min,
                max,
                supplied,
                path: path.to_string(),
                node: node.clone(),
            });
        }
        Ok(())
    }

    fn variable(&self, node: &JsonValue, path: &str) -> Result<String> {
        let object = self.object(node, path)?;
        match (
            object.get("variable").and_then(JsonValue::as_str),
            object.len(),
        ) {
            (Some(name), 1) => Ok(name.to_string()),
            _ => Err(unrepresentable(
                "expected a {\"variable\"} node",
                path,
                node,
            )),
        }
    }

    fn modify_op(&self, node: &JsonValue, path: &str) -> Result<ModifyOp> {
        let object = self.object(node, path)?;
        let (id, args) = self.call_parts(object, node, path)?;
        self.only_keys(object, &["call", "args"], path, node)?;
        if !args.is_empty() {
            return Err(StatementContentError::ArgumentCount {
                id: id.to_string(),
                min: 0,
                max: Some(0),
                supplied: args.len(),
                path: path.to_string(),
                node: node.clone(),
            });
        }
        ModifyOp::from_catalog_id(id)
            .ok_or_else(|| unrepresentable(format!("'{id}' is not a modify operation"), path, node))
    }

    fn value(&self, node: &JsonValue, path: &str) -> Result<Value> {
        match node {
            JsonValue::Number(number) => Ok(Value::Number(number.as_f64().ok_or_else(|| {
                unrepresentable("number is not representable as f64", path, node)
            })?)),
            JsonValue::Bool(value) => Ok(Value::Bool(*value)),
            JsonValue::Null => Ok(Value::Null),
            JsonValue::Object(object) => self.object_value(object, node, path),
            _ => Err(unrepresentable("expected a value node", path, node)),
        }
    }

    fn object_value(
        &self,
        object: &Map<String, JsonValue>,
        node: &JsonValue,
        path: &str,
    ) -> Result<Value> {
        self.object(node, path)?;
        let text = |key: &str| object.get(key).and_then(JsonValue::as_str);
        if object.contains_key("call") {
            self.only_keys(object, &["call", "args"], path, node)?;
            return self.value_call(object, node, path);
        }
        let tagged = |keys: &[&str]| -> Result<()> {
            if object.len() == keys.len() && keys.iter().all(|key| object.contains_key(*key)) {
                Ok(())
            } else {
                Err(unrepresentable("malformed tagged node", path, node))
            }
        };
        if object.contains_key("string") {
            tagged(&["string"])?;
            let string = text("string")
                .ok_or_else(|| unrepresentable("'string' must be text", path, node))?;
            return Ok(Value::string(string));
        }
        if object.contains_key("localizedString") {
            tagged(&["localizedString"])?;
            let id = text("localizedString")
                .ok_or_else(|| unrepresentable("'localizedString' must be an id", path, node))?;
            if !self.catalog.localized_strings().any(|entry| entry.id == id) {
                return Err(StatementContentError::UnknownId {
                    id: id.to_string(),
                    path: path.to_string(),
                    node: node.clone(),
                });
            }
            return Ok(Value::LocalizedString(id.to_string()));
        }
        if object.contains_key("enum") {
            tagged(&["enum", "member"])?;
            let (Some(domain), Some(member)) = (text("enum"), text("member")) else {
                return Err(unrepresentable("enum and member must be text", path, node));
            };
            let known = self
                .catalog
                .enum_domain(domain)
                .is_some_and(|domain| domain.members.iter().any(|m| m.member == member));
            if !known {
                return Err(StatementContentError::UnknownId {
                    id: format!("{domain}.{member}"),
                    path: path.to_string(),
                    node: node.clone(),
                });
            }
            return Ok(Value::Enum {
                value_type: domain.to_string(),
                value: member.to_string(),
            });
        }
        if object.contains_key("ambiguous") {
            return Err(unrepresentable(
                "an ambiguous enum member has no canonical identity to write",
                path,
                node,
            ));
        }
        if object.contains_key("subroutine") {
            tagged(&["subroutine"])?;
            let name = text("subroutine")
                .ok_or_else(|| unrepresentable("'subroutine' must be a name", path, node))?;
            return Ok(Value::Subroutine(name.to_string()));
        }
        Err(unrepresentable("unknown node form", path, node))
    }

    fn value_call(
        &self,
        object: &Map<String, JsonValue>,
        node: &JsonValue,
        path: &str,
    ) -> Result<Value> {
        let (id, args) = self.call_parts(object, node, path)?;
        let arg_path = |index: usize| child(&child(path, "args"), index);
        let count_error = |min: usize, max: Option<usize>| StatementContentError::ArgumentCount {
            id: id.to_string(),
            min,
            max,
            supplied: args.len(),
            path: path.to_string(),
            node: node.clone(),
        };
        match id {
            GLOBAL_VARIABLE_CALL => {
                if args.len() != 1 {
                    return Err(count_error(1, Some(1)));
                }
                Ok(Value::GlobalVariable(
                    self.variable(&args[0], &arg_path(0))?,
                ))
            }
            PLAYER_VARIABLE_CALL => {
                if args.len() != 2 {
                    return Err(count_error(2, Some(2)));
                }
                Ok(Value::player_variable(
                    self.value(&args[0], &arg_path(0))?,
                    self.variable(&args[1], &arg_path(1))?,
                ))
            }
            MEMBER_ACCESS_CALL => {
                if !(2..=3).contains(&args.len()) {
                    return Err(count_error(2, Some(3)));
                }
                let values = self.values(args, path)?;
                if !matches!(values[1], Value::String(_)) {
                    return Err(unrepresentable(
                        "memberAccess member must be a string",
                        &arg_path(1),
                        &args[1],
                    ));
                }
                Ok(Value::Call {
                    name: id.to_string(),
                    args: values,
                })
            }
            _ => {
                let values = self.values(args, path)?;
                // A modify operand (`add`, `appendToArray`, …) shares its id
                // with a value of the same name; with no arguments it is the
                // operand.
                let operand = args.is_empty() && wir::is_operation_helper(id);
                if !operand && let Some(entry) = self.catalog.entry(Kind::Value, id) {
                    self.check_arity(entry, &values, node, path)?;
                    return Ok(self.shaped_call(id, values));
                }
                if self.catalog.entry(Kind::Operator, id).is_some()
                    || wir::is_canonical_helper_call(id, args.len())
                {
                    return Ok(Value::Call {
                        name: id.to_string(),
                        args: values,
                    });
                }
                if wir::is_canonical_helper(id) {
                    return Err(count_error(0, Some(0)));
                }
                Err(StatementContentError::UnknownId {
                    id: id.to_string(),
                    path: path.to_string(),
                    node: node.clone(),
                })
            }
        }
    }

    fn values(&self, args: &[JsonValue], path: &str) -> Result<Vec<Value>> {
        args.iter()
            .enumerate()
            .map(|(index, arg)| self.value(arg, &child(&child(path, "args"), index)))
            .collect()
    }

    /// The model shape the parser gives a catalog value call: `array` and
    /// `vector` have their own variants.
    fn shaped_call(&self, id: &str, mut values: Vec<Value>) -> Value {
        match (id, values.len()) {
            ("eventPlayer", 0) => Value::EventPlayer,
            ("array", _) => Value::Array(values),
            ("vector", 3) => {
                let z = Box::new(values.pop().expect("three values"));
                let y = Box::new(values.pop().expect("three values"));
                let x = Box::new(values.pop().expect("three values"));
                Value::Vector { x, y, z }
            }
            _ => Value::call(id, values),
        }
    }
}
