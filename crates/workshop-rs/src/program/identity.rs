use std::collections::hash_map::DefaultHasher;
use std::fmt::Debug;
use std::hash::Hasher;

use super::{
    Action, Condition, ModifyOp, Program, ProgramProvenance, Rule, Value, ValueProvenance,
    action_argument_values, value_children,
};

pub(super) type NodeIdentity = u64;

pub(super) fn rule_identity(rule: &Rule) -> NodeIdentity {
    declaration_identity(&(&rule.name, rule.disabled, &rule.event))
}

pub(super) fn condition_identity(condition: &Condition) -> NodeIdentity {
    condition_identity_from(condition, value_identity(&condition.value))
}

pub(super) fn condition_identity_from(condition: &Condition, value: NodeIdentity) -> NodeIdentity {
    let mut hasher = DefaultHasher::new();
    hasher.write_u8(condition.disabled as u8);
    hasher.write_u64(value);
    hasher.finish()
}

pub(super) fn action_identity(action: &Action) -> NodeIdentity {
    let arguments = action_argument_values(action)
        .into_iter()
        .map(value_identity)
        .collect::<Vec<_>>();
    action_identity_from(action, &arguments)
}

fn action_identity_from(action: &Action, arguments: &[NodeIdentity]) -> NodeIdentity {
    let mut hasher = DefaultHasher::new();
    feed_action(&mut hasher, action);
    for &argument in arguments {
        hasher.write_u64(argument);
    }
    hasher.finish()
}

pub(super) fn value_identity(value: &Value) -> NodeIdentity {
    let children = value_children(value)
        .into_iter()
        .map(value_identity)
        .collect::<Vec<_>>();
    value_identity_from(value, &children)
}

/// The identity of `value` given the identities of its children, so a caller
/// that already walked the children does not hash each subtree again.
pub(super) fn value_identity_from(value: &Value, children: &[NodeIdentity]) -> NodeIdentity {
    let mut hasher = DefaultHasher::new();
    feed_value(&mut hasher, value);
    for &child in children {
        hasher.write_u64(child);
    }
    hasher.finish()
}

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
            let value = refresh_children(record, &condition.value);
            record.identity = condition_identity_from(condition, value);
        }
        for (record, action) in record.actions.iter_mut().zip(&rule.actions) {
            let arguments = action_argument_values(action)
                .into_iter()
                .enumerate()
                .map(|(index, value)| match record.arguments.get_mut(index) {
                    Some(argument) => refresh_value(argument, value),
                    None => value_identity(value),
                })
                .collect::<Vec<_>>();
            record.identity = action_identity_from(action, &arguments);
        }
    }
}

fn refresh_value(record: &mut ValueProvenance, value: &Value) -> NodeIdentity {
    record.identity = refresh_children(record, value);
    record.identity
}

/// Refresh the child records of `record` and return the identity of `value`.
fn refresh_children(record: &mut ValueProvenance, value: &Value) -> NodeIdentity {
    let children = value_children(value)
        .into_iter()
        .enumerate()
        .map(|(index, child)| match record.children.get_mut(index) {
            Some(record) => refresh_value(record, child),
            None => value_identity(child),
        })
        .collect::<Vec<_>>();
    value_identity_from(value, &children)
}

pub(super) fn declaration_identity(node: &impl Debug) -> NodeIdentity {
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
