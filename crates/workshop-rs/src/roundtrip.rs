//! Cross-language Workshop round-trip compatibility suite.
//!
//! [`round_trip`] proves `Workshop(locale) -> WIR -> Workshop(locale) -> WIR`
//! equivalence with a recorded evidence record, and [`equivalent`] compares
//! two WIR programs structurally, ignoring presentation-only differences
//! (source spans and file paths) while preserving operations, references,
//! control flow, and values.
//!
//! The v0.2 catalog supports `en-US`, so cross-locale equivalence is
//! trivially identity for now; the suite is locale-generic so additional
//! locales (a data-pipeline change) automatically extend coverage.

use crate::wir;

use crate::catalog::{Catalog, Locale};
use crate::emitter;
use crate::parser;
use crate::signatures::{ExpectedDomain, NoExpectedDomain};

/// A recorded round-trip result with the evidence needed for a compatibility
/// report.
#[derive(Debug, Clone, PartialEq)]
pub struct RoundTripRecord {
    /// SHA-256 of the input Workshop text.
    pub input_identity: String,
    /// The locale the text was parsed and emitted in.
    pub locale: Locale,
    /// The catalog schema version used.
    pub catalog_version: u32,
    /// Whether the input parsed.
    pub parse_ok: bool,
    /// Whether the parsed program emitted.
    pub emit_ok: bool,
    /// Whether the emitted text reparsed.
    pub reparse_ok: bool,
    /// Whether the original and round-tripped WIR are equivalent.
    pub equivalent: bool,
    /// A structured failure message, when any stage failed.
    pub error: Option<String>,
}

/// Run `Workshop -> WIR -> Workshop -> WIR` and record the evidence. The
/// record is always produced; failures are captured in its `error` field.
/// Ambiguous bare enum members stay rejected (no signature context).
pub fn round_trip(input: &str, catalog: &Catalog, locale: &Locale) -> RoundTripRecord {
    round_trip_with_context(input, catalog, locale, &NoExpectedDomain)
}

/// The context-sensitive form of [`round_trip`] (#111): reparsing the emitted
/// text uses the supplied canonical signature context so an ambiguous bare
/// enum member that the emitter produced (e.g. `Chase Global Variable Over
/// Time(..., None)`) resolves to the domain the signature pins.
pub fn round_trip_with_context(
    input: &str,
    catalog: &Catalog,
    locale: &Locale,
    context: &dyn ExpectedDomain,
) -> RoundTripRecord {
    let input_identity = sha256(input);
    let mut record = RoundTripRecord {
        input_identity,
        locale: locale.clone(),
        catalog_version: catalog.schema_version,
        parse_ok: false,
        emit_ok: false,
        reparse_ok: false,
        equivalent: false,
        error: None,
    };
    let first = match parser::parse_with_context(input, catalog, locale, context) {
        Ok(program) => program,
        Err(error) => {
            record.error = Some(error.to_string());
            return record;
        }
    };
    record.parse_ok = true;
    let emitted = match emitter::emit(&first, catalog, locale) {
        Ok(text) => text,
        Err(error) => {
            record.error = Some(error.to_string());
            return record;
        }
    };
    record.emit_ok = true;
    let second = match parser::parse_with_context(&emitted, catalog, locale, context) {
        Ok(program) => program,
        Err(error) => {
            record.error = Some(error.to_string());
            return record;
        }
    };
    record.reparse_ok = true;
    record.equivalent = equivalent(&first, &second);
    record
}

