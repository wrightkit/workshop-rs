//! CLI surface tests: parse/emit/convert a file, list locales, report the
//! machine-readable catalog identity, and fail explicitly on missing
//! target-locale mappings (with opt-in fallback).

use std::path::PathBuf;
use std::process::Command;

use workshop_rs::catalog::{Catalog, Locale};
use workshop_rs::conformance::CONFORMANCE_SCHEMA_VERSION;
use workshop_rs::conformance::{
    Comparison, ConformanceReason, ConformanceResult, ConformanceStatus, Equivalence, Evidence,
    EvidenceArtifact, EvidenceBasis, EvidenceClass, ExpectationSource, FeatureId, FeatureKind,
    FeatureNamespace, ReasonCode,
};
use workshop_rs::live_capture::{
    CENSUS_IDENTITY_SCHEMA_VERSION, CensusIdentity, LIVE_CAPTURE_SCHEMA_VERSION, LiveCapture,
};

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_workshop-rs-cli")
}

fn fixture(fixture_id: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../workshop-rs/tests/fixtures/corpus")
        .join(format!("{fixture_id}.ws"))
}

fn run(args: &[&str]) -> std::process::Output {
    Command::new(bin()).args(args).output().expect("cli runs")
}

// Synthetic schema/diff input only. This helper deliberately creates no
// client artifact and is not an Overwatch evidence fixture.
fn synthetic_capture(id: &str) -> String {
    let catalog = Catalog::builtin().unwrap();
    let locale = Locale::new("en-US");
    let raw = EvidenceArtifact {
        name: "synthetic-cli/raw.ws".to_string(),
        revision: Some("synthetic-cli".to_string()),
        path: Some("synthetic-cli/raw.ws".to_string()),
        sha256: Some(
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
        ),
        license: Some("test-only".to_string()),
    };
    let result = ConformanceResult {
        schema_version: CONFORMANCE_SCHEMA_VERSION,
        case_id: "wir/variables-global".to_string(),
        features: vec![
            FeatureId::owned(FeatureNamespace::Wir, FeatureKind::Variable, "global").unwrap(),
        ],
        status: ConformanceStatus::Inconclusive,
        comparison: Comparison {
            mode: Equivalence::NotComparable,
            expected: None,
            observed: None,
            normalizer: None,
        },
        evidence: Evidence {
            class: EvidenceClass::LiveClient,
            fixture: raw.clone(),
            expectation: ExpectationSource {
                basis: EvidenceBasis::WorkshopClient,
                artifact: EvidenceArtifact::new("synthetic-cli/client-expectation"),
                tracking_ref: None,
            },
            catalog: catalog.identity(),
            locale: Some(locale.clone()),
            client: Some(workshop_rs::conformance::ClientEvidence {
                game: "overwatch-2".to_string(),
                client_version: Some("synthetic-client".to_string()),
                season: Some("synthetic-season".to_string()),
                captured_at: "2026-08-18T00:00:00Z".to_string(),
                environment: Some("synthetic CLI unit input".to_string()),
            }),
            implementation: None,
        },
        reason: Some(ConformanceReason {
            code: ReasonCode::Inconclusive,
            detail: "synthetic CLI schema input".to_string(),
            tracking_ref: None,
        }),
    };
    LiveCapture {
        schema_version: LIVE_CAPTURE_SCHEMA_VERSION,
        capture_id: id.to_string(),
        game: "overwatch-2".to_string(),
        client: "synthetic-client".to_string(),
        season: "synthetic-season".to_string(),
        captured_at: "2026-08-18T00:00:00Z".to_string(),
        environment: "synthetic CLI unit input".to_string(),
        locale,
        catalog: catalog.identity(),
        census: CensusIdentity {
            schema_version: CENSUS_IDENTITY_SCHEMA_VERSION,
            digest: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
            shards: vec!["synthetic-cli-shard".to_string()],
        },
        raw_artifact: raw,
        results: vec![result],
    }
    .to_json()
    .unwrap()
}

#[test]
fn version_reports_the_machine_readable_identity() {
    let output = run(&["version"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("implementation version: "));
    assert!(stdout.contains("catalog version: "));
    assert!(stdout.contains("catalog digest: "));
    assert!(stdout.contains("locale en-us"));
    assert!(stdout.contains("locale zh-cn"));

    let output = run(&["version", "--json"]);
    assert!(output.status.success());
    let identity: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("identity is valid JSON");
    assert!(identity["implementation-version"].is_string());
    assert!(identity["catalog-digest"].is_string());
    assert!(identity["locale-coverage"].is_array());
}

#[test]
fn locales_lists_declared_locales_with_coverage() {
    let output = run(&["locales"]);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 2);
    for (line, (locale, expected)) in lines
        .iter()
        .zip([("en-us", None), ("zh-cn", Some(("1240", "1259")))])
    {
        let (reported_locale, coverage) = line.split_once(' ').expect("locale coverage line");
        let (mapped, total) = coverage.split_once('/').expect("mapped/total coverage");
        assert_eq!(reported_locale, locale);
        if let Some((expected_mapped, expected_total)) = expected {
            assert_eq!((mapped, total), (expected_mapped, expected_total), "{line}");
        } else {
            assert_eq!(mapped, total, "{line}");
        }
    }
}

#[test]
fn parse_parses_a_corpus_file() {
    let file = fixture("basic-rule");
    let output = run(&["parse", file.to_str().unwrap()]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("rule \"setup\""), "{stdout}");
}

