//! Offline execution for real-project and regression test manifests.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;

use super::conformance::{
    CONFORMANCE_SCHEMA_VERSION, Comparison, ConformanceReason, ConformanceResult,
    ConformanceStatus, Equivalence, FeatureId, ReasonCode, TestArtifact,
};
use workshop_rs::catalog::{Catalog, CatalogIdentity, Locale};
use workshop_rs::{parser, validate};

#[derive(Debug, Deserialize)]
struct CorpusManifest {
    #[serde(rename = "schemaVersion")]
    schema_version: u32,
    id: String,
    locale: String,
    expected: TestArtifact,
    cases: Vec<CorpusCase>,
}

#[derive(Debug, Deserialize)]
struct CorpusCase {
    id: String,
    fixture: String,
    source: TestArtifact,
    features: Vec<FeatureId>,
    #[serde(rename = "expectedStatus")]
    expected_status: ExpectedStatus,
    #[serde(rename = "failureContains")]
    failure_contains: Option<String>,
    #[serde(rename = "knownGap")]
    known_gap: Option<KnownGap>,
    expected: Option<TestArtifact>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum ExpectedStatus {
    Success,
}

#[derive(Debug, Deserialize)]
struct KnownGap {
    detail: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct CorpusReport {
    #[serde(rename = "schemaVersion")]
    schema_version: u32,
    manifest: String,
    catalog: CatalogIdentity,
    results: Vec<ConformanceResult>,
    summary: CorpusSummary,
}

#[derive(Debug, Default, Serialize)]
struct CorpusSummary {
    matched: usize,
    unsupported: usize,
    #[serde(rename = "known-gap")]
    known_gap: usize,
    #[serde(rename = "unexpected-regression")]
    unexpected_regression: usize,
    inconclusive: usize,
}

impl CorpusSummary {
    fn record(&mut self, status: ConformanceStatus) {
        match status {
            ConformanceStatus::Matched => self.matched += 1,
            ConformanceStatus::Unsupported => self.unsupported += 1,
            ConformanceStatus::KnownGap => self.known_gap += 1,
            ConformanceStatus::UnexpectedRegression => self.unexpected_regression += 1,
            ConformanceStatus::Inconclusive => self.inconclusive += 1,
        }
    }
}

impl CorpusReport {
    pub(crate) fn has_unexpected_regression(&self) -> bool {
        self.summary.unexpected_regression != 0
    }

