use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use workshop_rs::catalog::{Catalog, Locale, build_canonical};
use workshop_rs::settings;

const DEFAULT_FILE: &str = "crates/workshop-rs/src/catalog/data/catalog.json";

const DEFAULT_OUT_DIR: &str = "tools/corpus";

const DEFAULT_SETTINGS_OUT: &str = "crates/workshop-rs/src/settings/data/locales.json";

const DEFAULT_DESCRIPTIONS_FILE: &str = "crates/workshop-rs/src/catalog/data/descriptions.json";

const EXPORT_ENV: &str = "WORKSHOP_DATA_EXPORT";

fn usage() -> &'static str {
    "usage: workshop-catalog-gen <check|build|corpus|descriptions> [--file catalog.json] [--descriptions descriptions.json] [--json] [--export PATH] [--zh-source PATH] [--out-dir tools/corpus] [--settings-out PATH]"
}

fn export_path(export: Option<PathBuf>) -> Result<PathBuf, String> {
    export
        .or_else(|| std::env::var_os(EXPORT_ENV).map(PathBuf::from))
        .ok_or_else(|| {
            format!(
                "requires the export path via --export or the {EXPORT_ENV} environment variable"
            )
        })
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
    // `args_os` + lossy conversion: `env::args` panics on non-UTF-8 argv.
    let mut args = std::env::args_os()
        .skip(1)
        .map(|arg| arg.to_string_lossy().into_owned());
    let command = args.next();
    let mut file = PathBuf::from(DEFAULT_FILE);
    let mut descriptions_file = PathBuf::from(DEFAULT_DESCRIPTIONS_FILE);
    let mut zh_source = PathBuf::from(descriptions::DEFAULT_ZH_SOURCE);
    let mut json = false;
    let mut export: Option<PathBuf> = None;
    let mut out_dir = PathBuf::from(DEFAULT_OUT_DIR);
    let mut settings_out = PathBuf::from(DEFAULT_SETTINGS_OUT);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--file" => file = flag_value(&mut args, "--file"),
            "--descriptions" => descriptions_file = flag_value(&mut args, "--descriptions"),
            "--zh-source" => zh_source = flag_value(&mut args, "--zh-source"),
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
            Ok(mut catalog) => {
                let identity = catalog.identity();
                if identity.catalog_digest.is_none() {
                    eprintln!(
                        "workshop-catalog-gen: catalog declares no content digest — run `workshop-catalog-gen build` to record one"
                    );
                    return ExitCode::from(1);
                }
                if identity.catalog_version == "dev" {
                    eprintln!(
                        "workshop-catalog-gen: catalog declares no version — run `workshop-catalog-gen build`"
                    );
                    return ExitCode::from(1);
                }
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
                let descriptions_text = match std::fs::read_to_string(&descriptions_file) {
                    Ok(text) => text,
                    Err(error) => {
                        eprintln!(
                            "workshop-catalog-gen: cannot read {}: {error}",
                            descriptions_file.display()
                        );
                        return ExitCode::from(2);
                    }
                };
                if let Err(error) = catalog.load_descriptions(&descriptions_text) {
                    eprintln!("workshop-catalog-gen: descriptions: {error}");
                    return ExitCode::from(1);
                }
                if catalog.descriptions_digest().is_none() {
                    eprintln!(
                        "workshop-catalog-gen: descriptions declare no content digest — run `workshop-catalog-gen build` to record one"
                    );
                    return ExitCode::from(1);
                }
                if catalog.descriptions_version() == "dev" {
                    eprintln!(
                        "workshop-catalog-gen: descriptions declare no version — run `workshop-catalog-gen build`"
                    );
                    return ExitCode::from(1);
                }
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
                    println!(
                        "descriptions version {} digest {}",
                        catalog.descriptions_version(),
                        catalog.descriptions_digest().unwrap_or("<none>")
                    );
                }
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("workshop-catalog-gen: {error}");
                ExitCode::from(1)
            }
        },
        Some("build") => match build_canonical(&content) {
            Ok(output) => {
                if let Err(error) = std::fs::write(&file, &output) {
                    eprintln!(
                        "workshop-catalog-gen: cannot write {}: {error}",
                        file.display()
                    );
                    return ExitCode::from(2);
                }
                println!("wrote {}", file.display());
                let descriptions_text = match std::fs::read_to_string(&descriptions_file) {
                    Ok(text) => text,
                    Err(error) => {
                        eprintln!(
                            "workshop-catalog-gen: cannot read {}: {error}",
                            descriptions_file.display()
                        );
                        return ExitCode::from(2);
                    }
                };
                let catalog = match Catalog::load(&output) {
                    Ok(catalog) => catalog,
                    Err(error) => {
                        eprintln!("workshop-catalog-gen: {error}");
                        return ExitCode::from(1);
                    }
                };
                match workshop_rs::catalog::descriptions::build_canonical_descriptions(
                    &catalog,
                    &descriptions_text,
                ) {
                    Ok(rebuilt) => match std::fs::write(&descriptions_file, rebuilt) {
                        Ok(()) => {
                            println!("wrote {}", descriptions_file.display());
                            ExitCode::SUCCESS
                        }
                        Err(error) => {
                            eprintln!(
                                "workshop-catalog-gen: cannot write {}: {error}",
                                descriptions_file.display()
                            );
                            ExitCode::from(2)
                        }
                    },
                    Err(error) => {
                        eprintln!("workshop-catalog-gen: descriptions: {error}");
                        ExitCode::from(1)
                    }
                }
            }
            Err(error) => {
                eprintln!("workshop-catalog-gen: {error}");
                ExitCode::from(1)
            }
        },
        Some("corpus") => {
            let export_path = match export_path(export) {
                Ok(path) => path,
                Err(error) => {
                    eprintln!("workshop-catalog-gen: corpus {error}");
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
        Some("descriptions") => {
            let export_path = match export_path(export) {
                Ok(path) => path,
                Err(error) => {
                    eprintln!("workshop-catalog-gen: descriptions {error}");
                    return ExitCode::from(2);
                }
            };
            match descriptions::generate(&content, &descriptions_file, &export_path, &zh_source) {
                Ok(report) => {
                    for line in report {
                        println!("{line}");
                    }
                    ExitCode::SUCCESS
                }
                Err(error) => {
                    eprintln!("workshop-catalog-gen: descriptions: {error}");
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

const WIKI_INVENTORY: &str = include_str!("../../catalog/data/wiki-inventory.json");

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

mod corpus;
mod descriptions;

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
