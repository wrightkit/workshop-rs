//! Public contracts for canonical Workshop conformance evidence.
//!
//! This module identifies Workshop capabilities by locale-independent
//! canonical names and records results against evidence that is independent
//! from the implementation producing the observed output. It deliberately
//! does not define source-language provider identities or semantics.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::catalog::{Catalog, CatalogIdentity, Kind, Locale};

/// The current machine-readable conformance schema version.
pub const CONFORMANCE_SCHEMA_VERSION: u32 = 1;

/// The owner namespace of a Workshop capability identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum FeatureNamespace {
    /// An identity declared by the canonical Workshop catalog.
    Catalog,
    /// A Workshop IR or structural identity owned by this crate.
    Wir,
    /// A canonical custom-game settings path owned by this crate.
    Settings,
    /// A locale conversion or localization capability owned by this crate.
    Localization,
}

impl FeatureNamespace {
    /// The stable serialized spelling of this namespace.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Catalog => "catalog",
            Self::Wir => "wir",
            Self::Settings => "settings",
            Self::Localization => "localization",
        }
    }
}

/// The canonical category of a Workshop capability.
///
/// Catalog-backed categories use the canonical catalog identity as `name`.
/// The remaining categories identify WIR or Workshop structural capabilities;
/// none of these names are localized or borrowed from a source-language
/// provider.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum FeatureKind {
    /// A Workshop event declaration.
    Event,
    /// A Workshop action.
    Action,
    /// A Workshop value.
    Value,
    /// A Workshop operator.
    Operator,
    /// An enumerated value domain.
    Enum,
    /// A member of an enumerated value domain.
    EnumMember,
    /// A Workshop custom-game setting.
    Setting,
    /// A Workshop variable declaration or reference.
    Variable,
    /// A Workshop subroutine declaration or call.
    Subroutine,
    /// A control-flow construct represented by Workshop IR.
    ControlFlow,
    /// A user-visible Workshop string value.
    String,
    /// A locale or localized spelling operation.
    Localization,
    /// A Workshop content identity such as a hero, map, or mode.
    ContentId,
    /// A structural Workshop construct not covered by another category.
    Structural,
}

impl FeatureKind {
    /// The stable serialized spelling of this category.
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

/// A stable, locale-independent Workshop feature identity.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct FeatureId {
    /// The WrightKit repository-owned namespace for this identity.
    pub namespace: FeatureNamespace,
    /// The semantic category of the feature.
    pub kind: FeatureKind,
    /// The canonical name within the category.
    pub name: String,
}

impl FeatureId {
    /// Construct a feature identity from a canonical category and name.
    ///
    /// Names are intentionally opaque to this contract because catalog and
    /// WIR owners define their canonical names. Whitespace and control
    /// characters are rejected so serialized identities remain unambiguous.
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

    /// Construct a feature identity from a canonical catalog kind and id.
    pub fn from_catalog(kind: Kind, id: impl Into<String>) -> Result<Self, ConformanceError> {
        Self::new(FeatureNamespace::Catalog, kind.into(), id)
    }

    /// Construct a canonical enum-member identity that retains its domain.
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

    /// Construct a feature identity owned by Workshop IR or another
    /// `workshop-rs` namespace.
    pub fn owned(
        namespace: FeatureNamespace,
        kind: FeatureKind,
        name: impl Into<String>,
    ) -> Result<Self, ConformanceError> {
        Self::new(namespace, kind, name)
    }
}

/// The kind of evidence supporting a conformance expectation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum EvidenceBasis {
    /// A reproducible observation from the Overwatch Workshop client.
    WorkshopClient,
    /// A pinned, independently maintained compatibility oracle.
    PinnedExternalOracle,
    /// An accepted semantic or public API contract.
    SemanticContract,
    /// A preserved behavior from a provenance-linked real project.
    PreservedRegression,
}

/// The corpus/evidence layer containing a conformance case.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum EvidenceClass {
    /// A small synthetic, unit, or property case.
    Synthetic,
    /// A minimized regression extracted from a real project.
    MinimizedRegression,
    /// A complete real-world project or corpus case.
    RealProject,
    /// An observation captured from a live Workshop client.
    LiveClient,
}

