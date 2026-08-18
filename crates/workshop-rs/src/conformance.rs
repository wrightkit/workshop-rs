//! Public contracts for canonical Workshop conformance evidence.
//!
//! This module identifies Workshop capabilities by locale-independent
//! canonical names and records results against evidence that is independent
//! from the implementation producing the observed output. It deliberately
//! does not define source-language provider identities or semantics.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::catalog::{CatalogIdentity, Kind, Locale};

/// The current machine-readable conformance schema version.
pub const CONFORMANCE_SCHEMA_VERSION: u32 = 1;

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
    pub fn new(kind: FeatureKind, name: impl Into<String>) -> Result<Self, ConformanceError> {
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
        Ok(Self { kind, name })
    }

    /// Construct a feature identity from a canonical catalog kind and id.
    pub fn from_catalog(kind: Kind, id: impl Into<String>) -> Result<Self, ConformanceError> {
        Self::new(kind.into(), id)
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

/// The immutable source identity for a conformance case.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct EvidenceSource {
    /// Repository, fixture, oracle, or client-capture name.
    pub name: String,
    /// Immutable revision, release, or capture identifier where available.
    pub revision: Option<String>,
    /// Source path or artifact path within the named source.
    pub path: Option<String>,
    /// License or redistribution note for preserved source material.
    pub license: Option<String>,
}

impl EvidenceSource {
    /// Construct a source with no optional provenance fields.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            revision: None,
            path: None,
            license: None,
        }
    }
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
}

/// Provenance attached to one conformance result.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Evidence {
    pub class: EvidenceClass,
    pub basis: EvidenceBasis,
    pub source: EvidenceSource,
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
    pub equivalence: Equivalence,
    pub evidence: Evidence,
    /// Required for non-matching states and useful for diagnostics in all
    /// states. This is a diagnostic, not an expected-output oracle.
    pub detail: Option<String>,
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
            FeatureId::new(feature.kind, feature.name.clone())
                .map_err(|error| error.at(format!("features[{index}]")))?;
            if !seen_features.insert(feature) {
                return Err(ConformanceError::invalid(
                    format!("features[{index}]"),
                    "must not contain duplicate feature identities",
                ));
            }
        }
        validate_evidence(&self.evidence)?;
        if self.status.is_match() && self.equivalence == Equivalence::NotComparable {
            return Err(ConformanceError::invalid(
                "equivalence",
                "matched results must declare semantic, normalized, or exact-text equivalence",
            ));
        }
        if !self.status.is_match()
            && self
                .detail
                .as_deref()
                .is_none_or(|detail| detail.trim().is_empty())
        {
            return Err(ConformanceError::invalid(
                "detail",
                "non-matching results must explain the unsupported, gap, regression, or inconclusive state",
            ));
        }
        if self.status == ConformanceStatus::UnexpectedRegression
            && self.equivalence == Equivalence::NotComparable
        {
            return Err(ConformanceError::invalid(
                "equivalence",
                "an unexpected regression must identify the comparison contract",
            ));
        }
        Ok(())
    }

    /// Whether this result contributes to a successful conformance count.
    pub const fn is_match(&self) -> bool {
        self.status.is_match()
    }
}

fn validate_evidence(evidence: &Evidence) -> Result<(), ConformanceError> {
    validate_non_empty("evidence.source.name", &evidence.source.name)?;
    if matches!(
        evidence.basis,
        EvidenceBasis::PinnedExternalOracle | EvidenceBasis::PreservedRegression
    ) && evidence
        .source
        .revision
        .as_deref()
        .is_none_or(str::is_empty)
    {
        return Err(ConformanceError::invalid(
            "evidence.source.revision",
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
        if evidence.basis != EvidenceBasis::WorkshopClient {
            return Err(ConformanceError::invalid(
                "evidence.basis",
                "live-client evidence must use workshop-client evidence basis",
            ));
        }
    } else if evidence.client.is_some() {
        return Err(ConformanceError::invalid(
            "evidence.client",
            "client provenance is only valid for live-client evidence",
        ));
    }
    if evidence.class == EvidenceClass::MinimizedRegression
        && evidence.basis != EvidenceBasis::PreservedRegression
    {
        return Err(ConformanceError::invalid(
            "evidence.basis",
            "minimized-regression evidence must use preserved-regression basis",
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
            basis,
            source: EvidenceSource {
                name: "fixture".to_string(),
                revision: Some("abc123".to_string()),
                path: Some("cases/basic.ws".to_string()),
                license: Some("MIT".to_string()),
            },
            catalog: catalog(),
            locale: Some(Locale::new("en-US")),
            client: None,
            implementation: Some(ImplementationIdentity {
                name: "workshop-rs".to_string(),
                version: "0.1.0".to_string(),
                revision: Some("impl123".to_string()),
            }),
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
            equivalence: Equivalence::Semantic,
            evidence: evidence(EvidenceClass::Synthetic, EvidenceBasis::SemanticContract),
            detail: None,
        }
    }

    #[test]
    fn feature_ids_are_locale_and_provider_independent() {
        let feature = FeatureId::from_catalog(Kind::Action, "setHealth").expect("valid feature");
        assert_eq!(feature.kind, FeatureKind::Action);
        assert_eq!(feature.name, "setHealth");
        assert_eq!(
            serde_json::to_string(&feature).unwrap(),
            r#"{"kind":"action","name":"setHealth"}"#
        );
    }

    #[test]
    fn result_serializes_and_round_trips() {
        let result = matched();
        result.validate().expect("valid result");
        let json = serde_json::to_string(&result).expect("serialize result");
        let decoded: ConformanceResult = serde_json::from_str(&json).expect("deserialize result");
        assert_eq!(decoded, result);
    }

    #[test]
    fn known_gap_is_not_a_match_and_requires_detail() {
        let mut result = matched();
        result.status = ConformanceStatus::KnownGap;
        result.equivalence = Equivalence::NotComparable;
        assert!(result.validate().is_err());
        result.detail = Some("client spelling is not yet evidenced".to_string());
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
        result.detail = Some("  \n".to_string());
        assert!(result.validate().is_err());
    }

    #[test]
    fn live_client_requires_client_and_locale_provenance() {
        let mut result = matched();
        result.evidence.class = EvidenceClass::LiveClient;
        result.evidence.basis = EvidenceBasis::WorkshopClient;
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
        });
        result
            .validate()
            .expect("implementation metadata is allowed");
        assert_ne!(
            result.evidence.basis,
            EvidenceBasis::PinnedExternalOracle,
            "implementation metadata is not an oracle"
        );
    }
}
