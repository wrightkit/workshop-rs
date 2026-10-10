//! Structured rule content in the versioned `workshop-rs/rule-content-v1`
//! format.
//!
//! [`rule_content`] renders one [`Rule`] as the canonical call tree defined
//! by [`RULE_CONTENT_V1_SCHEMA`]. Every value and action is a call node named
//! by its catalog id; literals, strings, enum members, and declared variable
//! or subroutine names use the tagged forms the schema defines. Output is
//! deterministic for a given rule and independent of the source locale.
//!
//! Constructs the canonical model specializes but the catalog does not name
//! use the reserved ids declared in the schema: `globalVariable` and
//! `playerVariable` for variable reads, `assignMember` and `modifyMember` for
//! member-assignment actions, and a zero-argument operator call such as
//! `{"call":"add","args":[]}` for the modify operand.

use serde_json::{Map, Value as JsonValue, json};

use crate::core::error::{Result, WorkshopError};
use crate::program::{Action, Condition, Event, EventTarget, EventTeam, ModifyOp, Rule, Value};
use crate::wir::AMBIGUOUS_ENUM_CALL;

/// The format identity every produced document carries in `"format"`.
pub const RULE_CONTENT_V1: &str = "workshop-rs/rule-content-v1";

/// The published JSON Schema for [`RULE_CONTENT_V1`] documents.
pub const RULE_CONTENT_V1_SCHEMA: &str =
    include_str!("../../../../schemas/rule-content-v1.schema.json");

/// The reserved call id for a global variable read.
pub const GLOBAL_VARIABLE_CALL: &str = "globalVariable";
/// The reserved call id for a player variable read.
pub const PLAYER_VARIABLE_CALL: &str = "playerVariable";
/// The reserved call id for a plain member assignment (`target = value`).
pub const ASSIGN_MEMBER_CALL: &str = "assignMember";
/// The reserved call id for a compound member assignment (`target op= value`).
pub const MODIFY_MEMBER_CALL: &str = "modifyMember";

impl Rule {
    /// Render this rule in the [`RULE_CONTENT_V1`] format.
    ///
    /// The result is deterministic: it depends only on the rule's canonical
    /// content, never on source spelling or locale. An internal sentinel
    /// that has no form in the format, such as an unresolved ambiguous enum,
    /// fails the whole render rather than leaking into the document.
    pub fn content(&self) -> Result<JsonValue> {
        rule_content(self)
    }
}

/// Render one rule in the [`RULE_CONTENT_V1`] format; see [`Rule::content`].
pub fn rule_content(rule: &Rule) -> Result<JsonValue> {
    let mut document = Map::new();
    document.insert("format".to_string(), RULE_CONTENT_V1.into());
    document.insert("event".to_string(), event_node(&rule.event));
    document.insert("disabled".to_string(), rule.disabled.into());
    document.insert(
        "conditions".to_string(),
        rule.conditions
            .iter()
            .map(condition_node)
            .collect::<Result<_>>()?,
    );
    document.insert(
        "actions".to_string(),
        rule.actions
            .iter()
            .map(|action| action_node(action, false))
            .collect::<Result<_>>()?,
    );
    Ok(JsonValue::Object(document))
}

fn event_node(event: &Event) -> JsonValue {
    let mut node = Map::new();
    node.insert("id".to_string(), event.catalog_id().into());
    match event {
        Event::EachPlayerWithFilters { team, target } | Event::Player { team, target, .. } => {
            node.insert("team".to_string(), team_id(*team).into());
            node.insert("player".to_string(), target_id(target).into());
        }
        Event::Subroutine(name) => {
            node.insert("subroutine".to_string(), name.clone().into());
        }
        Event::Global | Event::EachPlayer => {}
    }
    JsonValue::Object(node)
}

/// The `EventTeam` filter id: the canonical `EventTeam` enum member id.
fn team_id(team: EventTeam) -> &'static str {
    match team {
        EventTeam::All => "ALL",
        EventTeam::Team1 => "TEAM_1",
        EventTeam::Team2 => "TEAM_2",
    }
}

/// The `EventTarget` filter id: the canonical `EventPlayer` member id for
/// `ALL`/`SLOT_n`, or the `Hero` member id for a hero target.
fn target_id(target: &EventTarget) -> String {
    match target {
        EventTarget::All => "ALL".to_string(),
        EventTarget::Slot(slot) => format!("SLOT_{slot}"),
        EventTarget::Hero(hero) => hero.clone(),
    }
}

fn condition_node(condition: &Condition) -> Result<JsonValue> {
    let mut node = Map::new();
    node.insert("value".to_string(), value_node(&condition.value)?);
    if condition.disabled {
        node.insert("disabled".to_string(), true.into());
    }
    Ok(JsonValue::Object(node))
}

