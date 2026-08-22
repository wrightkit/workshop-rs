//! Re-runnable P0 corpus gate over the pinned, repository-owned source inputs.

use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use workshop_rs::{
    WorkshopError,
    catalog::{Catalog, Locale},
    convert, emitter, parser, roundtrip, semantic, validate,
};

const CASES: &[(&str, &str, &str)] = &[
    (
        "ai-pve",
        "zh-CN",
        "d9c6460ca550e40083efcc2b57de16360088631970824599a22c0aa2cb7f11f9",
    ),
    (
        "bastion",
        "en-US",
        "44e453ddf7f373be65aea82d019abd45dd60f5ecb57c8d1607d3576a8bc60259",
    ),
    (
        "defend",
        "en-US",
        "06a956b650313ee2d6e24ec989f907244dc4444579bdba27c580b031de97b268",
    ),
    (
        "illari",
        "zh-CN",
        "f3aff73b9e677730bddc9c85b04c2bd38439bb7a4ba4fa2e80dc28db2e4a0860",
    ),
    (
        "rework",
        "en-US",
        "aa32cda640dba41fd99245a7d425d9897b53875d15cf071862197a8e6840258c",
    ),
];

#[derive(Debug, Clone, Copy)]
enum Stage {
    CanonicalValidation,
    Emission,
    Reparse,
    LocaleConversion,
}

impl Stage {
    fn as_str(self) -> &'static str {
        match self {
            Stage::CanonicalValidation => "canonical-validation",
            Stage::Emission => "emission",
            Stage::Reparse => "reparse",
            Stage::LocaleConversion => "locale-conversion",
        }
    }
}

fn known_gap(name: &str, stage: Stage, error: &WorkshopError) -> bool {
    matches!(
        (name, stage, error),
        (
            "defend",
            Stage::CanonicalValidation | Stage::Emission | Stage::LocaleConversion,
            WorkshopError::Unknown {
                kind: "action",
                spelling,
                ..
            },
        ) if spelling == "rawWorkshopAction"
    )
}

#[derive(Debug)]
struct SpanReport {
    file: usize,
    line: u32,
    column: u32,
    end_line: u32,
    end_column: u32,
}

fn residual_groups(issues: &[semantic::SemanticIssue]) -> Vec<serde_json::Value> {
    let mut groups: BTreeMap<
        (String, String),
        (usize, Option<SpanReport>, semantic::ResidualClassification),
    > = BTreeMap::new();
    for issue in issues {
        let entry = groups
            .entry((format!("{:?}", issue.kind), issue.name.clone()))
            .or_insert((0, None, issue.classification));
        entry.0 += 1;
        if entry.1.is_none() {
            entry.1 = issue.span.map(|span| SpanReport {
                file: span.file.index(),
                line: span.start.line,
                column: span.start.col,
                end_line: span.end.line,
                end_column: span.end.col,
            });
        }
    }
    groups
        .into_iter()
        .map(
            |((kind, source_canonical_name), (count, representative_span, classification))| {
                serde_json::json!({
                    "kind": kind,
                    "sourceCanonicalName": source_canonical_name,
                    "count": count,
                    "representativeSpan": representative_span.map(|span| serde_json::json!({
                        "file": span.file,
                        "line": span.line,
                        "column": span.column,
                        "endLine": span.end_line,
                        "endColumn": span.end_column,
                    })),
                    "classification": classification.as_str(),
                    "evidence": classification.evidence(),
                })
            },
        )
        .collect()
}

fn assert_residual_policy(name: &str, stage: &str, issues: &[semantic::SemanticIssue]) {
    let unexpected: Vec<_> = issues
        .iter()
        .filter(|issue| {
            !(name == "defend"
                && issue.kind == semantic::IncompletenessKind::OpaqueAction
                && issue.name == "rawWorkshopAction"
                && issue.classification == semantic::ResidualClassification::LegacyOpaque)
        })
        .collect();
    assert!(
        unexpected.is_empty(),
        "{name} {stage} has unexplained semantic residuals: {}",
        serde_json::to_string(&residual_groups(
            &unexpected.into_iter().cloned().collect::<Vec<_>>()
        ))
        .expect("residual JSON")
    );
}

