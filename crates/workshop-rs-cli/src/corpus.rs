//! Offline execution for provenance-linked real-project evidence manifests.
//!
//! The manifest describes source and expectation provenance; this runner only
//! measures the bundled Workshop text with the canonical parser and WIR
//! validation. It never creates an expectation from the observed output.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;

use workshop_rs::catalog::{Catalog, CatalogIdentity, Locale};
use workshop_rs::conformance::{
    Comparison, ConformanceReason, ConformanceResult, ConformanceStatus, Equivalence, Evidence,
    EvidenceArtifact, EvidenceClass, ExpectationSource, FeatureId, ReasonCode,
};
use workshop_rs::{parser, validate};

#[derive(Debug, Deserialize)]
struct CorpusManifest {
    #[serde(rename = "schemaVersion")]
    schema_version: u32,
    id: String,
    locale: String,
    source: EvidenceArtifact,
    expectation: ExpectationSource,
    cases: Vec<CorpusCase>,
}

#[derive(Debug, Deserialize)]
struct CorpusCase {
    id: String,
    class: EvidenceClass,
    fixture: String,
    features: Vec<FeatureId>,
    #[serde(rename = "expectedStatus")]
    expected_status: ExpectedStatus,
    #[serde(rename = "failureContains")]
    failure_contains: Option<String>,
    #[serde(rename = "knownGap")]
    known_gap: Option<KnownGap>,
    #[serde(rename = "derivedFrom")]
    derived_from: Option<String>,
    expectation: Option<ExpectationSource>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum ExpectedStatus {
    Success,
}

#[derive(Debug, Deserialize)]
struct KnownGap {
    detail: String,
    #[serde(rename = "trackingRef")]
    tracking_ref: String,
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
    if manifest.schema_version != 1 {
        return Err(format!(
            "unsupported manifest schema version {}; expected 1",
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
        let fixture = EvidenceArtifact {
            name: manifest.source.name.clone(),
            revision: manifest.source.revision.clone(),
            path: Some(case.fixture.clone()),
            sha256: Some(sha256(&input)),
            license: manifest.source.license.clone(),
        };
        let expectation = case
            .expectation
            .clone()
            .unwrap_or_else(|| manifest.expectation.clone());
        let evidence = Evidence {
            class: case.class,
            fixture,
            expectation: expectation.clone(),
            catalog: catalog_identity.clone(),
            locale: Some(locale.clone()),
            client: None,
            implementation: Some(workshop_rs::conformance::ImplementationIdentity {
                name: "workshop-rs".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                revision: None,
                artifact: None,
            }),
        };
        let result = execute_case(&case, &input, &catalog, &locale, evidence, expectation)?;
        result
            .validate_against(&catalog)
            .map_err(|error| format!("case {} produced invalid #18 result: {error}", case.id))?;
        summary.record(result.status);
        results.push(result);
    }

    Ok(CorpusReport {
        schema_version: 1,
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
    evidence: Evidence,
    expectation: ExpectationSource,
) -> Result<ConformanceResult, String> {
    let expected = expectation.artifact.clone();
    let parsed = parser::parse_with_context(input, catalog, locale, catalog);
    match (case.expected_status, parsed) {
        (ExpectedStatus::Success, Ok(program)) => {
            if let Err(error) = program.validate() {
                return Ok(non_match(
                    case,
                    evidence,
                    ConformanceStatus::UnexpectedRegression,
                    expected,
                    format!("WIR validation failed: {error}"),
                    ReasonCode::UnexpectedRegression,
                    None,
                ));
            }
            if let Err(error) = validate::validate_canonical_ids(&program, catalog) {
                return Ok(non_match(
                    case,
                    evidence,
                    ConformanceStatus::UnexpectedRegression,
                    expected,
                    format!("canonical identity validation failed: {error}"),
                    ReasonCode::UnexpectedRegression,
                    None,
                ));
            }
            let observed = EvidenceArtifact {
                name: "workshop-rs canonical WIR dump".to_string(),
                revision: None,
                path: Some(case.fixture.clone()),
                sha256: Some(sha256(&program.dump())),
                license: Some("MIT".to_string()),
            };
            Ok(ConformanceResult {
                schema_version: 1,
                case_id: case.id.clone(),
                features: case.features.clone(),
                status: ConformanceStatus::Matched,
                comparison: Comparison {
                    mode: Equivalence::Semantic,
                    expected: Some(expected),
                    observed: Some(observed),
                    normalizer: Some("parse-validate-canonical-wir-v1".to_string()),
                },
                evidence,
                reason: None,
            })
        }
        (ExpectedStatus::Success, Err(error)) => {
            let detail = format!("offline parser observation: {error}");
            let declared_gap = case.known_gap.as_ref().filter(|_gap| {
                case.failure_contains
                    .as_ref()
                    .is_some_and(|needle| error.to_string().contains(needle))
            });
            if let Some(gap) = declared_gap {
                Ok(non_match(
                    case,
                    evidence,
                    ConformanceStatus::KnownGap,
                    expected,
                    detail,
                    ReasonCode::KnownGap,
                    Some((gap.detail.clone(), gap.tracking_ref.clone())),
                ))
            } else {
                Ok(non_match(
                    case,
                    evidence,
                    ConformanceStatus::UnexpectedRegression,
                    expected,
                    detail,
                    ReasonCode::UnexpectedRegression,
                    case.derived_from.as_ref().map(|source| {
                        (format!("case is derived from {source}"), "#20".to_string())
                    }),
                ))
            }
        }
    }
}

fn non_match(
    case: &CorpusCase,
    evidence: Evidence,
    status: ConformanceStatus,
    expected: EvidenceArtifact,
    detail: String,
    code: ReasonCode,
    gap: Option<(String, String)>,
) -> ConformanceResult {
    let reason = gap.map(|(detail, tracking_ref)| ConformanceReason {
        code,
        detail,
        tracking_ref: Some(tracking_ref),
    });
    let reason = reason.or(Some(ConformanceReason {
        code,
        detail,
        tracking_ref: None,
    }));
    ConformanceResult {
        schema_version: 1,
        case_id: case.id.clone(),
        features: case.features.clone(),
        status,
        comparison: Comparison {
            mode: Equivalence::Semantic,
            expected: Some(expected),
            observed: None,
            normalizer: Some("parse-validate-canonical-wir-v1".to_string()),
        },
        evidence,
        reason,
    }
}

fn sha256(value: &str) -> String {
    let digest = Sha256::digest(value.as_bytes());
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}
