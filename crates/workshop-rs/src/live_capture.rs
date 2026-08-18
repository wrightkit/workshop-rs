//! Offline schema and comparison support for provenance-recorded client captures.
//!
//! This module admits already-recorded evidence; it does not start, control, or
//! query an Overwatch client. A valid [`LiveCapture`] is structurally
//! provenance-rich, but its metadata is still a claim that requires human
//! review and cannot establish gameplay/runtime correctness by itself.

use std::collections::{BTreeMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::catalog::{Catalog, CatalogIdentity, Locale};
pub use crate::census::{CENSUS_IDENTITY_SCHEMA_VERSION, CensusIdentity};
use crate::conformance::{
    ConformanceResult, ConformanceStatus, Equivalence, EvidenceArtifact, EvidenceBasis,
    EvidenceClass, FeatureId,
};

/// The current machine-readable live-capture schema version.
pub const LIVE_CAPTURE_SCHEMA_VERSION: u32 = 1;

/// One machine-readable capture from a manually operated Workshop client.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveCapture {
    pub schema_version: u32,
    pub capture_id: String,
    pub game: String,
    /// The client version/build string as observed by the maintainer.
    pub client: String,
    pub season: String,
    pub captured_at: String,
    /// Environment notes, including platform and any conditions relevant to
    /// interpreting import/export behavior.
    pub environment: String,
    pub locale: Locale,
    pub catalog: CatalogIdentity,
    pub census: CensusIdentity,
    /// The exact exported Workshop text or archive, pinned by revision/path
    /// and SHA-256. The bytes are intentionally not embedded in this schema.
    pub raw_artifact: EvidenceArtifact,
    /// #18 feature-attributed observations for the captured probes.
    pub results: Vec<ConformanceResult>,
}

impl LiveCapture {
    /// Validate a capture's structural and provenance contract. Historical
    /// captures may refer to an older catalog identity, so this does not
    /// silently substitute the current bundled catalog.
    pub fn validate(&self) -> Result<(), LiveCaptureError> {
        self.validate_structural(None)
    }

    /// Validate a current capture against the loaded canonical catalog.
    pub fn validate_against(&self, catalog: &Catalog) -> Result<(), LiveCaptureError> {
        self.validate_structural(Some(catalog))
    }