/// Structural equivalence of two WIR programs: identical settings, tables,
/// rules, actions, and values, ignoring source spans and file paths.
pub fn equivalent(a: &wir::Program, b: &wir::Program) -> bool {
    if !settings_equivalent(a.settings.as_ref(), b.settings.as_ref()) {
        return false;
    }
    let globals_a: Vec<_> = a
        .global_variables
        .iter()
        .map(|v| (v.name.as_str(), v.index))
        .collect();
    let globals_b: Vec<_> = b
        .global_variables
        .iter()
        .map(|v| (v.name.as_str(), v.index))
        .collect();
    if globals_a != globals_b {
        return false;
    }
    let players_a: Vec<_> = a
        .player_variables
        .iter()
        .map(|v| (v.name.as_str(), v.index))
        .collect();
    let players_b: Vec<_> = b
        .player_variables
        .iter()
        .map(|v| (v.name.as_str(), v.index))
        .collect();
    if players_a != players_b {
        return false;
    }
    let subs_a: Vec<_> = a
        .subroutines
        .iter()
        .map(|s| (s.name.as_str(), s.index))
        .collect();
    let subs_b: Vec<_> = b
        .subroutines
        .iter()
        .map(|s| (s.name.as_str(), s.index))
        .collect();
    if subs_a != subs_b {
        return false;
    }
    // Emission intentionally drops pass-only/condition-only rules because
    // they have no executable behavior. Ignore those presentation-only
    // source rules when comparing observable semantics.
    let rules_a: Vec<_> = a
        .rules
        .iter()
        .filter(|rule| !rule.actions.is_empty())
        .collect();
    let rules_b: Vec<_> = b
        .rules
        .iter()
        .filter(|rule| !rule.actions.is_empty())
        .collect();
    if rules_a.len() != rules_b.len() {
        return false;
    }
    for (rule_a, rule_b) in rules_a.into_iter().zip(rules_b) {
        if !rule_equivalent(a, b, rule_a, rule_b) {
            return false;
        }
    }
    true
}

fn settings_equivalent(
    left: Option<&crate::settings::Settings>,
    right: Option<&crate::settings::Settings>,
) -> bool {
    match (left, right) {
        (None, None) => true,
        (Some(left), Some(right)) => nodes_equivalent(&left.children, &right.children),
        _ => false,
    }
}

fn nodes_equivalent(
    left: &[crate::settings::SettingsNode],
    right: &[crate::settings::SettingsNode],
) -> bool {
    left.len() == right.len()
        && left
            .iter()
            .zip(right)
            .all(|(left, right)| match (left, right) {
                (
                    crate::settings::SettingsNode::Workshop { children: left, .. },
                    crate::settings::SettingsNode::Workshop {
                        children: right, ..
                    },
                ) => nodes_equivalent(left, right),
                (
                    crate::settings::SettingsNode::Group {
                        name: left_name,
                        children: left_children,
                        ..
                    },
                    crate::settings::SettingsNode::Group {
                        name: right_name,
                        children: right_children,
                        ..
                    },
                ) => left_name == right_name && nodes_equivalent(left_children, right_children),
                (
                    crate::settings::SettingsNode::Number {
                        name: left_name,
                        value: left_value,
                        ..
                    },
                    crate::settings::SettingsNode::Number {
                        name: right_name,
                        value: right_value,
                        ..
                    },
                ) => left_name == right_name && float_equivalent(*left_value, *right_value),
                (
                    crate::settings::SettingsNode::Bool {
                        name: left_name,
                        value: left_value,
                        ..
                    },
                    crate::settings::SettingsNode::Bool {
                        name: right_name,
                        value: right_value,
                        ..
                    },
                ) => left_name == right_name && left_value == right_value,
                (
                    crate::settings::SettingsNode::Flag {
                        name: left_name, ..
                    },
                    crate::settings::SettingsNode::Flag {
                        name: right_name, ..
                    },
                ) => left_name == right_name,
                (
                    crate::settings::SettingsNode::String {
                        name: left_name,
                        value: left_value,
                        ..
                    },
                    crate::settings::SettingsNode::String {
                        name: right_name,
                        value: right_value,
                        ..
                    },
                ) => left_name == right_name && left_value == right_value,
                (
                    crate::settings::SettingsNode::List {
                        name: left_name,
                        elements: left_elements,
                        ..
                    },
                    crate::settings::SettingsNode::List {
                        name: right_name,
                        elements: right_elements,
                        ..
                    },
                ) => {
                    left_name == right_name
                        && left_elements.len() == right_elements.len()
                        && left_elements
                            .iter()
                            .zip(right_elements)
                            .all(|(left, right)| left.value == right.value)
                }
                (
                    crate::settings::SettingsNode::Raw {
                        name: left_name,
                        value: left_value,
                        ..
                    },
                    crate::settings::SettingsNode::Raw {
                        name: right_name,
                        value: right_value,
                        ..
                    },
                ) => left_name == right_name && left_value == right_value,
                _ => false,
            })
}