fn action_node(action: &Action, disabled: bool) -> Result<JsonValue> {
    let (call, args) = match action {
        Action::SetGlobalVariable { variable, value } => (
            "setGlobalVariable",
            vec![variable_node(variable), value_node(value)?],
        ),
        Action::ModifyGlobalVariable {
            variable,
            op,
            value,
        } => (
            "modifyGlobalVariable",
            vec![
                variable_node(variable),
                modify_op_node(*op),
                value_node(value)?,
            ],
        ),
        Action::SetPlayerVariable {
            player,
            variable,
            value,
        } => (
            "setPlayerVariable",
            vec![
                value_node(player)?,
                variable_node(variable),
                value_node(value)?,
            ],
        ),
        Action::ModifyPlayerVariable {
            player,
            variable,
            op,
            value,
        } => (
            "modifyPlayerVariable",
            vec![
                value_node(player)?,
                variable_node(variable),
                modify_op_node(*op),
                value_node(value)?,
            ],
        ),
        Action::AssignMember { target, op, value } => {
            let args = match op {
                None => vec![value_node(target)?, value_node(value)?],
                Some(op) => vec![value_node(target)?, modify_op_node(*op), value_node(value)?],
            };
            (
                if op.is_some() {
                    MODIFY_MEMBER_CALL
                } else {
                    ASSIGN_MEMBER_CALL
                },
                args,
            )
        }
        Action::CallSubroutine { subroutine } => {
            ("callSubroutine", vec![subroutine_node(subroutine)])
        }
        Action::If { condition } => ("if", vec![value_node(condition)?]),
        Action::ElseIf { condition } => ("elseIf", vec![value_node(condition)?]),
        Action::Else => ("else", Vec::new()),
        Action::While { condition } => ("while", vec![value_node(condition)?]),
        Action::ForGlobalVariable {
            variable,
            start,
            stop,
            step,
        } => (
            "forGlobalVariable",
            vec![
                variable_node(variable),
                value_node(start)?,
                value_node(stop)?,
                value_node(step)?,
            ],
        ),
        Action::ForPlayerVariable {
            player,
            variable,
            start,
            stop,
            step,
        } => (
            "forPlayerVariable",
            vec![
                value_node(player)?,
                variable_node(variable),
                value_node(start)?,
                value_node(stop)?,
                value_node(step)?,
            ],
        ),
        Action::End => ("end", Vec::new()),
        Action::Disabled { action } => return action_node(action, true),
        Action::Call { name, args } => (
            name.as_str(),
            args.iter().map(value_node).collect::<Result<_>>()?,
        ),
    };
    reject_sentinel(call)?;
    let mut node = Map::new();
    node.insert("call".to_string(), call.into());
    node.insert("args".to_string(), JsonValue::Array(args));
    if disabled {
        node.insert("disabled".to_string(), true.into());
    }
    Ok(JsonValue::Object(node))
}

fn value_node(value: &Value) -> Result<JsonValue> {
    Ok(match value {
        Value::Number(number) => serde_json::Number::from_f64(*number)
            .ok_or_else(|| {
                WorkshopError::unsupported(
                    format!("rule-content-v1 cannot express non-finite number {number}"),
                    None,
                )
            })?
            .into(),
        Value::Bool(value) => (*value).into(),
        Value::Null => JsonValue::Null,
        Value::String(text) => json!({ "string": text }),
        Value::LocalizedString(id) => json!({ "localizedString": id }),
        Value::Enum { value_type, value } => {
            json!({ "enum": value_type, "member": value })
        }
        Value::GlobalVariable(variable) => {
            call_node(GLOBAL_VARIABLE_CALL, vec![variable_node(variable)])
        }
        Value::PlayerVariable { player, variable } => call_node(
            PLAYER_VARIABLE_CALL,
            vec![value_node(player)?, variable_node(variable)],
        ),
        Value::Subroutine(name) => subroutine_node(name),
        Value::EventPlayer => call_node("eventPlayer", Vec::new()),
        Value::Array(elements) => call_node(
            "array",
            elements.iter().map(value_node).collect::<Result<_>>()?,
        ),
        Value::Vector { x, y, z } => call_node(
            "vector",
            vec![value_node(x)?, value_node(y)?, value_node(z)?],
        ),
        Value::Call { name, args } => {
            reject_sentinel(name)?;
            call_node(name, args.iter().map(value_node).collect::<Result<_>>()?)
        }
    })
}

fn call_node(call: &str, args: Vec<JsonValue>) -> JsonValue {
    let mut node = Map::new();
    node.insert("call".to_string(), call.into());
    node.insert("args".to_string(), JsonValue::Array(args));
    JsonValue::Object(node)
}

fn variable_node(name: &str) -> JsonValue {
    json!({ "variable": name })
}

fn subroutine_node(name: &str) -> JsonValue {
    json!({ "subroutine": name })
}

/// The modify operand is an operator call node with no arguments, matching
/// how `ModifyOp` operands and comparison operators appear elsewhere.
fn modify_op_node(op: ModifyOp) -> JsonValue {
    call_node(op.catalog_id(), Vec::new())
}

/// Internal sentinels such as `__ambiguous_enum` have no form in
/// `rule-content-v1`; reject them instead of leaking them into the document.
fn reject_sentinel(call: &str) -> Result<()> {
    if call == AMBIGUOUS_ENUM_CALL {
        return Err(WorkshopError::unsupported(
            format!("rule-content-v1 cannot express '{call}'"),
            None,
        ));
    }
    Ok(())
}
