//! Standalone command-line interface for the canonical Workshop core
//! (`workshop-rs`). Operates on raw Workshop text files: parse to the public
//! `Program` model, emit localized Workshop text, convert between locales, list declared
//! locales with coverage, and print the machine-readable catalog identity.
//!
//! Exit codes: `0` success, `1` parse/emit/conversion/catalog failure,
//! `2` usage error.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use workshop_rs::Program;
use workshop_rs::catalog::{Catalog, Locale};
use workshop_rs::convert::{self, ConvertOptions};
use workshop_rs::detect;
use workshop_rs::emitter::{self, EmitOptions};
use workshop_rs::parser;

pub mod census;
pub mod conformance;
mod corpus;
pub mod live_capture;

/// The default locale override for parsing when the input locale is not
/// specified explicitly.
const USAGE: &str = "\
usage: workshop-rs-cli <command> [options]

commands:
  parse <file> [--locale LOCALE]
      Parse raw Workshop text into the validated public Program model and
      print a deterministic debug dump. Without --locale the locale is auto-detected.
  emit <file> [--locale LOCALE] [--fallback-locale LOCALE]
      Parse and emit localized Workshop text (fail-explicit on missing
      target-locale mappings; --fallback-locale opts into fallback, which is
      reported on stderr).
  convert <file> --from LOCALE --to LOCALE [--fallback-locale LOCALE]
      Convert raw Workshop text between locales (parse -> canonical
      semantics -> emit). Missing target-locale mappings fail explicitly
      unless --fallback-locale is given.
  locales
      List the declared locales with per-locale mapping coverage.
  version [--json]
      Print the machine-readable catalog identity: implementation version,
      catalog version and content digest, locale coverage, target source,
      and provenance.
  census [--json]
      Run the deterministic offline Workshop feature census. Unexpected
      regressions exit with status 1; known gaps remain visible.
  corpus <manifest> [--json]
      Run an offline provenance-linked real-project corpus manifest and print
      its conformance report. Known gaps remain visible and do not count
      as matches; unexpected regressions return exit code 1.
  seasonal-diff <previous.json> <current.json> [--json]
      Validate two provenance-rich live-client capture documents and emit a
      structured offline drift report. This command never captures a client.
";

pub fn run(args: Vec<String>) -> i32 {
    let mut args = args.into_iter();
    let Some(command) = args.next() else {
        eprintln!("{USAGE}");
        return 2;
    };
    let rest: Vec<String> = args.collect();
    match command.as_str() {
        "parse" => parse_command(rest),
        "emit" => emit_command(rest),
        "convert" => convert_command(rest),
        "locales" => locales_command(rest),
        "version" => version_command(rest),
        "census" => census_command(rest),
        "corpus" => corpus_command(rest),
        "seasonal-diff" => seasonal_diff_command(rest),
        "help" | "--help" | "-h" => {
            print!("{USAGE}");
            0
        }
        other => {
            eprintln!("workshop-rs-cli: unknown command '{other}'");
            eprintln!("{USAGE}");
            2
        }
    }
}

/// Parsed command arguments: positionals plus declared `--flag value` options
/// and standalone `--switch` toggles.
#[derive(Default)]
struct CliOptions {
    positional: Vec<String>,
    values: HashMap<String, String>,
    switches: HashSet<String>,
}

impl CliOptions {
    fn locale(&self, flag: &str) -> Option<Locale> {
        self.values.get(flag).map(|value| Locale::new(value))
    }

    fn has(&self, switch: &str) -> bool {
        self.switches.contains(switch)
    }

    /// Exactly `count` positional arguments mapped to paths.
    fn paths(&self, count: usize, missing: &str) -> Result<Vec<PathBuf>, String> {
        if self.positional.len() < count {
            return Err(missing.to_string());
        }
        if let Some(extra) = self.positional.get(count) {
            return Err(format!("unexpected argument '{extra}'"));
        }
        Ok(self.positional.iter().map(PathBuf::from).collect())
    }

    fn file(&self, missing: &str) -> Result<PathBuf, String> {
        Ok(self.paths(1, missing)?.remove(0))
    }
}

/// Consume `args`, collecting declared `--flag value` pairs, declared
/// `--switch` toggles, and positional arguments.
fn parse_cli(args: Vec<String>, flags: &[&str], switches: &[&str]) -> Result<CliOptions, String> {
    let mut options = CliOptions::default();
    let mut args = args.into_iter();
    while let Some(argument) = args.next() {
        if switches.contains(&argument.as_str()) {
            options.switches.insert(argument);
        } else if flags.contains(&argument.as_str()) {
            let value = args
                .next()
                .ok_or_else(|| format!("missing value for {argument}"))?;
            options.values.insert(argument, value);
        } else {
            options.positional.push(argument);
        }
    }
    Ok(options)
}