fn float_equivalent(left: f64, right: f64) -> bool {
    if left == right {
        return true;
    }
    let scale = left.abs().max(right.abs()).max(1.0);
    (left - right).abs() <= f64::EPSILON * scale * 4.0
}

fn rule_equivalent(
    a: &wir::Program,
    b: &wir::Program,
    left: &wir::Rule,
    right: &wir::Rule,
) -> bool {
    if left.name != right.name || left.disabled != right.disabled {
        return false;
    }
    let event_a = event_equivalent(a, b, &left.event, &right.event);
    if !event_a {
        return false;
    }
    if left.conditions.len() != right.conditions.len() {
        return false;
    }
    for (ca, cb) in left.conditions.iter().zip(right.conditions.iter()) {
        if !value_equivalent(a, b, *ca, *cb) {
            return false;
        }
    }
    if left.actions.len() != right.actions.len() {
        return false;
    }
    for (aa, ab) in left.actions.iter().zip(right.actions.iter()) {
        if !action_equivalent(a, b, *aa, *ab) {
            return false;
        }
    }
    true
}

fn event_equivalent(
    a: &wir::Program,
    b: &wir::Program,
    left: &wir::Event,
    right: &wir::Event,
) -> bool {
    match (left, right) {
        (wir::Event::Global, wir::Event::Global) => true,
        (wir::Event::EachPlayer, wir::Event::EachPlayer) => true,
        (wir::Event::EachPlayer, wir::Event::EachPlayerWithFilters { team, target })
        | (wir::Event::EachPlayerWithFilters { team, target }, wir::Event::EachPlayer) => {
            *team == wir::EventTeam::All && *target == wir::EventTarget::All
        }
        (
            wir::Event::EachPlayerWithFilters {
                team: team_a,
                target: target_a,
            },
            wir::Event::EachPlayerWithFilters {
                team: team_b,
                target: target_b,
            },
        ) => team_a == team_b && target_a == target_b,
        (
            wir::Event::Player {
                kind: kind_a,
                team: team_a,
                target: target_a,
            },
            wir::Event::Player {
                kind: kind_b,
                team: team_b,
                target: target_b,
            },
        ) => kind_a == kind_b && team_a == team_b && target_a == target_b,
        (wir::Event::Subroutine(sa), wir::Event::Subroutine(sb)) => {
            let name_a = a.subroutines.get(*sa).map(|s| s.name.as_str());
            let name_b = b.subroutines.get(*sb).map(|s| s.name.as_str());
            name_a == name_b
        }
        _ => false,
    }
}