#[test]
fn parse_with_explicit_locale_works() {
    let file = fixture("basic-rule");
    let output = run(&["parse", "--locale", "en-US", file.to_str().unwrap()]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn parse_reports_unknown_spellings_with_failure_exit() {
    let dir = std::env::temp_dir().join("workshop-rs-cli-parse-bad");
    std::fs::create_dir_all(&dir).unwrap();
    let bad = dir.join("bad.ws");
    std::fs::write(
        &bad,
        "rule (\"x\") { event { Ongoing - Global; } actions { Totally Unknown Thing(1); } }",
    )
    .unwrap();
    let output = run(&["parse", "--locale", "en-US", bad.to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Totally Unknown Thing"), "{stderr}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn emit_emits_localized_text() {
    let file = fixture("basic-rule");
    let output = run(&["emit", file.to_str().unwrap()]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("rule (\"setup\") {"), "{stdout}");
    assert!(stdout.contains("Disable Inspector Recording;"), "{stdout}");
}

#[test]
fn convert_to_zh_cn_uses_the_corpus_without_fallback() {
    let file = fixture("basic-rule");
    let output = run(&[
        "convert",
        file.to_str().unwrap(),
        "--from",
        "en-US",
        "--to",
        "zh-CN",
    ]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("持续 - 全局"), "{stdout}");
    assert!(stdout.contains("禁用查看器录制"), "{stdout}");
}

#[test]
fn convert_to_zh_cn_with_fallback_reports_the_choice() {
    let dir = std::env::temp_dir().join("workshop-rs-cli-convert-fallback");
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("unmapped.ws");
    std::fs::write(
        &file,
        "rule (\"setup\") { event { Ongoing - Global; } actions { Disable Inspector Recording; } }",
    )
    .unwrap();
    let output = run(&[
        "convert",
        file.to_str().unwrap(),
        "--from",
        "en-US",
        "--to",
        "fr-FR",
        "--fallback-locale",
        "en-US",
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Ongoing - Global"), "{stdout}");
    assert!(stdout.contains("Disable Inspector Recording"), "{stdout}");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("fallback-locale spelling") && stderr.contains("disableInspector"),
        "the fallback choice is visible in tooling output: {stderr}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn usage_errors_exit_2() {
    let output = run(&["convert"]);
    assert_eq!(output.status.code(), Some(2));
    let output = run(&["no-such-command"]);
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn seasonal_diff_requires_two_capture_files() {
    let output = run(&["seasonal-diff"]);
    assert_eq!(output.status.code(), Some(2));
    let output = run(&["seasonal-diff", "previous.json"]);
    assert_eq!(output.status.code(), Some(2));
    let output = run(&["seasonal-diff", "previous.json", "current.json", "extra"]);
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn seasonal_diff_missing_capture_fails_without_fabricating_evidence() {
    let output = run(&["seasonal-diff", "/no/previous.json", "/no/current.json"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("cannot read"));
}

#[test]
fn seasonal_diff_runs_text_and_json_success_paths_with_synthetic_schema_input() {
    let dir = std::env::temp_dir().join(format!("workshop-rs-cli-seasonal-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let previous = dir.join("previous.json");
    let current = dir.join("current.json");
    std::fs::write(&previous, synthetic_capture("capture-a")).unwrap();
    std::fs::write(&current, synthetic_capture("capture-b")).unwrap();
    let previous_arg = previous.to_string_lossy().to_string();
    let current_arg = current.to_string_lossy().to_string();

    let output = run(&["seasonal-diff", &previous_arg, &current_arg]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("RuntimeUncertainty"));

    let output = run(&["seasonal-diff", &previous_arg, &current_arg, "--json"]);
    assert!(output.status.success());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(
        report["runtimeUncertainty"]
            .as_array()
            .is_some_and(|entries| !entries.is_empty())
    );

    let _ = std::fs::remove_file(previous);
    let _ = std::fs::remove_file(current);
    let _ = std::fs::remove_dir(&dir);
}

#[test]
fn census_runs_with_machine_readable_results() {
    let output = run(&["census", "--json"]);
    assert_eq!(output.status.code(), Some(1));
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).expect("valid report");
    assert!(
        report["census"]["shards"]
            .as_array()
            .is_some_and(|shards| !shards.is_empty())
    );
    assert!(report["results"].as_array().is_some_and(|results| {
        results
            .iter()
            .any(|result| result["status"] == "inconclusive")
            && results
                .iter()
                .any(|result| result["status"] == "unexpected-regression")
    }));

    let output = run(&["census", "--json", "extra"]);
    assert_eq!(output.status.code(), Some(2));
    let output = run(&["census", "--bogus"]);
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn corpus_runs_full_and_minimized_cases_with_visible_gap() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../workshop-rs/tests/fixtures/corpus/real-projects.json");
    let output = run(&["corpus", manifest.to_str().unwrap()]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains("real-project/overpy-cake/full: Matched"),
        "{stdout}"
    );
    assert!(
        stdout.contains("minimized-regression/overpy-cake-loop: Matched"),
        "{stdout}"
    );
    assert!(
        stdout.contains("matched=2") && stdout.contains("known-gap=0"),
        "{stdout}"
    );

    let output = run(&["corpus", manifest.to_str().unwrap(), "--json"]);
    assert!(output.status.success());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).expect("valid report");
    assert_eq!(report["results"].as_array().unwrap().len(), 2);
    assert_eq!(report["summary"]["known-gap"], 0);
}

#[test]
fn corpus_missing_manifest_fails_explicitly() {
    let output = run(&["corpus", "/no/such/workshop-rs-manifest.json"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("cannot read"));
}
