use super::common;

use workshop_rs::settings::{PathPart, SettingValueDomain, definition};
use workshop_rs::{emitter, parser, roundtrip, validate};

const BELOW_MINIMUM: &str =
    include_str!("fixtures/settings/movement-speed-below-minimum.settings.ws");

#[test]
fn hero_movement_speed_rejects_values_below_the_client_minimum() {
    let catalog = common::catalog();
    let locale = common::en();
    let program = parser::parse(BELOW_MINIMUM, &catalog, &locale).expect("fixture parses");
    let diagnostic = program
        .validate()
        .expect_err("45% is below the client minimum");
    let message = diagnostic.to_string();
    for detail in ["movementSpeed%", "45", "50"] {
        assert!(message.contains(detail), "missing {detail:?}: {message}");
    }
    assert!(diagnostic.span().is_some());
    let diagnostics = program.settings_diagnostics();
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].error, diagnostic);
    assert_eq!(
        validate::validate_canonical_ids(&program, &catalog).expect_err("strict validation"),
        diagnostic
    );
    assert_eq!(
        validate::validate_canonical_ids_tolerating_residuals(&program, &catalog)
            .expect_err("tolerant validation"),
        diagnostic
    );
    assert_eq!(
        emitter::emit(&program, &catalog, &locale).expect_err("emission rejects invalid settings"),
        diagnostic
    );

    for value in ["50", "51", "150"] {
        let source = BELOW_MINIMUM.replace("45%", &format!("{value}%"));
        let program = parser::parse(&source, &catalog, &locale).expect("boundary fixture parses");
        program.validate().expect("value is within the known bound");
        validate::validate_canonical_ids(&program, &catalog).expect("canonical validation");
        let output = emitter::emit(&program, &catalog, &locale).expect("boundary fixture emits");
        let reparsed = parser::parse(&output, &catalog, &locale).expect("emitted settings reparse");
        assert!(roundtrip::equivalent(&program, &reparsed));
    }
}

#[test]
fn each_out_of_bounds_hero_setting_has_its_own_diagnostic() {
    let source = "settings { heroes { Team 1 {\n\
        Soldier: 76 { Movement Speed: 45% }\n\
        Ana { Movement Speed: 49% }\n\
    } } }";
    let program = parser::parse(source, &common::catalog(), &common::en()).expect("settings parse");
    let diagnostics = program.settings_diagnostics();
    assert_eq!(diagnostics.len(), 2);
    assert!(diagnostics[0].error.to_string().contains("45"));
    assert!(diagnostics[1].error.to_string().contains("49"));
    assert!(diagnostics[0].error.span().is_some());
    assert!(diagnostics[1].error.span().is_some());
    assert_ne!(diagnostics[0].error.span(), diagnostics[1].error.span());
}

#[test]
fn input_bounds_are_public_and_distinct_from_effective_clamping() {
    let hero_path = [
        PathPart::Part("heroes"),
        PathPart::Team,
        PathPart::Hero,
        PathPart::Part("movementSpeed%"),
    ];
    let speed = definition(&hero_path).expect("per-hero movement speed");
    let bounds = speed.input_bounds().expect("reviewed input bounds");
    assert_eq!(bounds.min(), Some(50.0));
    assert_eq!(bounds.max(), None);
    assert!(matches!(speed.domain(), SettingValueDomain::Percent(_)));
    assert!(speed.effective_number(45.0).is_none());

    let team_speed = definition(&[
        PathPart::Part("heroes"),
        PathPart::Team,
        PathPart::Part("movementSpeed%"),
    ])
    .expect("team movement speed");
    let health = definition(&[
        PathPart::Part("heroes"),
        PathPart::Team,
        PathPart::Hero,
        PathPart::Part("health%"),
    ])
    .expect("per-hero health");
    assert!(team_speed.input_bounds().is_none());
    assert!(health.input_bounds().is_none());
}

#[test]
fn known_bounds_apply_across_teams_and_heroes_without_inventing_other_bounds() {
    let catalog = common::catalog();
    let locale = common::en();
    for (team, hero) in [("General", "Ana"), ("Team 2", "Genji")] {
        let source = BELOW_MINIMUM
            .replace("Team 1", team)
            .replace("Soldier: 76", hero);
        let program = parser::parse(&source, &catalog, &locale).expect("fixture parses");
        let error = program.validate().expect_err("the hero bound applies");
        assert!(error.to_string().contains("movementSpeed%"), "{error}");
    }

    for source in [
        "settings { heroes { Team 1 { Movement Speed: 45% } } }".to_string(),
        BELOW_MINIMUM.replace("Movement Speed", "Health"),
    ] {
        let program = parser::parse(&source, &catalog, &locale).expect("fixture parses");
        program.validate().expect("no reviewed bound applies");
        validate::validate_canonical_ids(&program, &catalog).expect("canonical validation");
        emitter::emit(&program, &catalog, &locale).expect("unbounded fixture emits");
    }
}