fn action_equivalent(
    a: &wir::Program,
    b: &wir::Program,
    left: wir::ActionId,
    right: wir::ActionId,
) -> bool {
    let (Some(la), Some(rb)) = (a.actions.get(left), b.actions.get(right)) else {
        return false;
    };
    match (la, rb) {
        (
            wir::Action::SetGlobalVariable {
                variable: va,
                value: x,
                ..
            },
            wir::Action::SetGlobalVariable {
                variable: vb,
                value: y,
                ..
            },
        ) => {
            name_eq(a.global_variables.get(*va), b.global_variables.get(*vb))
                && value_equivalent(a, b, *x, *y)
        }
        (
            wir::Action::ModifyGlobalVariable {
                variable: va,
                op: oa,
                value: x,
                ..
            },
            wir::Action::ModifyGlobalVariable {
                variable: vb,
                op: ob,
                value: y,
                ..
            },
        ) => {
            name_eq(a.global_variables.get(*va), b.global_variables.get(*vb))
                && oa == ob
                && value_equivalent(a, b, *x, *y)
        }
        (
            wir::Action::SetPlayerVariable {
                player: pa,
                variable: va,
                value: x,
                ..
            },
            wir::Action::SetPlayerVariable {
                player: pb,
                variable: vb,
                value: y,
                ..
            },
        ) => {
            value_equivalent(a, b, *pa, *pb)
                && name_eq(a.player_variables.get(*va), b.player_variables.get(*vb))
                && value_equivalent(a, b, *x, *y)
        }
        (
            wir::Action::ModifyPlayerVariable {
                player: pa,
                variable: va,
                op: oa,
                value: x,
                ..
            },
            wir::Action::ModifyPlayerVariable {
                player: pb,
                variable: vb,
                op: ob,
                value: y,
                ..
            },
        ) => {
            value_equivalent(a, b, *pa, *pb)
                && name_eq(a.player_variables.get(*va), b.player_variables.get(*vb))
                && oa == ob
                && value_equivalent(a, b, *x, *y)
        }
        (
            wir::Action::AssignMember {
                target: ta,
                op: oa,
                value: xa,
                ..
            },
            wir::Action::AssignMember {
                target: tb,
                op: ob,
                value: xb,
                ..
            },
        ) => oa == ob && value_equivalent(a, b, *ta, *tb) && value_equivalent(a, b, *xa, *xb),
        (
            wir::Action::CallSubroutine { subroutine: sa, .. },
            wir::Action::CallSubroutine { subroutine: sb, .. },
        ) => name_eq(a.subroutines.get(*sa), b.subroutines.get(*sb)),
        (
            wir::Action::If {
                branches: ba,
                else_body: ea,
                ..
            },
            wir::Action::If {
                branches: bb,
                else_body: eb,
                ..
            },
        ) => branches_equivalent(a, b, ba, bb) && bodies_equivalent(a, b, ea, eb),
        (
            wir::Action::While {
                condition: ca,
                body: ba,
                ..
            },
            wir::Action::While {
                condition: cb,
                body: bb,
                ..
            },
        ) => value_equivalent(a, b, *ca, *cb) && actions_equivalent(a, b, ba, bb),
        (
            wir::Action::ForGlobalVariable {
                variable: va,
                start: sa,
                stop: ea,
                step: pa,
                body: ba,
                ..
            },
            wir::Action::ForGlobalVariable {
                variable: vb,
                start: sb,
                stop: eb,
                step: pb,
                body: bb,
                ..
            },
        ) => {
            name_eq(a.global_variables.get(*va), b.global_variables.get(*vb))
                && value_equivalent(a, b, *sa, *sb)
                && value_equivalent(a, b, *ea, *eb)
                && value_equivalent(a, b, *pa, *pb)
                && actions_equivalent(a, b, ba, bb)
        }
        (
            wir::Action::ForPlayerVariable {
                player: pa,
                variable: va,
                start: sa,
                stop: ea,
                step: sta,
                body: ba,
                ..
            },
            wir::Action::ForPlayerVariable {
                player: pb,
                variable: vb,
                start: sb,
                stop: eb,
                step: stb,
                body: bb,
                ..
            },
        ) => {
            value_equivalent(a, b, *pa, *pb)
                && name_eq(a.player_variables.get(*va), b.player_variables.get(*vb))
                && value_equivalent(a, b, *sa, *sb)
                && value_equivalent(a, b, *ea, *eb)
                && value_equivalent(a, b, *sta, *stb)
                && actions_equivalent(a, b, ba, bb)
        }
        (
            wir::Action::Call {
                name: na, args: xa, ..
            },
            wir::Action::Call {
                name: nb, args: xb, ..
            },
        ) => na == nb && values_equivalent(a, b, xa, xb),
        _ => false,
    }
}

fn branches_equivalent(
    a: &wir::Program,
    b: &wir::Program,
    left: &[wir::IfBranch],
    right: &[wir::IfBranch],
) -> bool {
    left.len() == right.len()
        && left.iter().zip(right.iter()).all(|(la, rb)| {
            value_equivalent(a, b, la.condition, rb.condition)
                && actions_equivalent(a, b, &la.body, &rb.body)
        })
}

fn actions_equivalent(
    a: &wir::Program,
    b: &wir::Program,
    left: &[wir::ActionId],
    right: &[wir::ActionId],
) -> bool {
    left.len() == right.len()
        && left
            .iter()
            .zip(right.iter())
            .all(|(la, rb)| action_equivalent(a, b, *la, *rb))
}

fn bodies_equivalent(
    a: &wir::Program,
    b: &wir::Program,
    left: &Option<Vec<wir::ActionId>>,
    right: &Option<Vec<wir::ActionId>>,
) -> bool {
    match (left, right) {
        (Some(la), Some(rb)) => actions_equivalent(a, b, la, rb),
        (None, None) => true,
        _ => false,
    }
}

