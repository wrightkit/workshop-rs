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
//! workshop-catalog-gen corpus [--file catalog.json] [--export PATH] [--out-dir tools/corpus] [--settings-out PATH]
//! ```
//!
//! * `check` validates the catalog (schema, duplicate ids, colliding or
//!   missing primary-locale aliases, undeclared locales, param arity) and
//!   verifies the declared content digest, printing the machine-readable
//!   identity (with `--json` as a JSON document).
//! * `build` validates, canonicalizes, and (re)writes the file with a fresh
//!   content digest. Re-running is byte-idempotent.
//! * `corpus` applies locale corpus evidence to the catalog data (ADR-0001
//!   Decision 6): it reads the user-provided Workshop data export (`--export`,
//!   or the `WORKSHOP_DATA_EXPORT` environment variable), matches every
//!   catalog entry and enum member by its exact en-US spelling against the
//!   export's localized index, and writes
//!   - the merged catalog data file (reviewed locale aliases added; data
//!     change only, the declared digest is left stale for `build` to
//!     recompute),
//!   - the machine-readable corpus manifest with every match, every exclusion
//!     and its reason, and per-category match statistics, and
//!   - the settings locale corpus for the declared settings surface.
//!     Unmatched entries are excluded from the corpus with a recorded reason
//!     and keep fail-explicit behavior (ADR-0001 Decision 7); no spelling is
//!     fabricated. Re-running on the merged data is byte-idempotent.
//!
//! Updating localization data is a bounded data change: edit the JSON and
//! re-run the pipeline; no parser or emitter code changes. The full locale
//! corpus flow is: `corpus` (data merge) -> `build` (fresh digest) ->
//! `check` (verify); commit data and regenerated files together.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use workshop_rs::catalog::{Catalog, Locale, build_canonical};
use workshop_rs::settings;

/// The committed catalog data, relative to the workspace root (where CI and
/// the documented pipeline commands run); `--file` overrides it.
const DEFAULT_FILE: &str = "crates/workshop-rs/src/catalog/data/catalog.json";

/// The default corpus manifest output directory.
const DEFAULT_OUT_DIR: &str = "tools/corpus";

/// The default settings locale corpus output file.
const DEFAULT_SETTINGS_OUT: &str = "crates/workshop-rs/src/settings/data/locales.json";

/// The environment variable naming the Workshop data export path.
const EXPORT_ENV: &str = "WORKSHOP_DATA_EXPORT";

fn usage() -> &'static str {
    "usage: workshop-catalog-gen <check|build|corpus> [--file catalog.json] [--json] [--export PATH] [--out-dir tools/corpus] [--settings-out PATH]"
}

fn flag_value(args: &mut impl Iterator<Item = String>, flag: &str) -> PathBuf {
    match args.next() {
        Some(path) => PathBuf::from(path),
        None => {
            eprintln!("workshop-catalog-gen: missing value for {flag}");
            std::process::exit(2);
        }
    }
}

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let command = args.next();
    let mut file = PathBuf::from(DEFAULT_FILE);
    let mut json = false;
    let mut export: Option<PathBuf> = None;
    let mut out_dir = PathBuf::from(DEFAULT_OUT_DIR);
    let mut settings_out = PathBuf::from(DEFAULT_SETTINGS_OUT);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--file" => file = flag_value(&mut args, "--file"),
            "--export" => export = Some(flag_value(&mut args, "--export")),
            "--out-dir" => out_dir = flag_value(&mut args, "--out-dir"),
            "--settings-out" => settings_out = flag_value(&mut args, "--settings-out"),
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
                if let Err(errors) = native_spellings(&content) {
                    for error in errors {
                        eprintln!("workshop-catalog-gen: {error}");
                    }
                    return ExitCode::from(1);
                }
                if let Err(errors) = settings::validate_catalog() {
                    for error in errors {
                        eprintln!("workshop-catalog-gen: settings catalog: {error}");
                    }
                    return ExitCode::from(1);
                }
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
        Some("corpus") => {
            let export_path =
                match export.or_else(|| std::env::var_os(EXPORT_ENV).map(PathBuf::from)) {
                    Some(path) => path,
                    None => {
                        eprintln!(
                            "workshop-catalog-gen: corpus requires the export path via --export or \
                         the {EXPORT_ENV} environment variable"
                        );
                        return ExitCode::from(2);
                    }
                };
            match corpus::generate(&content, &file, &export_path, &out_dir, &settings_out) {
                Ok(report) => {
                    for line in report {
                        println!("{line}");
                    }
                    ExitCode::SUCCESS
                }
                Err(error) => {
                    eprintln!("workshop-catalog-gen: {error}");
                    ExitCode::from(1)
                }
            }
        }
        _ => {
            eprintln!("{}", usage());
            ExitCode::from(2)
        }
    }
}

