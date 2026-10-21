use workshop_rs::WorkshopError;
use workshop_rs::rules::{IncompletenessKind, ResidualClassification, SemanticIssue};

pub(crate) const REAL_PROJECT_EXPECTATION_SCHEMA_VERSION: u32 = 1;

pub(crate) const REAL_PROJECT_CORPUS_ID: &str = "raw-workshop-real-projects/v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RealProjectStage {
    CanonicalValidation,
    Emission,
    LocaleConversion,
}

impl RealProjectStage {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::CanonicalValidation => "canonical-validation",
            Self::Emission => "emission",
            Self::LocaleConversion => "locale-conversion",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RealProjectGapKind {
    UnknownAction,
}

impl RealProjectGapKind {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::UnknownAction => "action",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RealProjectResidualExpectation {
    pub(crate) kind: IncompletenessKind,
    pub(crate) identity: &'static str,
    pub(crate) classification: ResidualClassification,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RealProjectGapExpectation {
    pub(crate) stage: RealProjectStage,
    pub(crate) kind: RealProjectGapKind,
    pub(crate) identity: &'static str,
    pub(crate) classification: ResidualClassification,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RealProjectCaseExpectation {
    pub(crate) id: &'static str,
    pub(crate) locale: &'static str,
    pub(crate) source_fixture: &'static str,
    pub(crate) source_sha256: &'static str,
    pub(crate) residuals: &'static [RealProjectResidualExpectation],
    pub(crate) gaps: &'static [RealProjectGapExpectation],
}

impl RealProjectCaseExpectation {
    pub(crate) fn admits_residual(&self, issue: &SemanticIssue) -> bool {
        self.residuals.iter().any(|expected| {
            expected.kind == issue.kind
                && expected.identity == issue.name
                && expected.classification == issue.classification
        })
    }

    pub(crate) fn admits_gap(&self, stage: RealProjectStage, error: &WorkshopError) -> bool {
        self.gaps.iter().any(|expected| {
            expected.stage == stage
                && match (expected.kind, error) {
                    (
                        RealProjectGapKind::UnknownAction,
                        WorkshopError::Unknown { kind, spelling, .. }
                        | WorkshopError::UnknownWithCandidates { kind, spelling, .. },
                    ) => *kind == expected.kind.as_str() && spelling == expected.identity,
                    _ => false,
                }
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RealProjectExpectation {
    pub(crate) schema_version: u32,
    pub(crate) corpus_id: &'static str,
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