fn value_equivalent(
    a: &wir::Program,
    b: &wir::Program,
    left: wir::ValueId,
    right: wir::ValueId,
) -> bool {
    let (Some(la), Some(rb)) = (a.values.get(left), b.values.get(right)) else {
        return false;
    };
    match (&la.value, &rb.value) {
        (wir::Value::Number { value: x, .. }, wir::Value::Number { value: y, .. }) => x == y,
        (wir::Value::String(x), wir::Value::String(y)) => x == y,
        (wir::Value::Bool(x), wir::Value::Bool(y)) => x == y,
        (wir::Value::Null, wir::Value::Null) => true,
        (wir::Value::Array(xa), wir::Value::Array(xb)) => values_equivalent(a, b, xa, xb),
        (
            wir::Value::Vector {
                x: x1,
                y: y1,
                z: z1,
            },
            wir::Value::Vector {
                x: x2,
                y: y2,
                z: z2,
            },
        ) => {
            value_equivalent(a, b, *x1, *x2)
                && value_equivalent(a, b, *y1, *y2)
                && value_equivalent(a, b, *z1, *z2)
        }
        (
            wir::Value::Enum {
                value_type: t1,
                value: v1,
            },
            wir::Value::Enum {
                value_type: t2,
                value: v2,
            },
        ) => t1 == t2 && v1 == v2,
        (wir::Value::GlobalVariable(v1), wir::Value::GlobalVariable(v2)) => {
            name_eq(a.global_variables.get(*v1), b.global_variables.get(*v2))
        }
        (
            wir::Value::PlayerVariable {
                player: p1,
                variable: v1,
            },
            wir::Value::PlayerVariable {
                player: p2,
                variable: v2,
            },
        ) => {
            value_equivalent(a, b, *p1, *p2)
                && name_eq(a.player_variables.get(*v1), b.player_variables.get(*v2))
        }
        (wir::Value::Subroutine(s1), wir::Value::Subroutine(s2)) => {
            name_eq(a.subroutines.get(*s1), b.subroutines.get(*s2))
        }
        (wir::Value::EventPlayer, wir::Value::EventPlayer) => true,
        (wir::Value::PlayerVariable { player, variable }, wir::Value::Call { name, args })
            if name == "memberAccess" && args.len() == 2 =>
        {
            let Some(wir::ValueNode {
                value: wir::Value::String(member),
                ..
            }) = b.values.get(args[1])
            else {
                return false;
            };
            value_equivalent(a, b, *player, args[0])
                && a.player_variables
                    .get(*variable)
                    .is_some_and(|value| value.name == *member)
        }
        (wir::Value::Call { name, args }, wir::Value::PlayerVariable { player, variable })
            if name == "memberAccess" && args.len() == 2 =>
        {
            let Some(wir::ValueNode {
                value: wir::Value::String(member),
                ..
            }) = a.values.get(args[1])
            else {
                return false;
            };
            value_equivalent(a, b, args[0], *player)
                && b.player_variables
                    .get(*variable)
                    .is_some_and(|value| value.name == *member)
        }
        (wir::Value::Call { name: n1, args: x1 }, wir::Value::Call { name: n2, args: x2 }) => {
            canonical_value_name(n1) == canonical_value_name(n2) && values_equivalent(a, b, x1, x2)
        }
        _ => false,
    }
}

fn canonical_value_name(name: &str) -> &str {
    match name {
        "+" => "add",
        "-" => "subtract",
        "*" => "multiply",
        "/" => "divide",
        "len" => "countOf",
        "abs" => "absoluteValue",
        "sqrt" => "squareRoot",
        _ => name,
    }
}

fn values_equivalent(
    a: &wir::Program,
    b: &wir::Program,
    left: &[wir::ValueId],
    right: &[wir::ValueId],
) -> bool {
    left.len() == right.len()
        && left
            .iter()
            .zip(right.iter())
            .all(|(la, rb)| value_equivalent(a, b, *la, *rb))
}

fn name_eq<T: Named>(left: Option<&T>, right: Option<&T>) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => left.name() == right.name(),
        (None, None) => true,
        _ => false,
    }
}

trait Named {
    fn name(&self) -> &str;
}

impl Named for wir::WorkshopVariable {
    fn name(&self) -> &str {
        &self.name
    }
}

impl Named for wir::WorkshopSubroutine {
    fn name(&self) -> &str {
        &self.name
    }
}

fn sha256(input: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    format!("{:x}", hasher.finalize())
}