fn assert_custom_workshop_settings(name: &str, source: &str, target: &str) {
    let marker = match name {
        "ai-pve" => "自定义 AI",
        "bastion" => "自动重开时间间隔（小时）",
        _ => return,
    };
    assert!(
        source.contains(marker),
        "{name} fixture lacks expected custom setting marker"
    );
    assert!(
        target.contains(marker),
        "{name} locale conversion changed custom setting data"
    );
}

fn assert_target_locale_spellings(
    name: &str,
    source_locale: &Locale,
    target_locale: &Locale,
    source: &str,
    target: &str,
    canonical_dump: &str,
    catalog: &Catalog,
) {
    let canonical_uses = |identity: &str| {
        canonical_dump
            .split(|character: char| !character.is_ascii_alphanumeric())
            .any(|token| token == identity)
    };
    for kind in [
        workshop_rs::catalog::Kind::Structural,
        workshop_rs::catalog::Kind::Action,
        workshop_rs::catalog::Kind::Value,
        workshop_rs::catalog::Kind::Event,
        workshop_rs::catalog::Kind::Operator,
        workshop_rs::catalog::Kind::Setting,
    ] {
        for entry in catalog.entries_of(kind) {
            let Some(source_spelling) = entry.spelling(source_locale) else {
                continue;
            };
            let Some(target_spelling) = entry.spelling(target_locale) else {
                continue;
            };
            let used = if kind == workshop_rs::catalog::Kind::Structural {
                entry.id != "workshop" && source.contains(source_spelling)
            } else {
                canonical_uses(&entry.id)
            };
            if source_spelling != target_spelling && used {
                let target_spelling_present = target.contains(target_spelling)
                    || (matches!(entry.id.as_str(), "chaseAtRate" | "chaseOverTime")
                        && catalog
                            .spelling(
                                workshop_rs::catalog::Kind::Action,
                                target_locale,
                                if entry.id == "chaseAtRate" {
                                    "chasePlayerVariableAtRate"
                                } else {
                                    "chasePlayerVariableOverTime"
                                },
                            )
                            .is_some_and(|spelling| target.contains(spelling)));
                assert!(
                    target_spelling_present,
                    "{name} target locale omitted {kind:?} '{}' spelling '{}', source spelling '{}' remains the only evidence",
                    entry.id, target_spelling, source_spelling
                );
            }
        }
    }
    for domain in catalog.enum_domains() {
        if let (Some(source_spelling), Some(target_spelling)) = (
            domain.spelling(source_locale),
            domain.spelling(target_locale),
        ) {
            if source_spelling != target_spelling
                && canonical_uses(&domain.domain)
                && source.contains(source_spelling)
            {
                assert!(
                    target.contains(target_spelling),
                    "{name} target locale omitted enum domain '{}' spelling '{}', source spelling '{}' remains the only evidence",
                    domain.domain,
                    target_spelling,
                    source_spelling
                );
            }
        }
        for member in &domain.members {
            if let (Some(source_spelling), Some(target_spelling)) = (
                member.spelling(source_locale),
                member.spelling(target_locale),
            ) {
                if source_spelling != target_spelling
                    && canonical_uses(&member.member)
                    && source.contains(source_spelling)
                {
                    assert!(
                        target.contains(target_spelling),
                        "{name} target locale omitted enum member '{}.{}' spelling '{}', source spelling '{}' remains the only evidence",
                        domain.domain,
                        member.member,
                        target_spelling,
                        source_spelling
                    );
                }
            }
        }
    }
}

