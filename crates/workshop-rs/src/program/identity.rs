use std::fmt::Debug;
use std::hash::{BuildHasher, Hasher};

use ahash::{AHasher, RandomState};

use super::{
    Action, Condition, Event, EventTarget, EventTeam, ModifyOp, Program, ProgramProvenance, Rule,
    Value, ValueProvenance, action_argument_values, value_children,
};

pub(super) type NodeIdentity = u64;

/// Fixed seeds (digits of pi) keep identities equal across runs and builds. ahash
/// draws runtime-random keys in `Default` when a dependency enables `runtime-rng`
/// (test builds do, through `jsonschema`), so the seeds are passed explicitly.
const IDENTITY_SEEDS: RandomState = RandomState::with_seeds(
    0x243f_6a88_85a3_08d3,
    0x1319_8a2e_0370_7344,
    0xa409_3822_299f_31d0,
    0x082e_fa98_ec4e_6c89,
);

fn identity_hasher() -> AHasher {
    IDENTITY_SEEDS.build_hasher()
}

pub(super) fn rule_identity(rule: &Rule) -> NodeIdentity {
    let mut hasher = identity_hasher();
    feed_str(&mut hasher, &rule.name);
    hasher.write_u8(rule.disabled as u8);
    feed_event(&mut hasher, &rule.event);
    hasher.finish()
}

pub(super) fn condition_identity(condition: &Condition) -> NodeIdentity {
    condition_identity_from(condition, value_identity(&condition.value))
}

pub(super) fn condition_identity_from(condition: &Condition, value: NodeIdentity) -> NodeIdentity {
    let mut hasher = identity_hasher();
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
    let mut hasher = identity_hasher();
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
    let mut hasher = identity_hasher();
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
    struct Sink<'a>(&'a mut AHasher);
    impl std::fmt::Write for Sink<'_> {
        fn write_str(&mut self, text: &str) -> std::fmt::Result {
            self.0.write(text.as_bytes());
            Ok(())
        }
    }
    let mut hasher = identity_hasher();
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

fn feed_event(hasher: &mut impl Hasher, event: &Event) {
    match event {
        Event::Global => hasher.write_u8(0),
        Event::EachPlayer => hasher.write_u8(1),
        Event::EachPlayerWithFilters { team, target } => {
            hasher.write_u8(2);
            feed_team(hasher, *team);
            feed_target(hasher, target);
        }
        Event::Player { kind, team, target } => {
            hasher.write_u8(3);
            hasher.write_u8(*kind as u8);
            feed_team(hasher, *team);
            feed_target(hasher, target);
        }
        Event::Subroutine(name) => {
            hasher.write_u8(4);
            feed_str(hasher, name);
        }
    }
}

fn feed_team(hasher: &mut impl Hasher, team: EventTeam) {
    hasher.write_u8(team as u8);
}

