//! Validation-entry agreement tests (workshop-rs#460).
//!
//! The CLI `parse` command, the `corpus` runner's per-case validation path,
//! and the library validation entry point are three ways the same program is
//! validated. For every input they must accept or reject identically: the
//! canonical-identity check, WIR structural validation, and parsing itself
//! are one contract, not three weaker subsets (workshop-rs#439).
//!
//! Both CLI surfaces are driven through the compiled binary, so a check
//! dropped from either entry point — not only from the library calls —
//! surfaces as a disagreement here.

use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

use sha2::{Digest, Sha256};
use workshop_rs::catalog::{Catalog, Locale};
use workshop_rs::{parser, validate};

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_workshop-rs-cli")
}

/// One program under test: `actions` is placed inside an `Ongoing - Global`
/// rule; `accepted` is the shared accept/reject contract.
struct Case {
    id: &'static str,
    actions: &'static str,
    accepted: bool,
}

const CASES: &[Case] = &[
    // A fully canonical program: every entry point must accept it.
    Case {
        id: "valid",
        actions: "Wait(1);",
        accepted: true,
    },
    // Surplus arguments past the declared signature: parse and WIR
    // validation accept this; only canonical-ID validation rejects it
    // (the workshop-rs#439 defect class).
    Case {
        id: "surplus-args",
        actions: "Wait(1, 2, 3, 4, 5);",
        accepted: false,
    },
    // A call missing every required argument.
    Case {
        id: "missing-args",
        actions: "Wait();",
        accepted: false,
    },
    // A partially supplied signature is a missing argument too
    // (workshop-rs#341).
    Case {
        id: "partial-args",
        actions: "Big Message(All Players(All Teams));",
        accepted: false,
    },
    // An unknown action spelling is rejected while parsing; both entry
    // points must agree on that rejection.
    Case {
        id: "unknown-action",
        actions: "Totally Unknown Thing(1);",
        accepted: false,
    },
    // An unresolved value call is preserved by the parser and rejected only
    // by canonical-ID validation.
    Case {
        id: "unknown-value",
        actions: "Wait(Totally Bogus Value(1));",
        accepted: false,
    },
    // An unresolved enum member is preserved by the parser for canonical
    // validation, which rejects it.
    Case {
        id: "unknown-enum-member",
        actions: "Set Player Allowed Heroes(All Players(All Teams), Hero(Not A Hero));",
        accepted: false,
    },
    // A literal overflowing to a non-finite number is rejected by
    // `Program::validate` (WIR invariants), not by canonical-ID
    // validation — so this leg is guarded in both entry points too.
    Case {
        id: "non-finite-number",
        actions: "Wait(1234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890);",
        accepted: false,
    },
];

fn program(case: &Case) -> String {
    format!(
        "rule (\"{}\") {{ event {{ Ongoing - Global; }} actions {{ {} }} }}",
        case.id, case.actions
    )
}

fn sha256(text: &str) -> String {
    format!("{:x}", Sha256::digest(text.as_bytes()))
}

/// The library validation path as the corpus runner applies it: parse, then
/// WIR structural validation, then canonical-ID validation.
fn library_accepts(source: &str, catalog: &Catalog, locale: &Locale) -> bool {
    let Ok(program) = parser::parse_with_context(source, catalog, locale, catalog) else {
        return false;
    };
    program.validate().is_ok() && validate::validate_canonical_ids(&program, catalog).is_ok()
}