#[test]
fn pinned_p0_corpus_has_explicit_stage_and_residual_gates() {
    let root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/real-projects");
    let catalog = Catalog::builtin().expect("built-in catalog");
    let mut inventory = BTreeMap::new();
    for (name, locale, expected_sha) in CASES {
        let path = std::path::Path::new(&root).join(format!("{name}.ow"));
        let bytes = std::fs::read(&path).unwrap_or_else(|error| panic!("{path:?}: {error}"));
        assert_eq!(
            format!("{:x}", Sha256::digest(&bytes)),
            *expected_sha,
            "pinned digest mismatch for {name}"
        );
        let source = String::from_utf8(bytes).expect("artifact is UTF-8 Workshop text");
        let locale = Locale::new(locale);
        let program = parser::parse_with_context(&source, &catalog, &locale, &catalog)
            .unwrap_or_else(|error| panic!("{name} parse failed: {error:?}"));
        assert_residual_policy(name, "source-parse", &semantic::inspect(&program, &catalog));
        if let Err(error) = validate::validate_canonical_ids(&program, &catalog) {
            assert!(
                known_gap(name, Stage::CanonicalValidation, &error),
                "{name} canonical validation failed outside known gaps: {error:?}"
            );
            println!(
                "{name}: known {} gap: {error:?}",
                Stage::CanonicalValidation.as_str()
            );
        }
        let emitted = match emitter::emit(&program, &catalog, &locale) {
            Ok(text) => text,
            Err(error) => {
                assert!(
                    known_gap(name, Stage::Emission, &error),
                    "{name} emission failed outside known gaps: {error:?}"
                );
                println!("{name}: known emission gap: {error:?}");
                inventory.insert(
                    name.to_string(),
                    residual_groups(&semantic::inspect(&program, &catalog)),
                );
                continue;
            }
        };
        let reparsed = parser::parse_with_context(&emitted, &catalog, &locale, &catalog)
            .unwrap_or_else(|error| panic!("{name} {} failed: {error:?}", Stage::Reparse.as_str()));
        assert_residual_policy(
            name,
            "source-locale-reparse",
            &semantic::inspect(&reparsed, &catalog),
        );
        if !roundtrip::equivalent(&program, &reparsed) {
            panic!("{name} semantic round-trip changed WIR");
        }
        let emitted_again = emitter::emit(&reparsed, &catalog, &locale)
            .unwrap_or_else(|error| panic!("{name} deterministic re-emission failed: {error:?}"));
        assert_eq!(
            emitted, emitted_again,
            "{name} emission is not deterministic"
        );
        let target_locale = if locale.as_str() == "en-US" {
            Locale::new("zh-CN")
        } else {
            Locale::new("en-US")
        };
        let converted = match convert::convert(
            &source,
            &catalog,
            &locale,
            &target_locale,
            &convert::ConvertOptions::default(),
        ) {
            Ok(converted) => converted,
            Err(error) => {
                assert!(
                    known_gap(name, Stage::LocaleConversion, &error),
                    "{name} locale conversion failed outside known gaps: {error:?}"
                );
                println!("{name}: known locale conversion gap: {error:?}");
                inventory.insert(
                    name.to_string(),
                    residual_groups(&semantic::inspect(&program, &catalog)),
                );
                continue;
            }
        };
        let converted_program =
            parser::parse_with_context(&converted.text, &catalog, &target_locale, &catalog)
                .unwrap_or_else(|error| panic!("{name} target-locale reparse failed: {error:?}"));
        assert_residual_policy(
            name,
            "target-locale-reparse",
            &semantic::inspect(&converted_program, &catalog),
        );
        assert_custom_workshop_settings(name, &source, &converted.text);
        assert_target_locale_spellings(
            name,
            &locale,
            &target_locale,
            &source,
            &converted.text,
            &program.dump(),
            &catalog,
        );
        assert!(
            roundtrip::equivalent(&program, &converted_program),
            "{name} target-locale conversion changed WIR"
        );
        let converted_back = emitter::emit(&converted_program, &catalog, &locale)
            .unwrap_or_else(|error| panic!("{name} reverse locale emission failed: {error:?}"));
        let converted_back_program =
            parser::parse_with_context(&converted_back, &catalog, &locale, &catalog)
                .unwrap_or_else(|error| panic!("{name} reverse locale reparse failed: {error:?}"));
        assert!(
            roundtrip::equivalent(&program, &converted_back_program),
            "{name} reverse locale conversion changed WIR"
        );
        inventory.insert(
            name.to_string(),
            residual_groups(&semantic::inspect(&program, &catalog)),
        );
    }
    for (name, groups) in &inventory {
        println!(
            "p0-artifact={name} residual-groups={}",
            serde_json::to_string(groups).expect("residual JSON")
        );
    }
}
