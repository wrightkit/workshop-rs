//! Node identity digests for positional provenance records.
//!
//! A record's identity is a process-local digest of the public node's
//! content. For condition, action, and value records the content is the
//! node's whole subtree — its own scalar fields plus the deep identity of
//! every value argument or child — because the node's authored span covers
//! the whole expression text. Rules and declarations keep the digest at
//! their own scalar fields: their conditions and actions live in positional
//! tables screened by count, the same way the public model exposes them.
//!
//! Identities are never serialized, so the digest format needs no stability
//! beyond a single process.

use std::collections::hash_map::DefaultHasher;
use std::fmt::Debug;
use std::hash::Hasher;

use super::{
    Action, Condition, ModifyOp, Program, ProgramProvenance, Rule, Value, ValueProvenance,
    action_argument_values, value_children,
};

/// The identity of a public node: a digest of the content a provenance
/// record describes. A record's own fields are returned only while the node
/// at its position still has the recorded identity; descending into a
/// record's positional child table only requires the table count to match.
pub(super) type NodeIdentity = u64;

/// A rule's identity is its own declared content — name, disabled flag, and
/// event. Conditions and actions are positional child tables screened by
/// count and are not part of it.
pub(super) fn rule_identity(rule: &Rule) -> NodeIdentity {
    scalar_identity(&(&rule.name, rule.disabled, &rule.event))
}

pub(super) fn condition_identity(condition: &Condition) -> NodeIdentity {
    let mut hasher = DefaultHasher::new();
    hasher.write_u8(condition.disabled as u8);
    hasher.write_u64(value_identity(&condition.value));
    hasher.finish()
}

/// An action's identity is its own scalar fields plus the deep identity of
/// every direct value argument: `Wait(1)` and `Wait(2)` are different
/// actions, and mutating an argument makes the whole action record stale.
pub(super) fn action_identity(action: &Action) -> NodeIdentity {
    let mut hasher = DefaultHasher::new();
    feed_action(&mut hasher, action);
    for value in action_argument_values(action) {
        hasher.write_u64(value_identity(value));
    }
    hasher.finish()
}

/// A value's identity is its own scalar fields plus the deep identity of
/// every child, covering the expression's whole subtree.
pub(super) fn value_identity(value: &Value) -> NodeIdentity {
    let mut hasher = DefaultHasher::new();
    feed_value(&mut hasher, value);
    for child in value_children(value) {
        hasher.write_u64(value_identity(child));
    }
    hasher.finish()
}

/// The identity of a declaration node, which has no positional children.
pub(super) fn declaration_identity(node: &impl Debug) -> NodeIdentity {
    scalar_identity(node)
}

/// Record every attached mapping's node identity so records are only
/// returned for the node content they were attached to.
pub(super) fn record_identities(provenance: &mut ProgramProvenance, program: &Program) {
    for (record, variable) in provenance
        .global_variables
        .iter_mut()
        .zip(&program.global_variables)
    {
        record.identity = declaration_identity(variable);
    }
    for (record, variable) in provenance
        .player_variables
        .iter_mut()
        .zip(&program.player_variables)
    {
        record.identity = declaration_identity(variable);
    }
    for (record, subroutine) in provenance.subroutines.iter_mut().zip(&program.subroutines) {
        record.identity = declaration_identity(subroutine);
    }
    for (record, rule) in provenance.rules.iter_mut().zip(&program.rules) {
        record.identity = rule_identity(rule);
        for (record, condition) in record.conditions.iter_mut().zip(&rule.conditions) {
            record.identity = condition_identity(condition);
            refresh_children(record, &condition.value);
        }
        for (record, action) in record.actions.iter_mut().zip(&rule.actions) {
            record.identity = action_identity(action);
            for (argument, value) in record
                .arguments
                .iter_mut()
                .zip(action_argument_values(action))
            {
                refresh_value(argument, value);
            }
        }
    }
}

fn refresh_value(record: &mut ValueProvenance, value: &Value) {
    record.identity = value_identity(value);
    refresh_children(record, value);
}

fn refresh_children(record: &mut ValueProvenance, value: &Value) {
    for (child, value) in record.children.iter_mut().zip(value_children(value)) {
        refresh_value(child, value);
    }
}

