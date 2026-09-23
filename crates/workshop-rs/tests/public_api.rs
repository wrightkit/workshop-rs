use workshop_rs::catalog::{Catalog, Locale};
use workshop_rs::settings::{PathPart, schema};
use workshop_rs::{emitter, parser, roundtrip, rules};

#[test]
fn canonical_program_operations_cover_parse_validate_inspect_emit_and_roundtrip() {
    let catalog = Catalog::builtin().expect("built-in catalog");
    let locale = Locale::new("en-US");
    let source = r#"rule ("public api") {
        event { Ongoing - Global; }
        actions { Wait(1, Ignore Condition); }
    }"#;

    let program = parser::parse_with_context(source, &catalog, &locale, &catalog)
        .expect("Workshop parses into the canonical program");
    program
        .validate()
        .expect("canonical program is structurally valid");
    rules::validate_canonical_ids(&program, &catalog)
        .expect("canonical ids resolve through the public rule API");
    assert!(program.semantic_issues(&catalog).is_empty());

    let emitted = emitter::emit(&program, &catalog, &locale).expect("canonical emission");
    let reparsed = parser::parse(&emitted, &catalog, &locale).expect("emitted Workshop reparses");
    assert!(roundtrip::equivalent(&program, &reparsed));
}

#[test]
fn settings_schema_exposes_enum_values_without_the_internal_table() {
    let definition = schema::definition(&[
        PathPart::Part("lobby"),
        PathPart::Part("enableMatchVoiceChat"),
    ])
    .expect("match voice setting is in the canonical schema");

    let enabled = definition
        .enum_members()
        .next()
        .expect("enum-backed boolean token");
    assert_eq!(enabled.domain(), "matchVoiceChat");
    assert_eq!(enabled.id(), "enabled");
    assert_eq!(enabled.english_name(), "Enabled");
}
