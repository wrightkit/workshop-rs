//! Owner-controlled expectations for the pinned real-project Workshop corpus.
//!
//! This is a test-support contract, not a conformance result or evidence
//! report. It contains only the current owner-defined input identities and
//! admitted Workshop semantic gaps. Consumers can use it to select and
//! validate the same inputs without maintaining a second expectation list.

use workshop_rs::WorkshopError;
use workshop_rs::semantic::{IncompletenessKind, ResidualClassification, SemanticIssue};

/// The schema version of [`REAL_PROJECT_EXPECTATION`].
pub(crate) const REAL_PROJECT_EXPECTATION_SCHEMA_VERSION: u32 = 1;

/// The stable identity of the pinned real-project source corpus.
pub(crate) const REAL_PROJECT_CORPUS_ID: &str = "raw-workshop-real-projects/v1";

/// The stage at which an admitted real-project gap is observed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RealProjectStage {
    /// Canonical builtin references are validated after parsing.
    CanonicalValidation,
    /// Canonical WIR is emitted back to Workshop text.
    Emission,
    /// Source text is converted to the other supported locale.
    LocaleConversion,
}

impl RealProjectStage {
    /// Return the stable machine-readable stage name.
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::CanonicalValidation => "canonical-validation",
            Self::Emission => "emission",
            Self::LocaleConversion => "locale-conversion",
        }
    }
}

/// The Workshop error identity admitted for a real-project stage gap.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RealProjectGapKind {
    /// An action spelling is not present in the canonical Workshop catalog.
    UnknownAction,
}

impl RealProjectGapKind {
    /// Return the stable machine-readable error kind name.
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::UnknownAction => "action",
        }
    }
}

/// An admitted semantic residual for one real-project source case.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RealProjectResidualExpectation {
    /// The semantic issue kind reported by [`workshop_rs::semantic::inspect`].
    pub(crate) kind: IncompletenessKind,
    /// The locale-independent Workshop identity of the residual.
    pub(crate) identity: &'static str,
    /// The owner-defined classification of the residual.
    pub(crate) classification: ResidualClassification,
}

/// An admitted Workshop error for one real-project processing stage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RealProjectGapExpectation {
    /// The processing stage where the error is admitted.
    pub(crate) stage: RealProjectStage,
    /// The structured Workshop error kind.
    pub(crate) kind: RealProjectGapKind,
    /// The localized spelling or identity carried by the error.
    pub(crate) identity: &'static str,
    /// The owner-defined classification corresponding to this gap.
    pub(crate) classification: ResidualClassification,
}

/// The pinned source identity and owner-defined expectation for one real-project case.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RealProjectCaseExpectation {
    /// Stable case identity.
    pub(crate) id: &'static str,
    /// Source locale of the pinned Workshop input.
    pub(crate) locale: &'static str,
    /// Crate-relative path to the pinned source input.
    pub(crate) source_fixture: &'static str,
    /// SHA-256 digest of [`Self::source_fixture`].
    pub(crate) source_sha256: &'static str,
    /// Semantic residuals admitted for this case at every inspection stage.
    pub(crate) residuals: &'static [RealProjectResidualExpectation],
    /// Stage-specific Workshop errors admitted for this case.
    pub(crate) gaps: &'static [RealProjectGapExpectation],
}

impl RealProjectCaseExpectation {
    /// Whether the inspected semantic issue is admitted for this case.
    pub(crate) fn admits_residual(&self, issue: &SemanticIssue) -> bool {
        self.residuals.iter().any(|expected| {
            expected.kind == issue.kind
                && expected.identity == issue.name
                && expected.classification == issue.classification
        })
    }

    /// Whether the error is an owner-admitted gap at the given stage.
    pub(crate) fn admits_gap(&self, stage: RealProjectStage, error: &WorkshopError) -> bool {
        self.gaps.iter().any(|expected| {
            expected.stage == stage
                && match (expected.kind, error) {
                    (
                        RealProjectGapKind::UnknownAction,
                        WorkshopError::Unknown { kind, spelling, .. },
                    ) => *kind == expected.kind.as_str() && spelling == expected.identity,
                    _ => false,
                }
        })
    }
}