    pub(crate) fn human_summary(&self) -> String {
        let mut output = format!("manifest {}\n", self.manifest);
        for result in &self.results {
            output.push_str(&format!("{}: {:?}\n", result.case_id, result.status));
        }
        output.push_str(&format!(
            "summary: matched={}, unsupported={}, known-gap={}, unexpected-regression={}, inconclusive={}\n",
            self.summary.matched,
            self.summary.unsupported,
            self.summary.known_gap,
            self.summary.unexpected_regression,
            self.summary.inconclusive,
        ));
        output
    }
}

pub(crate) fn run(manifest_path: &Path) -> Result<CorpusReport, String> {
    let manifest_text = std::fs::read_to_string(manifest_path)
        .map_err(|error| format!("cannot read {}: {error}", manifest_path.display()))?;
    let manifest: CorpusManifest = serde_json::from_str(&manifest_text)
        .map_err(|error| format!("invalid manifest JSON: {error}"))?;
    if manifest.schema_version != 2 {
        return Err(format!(
            "unsupported manifest schema version {}; expected 2",
            manifest.schema_version
        ));
    }
    if manifest.cases.is_empty() {
        return Err("manifest must contain at least one case".to_string());
    }

    let catalog = Catalog::builtin().map_err(|error| format!("catalog: {error}"))?;
    let catalog_identity = catalog.identity();
    let locale = Locale::new(&manifest.locale);
    let manifest_dir = manifest_path
        .parent()
        .ok_or_else(|| "manifest has no parent directory".to_string())?;
    let mut results = Vec::with_capacity(manifest.cases.len());
    let mut summary = CorpusSummary::default();

    for case in manifest.cases {
        let fixture_path = manifest_dir.join(&case.fixture);
        let input = std::fs::read_to_string(&fixture_path).map_err(|error| {
            format!(
                "cannot read case {} at {}: {error}",
                case.id,
                fixture_path.display()
            )
        })?;
        validate_source_artifact(&case.id, &case.source, &input)?;
        let expected = case
            .expected
            .clone()
            .unwrap_or_else(|| manifest.expected.clone());
        validate_expected_artifact(&case.id, &expected)?;
        let result = execute_case(
            &case,
            &input,
            &catalog,
            &locale,
            case.source.clone(),
            expected,
        )?;
        result
            .validate_against(&catalog)
            .map_err(|error| format!("case {} produced invalid result: {error}", case.id))?;
        summary.record(result.status);
        results.push(result);
    }

    Ok(CorpusReport {
        schema_version: 2,
        manifest: manifest.id,
        catalog: catalog_identity,
        results,
        summary,
    })
}

fn execute_case(
    case: &CorpusCase,
    input: &str,
    catalog: &Catalog,
    locale: &Locale,
    source: TestArtifact,
    expected: TestArtifact,
) -> Result<ConformanceResult, String> {
    let parsed = parser::parse_with_context(input, catalog, locale, catalog);
    match (case.expected_status, parsed) {
        (ExpectedStatus::Success, Ok(program)) => {
            if let Err(error) = program.validate() {
                return Ok(non_match(
                    case,
                    source,
                    catalog,
                    locale,
                    ConformanceStatus::UnexpectedRegression,
                    expected,
                    ConformanceReason {
                        code: ReasonCode::UnexpectedRegression,
                        detail: format!("WIR validation failed: {error}"),
                    },
                ));
            }
            if let Err(error) = validate::validate_canonical_ids(&program, catalog) {
                return Ok(non_match(
                    case,
                    source,
                    catalog,
                    locale,
                    ConformanceStatus::UnexpectedRegression,
                    expected,
                    ConformanceReason {
                        code: ReasonCode::UnexpectedRegression,
                        detail: format!("canonical identity validation failed: {error}"),
                    },
                ));
            }
            let observed = TestArtifact {
                name: "workshop-rs canonical WIR dump".to_string(),
                revision: None,
                path: Some(case.fixture.clone()),
                sha256: Some(sha256(&program.dump())),
                license: Some("MIT".to_string()),
            };
            Ok(ConformanceResult {
                schema_version: CONFORMANCE_SCHEMA_VERSION,
                case_id: case.id.clone(),
                features: case.features.clone(),
                status: ConformanceStatus::Matched,
                comparison: Comparison {
                    mode: Equivalence::Semantic,
                    expected: Some(expected),
                    observed: Some(observed),
                    normalizer: Some("parse-validate-canonical-wir-v1".to_string()),
                },
                source,
                catalog: catalog.identity(),
                locale: Some(locale.clone()),
                reason: None,
            })
        }
        (ExpectedStatus::Success, Err(error)) => {
            let detail = format!("offline parser result: {error}");
            let declared_gap = case.known_gap.as_ref().filter(|_| {
                case.failure_contains
                    .as_ref()
                    .is_some_and(|needle| error.to_string().contains(needle))
            });
            if let Some(gap) = declared_gap {
                Ok(non_match(
                    case,
                    source,
                    catalog,
                    locale,
                    ConformanceStatus::KnownGap,
                    expected,
                    ConformanceReason {
                        code: ReasonCode::KnownGap,
                        detail: gap.detail.clone(),
                    },
                ))
            } else {
                Ok(non_match(
                    case,
                    source,
                    catalog,
                    locale,
                    ConformanceStatus::UnexpectedRegression,
                    expected,
                    ConformanceReason {
                        code: ReasonCode::UnexpectedRegression,
                        detail,
                    },
                ))
            }
        }
    }
}

fn non_match(
    case: &CorpusCase,
    source: TestArtifact,
    catalog: &Catalog,
    locale: &Locale,
    status: ConformanceStatus,
    expected: TestArtifact,
    reason: ConformanceReason,
) -> ConformanceResult {
    ConformanceResult {
        schema_version: CONFORMANCE_SCHEMA_VERSION,
        case_id: case.id.clone(),
        features: case.features.clone(),
        status,
        comparison: Comparison {
            mode: Equivalence::Semantic,
            expected: Some(expected),
            observed: None,
            normalizer: Some("parse-validate-canonical-wir-v1".to_string()),
        },
        source,
        catalog: catalog.identity(),
        locale: Some(locale.clone()),
        reason: Some(reason),
    }
}

fn sha256(value: &str) -> String {
    let digest = Sha256::digest(value.as_bytes());
    format!("{digest:x}")
}

fn validate_source_artifact(
    case_id: &str,
    artifact: &TestArtifact,
    input: &str,
) -> Result<(), String> {
    validate_expected_artifact(case_id, artifact)?;
    let Some(expected_digest) = artifact.sha256.as_deref() else {
        return Err(format!("case {case_id} source must pin a SHA-256 digest"));
    };
    let actual_digest = sha256(input);
    if expected_digest != actual_digest {
        return Err(format!(
            "case {case_id} source digest mismatch: manifest {expected_digest}, fixture {actual_digest}"
        ));
    }
    Ok(())
}

fn validate_expected_artifact(case_id: &str, artifact: &TestArtifact) -> Result<(), String> {
    if artifact.name.trim().is_empty() {
        return Err(format!("case {case_id} test artifact has no identity"));
    }
    if artifact.revision.as_deref().is_none_or(str::is_empty) {
        return Err(format!("case {case_id} test artifact must pin a revision"));
    }
    if artifact.path.as_deref().is_none_or(str::is_empty) {
        return Err(format!("case {case_id} test artifact must pin a path"));
    }
    let Some(digest) = artifact.sha256.as_deref() else {
        return Err(format!(
            "case {case_id} test artifact must pin a SHA-256 digest"
        ));
    };
    if digest.len() != 64
        || !digest
            .chars()
            .all(|character| character.is_ascii_hexdigit())
    {
        return Err(format!(
            "case {case_id} test artifact has an invalid SHA-256 digest"
        ));
    }
    if artifact.license.as_deref().is_none_or(str::is_empty) {
        return Err(format!(
            "case {case_id} test artifact must record a license"
        ));
    }
    Ok(())
}