    fn validate_structural(&self, catalog: Option<&Catalog>) -> Result<(), LiveCaptureError> {
        if self.schema_version != LIVE_CAPTURE_SCHEMA_VERSION {
            return Err(invalid(format!(
                "unsupported live capture schema version {}; expected {}",
                self.schema_version, LIVE_CAPTURE_SCHEMA_VERSION
            )));
        }
        validate_name("captureId", &self.capture_id)?;
        validate_name("game", &self.game)?;
        validate_name("client", &self.client)?;
        validate_name("season", &self.season)?;
        validate_timestamp(&self.captured_at)?;
        validate_name("environment", &self.environment)?;
        validate_name("locale", self.locale.as_str())?;
        validate_catalog(&self.catalog)?;
        if normalize_game(&self.catalog.target.game) != normalize_game(&self.game) {
            return Err(invalid("capture game does not match catalog target game"));
        }
        if let Some(catalog) = catalog {
            if self.catalog != catalog.identity() {
                return Err(invalid(
                    "capture catalog identity does not match the loaded catalog",
                ));
            }
        }
        validate_census(&self.census)?;
        validate_artifact("rawArtifact", &self.raw_artifact, true)?;
        if self.results.is_empty() {
            return Err(invalid(
                "results must contain at least one #18 conformance result",
            ));
        }

        let mut case_ids = HashSet::with_capacity(self.results.len());
        for (index, result) in self.results.iter().enumerate() {
            let validation = match catalog {
                Some(catalog) => result.validate_against(catalog),
                None => result.validate(),
            };
            validation.map_err(|error| invalid(format!("results[{index}]: {error}")))?;
            if !case_ids.insert(&result.case_id) {
                return Err(invalid(format!(
                    "results[{index}].caseId duplicates another capture result"
                )));
            }
            let evidence = &result.evidence;
            if evidence.class != EvidenceClass::LiveClient {
                return Err(invalid(format!(
                    "results[{index}].evidence.class must be live-client"
                )));
            }
            if evidence.expectation.basis != EvidenceBasis::WorkshopClient {
                return Err(invalid(format!(
                    "results[{index}].evidence.expectation.basis must be workshop-client"
                )));
            }
            if evidence.catalog != self.catalog {
                return Err(invalid(format!(
                    "results[{index}].evidence.catalog does not match capture catalog"
                )));
            }
            if evidence.locale.as_ref() != Some(&self.locale) {
                return Err(invalid(format!(
                    "results[{index}].evidence.locale does not match capture locale"
                )));
            }
            if evidence.fixture != self.raw_artifact {
                return Err(invalid(format!(
                    "results[{index}].evidence.fixture must pin the capture raw artifact"
                )));
            }
            let client = evidence
                .client
                .as_ref()
                .ok_or_else(|| invalid(format!("results[{index}].evidence.client is required")))?;
            if client.game != self.game
                || client.client_version.as_deref() != Some(self.client.as_str())
                || client.season.as_deref() != Some(self.season.as_str())
                || client.captured_at != self.captured_at
                || client.environment.as_deref() != Some(self.environment.as_str())
            {
                return Err(invalid(format!(
                    "results[{index}].evidence.client does not match capture client provenance"
                )));
            }
            if result.status == ConformanceStatus::Matched {
                let observed = result.comparison.observed.as_ref().ok_or_else(|| {
                    invalid(format!(
                        "results[{index}].comparison.observed is required for matched evidence"
                    ))
                })?;
                if observed.sha256 != self.raw_artifact.sha256 {
                    return Err(invalid(format!(
                        "results[{index}].comparison.observed must pin the raw capture digest"
                    )));
                }
            }
        }
        Ok(())
    }

    /// Deserialize and validate a JSON capture in one operation.
    pub fn from_json(json: &str) -> Result<Self, LiveCaptureError> {
        let capture: Self = serde_json::from_str(json)
            .map_err(|error| invalid(format!("invalid live capture JSON: {error}")))?;
        capture.validate()?;
        Ok(capture)
    }

    /// Serialize a validated capture as stable, human-reviewable JSON.
    pub fn to_json(&self) -> Result<String, LiveCaptureError> {
        self.validate()?;
        serde_json::to_string_pretty(self)
            .map_err(|error| invalid(format!("cannot serialize live capture: {error}")))
    }