fn scalar_identity(node: &impl Debug) -> NodeIdentity {
    struct Sink<'a>(&'a mut DefaultHasher);
    impl std::fmt::Write for Sink<'_> {
        fn write_str(&mut self, text: &str) -> std::fmt::Result {
            self.0.write(text.as_bytes());
            Ok(())
        }
    }
    let mut hasher = DefaultHasher::new();
    let mut sink = Sink(&mut hasher);
    std::fmt::write(&mut sink, format_args!("{node:?}")).expect("a hash sink never fails");
    hasher.finish()
}

fn feed_str(hasher: &mut impl Hasher, text: &str) {
    hasher.write_usize(text.len());
    hasher.write(text.as_bytes());
}

/// The action's own scalar fields; its argument values are added by
/// [`action_identity`] as deep child digests.
fn feed_action(hasher: &mut impl Hasher, action: &Action) {
    match action {
        Action::SetGlobalVariable { variable, .. } => {
            hasher.write_u8(0);
            feed_str(hasher, variable);
        }
        Action::ModifyGlobalVariable { variable, op, .. } => {
            hasher.write_u8(1);
            feed_str(hasher, variable);
            feed_op(hasher, *op);
        }
        Action::SetPlayerVariable { variable, .. } => {
            hasher.write_u8(2);
            feed_str(hasher, variable);
        }
        Action::ModifyPlayerVariable { variable, op, .. } => {
            hasher.write_u8(3);
            feed_str(hasher, variable);
            feed_op(hasher, *op);
        }
        Action::AssignMember { op, .. } => {
            hasher.write_u8(4);
            hasher.write_u8(op.is_some() as u8);
            if let Some(op) = op {
                feed_op(hasher, *op);
            }
        }
        Action::CallSubroutine { subroutine } => {
            hasher.write_u8(5);
            feed_str(hasher, subroutine);
        }
        Action::If { .. } => hasher.write_u8(6),
        Action::ElseIf { .. } => hasher.write_u8(7),
        Action::Else => hasher.write_u8(8),
        Action::While { .. } => hasher.write_u8(9),
        Action::ForGlobalVariable { variable, .. } => {
            hasher.write_u8(10);
            feed_str(hasher, variable);
        }
        Action::ForPlayerVariable { variable, .. } => {
            hasher.write_u8(11);
            feed_str(hasher, variable);
        }
        Action::End => hasher.write_u8(12),
        Action::Disabled { action } => {
            hasher.write_u8(13);
            feed_action(hasher, action);
        }
        Action::Call { name, .. } => {
            hasher.write_u8(14);
            feed_str(hasher, name);
        }
    }
}

fn feed_op(hasher: &mut impl Hasher, op: ModifyOp) {
    std::hash::Hash::hash(&std::mem::discriminant(&op), hasher);
}

/// The value's own scalar fields; its children are added by
/// [`value_identity`] as deep child digests.
fn feed_value(hasher: &mut impl Hasher, value: &Value) {
    match value {
        Value::Number(number) => {
            hasher.write_u8(0);
            hasher.write_u64(number.to_bits());
        }
        Value::String(text) => {
            hasher.write_u8(1);
            feed_str(hasher, text);
        }
        Value::LocalizedString(text) => {
            hasher.write_u8(2);
            feed_str(hasher, text);
        }
        Value::Bool(flag) => {
            hasher.write_u8(3);
            hasher.write_u8(*flag as u8);
        }
        Value::Null => hasher.write_u8(4),
        Value::Array(_) => hasher.write_u8(5),
        Value::Vector { .. } => hasher.write_u8(6),
        Value::Enum { value_type, value } => {
            hasher.write_u8(7);
            feed_str(hasher, value_type);
            feed_str(hasher, value);
        }
        Value::GlobalVariable(name) => {
            hasher.write_u8(8);
            feed_str(hasher, name);
        }
        Value::PlayerVariable { variable, .. } => {
            hasher.write_u8(9);
            feed_str(hasher, variable);
        }
        Value::Subroutine(name) => {
            hasher.write_u8(10);
            feed_str(hasher, name);
        }
        Value::EventPlayer => hasher.write_u8(11),
        Value::Call { name, .. } => {
            hasher.write_u8(12);
            feed_str(hasher, name);
        }
    }
}
