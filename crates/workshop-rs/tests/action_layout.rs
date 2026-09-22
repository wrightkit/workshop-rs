use workshop_rs::actions::{self, ActionLayoutError};
use workshop_rs::catalog::{Catalog, Locale};
use workshop_rs::emitter;
use workshop_rs::{Action, Event, Program, Rule, Value, Variable};

fn program_with_structured_actions() -> (Program, Vec<Action>) {
    let mut program = Program::new();
    program
        .global_variables
        .push(Variable::with_index("index", 0));
    program
        .player_variables
        .push(Variable::with_index("index", 0));

    let number = || Value::number(0.0);
    let condition = || Value::Bool(true);
    let wait = || Action::call("wait", [number()]);
    let actions = vec![
        Action::If {
            condition: condition(),
        },
        wait(),
        Action::Else,
        wait(),
        Action::End,
        Action::While {
            condition: condition(),
        },
        wait(),
        Action::End,
        Action::ForGlobalVariable {
            variable: "index".to_string(),
            start: number(),
            stop: number(),
            step: number(),
        },
        wait(),
        Action::End,
        Action::ForPlayerVariable {
            player: Value::EventPlayer,
            variable: "index".to_string(),
            start: number(),
            stop: number(),
            step: number(),
        },
        Action::If {
            condition: condition(),
        },
        wait(),
        Action::Else,
        wait(),
        Action::End,
        Action::End,
        wait(),
    ];
    let mut rule = Rule::new("layout", Event::Global);
    rule.actions = actions.clone();
    program.rules.push(rule);
    (program, actions)
}

#[test]
fn structured_action_widths_count_native_expansion() {
    let (program, actions) = program_with_structured_actions();
    let catalog = Catalog::builtin().unwrap();
    let locale = Locale::new("en-US");

    for (range, width) in [((0..5), 5), ((5..8), 3), ((8..11), 3), ((11..18), 7)] {
        assert_eq!(
            actions::action_width(&program, &catalog, &locale, &actions[range])
                .unwrap()
                .width,
            width
        );
    }
}

#[test]
fn layout_matches_canonical_emission_for_a_nested_sequence() {
    let (program, actions) = program_with_structured_actions();
    let catalog = Catalog::builtin().unwrap();
    let locale = Locale::new("en-US");
    let emitted = emitter::emit(&program, &catalog, &locale).unwrap();
    let action_text = emitted
        .split_once("actions {\n")
        .unwrap()
        .1
        .split_once("\n    }")
        .unwrap()
        .0;
    let emitted_width = action_text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .count();
    let layout = actions::action_width(&program, &catalog, &locale, &actions).unwrap();
    assert_eq!(layout.width, emitted_width);
    assert_eq!(layout.width, 19);
}

#[test]
fn invalid_layout_requests_fail_as_invalid_programs() {
    let mut program = Program::new();
    let mut rule = Rule::new("invalid", Event::Global);
    rule.actions.push(Action::End);
    let actions = rule.actions.clone();
    program.rules.push(rule);

    let error = actions::action_width(
        &program,
        &Catalog::builtin().unwrap(),
        &Locale::new("en-US"),
        &actions,
    )
    .unwrap_err();
    assert!(matches!(error, ActionLayoutError::InvalidProgram { .. }));
}
