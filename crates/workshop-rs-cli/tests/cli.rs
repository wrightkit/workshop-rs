//! CLI surface tests: parse/emit/convert a file, list locales, report the
//! machine-readable catalog identity, and fail explicitly on missing
//! target-locale mappings (with opt-in fallback).

use std::path::PathBuf;
use std::process::Command;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_workshop-rs-cli")
}

fn fixture(fixture_id: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../workshop-rs/tests/fixtures/corpus")
        .join(format!("{fixture_id}.ws"))
}

fn run(args: &[&str]) -> std::process::Output {
    Command::new(bin()).args(args).output().expect("cli runs")
}

#[test]
fn version_reports_the_machine_readable_identity() {
    let output = run(&["version"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("implementation version: "));
    assert!(stdout.contains("catalog version: "));
    assert!(stdout.contains("catalog digest: "));
    assert!(stdout.contains("locale en-us"));
    assert!(stdout.contains("locale zh-cn"));

    let output = run(&["version", "--json"]);
    assert!(output.status.success());
    let identity: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("identity is valid JSON");
    assert!(identity["implementation-version"].is_string());
    assert!(identity["catalog-digest"].is_string());
    assert!(identity["locale-coverage"].is_array());
}

#[test]
fn locales_lists_declared_locales_with_coverage() {
    let output = run(&["locales"]);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 2);
    assert!(lines[0].starts_with("en-us 341/341"), "{stdout}");
    assert!(lines[1].starts_with("zh-cn 341/341"), "{stdout}");
}

#[test]
fn parse_parses_a_corpus_file() {
    let file = fixture("basic-rule");
    let output = run(&["parse", file.to_str().unwrap()]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("rule \"setup\""), "{stdout}");
}

#[test]
fn parse_with_explicit_locale_works() {
    let file = fixture("basic-rule");
    let output = run(&["parse", "--locale", "en-US", file.to_str().unwrap()]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn parse_reports_unknown_spellings_with_failure_exit() {
    let dir = std::env::temp_dir().join("workshop-rs-cli-parse-bad");
    std::fs::create_dir_all(&dir).unwrap();
    let bad = dir.join("bad.ws");
    std::fs::write(
        &bad,
        "rule (\"x\") { event { Ongoing - Global; } actions { Totally Unknown Thing(1); } }",
    )
    .unwrap();
    let output = run(&["parse", "--locale", "en-US", bad.to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Totally Unknown Thing"), "{stderr}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn emit_emits_localized_text() {
    let file = fixture("basic-rule");
    let output = run(&["emit", file.to_str().unwrap()]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("rule (\"setup\") {"), "{stdout}");
    assert!(stdout.contains("Disable Inspector Recording;"), "{stdout}");
}

#[test]
fn convert_to_zh_cn_uses_the_corpus_without_fallback() {
    let file = fixture("basic-rule");
    let output = run(&[
        "convert",
        file.to_str().unwrap(),
        "--from",
        "en-US",
        "--to",
        "zh-CN",
    ]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("持续 - 全局"), "{stdout}");
    assert!(stdout.contains("禁用查看器录制"), "{stdout}");
}

#[test]
fn convert_to_zh_cn_with_fallback_reports_the_choice() {
    let dir = std::env::temp_dir().join("workshop-rs-cli-convert-fallback");
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("unmapped.ws");
    std::fs::write(
        &file,
        "rule (\"setup\") { event { Ongoing - Global; } actions { Disable Inspector Recording; } }",
    )
    .unwrap();
    let output = run(&[
        "convert",
        file.to_str().unwrap(),
        "--from",
        "en-US",
        "--to",
        "fr-FR",
        "--fallback-locale",
        "en-US",
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Ongoing - Global"), "{stdout}");
    assert!(stdout.contains("Disable Inspector Recording"), "{stdout}");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("fallback-locale spelling") && stderr.contains("disableInspector"),
        "the fallback choice is visible in tooling output: {stderr}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn usage_errors_exit_2() {
    let output = run(&["convert"]);
    assert_eq!(output.status.code(), Some(2));
    let output = run(&["no-such-command"]);
    assert_eq!(output.status.code(), Some(2));
}