/// The owner-controlled real-project expectation contract consumed by the
/// harness and downstream conformance tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RealProjectExpectation {
    /// Contract schema version.
    pub(crate) schema_version: u32,
    /// Stable identity of the source corpus.
    pub(crate) corpus_id: &'static str,
    /// The complete current real-project case inventory and expectations.
    pub(crate) cases: &'static [RealProjectCaseExpectation],
}

const NO_RESIDUALS: &[RealProjectResidualExpectation] = &[];
const NO_GAPS: &[RealProjectGapExpectation] = &[];
const DEFEND_RESIDUALS: &[RealProjectResidualExpectation] = &[RealProjectResidualExpectation {
    kind: IncompletenessKind::OpaqueAction,
    identity: "rawWorkshopAction",
    classification: ResidualClassification::LegacyOpaque,
}];
const DEFEND_GAPS: &[RealProjectGapExpectation] = &[
    RealProjectGapExpectation {
        stage: RealProjectStage::CanonicalValidation,
        kind: RealProjectGapKind::UnknownAction,
        identity: "rawWorkshopAction",
        classification: ResidualClassification::LegacyOpaque,
    },
    RealProjectGapExpectation {
        stage: RealProjectStage::Emission,
        kind: RealProjectGapKind::UnknownAction,
        identity: "rawWorkshopAction",
        classification: ResidualClassification::LegacyOpaque,
    },
    RealProjectGapExpectation {
        stage: RealProjectStage::LocaleConversion,
        kind: RealProjectGapKind::UnknownAction,
        identity: "rawWorkshopAction",
        classification: ResidualClassification::LegacyOpaque,
    },
];

/// The single authoritative real-project case and expectation definition.
pub(crate) const REAL_PROJECT_EXPECTATION: RealProjectExpectation = RealProjectExpectation {
    schema_version: REAL_PROJECT_EXPECTATION_SCHEMA_VERSION,
    corpus_id: REAL_PROJECT_CORPUS_ID,
    cases: &[
        RealProjectCaseExpectation {
            id: "ai-pve",
            locale: "zh-CN",
            source_fixture: "tests/fixtures/real-projects/ai-pve.ow",
            source_sha256: "d9c6460ca550e40083efcc2b57de16360088631970824599a22c0aa2cb7f11f9",
            residuals: NO_RESIDUALS,
            gaps: NO_GAPS,
        },
        RealProjectCaseExpectation {
            id: "bastion",
            locale: "en-US",
            source_fixture: "tests/fixtures/real-projects/bastion.ow",
            source_sha256: "44e453ddf7f373be65aea82d019abd45dd60f5ecb57c8d1607d3576a8bc60259",
            residuals: NO_RESIDUALS,
            gaps: NO_GAPS,
        },
        RealProjectCaseExpectation {
            id: "defend",
            locale: "en-US",
            source_fixture: "tests/fixtures/real-projects/defend.ow",
            source_sha256: "06a956b650313ee2d6e24ec989f907244dc4444579bdba27c580b031de97b268",
            residuals: DEFEND_RESIDUALS,
            gaps: DEFEND_GAPS,
        },
        RealProjectCaseExpectation {
            id: "illari",
            locale: "zh-CN",
            source_fixture: "tests/fixtures/real-projects/illari.ow",
            source_sha256: "f3aff73b9e677730bddc9c85b04c2bd38439bb7a4ba4fa2e80dc28db2e4a0860",
            residuals: NO_RESIDUALS,
            gaps: NO_GAPS,
        },
        RealProjectCaseExpectation {
            id: "rework",
            locale: "en-US",
            source_fixture: "tests/fixtures/real-projects/rework.ow",
            source_sha256: "aa32cda640dba41fd99245a7d425d9897b53875d15cf071862197a8e6840258c",
            residuals: NO_RESIDUALS,
            gaps: NO_GAPS,
        },
    ],
};
