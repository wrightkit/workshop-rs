use super::common::{catalog, en};
use workshop_rs::{Action, Value, parser, validate};

fn program(actions: &str) -> workshop_rs::Program {
    program_with("", actions)
}

fn program_with(conditions: &str, actions: &str) -> workshop_rs::Program {
    let conditions = if conditions.is_empty() {
        String::new()
    } else {
        format!("    conditions\n    {{\n        {conditions}\n    }}\n")
    };
    let source = format!(
        r#"rule ("residual tolerance")
{{
    event
    {{
        Ongoing - Global;
    }}
{conditions}    actions
    {{
        {actions}
    }}
}}"#
    );
    parser::parse(&source, &catalog(), &en()).expect("test source parses")
}

fn tolerant_error(actions: &str) -> workshop_rs::WorkshopError {
    validate::validate_canonical_ids_tolerating_residuals(&program(actions), &catalog())
        .expect_err("expected a canonical rejection")
}

/// A catalog-unknown residual must not mask a later signature violation,
/// regardless of source order (#372, `wrightkit/wright#501` review).
#[test]
fn residuals_do_not_mask_later_canonical_violations_in_either_order() {
    for actions in [
        "Wait(sqrt(4));\n        Wait();",
        "Wait();\n        Wait(sqrt(4));",
    ] {
        let error = tolerant_error(actions);
        assert!(
            format!("{error}").contains("'wait' expects 1..2 argument(s), got 0"),
            "{actions:?} rejects the Wait() arity violation: {error}"
        );
    }
}

#[test]
fn a_residual_alone_does_not_reject() {
    for program in [
        program("Wait(sqrt(4));\n        Wait(1);"),
        program("Else;\n        Wait(1);"),
        program_with("sqrt(4);", "Wait(1);"),
    ] {
        validate::validate_canonical_ids_tolerating_residuals(&program, &catalog())
            .expect("residuals defer to completeness reporting");
    }
}

#[test]
fn nested_known_violations_inside_residual_arguments_still_reject() {
    // Unknown value call wrapping a slot-type-invalid known value (the
    // `defend` corpus pattern: `Players On Hero` is Array, not Player).
    let error = tolerant_error(
        "Wait(sqrt(Ability Cooldown(Players On Hero(Hero(Mercy), Team 1), Button(Ability 2))));",
    );
    assert!(
        format!("{error}")
            .contains("'getAbilityCooldown' argument 1 must have semantic type 'Player'"),
        "nested slot violation under a residual value: {error}"
    );

    // Unknown action calls are not authored through raw source (the parser
    // keeps statement-position spellings strict), so producer-shaped calls
    // build the residual directly.
    let mut program = program("Wait(1);");
    program.rules[0].actions.push(Action::call(
        "producerExtension",
        [Value::call("getAbilityCooldown", [])],
    ));
    let error = validate::validate_canonical_ids_tolerating_residuals(&program, &catalog())
        .expect_err("nested violation under an unknown action");
    assert!(
        format!("{error}").contains("'getAbilityCooldown' expects"),
        "nested arity violation under a residual action: {error}"
    );
}

#[test]
fn strict_validation_still_rejects_residuals() {
    let error = validate::validate_canonical_ids(&program("Wait(sqrt(4));"), &catalog())
        .expect_err("strict validation rejects the unknown value");
    assert!(
        matches!(
            error,
            workshop_rs::WorkshopError::Unknown { .. }
                | workshop_rs::WorkshopError::UnknownWithCandidates { .. }
        ),
        "strict validation reports the residual as Unknown: {error:?}"
    );
}

#[test]
fn catalog_defaults_stay_accepted() {
    for actions in ["Wait(1);", "Wait(1, Ignore Condition);"] {
        validate::validate_canonical_ids_tolerating_residuals(&program(actions), &catalog())
            .expect("{actions} supplies every required parameter");
    }
}
