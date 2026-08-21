//! Re-runnable P0 corpus gate. The source artifacts remain external because
//! their repository licenses do not permit redistribution here.

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

#[derive(Debug, serde::Serialize)]
struct ResidualGroup {
    kind: String,
    #[serde(rename = "sourceCanonicalName")]
    source_canonical_name: String,
    count: usize,
    #[serde(rename = "representativeSpan")]
    representative_span: Option<SpanReport>,
    classification: String,
    evidence: String,
}

#[derive(Debug, serde::Serialize)]
struct SpanReport {
    file: usize,
    line: u32,
    column: u32,
    end_line: u32,
    end_column: u32,
}

fn residual_groups(issues: &[semantic::SemanticIssue]) -> Vec<ResidualGroup> {
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
                ResidualGroup {
                    kind,
                    source_canonical_name,
                    count,
                    representative_span,
                    classification: classification.as_str().to_string(),
                    evidence: classification.evidence().to_string(),
                }
            },
        )
        .collect()
}

#[derive(Debug, serde::Serialize)]
struct ResidualInventory {
    schema: &'static str,
    artifacts: BTreeMap<String, Vec<ResidualGroup>>,
}

#[test]
#[ignore = "requires externally reacquired artifacts; run with WRIGHTKIT_P0_ARTIFACT_DIR"]
fn pinned_p0_corpus_has_explicit_stage_and_residual_gates() {
    let root = std::env::var("WRIGHTKIT_P0_ARTIFACT_DIR")
        .expect("set WRIGHTKIT_P0_ARTIFACT_DIR to reacquired artifacts");
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
    let report = ResidualInventory {
        schema: "workshop-p0-residual/v1",
        artifacts: inventory,
    };
    let actual = serde_json::to_value(&report).expect("inventory JSON");
    let expected: serde_json::Value = serde_json::from_str(include_str!(
        "../../../docs/evidence/workshop-p0-residual-v1.json"
    ))
    .expect("committed residual inventory JSON");
    assert_eq!(
        actual, expected,
        "residual inventory is not reconciled with the pinned artifact"
    );
    for (name, groups) in &report.artifacts {
        println!("{name}: {} grouped residuals", groups.len());
    }
}
