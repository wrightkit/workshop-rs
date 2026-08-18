//! Standalone command-line interface for the canonical Workshop core
//! (`workshop-rs`). Operates on raw Workshop text files: parse to WIR,
//! emit localized Workshop text, convert between locales, list declared
//! locales with coverage, and print the machine-readable catalog identity.
//!
//! Exit codes: `0` success, `1` parse/emit/conversion/catalog failure,
//! `2` usage error.

use std::path::{Path, PathBuf};

use workshop_rs::catalog::{Catalog, Locale};
use workshop_rs::census;
use workshop_rs::convert::{self, ConvertOptions};
use workshop_rs::detect;
use workshop_rs::emitter::{self, EmitOptions};
use workshop_rs::live_capture;
use workshop_rs::parser;

mod corpus;

/// The default locale override for parsing when the input locale is not
/// specified explicitly.
const USAGE: &str = "\
usage: workshop-rs-cli <command> [options]

commands:
  parse <file> [--locale LOCALE]
      Parse raw Workshop text into validated Workshop IR and print a
      deterministic WIR dump. Without --locale the locale is auto-detected.
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
      catalog version and content digest, locale coverage, target evidence,
      and provenance.
  census [--json]
      Run the deterministic offline Workshop feature census. Unexpected
      regressions exit with status 1; known gaps remain visible.
  corpus <manifest> [--json]
      Run an offline provenance-linked real-project corpus manifest and print
      its #18 conformance report. Known gaps remain visible and do not count
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

/// `--locale LOCALE`, `--fallback-locale LOCALE`, `--from`/`--to LOCALE`,
/// `--json`, `--file PATH`, and the positional file argument.
struct ArgParser {
    args: Vec<String>,
    position: usize,
}

impl ArgParser {
    fn new(args: Vec<String>) -> Self {
        ArgParser { args, position: 0 }
    }

    fn next(&mut self) -> Option<&str> {
        let value = self.args.get(self.position).map(String::as_str);
        if value.is_some() {
            self.position += 1;
        }
        value
    }

    fn value_after(&mut self, flag: &str) -> Result<String, String> {
        self.next()
            .map(str::to_string)
            .ok_or_else(|| format!("missing value for {flag}"))
    }