/// Unwrap a parsed-CLI result or a command result into a usage/failure exit.
fn cli_options(args: Vec<String>, flags: &[&str], switches: &[&str]) -> Result<CliOptions, i32> {
    parse_cli(args, flags, switches).map_err(|error| usage_error(&error))
}

fn catalog() -> Result<Catalog, String> {
    Catalog::builtin().map_err(|error| format!("catalog: {error}"))
}

fn read_file(path: &Path) -> Result<String, String> {
    std::fs::read_to_string(path)
        .map_err(|error| format!("cannot read {}: {error}", path.display()))
}

/// Resolve the parse locale: an explicit override always wins; otherwise
/// auto-detect with the documented confidence gate.
fn resolve_parse_locale(
    input: &str,
    catalog: &Catalog,
    explicit: Option<Locale>,
) -> Result<Locale, String> {
    detect::resolve_locale(input, catalog, explicit.as_ref()).map_err(|error| error.to_string())
}

fn parse_file(
    file: &Path,
    explicit_locale: Option<Locale>,
) -> Result<(Catalog, Locale, Program), String> {
    let (catalog, input) = match (catalog(), read_file(file)) {
        (Ok(catalog), Ok(input)) => (catalog, input),
        (Err(error), _) | (_, Err(error)) => return Err(error),
    };
    let locale = resolve_parse_locale(&input, &catalog, explicit_locale)?;
    let program = parser::parse_with_context(&input, &catalog, &locale, &catalog)
        .map_err(|error| error.to_string())?;
    Ok((catalog, locale, program))
}

fn parse_command(args: Vec<String>) -> i32 {
    let options = match cli_options(args, &["--locale"], &[]) {
        Ok(options) => options,
        Err(code) => return code,
    };
    let file = match options.file("parse requires a file argument") {
        Ok(file) => file,
        Err(error) => return usage_error(&error),
    };
    let (_, _, program) = match parse_file(&file, options.locale("--locale")) {
        Ok(parsed) => parsed,
        Err(error) => return fail(error),
    };
    if let Err(error) = program.validate() {
        return fail(format!("WIR validation failed: {error}"));
    }
    print!("{}", program.dump());
    0
}

fn emit_command(args: Vec<String>) -> i32 {
    let options = match cli_options(args, &["--locale", "--fallback-locale"], &[]) {
        Ok(options) => options,
        Err(code) => return code,
    };
    let file = match options.file("emit requires a file argument") {
        Ok(file) => file,
        Err(error) => return usage_error(&error),
    };
    let (catalog, locale, program) = match parse_file(&file, options.locale("--locale")) {
        Ok(parsed) => parsed,
        Err(error) => return fail(error),
    };
    let mut emit_options = EmitOptions::default();
    emit_options.fallback_locale = options.locale("--fallback-locale");
    match emitter::emit_with_options(&program, &catalog, &locale, &emit_options) {
        Ok(output) => {
            report_fallbacks(&output.fallback_ids);
            print!("{}", output.text);
            0
        }
        Err(error) => fail(error),
    }
}

fn convert_command(args: Vec<String>) -> i32 {
    let options = match cli_options(args, &["--from", "--to", "--fallback-locale"], &[]) {
        Ok(options) => options,
        Err(code) => return code,
    };
    let file = match options.file("convert requires a file argument") {
        Ok(file) => file,
        Err(error) => return usage_error(&error),
    };
    let (Some(from), Some(to)) = (options.locale("--from"), options.locale("--to")) else {
        return usage_error("convert requires --from and --to locales");
    };
    let (catalog, input) = match (catalog(), read_file(&file)) {
        (Ok(catalog), Ok(input)) => (catalog, input),
        (Err(error), _) | (_, Err(error)) => return fail(error),
    };
    let mut convert_options = ConvertOptions::default();
    convert_options.fallback_locale = options.locale("--fallback-locale");
    match convert::convert(&input, &catalog, &from, &to, &convert_options) {
        Ok(output) => {
            report_fallbacks(&output.fallback_ids);
            print!("{}", output.text);
            0
        }
        Err(error) => fail(error),
    }
}

/// Report opted-in fallback usage on stderr so the fallback choice is
/// visible in tooling output (ADR-0001 Decision 7).
fn report_fallbacks(fallback_ids: &[String]) {
    if fallback_ids.is_empty() {
        return;
    }
    eprintln!(
        "workshop-rs-cli: note: {} canonical id(s) emitted with a fallback-locale spelling: {}",
        fallback_ids.len(),
        fallback_ids.join(", ")
    );
}

fn locales_command(args: Vec<String>) -> i32 {
    let options = match cli_options(args, &[], &[]) {
        Ok(options) => options,
        Err(code) => return code,
    };
    if let Err(error) = options.paths(0, "") {
        return usage_error(&error);
    }
    let catalog = match catalog() {
        Ok(catalog) => catalog,
        Err(error) => return fail(error),
    };
    for coverage in catalog.locale_coverage_all() {
        println!("{} {}/{}", coverage.locale, coverage.mapped, coverage.total);
    }
    0
}