    /// Compare two validated captures without contacting a provider or client.
    pub fn diff(&self, newer: &Self) -> Result<LiveCaptureDiff, LiveCaptureError> {
        self.validate()?;
        newer.validate()?;

        let mut changes = Vec::new();
        if self.locale != newer.locale {
            changes.push(DiffEntry::metadata(
                DiffCategory::Locale,
                format!("locale changed from {} to {}", self.locale, newer.locale),
            ));
        }
        if self.catalog != newer.catalog {
            changes.push(DiffEntry::metadata(
                DiffCategory::Catalog,
                "catalog identity changed",
            ));
        }
        if self.census != newer.census {
            changes.push(DiffEntry::metadata(
                DiffCategory::SemanticSchema,
                "census identity or shard set changed",
            ));
        }
        if self.raw_artifact != newer.raw_artifact {
            changes.push(DiffEntry::metadata(
                DiffCategory::Content,
                "raw client artifact provenance or content changed",
            ));
        }

        let prior: BTreeMap<_, _> = self
            .results
            .iter()
            .map(|result| (result.case_id.as_str(), result))
            .collect();
        let current: BTreeMap<_, _> = newer
            .results
            .iter()
            .map(|result| (result.case_id.as_str(), result))
            .collect();
        let mut all_case_ids: Vec<&str> = prior.keys().chain(current.keys()).copied().collect();
        all_case_ids.sort_unstable();
        all_case_ids.dedup();

        let mut runtime_uncertainty = vec![DiffEntry::metadata(
            DiffCategory::RuntimeUncertainty,
            "import/export capture does not establish gameplay or runtime behavior",
        )];
        for case_id in all_case_ids {
            match (prior.get(case_id), current.get(case_id)) {
                (None, Some(result)) => changes.push(DiffEntry::result(
                    DiffCategory::Content,
                    result,
                    "feature-attributed result was added",
                )),
                (Some(result), None) => changes.push(DiffEntry::result(
                    DiffCategory::Content,
                    result,
                    "feature-attributed result was removed",
                )),
                (Some(previous), Some(current)) => {
                    let features_changed = !same_features(previous, current);
                    if features_changed {
                        changes.push(DiffEntry::result(
                            DiffCategory::SemanticSchema,
                            current,
                            "feature attribution changed",
                        ));
                    }

                    let uncertain = is_runtime_uncertain(previous) || is_runtime_uncertain(current);
                    if uncertain {
                        runtime_uncertainty.push(DiffEntry::result(
                            DiffCategory::RuntimeUncertainty,
                            current,
                            "result is not a comparable semantic match",
                        ));
                    }
                    if previous.status != current.status {
                        if !uncertain {
                            changes.push(DiffEntry::result(
                                DiffCategory::SemanticSchema,
                                current,
                                format!(
                                    "conformance status changed from {:?} to {:?}",
                                    previous.status, current.status
                                ),
                            ));
                        }
                    } else if previous.comparison.mode != current.comparison.mode {
                        changes.push(DiffEntry::result(
                            DiffCategory::SemanticSchema,
                            current,
                            "comparison mode changed",
                        ));
                    } else if previous.comparison != current.comparison {
                        changes.push(DiffEntry::result(
                            DiffCategory::Content,
                            current,
                            "expected or observed feature artifact changed",
                        ));
                    }
                }
                (None, None) => unreachable!("case ID came from one of the result maps"),
            }
        }

        Ok(LiveCaptureDiff {
            schema_version: LIVE_CAPTURE_SCHEMA_VERSION,
            prior_capture_id: self.capture_id.clone(),
            new_capture_id: newer.capture_id.clone(),
            changes,
            runtime_uncertainty,
        })
    }
}

/// The classification used by the offline capture comparison.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DiffCategory {
    Locale,
    Catalog,
    Content,
    SemanticSchema,
    RuntimeUncertainty,
}

/// One feature-attributed or capture-level diff observation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiffEntry {
    pub category: DiffCategory,
    pub case_id: Option<String>,
    pub features: Vec<FeatureId>,
    pub detail: String,
}

impl DiffEntry {
    fn metadata(category: DiffCategory, detail: impl Into<String>) -> Self {
        Self {
            category,
            case_id: None,
            features: Vec::new(),
            detail: detail.into(),
        }
    }

    fn result(
        category: DiffCategory,
        result: &ConformanceResult,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            category,
            case_id: Some(result.case_id.clone()),
            features: result.features.clone(),
            detail: detail.into(),
        }
    }
}

/// Machine-readable output of [`LiveCapture::diff`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveCaptureDiff {
    pub schema_version: u32,
    pub prior_capture_id: String,
    pub new_capture_id: String,
    pub changes: Vec<DiffEntry>,
    /// Runtime/gameplay uncertainty is intentionally separate from
    /// import/export changes and is always present for this workflow.
    pub runtime_uncertainty: Vec<DiffEntry>,
}

impl LiveCaptureDiff {
    pub fn to_json(&self) -> Result<String, LiveCaptureError> {
        serde_json::to_string_pretty(self)
            .map_err(|error| invalid(format!("cannot serialize live capture diff: {error}")))
    }

    pub fn human_summary(&self) -> String {
        let mut output = format!(
            "live capture diff schema {}\n{} -> {}\n",
            self.schema_version, self.prior_capture_id, self.new_capture_id
        );
        for entry in self.changes.iter().chain(self.runtime_uncertainty.iter()) {
            output.push_str(&format!("{:?}: {}\n", entry.category, entry.detail));
        }
        if self.changes.is_empty() {
            output.push_str("no import/export changes classified\n");
        }
        output
    }
}