/// An immutable source or artifact identity.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct EvidenceArtifact {
    /// Repository, fixture, oracle, or captured-artifact name.
    pub name: String,
    /// Immutable revision, release, or capture identifier where available.
    pub revision: Option<String>,
    /// Source path or artifact path within the named source.
    pub path: Option<String>,
    /// Content digest when the source is a materialized artifact.
    #[serde(rename = "sha256")]
    pub sha256: Option<String>,
    /// License or redistribution note for preserved source material.
    pub license: Option<String>,
}

impl EvidenceArtifact {
    /// Construct a source with no optional provenance fields.
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

/// The independent source that defines the expected behavior.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExpectationSource {
    pub basis: EvidenceBasis,
    pub artifact: EvidenceArtifact,
    /// Issue, review, or other tracking reference for a known classification.
    pub tracking_ref: Option<String>,
}

/// The live-client metadata attached to a client observation.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientEvidence {
    /// The game identity, normally `overwatch-2`.
    pub game: String,
    /// Client version when observable.
    pub client_version: Option<String>,
    /// Season or equivalent release identifier when observable.
    pub season: Option<String>,
    /// Capture date in an explicit ISO-8601 representation.
    pub captured_at: String,
    /// Environment notes that affect interpretation of the capture.
    pub environment: Option<String>,
}

/// The implementation identity that produced the observed output.
///
/// This is measurement metadata only. It is never an [`EvidenceBasis`] and
/// therefore cannot serve as its own correctness oracle.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ImplementationIdentity {
    pub name: String,
    pub version: String,
    pub revision: Option<String>,
    /// The materialized implementation artifact, when one is recorded.
    pub artifact: Option<EvidenceArtifact>,
}

/// Provenance attached to one conformance result.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Evidence {
    pub class: EvidenceClass,
    /// The fixture or observation being executed.
    pub fixture: EvidenceArtifact,
    /// The independent source that defines the expectation.
    pub expectation: ExpectationSource,
    /// The catalog identity used to interpret the case.
    pub catalog: CatalogIdentity,
    /// The source locale, when the case has localized input or output.
    pub locale: Option<Locale>,
    /// Live-client provenance, required for `LiveClient` evidence.
    pub client: Option<ClientEvidence>,
    /// The implementation under observation, when applicable.
    pub implementation: Option<ImplementationIdentity>,
}

/// The equivalence contract used to compare expected and observed behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Equivalence {
    /// Compare canonical Workshop semantics, ignoring presentation details.
    Semantic,
    /// Compare a normalized representation that preserves the claimed
    /// observable behavior.
    Normalized,
    /// Compare exact text only when text identity is part of the contract.
    ExactText,
    /// No comparison is claimed for this unsupported, gap, or inconclusive
    /// result.
    NotComparable,
}

/// The structured comparison attached to a conformance result.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Comparison {
    pub mode: Equivalence,
    pub expected: Option<EvidenceArtifact>,
    pub observed: Option<EvidenceArtifact>,
    /// The semantic/normalization procedure used for the comparison.
    pub normalizer: Option<String>,
}

/// A stable reason code for a non-matching result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ReasonCode {
    Unsupported,
    KnownGap,
    UnexpectedRegression,
    Inconclusive,
}

/// Structured detail for a non-matching result.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConformanceReason {
    pub code: ReasonCode,
    pub detail: String,
    pub tracking_ref: Option<String>,
}

/// The conformance state of one case for one or more Workshop features.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConformanceStatus {
    /// Observed behavior satisfies the declared comparison contract.
    Matched,
    /// The feature is outside the implementation's declared supported
    /// surface.
    Unsupported,
    /// A known mismatch remains tracked and visible.
    KnownGap,
    /// Behavior diverged unexpectedly from the independent expectation.
    UnexpectedRegression,
    /// Available evidence is insufficient to classify the behavior.
    Inconclusive,
}

impl ConformanceStatus {
    /// Whether this status is a successful conformance match.
    pub const fn is_match(self) -> bool {
        matches!(self, Self::Matched)
    }
}