fn version_command(args: Vec<String>) -> i32 {
    let options = match cli_options(args, &[], &["--json"]) {
        Ok(options) => options,
        Err(code) => return code,
    };
    if let Err(error) = options.paths(0, "") {
        return usage_error(&error);
    }
    let json = options.has("--json");
    let catalog = match catalog() {
        Ok(catalog) => catalog,
        Err(error) => return fail(error),
    };
    let identity = catalog.identity();
    if json {
        if let Err(error) = print_json(&identity, "identity") {
            return fail(error);
        }
    } else {
        println!(
            "implementation version: {}",
            identity.implementation_version
        );
        println!("catalog version: {}", identity.catalog_version);
        println!(
            "catalog digest: {}",
            identity.catalog_digest.as_deref().unwrap_or("<none>")
        );
        for coverage in &identity.locale_coverage {
            println!(
                "locale {}: {}/{} mapped",
                coverage.locale, coverage.mapped, coverage.total
            );
        }
        println!(
            "target: {} ({})",
            identity.target.surface, identity.target.game
        );
    }
    0
}

fn census_command(args: Vec<String>) -> i32 {
    let json = match args.as_slice() {
        [] => false,
        [flag] if flag == "--json" => true,
        _ => return usage_error("census accepts only the optional --json flag"),
    };
    let catalog = match Catalog::builtin() {
        Ok(catalog) => catalog,
        Err(error) => return usage_error(&format!("cannot load catalog: {error}")),
    };
    let census = match census::Census::builtin(&catalog) {
        Ok(census) => census,
        Err(error) => return usage_error(&format!("cannot build census: {error}")),
    };
    let report = census.run(&catalog);
    if let Err(error) = report.validate_against(&catalog) {
        return usage_error(&format!("invalid census report: {error}"));
    }
    if json {
        match report.to_json() {
            Ok(text) => println!("{text}"),
            Err(error) => return usage_error(&format!("cannot serialize census: {error}")),
        }
    } else {
        println!(
            "census schema {} / conformance schema {}",
            report.schema_version, report.conformance_schema_version
        );
        for result in &report.results {
            println!("{}: {:?}", result.case_id, result.status);
        }
    }
    if report
        .results
        .iter()
        .any(|result| result.status == conformance::ConformanceStatus::UnexpectedRegression)
    {
        1
    } else {
        0
    }
}

fn corpus_command(args: Vec<String>) -> i32 {
    let options = match cli_options(args, &[], &["--json"]) {
        Ok(options) => options,
        Err(code) => return code,
    };
    let manifest = match options.file("corpus requires a manifest file") {
        Ok(manifest) => manifest,
        Err(error) => return usage_error(&error),
    };
    match corpus::run(&manifest) {
        Ok(report) => {
            if options.has("--json") {
                if let Err(error) = print_json(&report, "corpus report") {
                    return fail(error);
                }
            } else {
                print!("{}", report.human_summary());
            }
            i32::from(report.has_unexpected_regression())
        }
        Err(error) => fail(format!("corpus: {error}")),
    }
}

fn seasonal_diff_command(args: Vec<String>) -> i32 {
    let options = match cli_options(args, &[], &["--json"]) {
        Ok(options) => options,
        Err(code) => return code,
    };
    let paths = match options.paths(
        2,
        "seasonal-diff requires previous and current capture files",
    ) {
        Ok(paths) => paths,
        Err(error) => return usage_error(&error),
    };
    let capture = |path: &Path| -> Result<live_capture::LiveCapture, String> {
        let text = read_file(path)?;
        live_capture::LiveCapture::from_json(&text)
            .map_err(|error| format!("seasonal-diff: {error}"))
    };
    let (previous, current) = match (capture(&paths[0]), capture(&paths[1])) {
        (Ok(previous), Ok(current)) => (previous, current),
        (Err(error), _) | (_, Err(error)) => return fail(error),
    };
    let diff = match previous.diff(&current) {
        Ok(diff) => diff,
        Err(error) => return fail(format!("seasonal-diff: {error}")),
    };
    if options.has("--json") {
        match diff.to_json() {
            Ok(text) => println!("{text}"),
            Err(error) => return fail(format!("cannot serialize seasonal diff: {error}")),
        }
    } else {
        print!("{}", diff.human_summary());
    }
    0
}

/// Print a command failure and return exit code 1.
fn fail(message: impl std::fmt::Display) -> i32 {
    eprintln!("workshop-rs-cli: {message}");
    1
}

fn print_json(value: &impl serde::Serialize, what: &str) -> Result<(), String> {
    serde_json::to_string_pretty(value)
        .map(|text| println!("{text}"))
        .map_err(|error| format!("cannot serialize {what}: {error}"))
}

fn usage_error(message: &str) -> i32 {
    eprintln!("workshop-rs-cli: {message}");
    eprintln!("{USAGE}");
    2
}