/// A validation or serialization failure in the offline capture workflow.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveCaptureError {
    pub message: String,
}

impl std::fmt::Display for LiveCaptureError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for LiveCaptureError {}

fn invalid(message: impl Into<String>) -> LiveCaptureError {
    LiveCaptureError {
        message: message.into(),
    }
}

fn validate_name(field: &str, value: &str) -> Result<(), LiveCaptureError> {
    if value.trim().is_empty() || value.chars().any(char::is_control) {
        Err(invalid(format!("{field} must be non-empty and printable")))
    } else {
        Ok(())
    }
}

fn validate_timestamp(value: &str) -> Result<(), LiveCaptureError> {
    validate_name("capturedAt", value)?;
    if !value.contains('T') || !(value.ends_with('Z') || value.contains('+')) {
        return Err(invalid(
            "capturedAt must use an ISO-8601 timestamp with a timezone",
        ));
    }
    Ok(())
}

fn validate_catalog(catalog: &CatalogIdentity) -> Result<(), LiveCaptureError> {
    validate_name(
        "catalog.implementationVersion",
        &catalog.implementation_version,
    )?;
    validate_name("catalog.catalogVersion", &catalog.catalog_version)?;
    let digest = catalog
        .catalog_digest
        .as_deref()
        .ok_or_else(|| invalid("catalog.catalogDigest is required to pin live evidence"))?;
    validate_sha256("catalog.catalogDigest", digest)?;
    validate_name("catalog.target.game", &catalog.target.game)?;
    validate_name("catalog.target.format", &catalog.target.format)?;
    validate_name("catalog.target.surface", &catalog.target.surface)?;
    if catalog.locale_coverage.is_empty() {
        return Err(invalid("catalog.localeCoverage must not be empty"));
    }
    Ok(())
}

fn validate_census(census: &CensusIdentity) -> Result<(), LiveCaptureError> {
    if census.schema_version != CENSUS_IDENTITY_SCHEMA_VERSION {
        return Err(invalid(format!(
            "unsupported census identity schema version {}; expected {}",
            census.schema_version, CENSUS_IDENTITY_SCHEMA_VERSION
        )));
    }
    validate_sha256("census.digest", &census.digest)?;
    if census.shards.is_empty() || census.shards.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(invalid(
            "census.shards must be non-empty and strictly sorted",
        ));
    }
    Ok(())
}

fn validate_artifact(
    field: &str,
    artifact: &EvidenceArtifact,
    require_pin: bool,
) -> Result<(), LiveCaptureError> {
    validate_name(&format!("{field}.name"), &artifact.name)?;
    if require_pin {
        validate_name(
            &format!("{field}.revision"),
            artifact.revision.as_deref().unwrap_or_default(),
        )?;
        validate_name(
            &format!("{field}.path"),
            artifact.path.as_deref().unwrap_or_default(),
        )?;
        validate_sha256(
            &format!("{field}.sha256"),
            artifact.sha256.as_deref().unwrap_or_default(),
        )?;
    }
    Ok(())
}

fn validate_sha256(field: &str, digest: &str) -> Result<(), LiveCaptureError> {
    if digest.len() != 64 || !digest.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(invalid(format!(
            "{field} must be a 64-character hexadecimal SHA-256 digest"
        )));
    }
    Ok(())
}

fn normalize_game(value: &str) -> String {
    value
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .flat_map(char::to_lowercase)
        .collect()
}

fn same_features(left: &ConformanceResult, right: &ConformanceResult) -> bool {
    left.features.len() == right.features.len()
        && left
            .features
            .iter()
            .all(|feature| right.features.contains(feature))
}

fn is_runtime_uncertain(result: &ConformanceResult) -> bool {
    !result.status.is_match() || result.comparison.mode == Equivalence::NotComparable
}

#[cfg(test)]
mod tests {
    //! These are constructed schema/diff unit tests only. They are not live
    //! client captures or runtime evidence.