    fn expect_end(&mut self) -> Result<(), String> {
        if let Some(extra) = self.next() {
            return Err(format!("unexpected argument '{extra}'"));
        }
        Ok(())
    }
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

fn parse_command(args: Vec<String>) -> i32 {
    let mut parser = ArgParser::new(args);
    let mut file: Option<PathBuf> = None;
    let mut locale: Option<Locale> = None;
    loop {
        match parser.next() {
            None => break,
            Some("--locale") => match parser.value_after("--locale") {
                Ok(value) => locale = Some(Locale::new(&value)),
                Err(error) => return usage_error(&error),
            },
            Some(value) if file.is_none() => file = Some(PathBuf::from(value)),
            Some(value) => return usage_error(&format!("unexpected argument '{value}'")),
        }
    }
    let Some(file) = file else {
        return usage_error("parse requires a file argument");
    };
    let (catalog, input) = match (catalog(), read_file(&file)) {
        (Ok(catalog), Ok(input)) => (catalog, input),
        (Err(error), _) | (_, Err(error)) => {
            eprintln!("workshop-rs-cli: {error}");
            return 1;
        }
    };
    let locale = match resolve_parse_locale(&input, &catalog, locale) {
        Ok(locale) => locale,
        Err(error) => {
            eprintln!("workshop-rs-cli: {error}");
            return 1;
        }
    };
    let program = match parser::parse_with_context(&input, &catalog, &locale, &catalog) {
        Ok(program) => program,
        Err(error) => {
            eprintln!("workshop-rs-cli: {error}");
            return 1;
        }
    };
    if let Err(error) = program.validate() {
        eprintln!("workshop-rs-cli: WIR validation failed: {error}");
        return 1;
    }
    print!("{}", program.dump());
    0
}

fn emit_command(args: Vec<String>) -> i32 {
    let mut parser = ArgParser::new(args);
    let mut file: Option<PathBuf> = None;
    let mut locale: Option<Locale> = None;
    let mut fallback: Option<Locale> = None;
    loop {
        match parser.next() {
            None => break,
            Some("--locale") => match parser.value_after("--locale") {
                Ok(value) => locale = Some(Locale::new(&value)),
                Err(error) => return usage_error(&error),
            },
            Some("--fallback-locale") => match parser.value_after("--fallback-locale") {
                Ok(value) => fallback = Some(Locale::new(&value)),
                Err(error) => return usage_error(&error),
            },
            Some(value) if file.is_none() => file = Some(PathBuf::from(value)),
            Some(value) => return usage_error(&format!("unexpected argument '{value}'")),
        }
    }
    let Some(file) = file else {
        return usage_error("emit requires a file argument");
    };
    let (catalog, input) = match (catalog(), read_file(&file)) {
        (Ok(catalog), Ok(input)) => (catalog, input),
        (Err(error), _) | (_, Err(error)) => {
            eprintln!("workshop-rs-cli: {error}");
            return 1;
        }
    };
    let locale = match resolve_parse_locale(&input, &catalog, locale) {
        Ok(locale) => locale,
        Err(error) => {
            eprintln!("workshop-rs-cli: {error}");
            return 1;
        }
    };
    let program = match parser::parse_with_context(&input, &catalog, &locale, &catalog) {
        Ok(program) => program,
        Err(error) => {
            eprintln!("workshop-rs-cli: {error}");
            return 1;
        }
    };
    let options = EmitOptions {
        fallback_locale: fallback,
    };
    match emitter::emit_with_options(&program, &catalog, &locale, &options) {
        Ok(output) => {
            report_fallbacks(&output.fallback_ids);
            print!("{}", output.text);
            0
        }
        Err(error) => {
            eprintln!("workshop-rs-cli: {error}");
            1
        }
    }
}

fn convert_command(args: Vec<String>) -> i32 {
    let mut parser = ArgParser::new(args);
    let mut file: Option<PathBuf> = None;
    let mut from: Option<Locale> = None;
    let mut to: Option<Locale> = None;
    let mut fallback: Option<Locale> = None;
    loop {
        match parser.next() {
            None => break,
            Some("--from") => match parser.value_after("--from") {
                Ok(value) => from = Some(Locale::new(&value)),
                Err(error) => return usage_error(&error),
            },
            Some("--to") => match parser.value_after("--to") {
                Ok(value) => to = Some(Locale::new(&value)),
                Err(error) => return usage_error(&error),
            },
            Some("--fallback-locale") => match parser.value_after("--fallback-locale") {
                Ok(value) => fallback = Some(Locale::new(&value)),
                Err(error) => return usage_error(&error),
            },
            Some(value) if file.is_none() => file = Some(PathBuf::from(value)),
            Some(value) => return usage_error(&format!("unexpected argument '{value}'")),
        }
    }
    let Some(file) = file else {
        return usage_error("convert requires a file argument");
    };
    let (Some(from), Some(to)) = (from, to) else {
        return usage_error("convert requires --from and --to locales");
    };
    let (catalog, input) = match (catalog(), read_file(&file)) {
        (Ok(catalog), Ok(input)) => (catalog, input),
        (Err(error), _) | (_, Err(error)) => {
            eprintln!("workshop-rs-cli: {error}");
            return 1;
        }
    };
    let options = ConvertOptions {
        fallback_locale: fallback,
    };
    match convert::convert(&input, &catalog, &from, &to, &options) {
        Ok(output) => {
            report_fallbacks(&output.fallback_ids);
            print!("{}", output.text);
            0
        }
        Err(error) => {
            eprintln!("workshop-rs-cli: {error}");
            1
        }
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
    let mut parser = ArgParser::new(args);
    if let Err(error) = parser.expect_end() {
        return usage_error(&error);
    }
    let catalog = match catalog() {
        Ok(catalog) => catalog,
        Err(error) => {
            eprintln!("workshop-rs-cli: {error}");
            return 1;
        }
    };
    for coverage in catalog.locale_coverage_all() {
        println!("{} {}/{}", coverage.locale, coverage.mapped, coverage.total);
    }
    0
}

fn version_command(args: Vec<String>) -> i32 {
    let mut parser = ArgParser::new(args);
    let mut json = false;
    loop {
        match parser.next() {
            None => break,
            Some("--json") => json = true,
            Some(value) => return usage_error(&format!("unexpected argument '{value}'")),
        }
    }
    let catalog = match catalog() {
        Ok(catalog) => catalog,
        Err(error) => {
            eprintln!("workshop-rs-cli: {error}");
            return 1;
        }
    };
    let identity = catalog.identity();
    if json {
        match serde_json::to_string_pretty(&identity) {
            Ok(text) => println!("{text}"),
            Err(error) => {
                eprintln!("workshop-rs-cli: cannot serialize identity: {error}");
                return 1;
            }
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
    if report.results.iter().any(|result| {
        result.status == workshop_rs::conformance::ConformanceStatus::UnexpectedRegression
    }) {
        1
    } else {
        0
    }
}

fn corpus_command(args: Vec<String>) -> i32 {
    let mut parser = ArgParser::new(args);
    let mut manifest: Option<PathBuf> = None;
    let mut json = false;
    loop {
        match parser.next() {
            None => break,
            Some("--json") => json = true,
            Some(value) if manifest.is_none() => manifest = Some(PathBuf::from(value)),
            Some(value) => return usage_error(&format!("unexpected argument '{value}'")),
        }
    }
    let Some(manifest) = manifest else {
        return usage_error("corpus requires a manifest file");
    };
    match corpus::run(&manifest) {
        Ok(report) => {
            if json {
                match serde_json::to_string_pretty(&report) {
                    Ok(text) => println!("{text}"),
                    Err(error) => {
                        eprintln!("workshop-rs-cli: cannot serialize corpus report: {error}");
                        return 1;
                    }
                }
            } else {
                print!("{}", report.human_summary());
            }
            if report.has_unexpected_regression() {
                1
            } else {
                0
            }
        }
        Err(error) => {
            eprintln!("workshop-rs-cli: corpus: {error}");
            1
        }
    }
}

fn seasonal_diff_command(args: Vec<String>) -> i32 {
    let mut paths = Vec::new();
    let mut json = false;
    for argument in args {
        if argument == "--json" {
            json = true;
        } else if paths.len() < 2 {
            paths.push(PathBuf::from(argument));
        } else {
            return usage_error("seasonal-diff accepts two capture files and --json");
        }
    }
    if paths.len() != 2 {
        return usage_error("seasonal-diff requires previous and current capture files");
    }
    let previous = match read_file(&paths[0]) {
        Ok(text) => text,
        Err(error) => {
            eprintln!("workshop-rs-cli: {error}");
            return 1;
        }
    };
    let current = match read_file(&paths[1]) {
        Ok(text) => text,
        Err(error) => {
            eprintln!("workshop-rs-cli: {error}");
            return 1;
        }
    };
    let previous = match live_capture::LiveCapture::from_json(&previous) {
        Ok(capture) => capture,
        Err(error) => {
            eprintln!("workshop-rs-cli: seasonal-diff: {error}");
            return 1;
        }
    };
    let current = match live_capture::LiveCapture::from_json(&current) {
        Ok(capture) => capture,
        Err(error) => {
            eprintln!("workshop-rs-cli: seasonal-diff: {error}");
            return 1;
        }
    };
    let diff = match previous.diff(&current) {
        Ok(diff) => diff,
        Err(error) => {
            eprintln!("workshop-rs-cli: seasonal-diff: {error}");
            return 1;
        }
    };
    if json {
        match diff.to_json() {
            Ok(text) => println!("{text}"),
            Err(error) => {
                eprintln!("workshop-rs-cli: cannot serialize seasonal diff: {error}");
                return 1;
            }
        }
    } else {
        print!("{}", diff.human_summary());
    }
    0
}

fn usage_error(message: &str) -> i32 {
    eprintln!("workshop-rs-cli: {message}");
    eprintln!("{USAGE}");
    2
}
