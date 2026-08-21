//! Re-runnable P0 corpus census. The source artifacts remain external because
//! their repository licenses do not permit redistribution here.

use sha2::{Digest, Sha256};
use workshop_rs::{
    catalog::{Catalog, Locale},
    emitter, parser, roundtrip, semantic, validate,
};

const CASES: &[(&str, &str, &str, usize)] = &[
    (
        "ai-pve",
        "zh-CN",
        "d9c6460ca550e40083efcc2b57de16360088631970824599a22c0aa2cb7f11f9",
        246,
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
        0,
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
        match validate::validate_canonical_ids(&program, &catalog) {
            Ok(()) => println!("{name}: canonical validation passed"),
            Err(error) => println!("{name}: canonical validation classified gap: {error:?}"),
        }
        match emitter::emit(&program, &catalog, &Locale::new(locale)) {
            Ok(emitted) => {
                match parser::parse_with_context(&emitted, &catalog, &Locale::new(locale), &catalog)
                {
                    Ok(reparsed) => {
                        if !roundtrip::equivalent(&program, &reparsed) {
                            println!("{name}: original WIR:\n{}", program.dump());
                            println!("{name}: reparsed WIR:\n{}", reparsed.dump());
                            panic!("{name} semantic round-trip changed WIR");
                        }
                        let emitted_again =
                            emitter::emit(&reparsed, &catalog, &Locale::new(locale))
                                .unwrap_or_else(|error| {
                                    panic!("{name} second emission failed: {error:?}")
                                });
                        assert_eq!(
                            emitted, emitted_again,
                            "{name} emission is not deterministic"
                        );
                    }
                    Err(error) => {
                        println!("{name}: emitted output reparse classified gap: {error:?}");
                        let line = match &error {
                            workshop_rs::WorkshopError::Unknown { span, .. }
                            | workshop_rs::WorkshopError::Malformed { span, .. }
                            | workshop_rs::WorkshopError::Unsupported { span, .. } => span
                                .map(|span| span.start.line as usize)
                                .unwrap_or_default(),
                            _ => 0,
                        };
                        if line > 0 {
                            for (index, text) in emitted.lines().enumerate() {
                                let number = index + 1;
                                if number.abs_diff(line) <= 2 {
                                    println!("{name}: emitted[{number}] {text}");
                                }
                            }
                        }
                    }
                }
            }
            Err(error) => println!("{name}: emission classified gap: {error:?}"),
        }
        let issues = semantic::inspect(&program, &catalog);
        assert_eq!(
            issues.len(),
            *expected_issues,
            "semantic census changed for {name}"
        );
        println!("{name}: {} semantic issues", issues.len());
    }
}
