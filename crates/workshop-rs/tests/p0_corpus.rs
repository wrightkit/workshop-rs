//! Re-runnable P0 corpus gate over the pinned, repository-owned source inputs.

use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use workshop_rs::{
    catalog::{Catalog, Locale},
    convert, emitter,
    p0::{P0_EXPECTATION, P0CaseExpectation, P0Stage},
    parser, roundtrip, semantic, validate,
};

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

fn assert_residual_policy(
    case: &P0CaseExpectation,
    stage: &str,
    issues: &[semantic::SemanticIssue],
) {
    let unexpected: Vec<_> = issues
        .iter()
        .filter(|issue| !case.admits_residual(issue))
        .collect();
    assert!(
        unexpected.is_empty(),
        "{} {stage} has unexplained semantic residuals: {}",
        case.id,
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
    let catalog = Catalog::builtin().expect("built-in catalog");
    let mut inventory = BTreeMap::new();
    for case in P0_EXPECTATION.cases {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(case.source_fixture);
        let bytes = std::fs::read(&path).unwrap_or_else(|error| panic!("{path:?}: {error}"));
        assert_eq!(
            format!("{:x}", Sha256::digest(&bytes)),
            case.source_sha256,
            "pinned digest mismatch for {}",
            case.id
        );
        let source = String::from_utf8(bytes).expect("artifact is UTF-8 Workshop text");
        let locale = Locale::new(case.locale);
        let program = parser::parse_with_context(&source, &catalog, &locale, &catalog)
            .unwrap_or_else(|error| panic!("{} parse failed: {error:?}", case.id));
        assert_residual_policy(case, "source-parse", &semantic::inspect(&program, &catalog));
        if let Err(error) = validate::validate_canonical_ids(&program, &catalog) {
            assert!(
                case.admits_gap(P0Stage::CanonicalValidation, &error),
                "{} canonical validation failed outside known gaps: {error:?}",
                case.id
            );
            println!(
                "{}: known {} gap: {error:?}",
                case.id,
                P0Stage::CanonicalValidation.as_str()
            );
        }
        let emitted = match emitter::emit(&program, &catalog, &locale) {
            Ok(text) => text,
            Err(error) => {
                assert!(
                    case.admits_gap(P0Stage::Emission, &error),
                    "{} emission failed outside known gaps: {error:?}",
                    case.id
                );
                println!("{}: known emission gap: {error:?}", case.id);
                inventory.insert(
                    case.id.to_string(),
                    residual_groups(&semantic::inspect(&program, &catalog)),
                );
                continue;
            }
        };
        let reparsed = parser::parse_with_context(&emitted, &catalog, &locale, &catalog)
            .unwrap_or_else(|error| panic!("{} reparse failed: {error:?}", case.id));
        assert_residual_policy(
            case,
            "source-locale-reparse",
            &semantic::inspect(&reparsed, &catalog),
        );
        if !roundtrip::equivalent(&program, &reparsed) {
            panic!("{} semantic round-trip changed WIR", case.id);
        }
        let emitted_again = emitter::emit(&reparsed, &catalog, &locale).unwrap_or_else(|error| {
            panic!("{} deterministic re-emission failed: {error:?}", case.id)
        });
        assert_eq!(
            emitted, emitted_again,
            "{} emission is not deterministic",
            case.id
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
                    case.admits_gap(P0Stage::LocaleConversion, &error),
                    "{} locale conversion failed outside known gaps: {error:?}",
                    case.id
                );
                println!("{}: known locale conversion gap: {error:?}", case.id);
                inventory.insert(
                    case.id.to_string(),
                    residual_groups(&semantic::inspect(&program, &catalog)),
                );
                continue;
            }
        };
        let converted_program =
            parser::parse_with_context(&converted.text, &catalog, &target_locale, &catalog)
                .unwrap_or_else(|error| {
                    panic!("{} target-locale reparse failed: {error:?}", case.id)
                });
        assert_residual_policy(
            case,
            "target-locale-reparse",
            &semantic::inspect(&converted_program, &catalog),
        );
        assert_custom_workshop_settings(case.id, &source, &converted.text);
        assert_target_locale_spellings(
            case.id,
            &locale,
            &target_locale,
            &source,
            &converted.text,
            &program.dump(),
            &catalog,
        );
        assert!(
            roundtrip::equivalent(&program, &converted_program),
            "{} target-locale conversion changed WIR",
            case.id
        );
        let converted_back =
            emitter::emit(&converted_program, &catalog, &locale).unwrap_or_else(|error| {
                panic!("{} reverse locale emission failed: {error:?}", case.id)
            });
        let converted_back_program =
            parser::parse_with_context(&converted_back, &catalog, &locale, &catalog)
                .unwrap_or_else(|error| {
                    panic!("{} reverse locale reparse failed: {error:?}", case.id)
                });
        assert!(
            roundtrip::equivalent(&program, &converted_back_program),
            "{} reverse locale conversion changed WIR",
            case.id
        );
        inventory.insert(
            case.id.to_string(),
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