/// One machine-readable Workshop conformance result.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConformanceResult {
    pub schema_version: u32,
    /// Stable identity for the fixture or probe case.
    pub case_id: String,
    /// Features exercised or implicated by this result.
    pub features: Vec<FeatureId>,
    pub status: ConformanceStatus,
    pub comparison: Comparison,
    pub evidence: Evidence,
    /// Required for non-matching states. This is a diagnostic, not an
    /// expected-output oracle.
    pub reason: Option<ConformanceReason>,
}

impl ConformanceResult {
    /// Validate the cross-field invariants of a serialized result.
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
        if self.features.is_empty() {
            return Err(ConformanceError::invalid(
                "features",
                "must contain at least one feature",
            ));
        }
        let mut seen_features: HashSet<&FeatureId> = HashSet::with_capacity(self.features.len());
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
        validate_evidence(&self.evidence)?;
        if self.status.is_match() && self.comparison.mode == Equivalence::NotComparable {
            return Err(ConformanceError::invalid(
                "comparison.mode",
                "matched results must declare semantic, normalized, or exact-text equivalence",
            ));
        }
        validate_comparison(&self.comparison, &self.evidence)?;
        if self.status.is_match() {
            if self.comparison.expected.is_none() || self.comparison.observed.is_none() {
                return Err(ConformanceError::invalid(
                    "comparison",
                    "matched results require expected and observed artifacts",
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

    /// Validate this result against the actual canonical catalog used for the
    /// case. Plain [`Self::validate`] checks the serialized contract only; it
    /// cannot prove that a catalog identity name exists without the catalog.
    pub fn validate_against(&self, catalog: &Catalog) -> Result<(), ConformanceError> {
        self.validate()?;
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

    /// Deserialize and validate a JSON result in one operation.
    pub fn from_json(json: &str) -> Result<Self, ConformanceDecodeError> {
        let result: Self = serde_json::from_str(json).map_err(ConformanceDecodeError::Json)?;
        result.validate().map_err(ConformanceDecodeError::Invalid)?;
        Ok(result)
    }

    /// Whether this result contributes to a successful conformance count.
    pub const fn is_match(&self) -> bool {
        self.status.is_match()
    }
}

fn validate_evidence(evidence: &Evidence) -> Result<(), ConformanceError> {
    validate_artifact(
        "evidence.fixture",
        &evidence.fixture,
        false,
        evidence.class == EvidenceClass::LiveClient,
    )?;
    validate_artifact(
        "evidence.expectation.artifact",
        &evidence.expectation.artifact,
        matches!(
            evidence.expectation.basis,
            EvidenceBasis::PinnedExternalOracle | EvidenceBasis::PreservedRegression
        ),
        false,
    )?;
    if evidence.class == EvidenceClass::LiveClient
        && evidence.expectation.basis != EvidenceBasis::WorkshopClient
    {
        return Err(ConformanceError::invalid(
            "evidence.expectation.basis",
            "live-client evidence must use workshop-client evidence basis",
        ));
    }
    if matches!(
        evidence.expectation.basis,
        EvidenceBasis::PinnedExternalOracle | EvidenceBasis::PreservedRegression
    ) && evidence
        .expectation
        .artifact
        .revision
        .as_deref()
        .is_none_or(str::is_empty)
    {
        return Err(ConformanceError::invalid(
            "evidence.expectation.artifact.revision",
            "pinned oracle and preserved regression evidence require an immutable revision",
        ));
    }
    if evidence.class == EvidenceClass::LiveClient {
        let client = evidence.client.as_ref().ok_or_else(|| {
            ConformanceError::invalid(
                "evidence.client",
                "live-client evidence requires client provenance",
            )
        })?;
        validate_non_empty("evidence.client.game", &client.game)?;
        validate_non_empty("evidence.client.capturedAt", &client.captured_at)?;
        if evidence.locale.is_none() {
            return Err(ConformanceError::invalid(
                "evidence.locale",
                "live-client evidence requires a client locale",
            ));
        }
    } else if evidence.client.is_some() {
        return Err(ConformanceError::invalid(
            "evidence.client",
            "client provenance is only valid for live-client evidence",
        ));
    }
    if evidence.class == EvidenceClass::MinimizedRegression
        && evidence.expectation.basis != EvidenceBasis::PreservedRegression
    {
        return Err(ConformanceError::invalid(
            "evidence.expectation.basis",
            "minimized-regression evidence must use preserved-regression basis",
        ));
    }
    validate_non_empty(
        "evidence.catalog.implementationVersion",
        &evidence.catalog.implementation_version,
    )?;
    validate_non_empty(
        "evidence.catalog.catalogVersion",
        &evidence.catalog.catalog_version,
    )?;
    if evidence
        .catalog
        .catalog_digest
        .as_deref()
        .is_none_or(str::is_empty)
    {
        return Err(ConformanceError::invalid(
            "evidence.catalog.catalogDigest",
            "conformance evidence requires a pinned catalog digest",
        ));
    }
    Ok(())
}

fn validate_artifact(
    field: &str,
    artifact: &EvidenceArtifact,
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
            "materialized evidence requires a SHA-256 digest",
        ));
    }
    Ok(())
}

fn validate_comparison(
    comparison: &Comparison,
    evidence: &Evidence,
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
    }
    if let Some(observed) = &comparison.observed {
        validate_artifact("comparison.observed", observed, false, false)?;
    }
    if let (Some(expected), Some(observed)) = (&comparison.expected, &comparison.observed) {
        if expected == observed {
            return Err(ConformanceError::invalid(
                "comparison",
                "expected and observed artifacts must be distinct",
            ));
        }
    }
    if let Some(expected) = &comparison.expected {
        if Some(expected) == Some(&evidence.fixture) {
            return Err(ConformanceError::invalid(
                "comparison.expected",
                "expected artifact must not be the executed fixture",
            ));
        }
        if evidence
            .implementation
            .as_ref()
            .and_then(|implementation| implementation.artifact.as_ref())
            == Some(expected)
        {
            return Err(ConformanceError::invalid(
                "comparison.expected",
                "expected artifact must not be the implementation artifact",
            ));
        }
    }
    if let Some(observed) = &comparison.observed {
        if Some(observed) == Some(&evidence.fixture)
            || Some(observed) == Some(&evidence.expectation.artifact)
            || evidence
                .implementation
                .as_ref()
                .and_then(|implementation| implementation.artifact.as_ref())
                == Some(observed)
        {
            return Err(ConformanceError::invalid(
                "comparison.observed",
                "observed artifact must be distinct from fixture, expectation, and implementation artifacts",
            ));
        }
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
                "matched results must not carry a non-matching reason",
            ));
        }
    };
    if reason.code != expected {
        return Err(ConformanceError::invalid(
            "reason.code",
            "reason code must match conformance status",
        ));
    }
    if reason.code == ReasonCode::KnownGap
        && reason
            .tracking_ref
            .as_deref()
            .is_none_or(|tracking_ref| tracking_ref.trim().is_empty())
    {
        return Err(ConformanceError::invalid(
            "reason.trackingRef",
            "known gaps require a tracking reference",
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

/// A validation error for the public conformance contract.
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

/// Errors returned by the validated JSON entry point.
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
    use crate::catalog::{Catalog, Locale};

    fn catalog() -> CatalogIdentity {
        Catalog::builtin().expect("built-in catalog").identity()
    }

    fn evidence(class: EvidenceClass, basis: EvidenceBasis) -> Evidence {
        Evidence {
            class,
            fixture: EvidenceArtifact {
                name: "fixture".to_string(),
                revision: Some("abc123".to_string()),
                path: Some("cases/basic.ws".to_string()),
                sha256: Some(
                    "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
                ),
                license: Some("MIT".to_string()),
            },
            expectation: ExpectationSource {
                basis,
                artifact: EvidenceArtifact {
                    name: "semantic-contract".to_string(),
                    revision: Some("contract-1".to_string()),
                    path: Some("docs/adr/0002-conformance-contract.md".to_string()),
                    sha256: None,
                    license: Some("MIT".to_string()),
                },
                tracking_ref: None,
            },
            catalog: catalog(),
            locale: Some(Locale::new("en-US")),
            client: None,
            implementation: Some(ImplementationIdentity {
                name: "workshop-rs".to_string(),
                version: "0.1.0".to_string(),
                revision: Some("impl123".to_string()),
                artifact: None,
            }),
        }
    }

    fn hashed_artifact(name: &str) -> EvidenceArtifact {
        EvidenceArtifact {
            name: name.to_string(),
            sha256: Some(
                "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string(),
            ),
            ..EvidenceArtifact::new(name)
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
                expected: Some(hashed_artifact("expected")),
                observed: Some(hashed_artifact("observed")),
                normalizer: Some("canonical-wir".to_string()),
            },
            evidence: evidence(EvidenceClass::Synthetic, EvidenceBasis::SemanticContract),
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
    fn result_serializes_and_round_trips() {
        let result = matched();
        result.validate().expect("valid result");
        result
            .validate_against(&Catalog::builtin().expect("built-in catalog"))
            .expect("catalog-backed feature exists");
        let json = serde_json::to_string(&result).expect("serialize result");
        let decoded = ConformanceResult::from_json(&json).expect("deserialize valid result");
        assert_eq!(decoded, result);
    }

    #[test]
    fn catalog_validation_rejects_fabricated_catalog_features() {
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
    }

    #[test]
    fn known_gap_is_not_a_match_and_requires_detail() {
        let mut result = matched();
        result.status = ConformanceStatus::KnownGap;
        result.comparison.mode = Equivalence::NotComparable;
        assert!(result.validate().is_err());
        result.reason = Some(ConformanceReason {
            code: ReasonCode::KnownGap,
            detail: "client spelling is not yet evidenced".to_string(),
            tracking_ref: Some("#18".to_string()),
        });
        result.validate().expect("documented gap");
        assert!(!result.is_match());
    }

    #[test]
    fn duplicate_features_and_blank_details_are_invalid() {
        let mut result = matched();
        result.features.push(result.features[0].clone());
        assert!(result.validate().is_err());

        let mut result = matched();
        result.status = ConformanceStatus::Inconclusive;
        result.reason = Some(ConformanceReason {
            code: ReasonCode::Inconclusive,
            detail: "  \n".to_string(),
            tracking_ref: None,
        });
        assert!(result.validate().is_err());
    }

    #[test]
    fn matched_artifacts_cannot_reuse_fixture_or_implementation_output() {
        let mut result = matched();
        result.comparison.observed = Some(result.evidence.fixture.clone());
        assert!(result.validate().is_err());

        let mut result = matched();
        let implementation_artifact = hashed_artifact("implementation-output");
        result.evidence.implementation.as_mut().unwrap().artifact =
            Some(implementation_artifact.clone());
        result.comparison.observed = Some(implementation_artifact);
        assert!(result.validate().is_err());

        let mut result = matched();
        let implementation_artifact = hashed_artifact("implementation-output");
        result.evidence.implementation.as_mut().unwrap().artifact =
            Some(implementation_artifact.clone());
        result.comparison.expected = Some(implementation_artifact);
        assert!(result.validate().is_err());
    }

    #[test]
    fn live_client_requires_client_and_locale_provenance() {
        let mut result = matched();
        result.evidence.class = EvidenceClass::LiveClient;
        result.evidence.expectation.basis = EvidenceBasis::WorkshopClient;
        result.evidence.client = Some(ClientEvidence {
            game: "overwatch-2".to_string(),
            client_version: Some("season-1".to_string()),
            season: Some("season-1".to_string()),
            captured_at: "2026-08-18T00:00:00Z".to_string(),
            environment: None,
        });
        result.validate().expect("complete live evidence");
        result.evidence.locale = None;
        assert!(result.validate().is_err());
    }

    #[test]
    fn implementation_output_cannot_be_the_evidence_basis() {
        let mut result = matched();
        result.evidence.implementation = Some(ImplementationIdentity {
            name: "changed-implementation".to_string(),
            version: "dev".to_string(),
            revision: None,
            artifact: None,
        });
        result
            .validate()
            .expect("implementation metadata is allowed");
        assert_ne!(
            result.evidence.expectation.basis,
            EvidenceBasis::PinnedExternalOracle,
            "implementation metadata is not an oracle"
        );
    }

    #[test]
    fn validated_json_rejects_an_invalid_status_reason() {
        let mut value = serde_json::to_value(matched()).expect("serialize result");
        value["status"] = serde_json::json!("known-gap");
        let json = serde_json::to_string(&value).expect("serialize invalid result");
        assert!(ConformanceResult::from_json(&json).is_err());
    }
}
