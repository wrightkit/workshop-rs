use workshop_rs::catalog::{Catalog, Locale};
use workshop_rs::source::{FileId, Position, SourceFile, Span};
use workshop_rs::{parser, roundtrip};

fn catalog() -> Catalog {
    Catalog::builtin().expect("builtin catalog")
}

#[test]
fn raw_parse_retains_comments_and_attaches_inner_comments_by_span() {
    let source = "// file header\nrule (\"comments\") { // rule header\n    event { Ongoing - Global; } // event note\n    actions { Wait(1, Ignore Condition); } // action note\n}\n// file footer\n";
    let program = parser::parse(source, &catalog(), &Locale::new("en-US")).expect("parses");
    let document = program
        .source(workshop_rs::source::FileId::from_index(0))
        .expect("raw parsing retains source");
    assert_eq!(document.text(), source);
    let comments: Vec<_> = document.comments().collect();
    assert_eq!(comments.len(), 5);
    assert_eq!(comments[0].text(document), "// file header");
    assert_eq!(comments[4].text(document), "// file footer");

    let rule_comments: Vec<_> = document
        .comments_for(program.rule_span(0).expect("public rule span"))
        .collect();
    assert_eq!(rule_comments.len(), 3);
    assert_eq!(rule_comments[0].text(document), "// rule header");
    assert_eq!(rule_comments[2].text(document), "// action note");
}

#[test]
fn source_span_operations_fail_closed_for_another_file() {
    let source =
        "rule (\"one\") { event { Ongoing - Global; } actions { Wait(1, Ignore Condition); } }";
    let program = parser::parse(source, &catalog(), &Locale::new("en-US")).expect("parses");
    let document = program.source(FileId::from_index(0)).unwrap();
    let rule_span = program.rule_span(0).expect("public rule span");
    let foreign_span = Span::new(FileId::from_index(1), rule_span.start, rule_span.end);
    assert!(document.byte_range(foreign_span).is_none());
    assert!(document.comments_for(foreign_span).next().is_none());
    assert!(document.edit_span(foreign_span, "").is_err());
}

#[test]
fn late_source_attachment_preserves_bound_file_identity() {
    let source = "// late source\nrule (\"late\") { event { Ongoing - Global; } actions { Wait(1, Ignore Condition); } }";
    let mut program = workshop_rs::wir::Program::default();
    let file = program.add_file(SourceFile::new("late.ws"));
    program.files.get_mut(file).unwrap().set_source(source);

    let document = program.source(file).expect("late source attachment");
    let span = Span::new(file, Position::new(2, 1), Position::new(2, 5));
    assert_eq!(document.byte_range(span), Some(15..19));
    let edit = document.edit_span(span, "Rule").expect("bound span edit");
    assert!(
        document
            .apply(&[edit])
            .unwrap()
            .text()
            .contains("Rule (\"late\")")
    );
}

#[test]
fn disabled_rule_span_includes_modifier_and_intervening_comment() {
    let source = "disabled // modifier note\nrule (\"disabled\") { event { Ongoing - Global; } actions { Wait(1, Ignore Condition); } }";
    let program = parser::parse(source, &catalog(), &Locale::new("en-US")).expect("parses");
    assert!(program.rules[0].disabled);
    let document = program.source(FileId::from_index(0)).unwrap();
    let span = program.rule_span(0).expect("public rule span");
    assert_eq!(&document.text()[document.byte_range(span).unwrap()], source);
    assert_eq!(document.comments_for(span).count(), 1);
}

#[test]
fn source_edit_preserves_unrelated_trivia_and_reparses_semantics() {
    let source = "// retain this\nrule (\"edit\") {\n  event { Ongoing - Global; }\n  actions { Wait(1, Ignore Condition); } // retain this too\n}\n";
    let catalog = catalog();
    let program = parser::parse(source, &catalog, &Locale::new("en-US")).expect("parses");
    let span = program
        .action_argument_span(0, 0, 0)
        .expect("public action argument span");
    let edit = program.edit_source(span, "2").expect("public number edit");
    let document = program
        .source(workshop_rs::source::FileId::from_index(0))
        .unwrap();
    let updated = document.apply(&[edit]).expect("edit applies");
    assert!(updated.text().contains("// retain this"));
    assert!(updated.text().contains("// retain this too"));
    assert!(updated.text().contains("Wait(2, Ignore Condition)"));

    let reparsed = parser::parse(updated.text(), &catalog, &Locale::new("en-US"))
        .expect("edited source reparses");
    assert!(!roundtrip::equivalent(&program, &reparsed));
    assert!(reparsed.action_argument_span(0, 0, 0).is_some());
}

#[test]
fn unsupported_mixed_source_is_rejected_without_fabricated_workshop_semantics() {
    let mixed = "rule (\"raw\") { event { Ongoing - Global; } actions { Wait(1, Ignore Condition); } }\n@if deltin_only_construct\n";
    let error = parser::parse(mixed, &catalog(), &Locale::new("en-US"))
        .expect_err("mixed source is outside raw Workshop parsing");
    assert!(matches!(
        error,
        workshop_rs::WorkshopError::Malformed { .. }
    ));
}