    use super::*;
    use crate::catalog::Catalog;
    use crate::conformance::{
        ClientEvidence, Comparison, ConformanceReason, Evidence, ExpectationSource, FeatureKind,
        FeatureNamespace, ReasonCode,
    };

    const DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const OTHER_DIGEST: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

    fn catalog() -> CatalogIdentity {
        Catalog::builtin()
            .expect("built-in catalog for constructed unit data")
            .identity()
    }

    fn raw(digest: &str) -> EvidenceArtifact {
        EvidenceArtifact {
            name: "constructed-unit-test/raw.ws".to_string(),
            revision: Some("unit-test-revision".to_string()),
            path: Some("constructed-unit-test/raw.ws".to_string()),
            sha256: Some(digest.to_string()),
            license: Some("MIT".to_string()),
        }
    }

    fn feature(name: &str) -> FeatureId {
        FeatureId::owned(FeatureNamespace::Wir, FeatureKind::Structural, name)
            .expect("constructed feature identity")
    }

    fn result(
        identity: &CatalogIdentity,
        locale: &Locale,
        raw: &EvidenceArtifact,
        case_id: &str,
        status: ConformanceStatus,
        feature_name: &str,
    ) -> ConformanceResult {
        let matched = status == ConformanceStatus::Matched;
        ConformanceResult {
            schema_version: CONFORMANCE_SCHEMA_VERSION,
            case_id: case_id.to_string(),
            features: vec![feature(feature_name)],
            status,
            comparison: Comparison {
                mode: if matched {
                    Equivalence::Normalized
                } else {
                    Equivalence::NotComparable
                },
                expected: matched.then(|| EvidenceArtifact::new("constructed-unit-test/oracle")),
                observed: matched.then(|| EvidenceArtifact {
                    name: "constructed-unit-test/observed.ws".to_string(),
                    revision: Some("unit-test-revision".to_string()),
                    path: Some("constructed-unit-test/raw.ws".to_string()),
                    sha256: Some(raw.sha256.clone().unwrap()),
                    license: Some("MIT".to_string()),
                }),
                normalizer: matched.then(|| "constructed-unit-test-normalizer".to_string()),
            },
            evidence: Evidence {
                class: EvidenceClass::LiveClient,
                fixture: raw.clone(),
                expectation: ExpectationSource {
                    basis: EvidenceBasis::WorkshopClient,
                    artifact: EvidenceArtifact::new("constructed-unit-test/client"),
                    tracking_ref: None,
                },
                catalog: identity.clone(),
                locale: Some(locale.clone()),
                client: Some(ClientEvidence {
                    game: "overwatch-2".to_string(),
                    client_version: Some("constructed-unit-test-client".to_string()),
                    season: Some("constructed-unit-test-season".to_string()),
                    captured_at: "2026-08-18T00:00:00Z".to_string(),
                    environment: Some(
                        "constructed schema/diff unit test; not live evidence".to_string(),
                    ),
                }),
                implementation: None,
            },
            reason: (!matched).then(|| ConformanceReason {
                code: ReasonCode::Inconclusive,
                detail: "constructed unit uncertainty".to_string(),
                tracking_ref: None,
            }),
        }
    }

    fn make_capture(
        capture_id: &str,
        locale: &str,
        digest: &str,
        result_status: ConformanceStatus,
        feature_name: &str,
    ) -> LiveCapture {
        let identity = catalog();
        let locale = Locale::new(locale);
        let raw = raw(digest);
        LiveCapture {
            schema_version: LIVE_CAPTURE_SCHEMA_VERSION,
            capture_id: capture_id.to_string(),
            game: "overwatch-2".to_string(),
            client: "constructed-unit-test-client".to_string(),
            season: "constructed-unit-test-season".to_string(),
            captured_at: "2026-08-18T00:00:00Z".to_string(),
            environment: "constructed schema/diff unit test; not live evidence".to_string(),
            locale: locale.clone(),
            catalog: identity.clone(),
            census: CensusIdentity {
                schema_version: CENSUS_SCHEMA_VERSION,
                conformance_schema_version: CONFORMANCE_SCHEMA_VERSION,
                identity: "constructed-unit-test-census".to_string(),
                shards: vec!["constructed-unit-test-shard".to_string()],
            },
            raw_artifact: raw.clone(),
            results: vec![result(
                &identity,
                &locale,
                &raw,
                "constructed-unit-test/case",
                result_status,
                feature_name,
            )],
        }
    }