/// Write one fixture file per case and a schema-2 corpus manifest pinning
/// every fixture's digest, then run the `corpus` command on it. Returns the
/// per-case conformance status reported by the runner.
fn corpus_statuses(dir: &Path, cases: &[(&Case, String)]) -> HashMap<String, String> {
    let case_entries: Vec<serde_json::Value> = cases
        .iter()
        .map(|(case, source)| {
            serde_json::json!({
                "id": format!("agreement/{}", case.id),
                "fixture": format!("{}.ws", case.id),
                "source": {
                    "name": format!("{}.ws", case.id),
                    "revision": "issue-460",
                    "path": format!("cases/{}.ws", case.id),
                    "sha256": sha256(source),
                    "license": "MIT",
                },
                "features": [{"namespace": "wir", "kind": "structural", "name": "rule"}],
                "expectedStatus": "success",
            })
        })
        .collect();
    let manifest = serde_json::json!({
        "schemaVersion": 2,
        "id": "validation-agreement",
        "locale": "en-US",
        "expected": {
            "name": "expected canonical WIR dump",
            "revision": "issue-460",
            "path": "expected/dump.txt",
            "sha256": sha256("reference artifact placeholder"),
            "license": "MIT",
        },
        "cases": case_entries,
    });
    let manifest_path = dir.join("manifest.json");
    std::fs::write(
        &manifest_path,
        serde_json::to_string_pretty(&manifest).unwrap(),
    )
    .unwrap();

    let output = Command::new(bin())
        .args(["corpus", manifest_path.to_str().unwrap(), "--json"])
        .output()
        .expect("corpus command runs");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let report: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_else(|error| {
        panic!(
            "corpus report is JSON: {error}\nstdout: {stdout}\nstderr: {}",
            String::from_utf8_lossy(&output.stderr)
        )
    });
    report["results"]
        .as_array()
        .expect("corpus report has results")
        .iter()
        .map(|result| {
            (
                result["caseId"].as_str().unwrap().to_string(),
                result["status"].as_str().unwrap().to_string(),
            )
        })
        .collect()
}

/// The `corpus` runner reports a validated case as `inconclusive` (the
/// pinned reference comparison cannot run offline) and a rejected case as
/// `unexpected-regression`.
fn corpus_accepts(status: &str) -> bool {
    match status {
        "inconclusive" => true,
        "unexpected-regression" => false,
        other => panic!("unexpected corpus conformance status '{other}'"),
    }
}

/// The CLI `parse` entry point as a user runs it: exit 0 accepts the
/// program, exit 1 rejects it (exit 2 would be a usage error).
fn cli_accepts(file: &Path) -> (bool, String) {
    let output = Command::new(bin())
        .args(["parse", "--locale", "en-US", file.to_str().unwrap()])
        .output()
        .expect("parse command runs");
    let code = output.status.code();
    assert!(
        matches!(code, Some(0) | Some(1)),
        "unexpected parse exit code {code:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    (
        code == Some(0),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

#[test]
fn cli_parse_and_corpus_validation_agree() {
    let catalog = Catalog::builtin().expect("built-in catalog");
    let locale = Locale::new("en-US");
    let dir = std::env::temp_dir().join(format!(
        "workshop-rs-cli-validation-agreement-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).unwrap();

    let cases: Vec<(&Case, String)> = CASES.iter().map(|case| (case, program(case))).collect();
    for (case, source) in &cases {
        std::fs::write(dir.join(format!("{}.ws", case.id)), source).unwrap();
    }
    let corpus_statuses = corpus_statuses(&dir, &cases);

    for (case, source) in &cases {
        let file = dir.join(format!("{}.ws", case.id));
        let (cli, cli_stderr) = cli_accepts(&file);
        let corpus = corpus_accepts(
            corpus_statuses
                .get(&format!("agreement/{}", case.id))
                .unwrap_or_else(|| panic!("corpus report misses case {}", case.id)),
        );
        let library = library_accepts(source, &catalog, &locale);
        assert_eq!(
            cli, corpus,
            "case {}: `parse` and `corpus` disagree (cli stderr: {cli_stderr})",
            case.id
        );
        assert_eq!(
            cli, library,
            "case {}: `parse` and library validation disagree (cli stderr: {cli_stderr})",
            case.id
        );
        assert_eq!(
            cli,
            case.accepted,
            "case {}: expected {}, got {} (cli stderr: {cli_stderr})",
            case.id,
            if case.accepted { "accept" } else { "reject" },
            if cli { "accept" } else { "reject" }
        );
    }

    let _ = std::fs::remove_dir_all(&dir);
}
