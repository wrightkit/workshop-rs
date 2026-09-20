//! Public contracts for Workshop test results.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use workshop_rs::catalog::{Catalog, CatalogIdentity, Kind, Locale};

pub const CONFORMANCE_SCHEMA_VERSION: u32 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum FeatureNamespace {
    Catalog,
    Wir,
    Settings,
    Localization,
}

impl FeatureNamespace {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Catalog => "catalog",
            Self::Wir => "wir",
            Self::Settings => "settings",
            Self::Localization => "localization",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum FeatureKind {
    Event,
    Action,
    Value,
    Operator,
    Enum,
    EnumMember,
    Setting,
    Variable,
    Subroutine,
    ControlFlow,
    String,
    Localization,
    ContentId,
    Structural,
}

impl FeatureKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Event => "event",
            Self::Action => "action",
            Self::Value => "value",
            Self::Operator => "operator",
            Self::Enum => "enum",
            Self::EnumMember => "enum-member",
            Self::Setting => "setting",
            Self::Variable => "variable",
            Self::Subroutine => "subroutine",
            Self::ControlFlow => "control-flow",
            Self::String => "string",
            Self::Localization => "localization",
            Self::ContentId => "content-id",
            Self::Structural => "structural",
        }
    }
}

impl From<Kind> for FeatureKind {
    fn from(kind: Kind) -> Self {
        match kind {
            Kind::Structural => Self::Structural,
            Kind::Action => Self::Action,
            Kind::Value => Self::Value,
            Kind::Event => Self::Event,
            Kind::Operator => Self::Operator,
            Kind::Enum => Self::Enum,
            Kind::Setting => Self::Setting,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct FeatureId {
    pub namespace: FeatureNamespace,
    pub kind: FeatureKind,
    pub name: String,
}

impl FeatureId {
    pub fn new(
        namespace: FeatureNamespace,
        kind: FeatureKind,
        name: impl Into<String>,
    ) -> Result<Self, ConformanceError> {
        let name = name.into();
        if name.is_empty() {
            return Err(ConformanceError::invalid(
                "feature.name",
                "must not be empty",
            ));
        }
        if name
            .chars()
            .any(|character| character.is_whitespace() || character.is_control())
        {
            return Err(ConformanceError::invalid(
                "feature.name",
                "must not contain whitespace or control characters",
            ));
        }
        Ok(Self {
            namespace,
            kind,
            name,
        })
    }

    pub fn from_catalog(kind: Kind, id: impl Into<String>) -> Result<Self, ConformanceError> {
        Self::new(FeatureNamespace::Catalog, kind.into(), id)
    }

    pub fn from_enum_member(
        domain: impl Into<String>,
        member: impl Into<String>,
    ) -> Result<Self, ConformanceError> {
        let domain = domain.into();
        let member = member.into();
        if domain.is_empty() || member.is_empty() {
            return Err(ConformanceError::invalid(
                "feature.name",
                "enum member identities require a domain and member",
            ));
        }
        Self::new(
            FeatureNamespace::Catalog,
            FeatureKind::EnumMember,
            format!("{domain}/{member}"),
        )
    }

    pub fn owned(
        namespace: FeatureNamespace,
        kind: FeatureKind,
        name: impl Into<String>,
    ) -> Result<Self, ConformanceError> {
        Self::new(namespace, kind, name)
    }
}

/// An immutable identity for a test input, expected result, or observed result.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct TestArtifact {
    pub name: String,
    pub revision: Option<String>,
    pub path: Option<String>,
    #[serde(rename = "sha256")]
    pub sha256: Option<String>,
    pub license: Option<String>,
}

impl TestArtifact {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            revision: None,
            path: None,
            sha256: None,
            license: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Equivalence {
    Semantic,
    Normalized,
    ExactText,
    NotComparable,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Comparison {
    pub mode: Equivalence,
    pub expected: Option<TestArtifact>,
    pub observed: Option<TestArtifact>,
    pub normalizer: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ReasonCode {
    Unsupported,
    KnownGap,
    UnexpectedRegression,
    Inconclusive,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConformanceReason {
    pub code: ReasonCode,
    pub detail: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConformanceStatus {
    Matched,
    Unsupported,
    KnownGap,
    UnexpectedRegression,
    Inconclusive,
}

impl ConformanceStatus {
    pub const fn is_match(self) -> bool {
        matches!(self, Self::Matched)
    }
}

/// One machine-readable result from a Workshop test or reference comparison.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConformanceResult {
    pub schema_version: u32,
    pub case_id: String,
    pub features: Vec<FeatureId>,
    pub status: ConformanceStatus,
    pub comparison: Comparison,
    pub source: TestArtifact,
    pub catalog: CatalogIdentity,
    pub locale: Option<Locale>,
    pub reason: Option<ConformanceReason>,
}

impl ConformanceResult {
    pub fn validate(&self) -> Result<(), ConformanceError> {
        if self.schema_version != CONFORMANCE_SCHEMA_VERSION {
            return Err(ConformanceError::invalid(
                "schemaVersion",
                format!(
                    "unsupported schema version {}; expected {}",
                    self.schema_version, CONFORMANCE_SCHEMA_VERSION
                ),
            ));
        }
        validate_non_empty("caseId", &self.case_id)?;
        let mut seen_features: HashSet<&FeatureId> = HashSet::with_capacity(self.features.len());
        if self.features.is_empty() {
            return Err(ConformanceError::invalid(
                "features",
                "must contain at least one feature",
            ));
        }
        for (index, feature) in self.features.iter().enumerate() {
            FeatureId::new(feature.namespace, feature.kind, feature.name.clone())
                .map_err(|error| error.at(format!("features[{index}]")))?;
            if !seen_features.insert(feature) {
                return Err(ConformanceError::invalid(
                    format!("features[{index}]"),
                    "must not contain duplicate feature identities",
                ));
            }
        }
        validate_artifact("source", &self.source, false, false)?;
        validate_catalog_identity(&self.catalog)?;
        validate_comparison(&self.comparison, &self.source)?;
        if self.status.is_match() {
            if self.comparison.mode == Equivalence::NotComparable {
                return Err(ConformanceError::invalid(
                    "comparison.mode",
                    "matched results must declare semantic, normalized, or exact-text equivalence",
                ));
            }
            if self.comparison.expected.is_none() || self.comparison.observed.is_none() {
                return Err(ConformanceError::invalid(
                    "comparison",
                    "matched results require expected and observed artifacts",
                ));
            }
            if self.reason.is_some() {
                return Err(ConformanceError::invalid(
                    "reason",
                    "matched results must not carry a reason",
                ));
            }
        } else {
            let reason = self.reason.as_ref().ok_or_else(|| {
                ConformanceError::invalid(
                    "reason",
                    "non-matching results require a structured reason",
                )
            })?;
            validate_reason(self.status, reason)?;
            if self.status == ConformanceStatus::UnexpectedRegression
                && self.comparison.mode == Equivalence::NotComparable
            {
                return Err(ConformanceError::invalid(
                    "comparison.mode",
                    "an unexpected regression must identify the comparison contract",
                ));
            }
        }
        Ok(())
    }

    pub fn validate_against(&self, catalog: &Catalog) -> Result<(), ConformanceError> {
        self.validate()?;
        if self.catalog != catalog.identity() {
            return Err(ConformanceError::invalid(
                "catalog",
                "must match the catalog supplied to validate_against",
            ));
        }
        for (index, feature) in self.features.iter().enumerate() {
            if feature.namespace != FeatureNamespace::Catalog {
                continue;
            }
            match feature.kind {
                FeatureKind::Enum => {
                    if catalog.enum_domain(&feature.name).is_none() {
                        return Err(ConformanceError::invalid(
                            format!("features[{index}]"),
                            format!("unknown canonical enum domain '{}'", feature.name),
                        ));
                    }
                }
                FeatureKind::EnumMember => {
                    let (domain, member) = feature.name.split_once('/').ok_or_else(|| {
                        ConformanceError::invalid(
                            format!("features[{index}]"),
                            "enum-member identity must contain domain/member",
                        )
                    })?;
                    let known = catalog.enum_domain(domain).is_some_and(|candidate| {
                        candidate.members.iter().any(|item| item.member == member)
                    });
                    if !known {
                        return Err(ConformanceError::invalid(
                            format!("features[{index}]"),
                            format!("unknown canonical enum member '{domain}/{member}'"),
                        ));
                    }
                }
                kind => {
                    let catalog_kind = match kind {
                        FeatureKind::Event => Kind::Event,
                        FeatureKind::Action => Kind::Action,
                        FeatureKind::Value => Kind::Value,
                        FeatureKind::Operator => Kind::Operator,
                        FeatureKind::Setting => Kind::Setting,
                        FeatureKind::Structural => Kind::Structural,
                        _ => {
                            return Err(ConformanceError::invalid(
                                format!("features[{index}]"),
                                "this feature kind cannot use the catalog namespace",
                            ));
                        }
                    };
                    if catalog.entry(catalog_kind, &feature.name).is_none() {
                        return Err(ConformanceError::invalid(
                            format!("features[{index}]"),
                            format!(
                                "unknown canonical {} '{}'",
                                catalog_kind.as_str(),
                                feature.name
                            ),
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    pub fn from_json(json: &str) -> Result<Self, ConformanceDecodeError> {
        let result: Self = serde_json::from_str(json).map_err(ConformanceDecodeError::Json)?;
        result.validate().map_err(ConformanceDecodeError::Invalid)?;
        Ok(result)
    }

    pub const fn is_match(&self) -> bool {
        self.status.is_match()
    }
}

fn validate_catalog_identity(catalog: &CatalogIdentity) -> Result<(), ConformanceError> {
    validate_non_empty(
        "catalog.implementationVersion",
        &catalog.implementation_version,
    )?;
    validate_non_empty("catalog.catalogVersion", &catalog.catalog_version)?;
    if catalog.catalog_digest.as_deref().is_none_or(str::is_empty) {
        return Err(ConformanceError::invalid(
            "catalog.catalogDigest",
            "must contain a catalog digest",
        ));
    }
    Ok(())
}

fn validate_artifact(
    field: &str,
    artifact: &TestArtifact,
    require_revision: bool,
    require_digest: bool,
) -> Result<(), ConformanceError> {
    validate_non_empty(&format!("{field}.name"), &artifact.name)?;
    if require_revision && artifact.revision.as_deref().is_none_or(str::is_empty) {
        return Err(ConformanceError::invalid(
            format!("{field}.revision"),
            "must identify an immutable revision",
        ));
    }
    if let Some(digest) = &artifact.sha256 {
        if digest.len() != 64 || !digest.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(ConformanceError::invalid(
                format!("{field}.sha256"),
                "must be a 64-character hexadecimal SHA-256 digest",
            ));
        }
    }
    if require_digest && artifact.sha256.is_none() {
        return Err(ConformanceError::invalid(
            format!("{field}.sha256"),
            "must contain a SHA-256 digest",
        ));
    }
    Ok(())
}

fn validate_comparison(
    comparison: &Comparison,
    source: &TestArtifact,
) -> Result<(), ConformanceError> {
    if comparison.mode == Equivalence::Normalized
        && comparison
            .normalizer
            .as_deref()
            .is_none_or(|normalizer| normalizer.trim().is_empty())
    {
        return Err(ConformanceError::invalid(
            "comparison.normalizer",
            "normalized comparisons require a named normalizer",
        ));
    }
    if let Some(expected) = &comparison.expected {
        validate_artifact("comparison.expected", expected, false, false)?;
        if expected == source {
            return Err(ConformanceError::invalid(
                "comparison.expected",
                "expected artifact must not be the test input",
            ));
        }
    }
    if let Some(observed) = &comparison.observed {
        validate_artifact("comparison.observed", observed, false, false)?;
        if observed == source {
            return Err(ConformanceError::invalid(
                "comparison.observed",
                "observed artifact must not be the test input",
            ));
        }
    }
    if comparison.expected.is_some() && comparison.expected == comparison.observed {
        return Err(ConformanceError::invalid(
            "comparison",
            "expected and observed artifacts must be distinct",
        ));
    }
    Ok(())
}

fn validate_reason(
    status: ConformanceStatus,
    reason: &ConformanceReason,
) -> Result<(), ConformanceError> {
    validate_non_empty("reason.detail", &reason.detail)?;
    let expected = match status {
        ConformanceStatus::Unsupported => ReasonCode::Unsupported,
        ConformanceStatus::KnownGap => ReasonCode::KnownGap,
        ConformanceStatus::UnexpectedRegression => ReasonCode::UnexpectedRegression,
        ConformanceStatus::Inconclusive => ReasonCode::Inconclusive,
        ConformanceStatus::Matched => {
            return Err(ConformanceError::invalid(
                "reason",
                "matched results must not carry a reason",
            ));
        }
    };
    if reason.code != expected {
        return Err(ConformanceError::invalid(
            "reason.code",
            "reason code must match conformance status",
        ));
    }
    Ok(())
}

fn validate_non_empty(field: &str, value: &str) -> Result<(), ConformanceError> {
    if value.trim().is_empty() {
        Err(ConformanceError::invalid(field, "must not be empty"))
    } else {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConformanceError {
    pub field: String,
    pub message: String,
}

impl ConformanceError {
    fn invalid(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            message: message.into(),
        }
    }

    fn at(self, field: String) -> Self {
        Self { field, ..self }
    }
}

impl std::fmt::Display for ConformanceError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "invalid conformance {}: {}",
            self.field, self.message
        )
    }
}

impl std::error::Error for ConformanceError {}

#[derive(Debug)]
pub enum ConformanceDecodeError {
    Json(serde_json::Error),
    Invalid(ConformanceError),
}

impl std::fmt::Display for ConformanceDecodeError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json(error) => write!(formatter, "invalid conformance JSON: {error}"),
            Self::Invalid(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for ConformanceDecodeError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn catalog() -> CatalogIdentity {
        Catalog::builtin().expect("built-in catalog").identity()
    }

    fn source() -> TestArtifact {
        TestArtifact {
            name: "fixture".to_string(),
            revision: Some("abc123".to_string()),
            path: Some("cases/basic.ws".to_string()),
            sha256: Some("a".repeat(64)),
            license: Some("MIT".to_string()),
        }
    }

    fn artifact(name: &str) -> TestArtifact {
        TestArtifact {
            name: name.to_string(),
            sha256: Some("b".repeat(64)),
            ..TestArtifact::new(name)
        }
    }

    fn matched() -> ConformanceResult {
        ConformanceResult {
            schema_version: CONFORMANCE_SCHEMA_VERSION,
            case_id: "basic-action".to_string(),
            features: vec![
                FeatureId::from_catalog(Kind::Action, "setHealth").expect("valid feature"),
            ],
            status: ConformanceStatus::Matched,
            comparison: Comparison {
                mode: Equivalence::Semantic,
                expected: Some(artifact("expected")),
                observed: Some(artifact("observed")),
                normalizer: None,
            },
            source: source(),
            catalog: catalog(),
            locale: Some(Locale::new("en-US")),
            reason: None,
        }
    }

    #[test]
    fn feature_ids_are_locale_and_provider_independent() {
        let feature = FeatureId::from_catalog(Kind::Action, "setHealth").expect("valid feature");
        assert_eq!(feature.kind, FeatureKind::Action);
        assert_eq!(feature.name, "setHealth");
        assert_eq!(
            serde_json::to_string(&feature).unwrap(),
            r#"{"namespace":"catalog","kind":"action","name":"setHealth"}"#
        );
        let member = FeatureId::from_enum_member("Hero", "ANA").expect("valid member");
        assert_eq!(member.name, "Hero/ANA");
    }

    #[test]
    fn result_serializes_as_test_data_without_evidence_taxonomy() {
        let result = matched();
        result.validate().expect("valid result");
        result
            .validate_against(&Catalog::builtin().expect("built-in catalog"))
            .expect("catalog-backed feature exists");
        let json = serde_json::to_string(&result).expect("serialize result");
        let document: serde_json::Value = serde_json::from_str(&json).expect("valid result JSON");
        assert!(document.get("evidence").is_none());
        assert!(document.get("trackingRef").is_none());
        let decoded = ConformanceResult::from_json(&json).expect("deserialize valid result");
        assert_eq!(decoded, result);
    }

    #[test]
    fn catalog_validation_rejects_fabricated_features_and_catalogs() {
        let mut result = matched();
        result.features = vec![
            FeatureId::from_catalog(Kind::Action, "notAWorkshopAction")
                .expect("syntactically valid feature"),
        ];
        assert!(
            result
                .validate_against(&Catalog::builtin().expect("built-in catalog"))
                .is_err()
        );

        let mut result = matched();
        result.catalog.catalog_digest = Some("f".repeat(64));
        let error = result
            .validate_against(&Catalog::builtin().expect("built-in catalog"))
            .expect_err("result must be bound to the supplied catalog");
        assert_eq!(error.field, "catalog");
    }

    #[test]
    fn non_matching_results_need_the_matching_reason() {
        let mut result = matched();
        result.status = ConformanceStatus::KnownGap;
        result.comparison.mode = Equivalence::NotComparable;
        assert!(result.validate().is_err());
        result.reason = Some(ConformanceReason {
            code: ReasonCode::KnownGap,
            detail: "client spelling is not available".to_string(),
        });
        result.validate().expect("documented gap");
        assert!(!result.is_match());
    }

    #[test]
    fn duplicate_features_blank_details_and_aliasing_are_invalid() {
        let mut result = matched();
        result.features.push(result.features[0].clone());
        assert!(result.validate().is_err());

        let mut result = matched();
        result.status = ConformanceStatus::Inconclusive;
        result.reason = Some(ConformanceReason {
            code: ReasonCode::Inconclusive,
            detail: "  \n".to_string(),
        });
        assert!(result.validate().is_err());

        let mut result = matched();
        result.comparison.observed = Some(result.source.clone());
        assert!(result.validate().is_err());
    }
}
