#![allow(dead_code)]

use sha2::{Digest, Sha256};
use workshop_rs::{
    WorkshopError,
    catalog::{Catalog, Locale},
    p0::{P0CaseExpectation, P0Stage},
    semantic,
};

#[derive(Debug)]
struct SpanReport {
    file: usize,
    line: u32,
    column: u32,
    end_line: u32,
    end_column: u32,
}

pub(crate) fn cases() -> &'static [P0CaseExpectation] {
    workshop_rs::p0::P0_EXPECTATION.cases
}

pub(crate) fn source(case: &P0CaseExpectation) -> (String, Locale) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(case.source_fixture);
    let bytes = std::fs::read(&path).unwrap_or_else(|error| panic!("{path:?}: {error}"));
    assert_eq!(
        format!("{:x}", Sha256::digest(&bytes)),
        case.source_sha256,
        "pinned digest mismatch for {}",
        case.id
    );
    let source = String::from_utf8(bytes).expect("artifact is UTF-8 Workshop text");
    (source, Locale::new(case.locale))
}

pub(crate) fn target_locale(source_locale: &Locale) -> Locale {
    if source_locale.as_str() == "en-US" {
        Locale::new("zh-CN")
    } else {
        Locale::new("en-US")
    }
}

pub(crate) fn assert_residual_policy(
    case: &P0CaseExpectation,
    stage: &str,
    issues: &[semantic::SemanticIssue],
) {
    if !issues.is_empty() {
        println!(
            "real-project={} stage={stage} residual-groups={}",
            case.id,
            serde_json::to_string(&residual_groups(issues)).expect("residual JSON")
        );
    }
    let unexpected: Vec<_> = issues
        .iter()
        .filter(|issue| !case.admits_residual(issue))
        .collect();
    assert!(
        unexpected.is_empty(),
        "{} {stage} has unexplained semantic residuals: {}",
        case.id,
        serde_json::to_string(&residual_groups(
            &unexpected.into_iter().cloned().collect::<Vec<_>>(),
        ))
        .expect("residual JSON")
    );
}

pub(crate) fn assert_gap(case: &P0CaseExpectation, stage: P0Stage, error: &WorkshopError) {
    assert!(
        case.admits_gap(stage, error),
        "{} {} failed outside known gaps: {error:?}",
        case.id,
        stage.as_str()
    );
}

fn residual_groups(issues: &[semantic::SemanticIssue]) -> Vec<serde_json::Value> {
    let mut groups = std::collections::BTreeMap::<
        (String, String),
        (usize, Option<SpanReport>, semantic::ResidualClassification),
    >::new();
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

pub(crate) fn assert_custom_workshop_settings(case_id: &str, source: &str, target: &str) {
    let marker = match case_id {
        "ai-pve" => "自定义 AI",
        "bastion" => "自动重开时间间隔（小时）",
        _ => return,
    };
    assert!(
        source.contains(marker),
        "{case_id} fixture lacks expected custom setting marker"
    );
    assert!(
        target.contains(marker),
        "{case_id} locale conversion changed custom setting data"
    );
}

pub(crate) fn assert_target_locale_spellings(
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