fn feed_target(hasher: &mut impl Hasher, target: &EventTarget) {
    match target {
        EventTarget::All => hasher.write_u8(0),
        EventTarget::Slot(slot) => {
            hasher.write_u8(1);
            hasher.write_u8(*slot);
        }
        EventTarget::Hero(hero) => {
            hasher.write_u8(2);
            feed_str(hasher, hero);
        }
    }
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

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;
    use crate::catalog::Catalog;
    use crate::program::PlayerEventKind;
    use crate::{detect, parser};

    const KINDS: [PlayerEventKind; 11] = [
        PlayerEventKind::DealtDamage,
        PlayerEventKind::DealtFinalBlow,
        PlayerEventKind::DealtHealing,
        PlayerEventKind::DealtKnockback,
        PlayerEventKind::Died,
        PlayerEventKind::EarnedElimination,
        PlayerEventKind::Joined,
        PlayerEventKind::Left,
        PlayerEventKind::ReceivedHealing,
        PlayerEventKind::ReceivedKnockback,
        PlayerEventKind::TookDamage,
    ];
    const TEAMS: [EventTeam; 3] = [EventTeam::All, EventTeam::Team1, EventTeam::Team2];

    fn rule(name: &str, disabled: bool, event: Event) -> Rule {
        let mut rule = Rule::new(name, event);
        rule.disabled = disabled;
        rule
    }

    fn targets() -> Vec<EventTarget> {
        vec![
            EventTarget::All,
            EventTarget::Slot(0),
            EventTarget::Slot(1),
            EventTarget::Hero("ana".into()),
            EventTarget::Hero(String::new()),
        ]
    }

    /// Every `Event` shape: each player kind, team, and target combination.
    fn every_event() -> Vec<Event> {
        let mut events = vec![
            Event::Global,
            Event::EachPlayer,
            Event::Subroutine("a".into()),
            Event::Subroutine("b".into()),
        ];
        for team in TEAMS {
            for target in targets() {
                events.push(Event::EachPlayerWithFilters { team, target });
            }
        }
        for kind in KINDS {
            for team in TEAMS {
                for target in targets() {
                    events.push(Event::Player { kind, team, target });
                }
            }
        }
        events
    }

    fn identities(program: &Program) -> Vec<NodeIdentity> {
        program
            .rules
            .iter()
            .flat_map(|rule| {
                std::iter::once(rule_identity(rule))
                    .chain(rule.conditions.iter().map(condition_identity))
                    .chain(rule.actions.iter().map(action_identity))
            })
            .collect()
    }

    #[test]
    fn rule_identity_separates_every_event_name_and_flag() {
        let mut rules = Vec::new();
        for name in ["r", "s"] {
            for disabled in [false, true] {
                for event in every_event() {
                    rules.push(rule(name, disabled, event));
                }
            }
        }
        // Name and string-valued fields must not alias across their boundary.
        rules.push(rule(
            "a",
            false,
            Event::EachPlayerWithFilters {
                team: EventTeam::Team1,
                target: EventTarget::Hero("bc".into()),
            },
        ));
        rules.push(rule(
            "ab",
            false,
            Event::EachPlayerWithFilters {
                team: EventTeam::Team1,
                target: EventTarget::Hero("c".into()),
            },
        ));
        rules.push(rule("a", false, Event::Subroutine("bc".into())));
        rules.push(rule("ab", false, Event::Subroutine("c".into())));
        let identities: HashSet<_> = rules.iter().map(rule_identity).collect();
        assert_eq!(identities.len(), rules.len());
    }

    #[test]
    fn identities_repeat_within_a_process_and_across_rebuilt_programs() {
        let event = rule("r", false, Event::Global);
        assert_eq!(rule_identity(&event), rule_identity(&event));

        let catalog = Catalog::builtin().expect("built-in catalog");
        let input = include_str!("../../tests/fixtures/real-projects/bastion.ow");
        let locale = detect::resolve_locale(input, &catalog, None).expect("locale");
        let first = parser::parse_with_context(input, &catalog, &locale, &catalog).expect("parse");
        let second = parser::parse_with_context(input, &catalog, &locale, &catalog).expect("parse");
        let identities_first = identities(&first);
        assert!(!identities_first.is_empty());
        assert_eq!(identities_first, identities(&second));
    }

    /// Identities of a fixed set of rules and one value, in a stable order.
    fn fixed_identities() -> Vec<NodeIdentity> {
        let mut identities: Vec<NodeIdentity> = every_event()
            .into_iter()
            .map(|event| rule_identity(&rule("r", false, event)))
            .collect();
        identities.push(value_identity(&Value::String("ana".into())));
        identities
    }

    /// Re-runs this test binary as a child process. Each process draws its own
    /// ahash keys under `runtime-rng`, which test builds enable, so equal values
    /// show that the explicit seeds, not the process, decide identities.
    #[test]
    fn identities_match_across_processes() {
        const CHILD_MARKER: &str = "WORKSHOP_RS_IDENTITY_CHILD";
        if std::env::var_os(CHILD_MARKER).is_some() {
            println!("IDENTITIES {:?}", fixed_identities());
            return;
        }
        let output = std::process::Command::new(std::env::current_exe().expect("test binary"))
            .args([
                "--exact",
                "program::identity::tests::identities_match_across_processes",
                "--nocapture",
            ])
            .env(CHILD_MARKER, "1")
            .output()
            .expect("spawn child test process");
        assert!(
            output.status.success(),
            "child test failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        let child = stdout
            .lines()
            .find_map(|line| line.split_once("IDENTITIES ").map(|(_, rest)| rest))
            .expect("child printed its identities");
        assert_eq!(child, format!("{:?}", fixed_identities()));
    }
}