/// The Workshop.codes wiki article titles of every native action and value.
const WIKI_INVENTORY: &str = include_str!("../../catalog/data/wiki-inventory.json");

/// Every action and value en-US spelling must be a native Workshop name from
/// the wiki inventory (compared case-insensitively, since the wiki
/// capitalizes some connecting words differently). A spelling outside it is a
/// non-native identity or alias, which the catalog does not hold.
fn native_spellings(catalog_json: &str) -> Result<(), Vec<String>> {
    let inventory: serde_json::Value =
        serde_json::from_str(WIKI_INVENTORY).map_err(|error| vec![error.to_string()])?;
    let catalog: serde_json::Value =
        serde_json::from_str(catalog_json).map_err(|error| vec![error.to_string()])?;
    let mut errors = Vec::new();
    for (section, kind) in [("actions", "action"), ("values", "value")] {
        let native: std::collections::HashSet<String> = inventory[section]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|name| name.as_str().map(str::to_lowercase))
            .collect();
        let mut owners = std::collections::HashMap::new();
        for entry in catalog[section].as_array().into_iter().flatten() {
            let spellings = match &entry["aliases"]["en-US"] {
                serde_json::Value::String(name) => vec![name.as_str()],
                serde_json::Value::Array(names) => {
                    names.iter().filter_map(serde_json::Value::as_str).collect()
                }
                _ => Vec::new(),
            };
            let id = entry["id"].as_str().unwrap_or("?");
            for name in spellings {
                if let Some(owner) = owners.insert(name.to_lowercase(), id) {
                    errors.push(format!(
                        "{kind} '{id}': en-US spelling '{name}' repeats the native name of '{owner}'"
                    ));
                }
                if !native.contains(&name.to_lowercase()) {
                    errors.push(format!(
                        "{kind} '{}': en-US spelling '{name}' is not a native Workshop name in the wiki inventory",
                        entry["id"].as_str().unwrap_or("?")
                    ));
                }
            }
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// The locale corpus pipeline (ADR-0001 Decision 6).
mod corpus;

#[cfg(test)]
mod tests {
    use super::native_spellings;

    #[test]
    fn committed_catalog_uses_only_native_spellings() {
        let catalog = include_str!("../../catalog/data/catalog.json");
        native_spellings(catalog).expect("catalog holds only native Workshop spellings");
    }

    #[test]
    fn case_variant_of_a_native_name_is_a_duplicate() {
        let catalog = r#"{"actions":[
            {"id":"movePlayerToTeam","aliases":{"en-US":"Move Player to Team"}},
            {"id":"moveToTeamAgain","aliases":{"en-US":"Move Player To Team"}}],"values":[]}"#;
        let errors = native_spellings(catalog).expect_err("case-only duplicate fails");
        assert_eq!(errors.len(), 1, "{errors:?}");
        assert!(errors[0].contains("repeats the native name"), "{errors:?}");
    }

    #[test]
    fn non_native_spelling_is_rejected() {
        let catalog = r#"{"actions":[{"id":"forceThrottle","aliases":{"en-US":"Force Throttle"}}],
            "values":[{"id":"isFiringSecondary","aliases":{"en-US":["Is Firing Secondary","Is Firing Secondary Fire"]}}]}"#;
        let errors = native_spellings(catalog).expect_err("non-native spellings fail");
        assert_eq!(errors.len(), 2, "{errors:?}");
    }
}
