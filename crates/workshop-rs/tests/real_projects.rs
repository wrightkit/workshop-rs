//! Pinned real-project corpus contract and provenance checks.

use common::real_projects::{
    REAL_PROJECT_CORPUS_ID, REAL_PROJECT_EXPECTATION, REAL_PROJECT_EXPECTATION_SCHEMA_VERSION,
    RealProjectGapKind, RealProjectStage,
};
use workshop_rs::{WorkshopError, semantic::ResidualClassification};

mod common;

#[test]
fn expectation_has_unique_pinned_case_identities() {
    assert_eq!(
        REAL_PROJECT_EXPECTATION.schema_version,
        REAL_PROJECT_EXPECTATION_SCHEMA_VERSION
    );
    assert_eq!(REAL_PROJECT_EXPECTATION.corpus_id, REAL_PROJECT_CORPUS_ID);

    for (index, case) in REAL_PROJECT_EXPECTATION.cases.iter().enumerate() {
        assert!(!case.id.is_empty());
        assert!(case.locale == "en-US" || case.locale == "zh-CN");
        assert!(
            case.source_fixture
                .starts_with("tests/fixtures/real-projects/")
        );
        assert_eq!(case.source_sha256.len(), 64);
        assert!(
            REAL_PROJECT_EXPECTATION.cases[index + 1..]
                .iter()
                .all(|other| other.id != case.id),
            "duplicate real-project case identity: {}",
            case.id
        );
    }
}

#[test]
fn expectation_keeps_the_admitted_gap_identity_and_classification() {
    let defend = REAL_PROJECT_EXPECTATION
        .cases
        .iter()
        .find(|case| case.id == "defend")
        .expect("defend case");
    assert_eq!(defend.residuals.len(), 1);
    assert_eq!(defend.residuals[0].identity, "rawWorkshopAction");
    assert_eq!(
        defend.residuals[0].classification,
        ResidualClassification::LegacyOpaque
    );
    assert_eq!(defend.gaps.len(), 3);
    assert!(defend.gaps.iter().all(|gap| {
        gap.kind == RealProjectGapKind::UnknownAction
            && gap.identity == "rawWorkshopAction"
            && gap.classification == ResidualClassification::LegacyOpaque
    }));

    let error = WorkshopError::Unknown {
        kind: "action",
        spelling: "rawWorkshopAction".to_string(),
        locale: workshop_rs::catalog::Locale::new("en-US"),
        span: None,
    };
    assert!(defend.admits_gap(RealProjectStage::Emission, &error));
    assert!(!defend.admits_gap(
        RealProjectStage::CanonicalValidation,
        &WorkshopError::Unknown {
            kind: "value",
            spelling: "rawWorkshopAction".to_string(),
            locale: workshop_rs::catalog::Locale::new("en-US"),
            span: None,
        }
    ));
}
