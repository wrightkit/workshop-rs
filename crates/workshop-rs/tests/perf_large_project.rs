//! Stage-level timing for the pinned large real-project input.
//!
//! Run with:
//! `cargo test -p workshop-rs --release --lib perf_large_project -- --ignored --nocapture`

use super::common::en;
use super::internal;
use std::hint::black_box;
use std::time::{Duration, Instant};
use workshop_rs::catalog::Catalog;
use workshop_rs::program::{Action, Program, Value};

const BASTION: &str = include_str!("fixtures/real-projects/bastion.ow");

fn time<T>(label: &str, iterations: usize, mut operation: impl FnMut() -> T) -> Duration {
    // warmup
    black_box(operation());
    let start = Instant::now();
    for _ in 0..iterations {
        black_box(operation());
    }
    let elapsed = start.elapsed();
    println!(
        "{label:<42} total {elapsed:?}  ({:?}/op)",
        elapsed / iterations as u32
    );
    elapsed
}

fn value_children(value: &Value) -> Vec<&Value> {
    match value {
        Value::Array(values) => values.iter().collect(),
        Value::Vector { x, y, z } => vec![x.as_ref(), y.as_ref(), z.as_ref()],
        Value::PlayerVariable { player, .. } => vec![player.as_ref()],
        Value::Call { args, .. } => args.iter().collect(),
        _ => Vec::new(),
    }
}

fn action_argument_values(action: &Action) -> Vec<&Value> {
    match action {
        Action::SetGlobalVariable { value, .. } | Action::ModifyGlobalVariable { value, .. } => {
            vec![value]
        }
        Action::SetPlayerVariable { player, value, .. }
        | Action::ModifyPlayerVariable { player, value, .. } => vec![player, value],
        Action::AssignMember { target, value, .. } => vec![target, value],
        Action::If { condition } | Action::ElseIf { condition } | Action::While { condition } => {
            vec![condition]
        }
        Action::ForGlobalVariable {
            start, stop, step, ..
        } => vec![start, stop, step],
        Action::ForPlayerVariable {
            player,
            start,
            stop,
            step,
            ..
        } => vec![player, start, stop, step],
        Action::Call { args, .. } => args.iter().collect(),
        Action::Disabled { action } => action_argument_values(action),
        Action::CallSubroutine { .. } | Action::Else | Action::End => Vec::new(),
    }
}

/// Traverse every public value node the way a consumer resolving every node's
/// span does: one root-to-node lookup per node.
fn span_traversal(program: &Program) -> usize {
    let mut lookups = 0;
    for (rule_index, rule) in program.rules.iter().enumerate() {
        for (condition_index, condition) in rule.conditions.iter().enumerate() {
            let mut stack = vec![(&condition.value, Vec::new())];
            while let Some((value, path)) = stack.pop() {
                lookups += 1;
                black_box(program.condition_value_span(rule_index, condition_index, &path));
                for (index, child) in value_children(value).into_iter().enumerate() {
                    let mut child_path = path.clone();
                    child_path.push(index);
                    stack.push((child, child_path));
                }
            }
        }
        for (action_index, action) in rule.actions.iter().enumerate() {
            for (argument, value) in action_argument_values(action).into_iter().enumerate() {
                let mut stack = vec![(value, Vec::new())];
                while let Some((value, path)) = stack.pop() {
                    lookups += 1;
                    black_box(program.action_argument_value_span(
                        rule_index,
                        action_index,
                        argument,
                        &path,
                    ));
                    for (index, child) in value_children(value).into_iter().enumerate() {
                        let mut child_path = path.clone();
                        child_path.push(index);
                        stack.push((child, child_path));
                    }
                }
            }
        }
    }
    lookups
}

#[test]
#[ignore = "performance measurement"]
fn large_project_stages() {
    let catalog = Catalog::builtin().expect("builtin catalog");
    let locale = en();
    let iterations = 3;

    println!(
        "\n=== bastion.ow stage timing ({} bytes) ===",
        BASTION.len()
    );

    time("tokenize", iterations, || {
        crate::frontend::lexer::tokenize(BASTION).expect("tokenize")
    });

    time("parse_wir (lex+parse)", iterations, || {
        internal::parse_in(BASTION, &locale)
    });

    let wir = internal::parse_in(BASTION, &locale);
    println!(
        "wir values={} actions={} rules={}",
        wir.values.len(),
        wir.actions.len(),
        wir.rules.len()
    );

    time("Program::from_wir", iterations, || {
        Program::from_wir(wir.clone()).expect("from_wir")
    });

    time("parse (end to end)", iterations, || {
        workshop_rs::parser::parse(BASTION, &catalog, &locale).expect("parse")
    });

    let program = workshop_rs::parser::parse(BASTION, &catalog, &locale).expect("parse");

    time("program.to_wir", iterations, || {
        program.to_wir().expect("to_wir")
    });

    time("program.validate", iterations, || {
        program.validate().expect("validate")
    });

    time("emit", iterations, || {
        workshop_rs::emitter::emit(&program, &catalog, &locale).expect("emit")
    });

    time("emit_wir", iterations, || internal::emit_in(&wir, &locale));

    time("semantic_issues", iterations, || {
        program.semantic_issues(&catalog)
    });

    time("element_count", iterations, || {
        program.element_count(&catalog).expect("element_count")
    });

    time("dump", iterations, || program.dump());

    time("span traversal (per-node path lookup)", 1, || {
        span_traversal(&program)
    });
    println!("span lookups = {}", span_traversal(&program));
}
