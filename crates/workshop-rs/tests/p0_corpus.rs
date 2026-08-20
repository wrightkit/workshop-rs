//! Re-runnable P0 corpus census. The source artifacts remain external because
//! their repository licenses do not permit redistribution here.

use sha2::{Digest, Sha256};
use workshop_rs::{
    catalog::{Catalog, Locale},
    parser, semantic,
};

const CASES: &[(&str, &str, &str, usize)] = &[
    (
        "ai-pve",
        "zh-CN",
        "b0405707d54fd30e20f285ce4a3fdaf9899e3959fb5dd223c898040c63a18773",
        247,
    ),
    (
        "bastion",
        "en-US",
        "44e453ddf7f373be65aea82d019abd45dd60f5ecb57c8d1607d3576a8bc60259",
        396,
    ),
    (
        "defend",
        "en-US",
        "06a956b650313ee2d6e24ec989f907244dc4444579bdba27c580b031de97b268",
        3,
    ),
    (
        "illari",
        "zh-CN",
        "f3aff73b9e677730bddc9c85b04c2bd38439bb7a4ba4fa2e80dc28db2e4a0860",
        0,
    ),
    (
        "rework",
        "en-US",
        "aa32cda640dba41fd99245a7d425d9897b53875d15cf071862197a8e6840258c",
        1,
    ),
];

#[test]
#[ignore = "requires externally reacquired artifacts; run with WRIGHTKIT_P0_ARTIFACT_DIR"]
fn pinned_p0_corpus_has_expected_semantic_census() {
    let root = std::env::var("WRIGHTKIT_P0_ARTIFACT_DIR")
        .expect("set WRIGHTKIT_P0_ARTIFACT_DIR to the reacquired artifact directory");
    let catalog = Catalog::builtin().expect("built-in catalog");
    for (name, locale, expected_sha, expected_issues) in CASES {
        let path = std::path::Path::new(&root).join(format!("{name}.ow"));
        let bytes = std::fs::read(&path).unwrap_or_else(|error| panic!("{path:?}: {error}"));
        let actual_sha = format!("{:x}", Sha256::digest(&bytes));
        assert_eq!(
            &actual_sha, expected_sha,
            "pinned digest mismatch for {name}"
        );
        let source = String::from_utf8(bytes).expect("artifact is UTF-8 Workshop text");
        let program = parser::parse_with_context(&source, &catalog, &Locale::new(locale), &catalog)
            .unwrap_or_else(|error| panic!("{name} parse failed: {error:?}"));
        let issues = semantic::inspect(&program, &catalog);
        assert_eq!(
            issues.len(),
            *expected_issues,
            "semantic census changed for {name}"
        );
        println!("{name}: {} semantic issues", issues.len());
    }
}
