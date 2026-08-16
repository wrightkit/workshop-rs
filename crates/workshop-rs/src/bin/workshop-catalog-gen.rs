//! `workshop-catalog-gen` — the reproducible Workshop catalog data pipeline.
//!
//! Validates and deterministically canonicalizes the catalog data file, and
//! maintains the dataset's machine-readable identity (version + content
//! digest, ADR-0001).
//!
//! Usage:
//! ```sh
//! workshop-catalog-gen check [--file catalog.json] [--json]
//! workshop-catalog-gen build [--file catalog.json]
//! ```
//!
//! * `check` validates the catalog (schema, duplicate ids, colliding or
//!   missing primary-locale aliases, undeclared locales, param arity) and
//!   verifies the declared content digest, printing the machine-readable
//!   identity (with `--json` as a JSON document).
//! * `build` validates, canonicalizes, and (re)writes the file with a fresh
//!   content digest. Re-running is byte-idempotent.
//!
//! Updating localization data is a bounded data change: edit the JSON and
//! re-run `check`/`build`; no parser or emitter code changes.

use std::path::PathBuf;
use std::process::ExitCode;

use workshop_rs::catalog::{Catalog, build_canonical};

/// The committed catalog data, relative to the workspace root (where CI and
/// the documented pipeline commands run); `--file` overrides it.
const DEFAULT_FILE: &str = "crates/workshop-rs/src/catalog/data/catalog.json";

fn usage() -> &'static str {
    "usage: workshop-catalog-gen <check|build> [--file catalog.json] [--json]"
}

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let command = args.next();
    let mut file = PathBuf::from(DEFAULT_FILE);
    let mut json = false;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--file" => match args.next() {
                Some(path) => file = PathBuf::from(path),
                None => {
                    eprintln!("workshop-catalog-gen: missing value for --file");
                    return ExitCode::from(2);
                }
            },
            "--json" => json = true,
            other => {
                eprintln!("workshop-catalog-gen: unknown argument '{other}'");
                eprintln!("{}", usage());
                return ExitCode::from(2);
            }
        }
    }

    let content = match std::fs::read_to_string(&file) {
        Ok(content) => content,
        Err(error) => {
            eprintln!(
                "workshop-catalog-gen: cannot read {}: {error}",
                file.display()
            );
            return ExitCode::from(2);
        }
    };

    match command.as_deref() {
        Some("check") => match Catalog::load(&content) {
            Ok(catalog) => {
                let identity = catalog.identity();
                if json {
                    match serde_json::to_string_pretty(&identity) {
                        Ok(text) => println!("{text}"),
                        Err(error) => {
                            eprintln!("workshop-catalog-gen: cannot serialize identity: {error}");
                            return ExitCode::from(1);
                        }
                    }
                } else {
                    println!(
                        "OK {} entries, {} enum domains, {} locale(s)",
                        catalog.entry_count(),
                        catalog.enum_domains_count(),
                        catalog.locales().len(),
                    );
                    println!(
                        "version {} digest {}",
                        identity.catalog_version,
                        identity.catalog_digest.as_deref().unwrap_or("<none>")
                    );
                    for coverage in &identity.locale_coverage {
                        println!(
                            "locale {}: {}/{} mapped",
                            coverage.locale, coverage.mapped, coverage.total
                        );
                    }
                }
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("workshop-catalog-gen: {error}");
                ExitCode::from(1)
            }
        },
        Some("build") => match build_canonical(&content) {
            Ok(output) => match std::fs::write(&file, output) {
                Ok(()) => {
                    println!("wrote {}", file.display());
                    ExitCode::SUCCESS
                }
                Err(error) => {
                    eprintln!(
                        "workshop-catalog-gen: cannot write {}: {error}",
                        file.display()
                    );
                    ExitCode::from(2)
                }
            },
            Err(error) => {
                eprintln!("workshop-catalog-gen: {error}");
                ExitCode::from(1)
            }
        },
        _ => {
            eprintln!("{}", usage());
            ExitCode::from(2)
        }
    }
}