    #[test]
    fn constructed_capture_schema_round_trips_without_live_evidence_claim() {
        let capture = make_capture(
            "capture-a",
            "en-US",
            DIGEST,
            ConformanceStatus::Matched,
            "one",
        );
        let json = capture.to_json().expect("constructed schema serializes");
        let decoded = LiveCapture::from_json(&json).expect("constructed schema validates");
        assert_eq!(decoded, capture);
    }

    #[test]
    fn schema_rejects_missing_raw_pin_and_client_provenance() {
        let mut capture = make_capture(
            "capture-a",
            "en-US",
            DIGEST,
            ConformanceStatus::Matched,
            "one",
        );
        capture.raw_artifact.sha256 = None;
        assert!(capture.validate().is_err());

        let mut capture = make_capture(
            "capture-a",
            "en-US",
            DIGEST,
            ConformanceStatus::Matched,
            "one",
        );
        capture.results[0].evidence.client = None;
        assert!(capture.validate().is_err());

        let mut capture = make_capture(
            "capture-a",
            "en-US",
            DIGEST,
            ConformanceStatus::Matched,
            "one",
        );
        capture.game = "not-overwatch".to_string();
        assert!(capture.validate().is_err());

        let mut capture = make_capture(
            "capture-a",
            "en-US",
            DIGEST,
            ConformanceStatus::Matched,
            "one",
        );
        capture.results[0]
            .evidence
            .client
            .as_mut()
            .unwrap()
            .environment = Some("different environment".to_string());
        assert!(capture.validate().is_err());
    }

    #[test]
    fn diff_reports_requested_categories_and_separates_runtime_uncertainty() {
        let prior = make_capture(
            "capture-a",
            "en-US",
            DIGEST,
            ConformanceStatus::Matched,
            "one",
        );
        let mut newer = make_capture(
            "capture-b",
            "zh-CN",
            OTHER_DIGEST,
            ConformanceStatus::Inconclusive,
            "two",
        );
        newer.census.identity = "constructed-unit-test-census-v2".to_string();
        newer.catalog.catalog_digest = Some(OTHER_DIGEST.to_string());
        newer.results[0].evidence.catalog = newer.catalog.clone();
        let diff = prior.diff(&newer).expect("constructed diff validates");
        let categories: HashSet<_> = diff.changes.iter().map(|entry| entry.category).collect();
        assert!(categories.contains(&DiffCategory::Locale));
        assert!(categories.contains(&DiffCategory::Catalog));
        assert!(categories.contains(&DiffCategory::Content));
        assert!(categories.contains(&DiffCategory::SemanticSchema));
        assert!(!diff.runtime_uncertainty.is_empty());
        assert!(
            diff.runtime_uncertainty
                .iter()
                .all(|entry| entry.category == DiffCategory::RuntimeUncertainty)
        );
        let json = diff.to_json().expect("structured diff serializes");
        let document: serde_json::Value = serde_json::from_str(&json).expect("valid diff JSON");
        assert!(document["changes"].is_array());
        assert!(document["runtimeUncertainty"].is_array());
    }

    #[test]
    fn diff_refuses_capture_without_live_client_provenance() {
        let prior = make_capture(
            "capture-a",
            "en-US",
            DIGEST,
            ConformanceStatus::Matched,
            "one",
        );
        let mut newer = make_capture(
            "capture-b",
            "en-US",
            DIGEST,
            ConformanceStatus::Matched,
            "one",
        );
        newer.results[0].evidence.expectation.basis = EvidenceBasis::SemanticContract;
        assert!(prior.diff(&newer).is_err());
    }
}
