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
use workshop_rs::settings::schema;

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
            "--file" => match args.next() {
                Some(path) => file = PathBuf::from(path),
                None => {
                    eprintln!("workshop-catalog-gen: missing value for --file");
                    return ExitCode::from(2);
                }
            },
            "--export" => match args.next() {
                Some(path) => export = Some(PathBuf::from(path)),
                None => {
                    eprintln!("workshop-catalog-gen: missing value for --export");
                    return ExitCode::from(2);
                }
            },
            "--out-dir" => match args.next() {
                Some(path) => out_dir = PathBuf::from(path),
                None => {
                    eprintln!("workshop-catalog-gen: missing value for --out-dir");
                    return ExitCode::from(2);
                }
            },
            "--settings-out" => match args.next() {
                Some(path) => settings_out = PathBuf::from(path),
                None => {
                    eprintln!("workshop-catalog-gen: missing value for --settings-out");
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
                if let Err(errors) = native_spellings(&content) {
                    for error in errors {
                        eprintln!("workshop-catalog-gen: {error}");
                    }
                    return ExitCode::from(1);
                }
                if let Err(errors) = schema::validate_catalog() {
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
const WIKI_INVENTORY: &str = include_str!("../catalog/data/wiki-inventory.json");

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
mod corpus {
    use super::*;
    use serde_json::{Map, Value};

    /// A match candidate inside the export: the export key (for provenance)
    /// and every reviewed locale spelling.
    #[derive(Debug, Clone)]
    struct Candidate {
        key: String,
        locales: std::collections::BTreeMap<String, String>,
    }

    /// An exact-en-US-spelling index over a slice of the export.
    #[derive(Debug, Clone, Default)]
    struct Index {
        by_en: HashMap<String, Vec<Candidate>>,
    }

    impl Index {
        fn add_translations(&mut self, key: &str, en: &str, locales: Map<String, Value>) {
            if !en.is_empty() {
                let aliases = locales
                    .into_iter()
                    .filter_map(|(locale, value)| {
                        value.as_str().map(|value| (locale, value.to_string()))
                    })
                    .collect();
                self.by_en
                    .entry(en.to_string())
                    .or_default()
                    .push(Candidate {
                        key: key.to_string(),
                        locales: aliases,
                    });
            }
        }

        /// Match an exact en-US spelling. The source identities must agree on
        /// the zh-CN spelling so the existing corpus contract remains strict;
        /// other locale aliases are merged independently below.
        fn match_spelling(&self, en: &str) -> Result<Vec<Candidate>, String> {
            let Some(candidates) = self.by_en.get(en) else {
                return Err("no exact en-US match in the export".to_string());
            };
            let zh = candidates
                .iter()
                .find_map(|candidate| candidate.locales.get("zh-CN"));
            for candidate in candidates {
                if zh.is_some()
                    && candidate
                        .locales
                        .get("zh-CN")
                        .is_some_and(|value| Some(value) != zh)
                {
                    let keys: Vec<&str> = candidates.iter().map(|c| c.key.as_str()).collect();
                    return Err(format!(
                        "ambiguous: export candidates disagree on zh-CN ({})",
                        keys.join(", ")
                    ));
                }
            }
            Ok(candidates.clone())
        }

        fn locale_spelling(
            candidates: &[Candidate],
            locale: &str,
        ) -> Result<Option<String>, String> {
            let mut spelling: Option<&str> = None;
            for candidate in candidates {
                let Some(candidate_spelling) = candidate.locales.get(locale) else {
                    return Ok(None);
                };
                if let Some(previous) = spelling {
                    if previous != candidate_spelling {
                        let keys: Vec<&str> = candidates.iter().map(|c| c.key.as_str()).collect();
                        return Err(format!(
                            "ambiguous: export candidates disagree on {locale} ({})",
                            keys.join(", ")
                        ));
                    }
                } else {
                    spelling = Some(candidate_spelling);
                }
            }
            Ok(spelling.map(str::to_string))
        }
    }

    /// Build an index from `localized` entries with any of the given key
    /// prefixes.
    fn localized_index(export: &Value, prefixes: &[&str]) -> Index {
        let mut index = Index::default();
        let Some(localized) = export.get("localized").and_then(Value::as_object) else {
            return index;
        };
        for (key, entry) in localized {
            if !prefixes.iter().any(|prefix| key.starts_with(prefix)) {
                continue;
            }
            let Some(translations) = entry.get("translations") else {
                continue;
            };
            let en = translations.get("en-US").and_then(Value::as_str);
            if let Some(en) = en {
                index.add_translations(
                    key,
                    en,
                    translations.as_object().cloned().unwrap_or_default(),
                );
            }
        }
        index
    }

    /// Build an index from a `data.*` section with direct locale fields
    /// (maps, heroes), keyed `data.<id>` for provenance.
    fn data_index(export: &Value, section: &str) -> Index {
        let mut index = Index::default();
        let Some(entries) = export
            .get("data")
            .and_then(|data| data.get(section))
            .and_then(Value::as_object)
        else {
            return index;
        };
        for (id, entry) in entries {
            let en = entry.get("en-US").and_then(Value::as_str);
            if let Some(en) = en {
                index.add_translations(
                    &format!("data.{section}.{id}"),
                    en,
                    entry.clone().as_object().cloned().unwrap_or_default(),
                );
            }
        }
        index
    }

    /// Build an index from a nested `data.<parent>.<section>` table with
    /// direct locale fields.
    fn nested_data_index(export: &Value, parent: &str, section: &str) -> Index {
        let mut index = Index::default();
        let Some(entries) = export
            .get("data")
            .and_then(|data| data.get(parent))
            .and_then(|parent| parent.get(section))
            .and_then(Value::as_object)
        else {
            return index;
        };
        for (id, entry) in entries {
            let en = entry.get("en-US").and_then(Value::as_str);
            if let Some(en) = en {
                index.add_translations(
                    &format!("data.{parent}.{section}.{id}"),
                    en,
                    entry.clone().as_object().cloned().unwrap_or_default(),
                );
            }
        }
        index
    }

    /// Resolve a user-confirmed legacy identity whose export wording differs
    /// from the canonical English spelling. The export entry is accepted only
    /// when its category, identity, GUID, and both locale values match the
    /// confirmed mapping.
    fn confirmed_identity_match(export: &Value, kind: &str, id: &str) -> Option<Vec<Candidate>> {
        let (key, category, export_id, guid, en_us) = match (kind, id) {
            ("action", "setAllowedHeroes") => (
                "actions..setAllowedHeroes",
                "actions",
                ".setAllowedHeroes",
                "00000000BA5B",
                "Set Player Allowed Heroes",
            ),
            ("operator", "==") => (
                "localizedStrings.{0} == {1}",
                "localizedStrings",
                "{0} == {1}",
                "00000000BFA3",
                "{0} == {1}",
            ),
            ("operator", "!=") => (
                "localizedStrings.{0} != {1}",
                "localizedStrings",
                "{0} != {1}",
                "00000000BFA2",
                "{0} != {1}",
            ),
            ("operator", "<=") => (
                "localizedStrings.{0} <= {1}",
                "localizedStrings",
                "{0} <= {1}",
                "00000000BFA1",
                "{0} <= {1}",
            ),
            ("operator", ">=") => (
                "localizedStrings.{0} >= {1}",
                "localizedStrings",
                "{0} >= {1}",
                "00000000BF9F",
                "{0} >= {1}",
            ),
            ("operator", "<") => (
                "localizedStrings.{0} < {1}",
                "localizedStrings",
                "{0} < {1}",
                "00000000BFA6",
                "{0} < {1}",
            ),
            ("operator", ">") => (
                "localizedStrings.{0} > {1}",
                "localizedStrings",
                "{0} > {1}",
                "00000000BFA0",
                "{0} > {1}",
            ),
            ("enum member", "Map.LIJIANG_TOWER_LUNAR") => (
                "maps.lijiangTowerLny",
                "maps",
                "lijiangTowerLny",
                "000000005A33",
                "Lijiang Tower Lunar New Year",
            ),
            ("enum member", "ProgressBarWorldReeval.VISIBLE_TO_AND_VALUES") => (
                "constants.ProgressHudReeval.VISIBILITY_AND_VALUES",
                "constants",
                "ProgressHudReeval.VISIBILITY_AND_VALUES",
                "0000000122EF",
                "Visible To and Values",
            ),
            ("enum member", "Rounding.NEAREST") => (
                "constants.__Rounding__.__roundToNearest__",
                "constants",
                "__Rounding__.__roundToNearest__",
                "00000000C34D",
                "To Nearest",
            ),
            _ => return None,
        };
        let entry = export.get("localized")?.get(key)?;
        if entry.get("category")?.as_str()? != category
            || entry.get("id")?.as_str()? != export_id
            || entry.get("guid")?.as_str()? != guid
        {
            return None;
        }
        let translations = entry.get("translations")?;
        if translations.get("en-US")?.as_str()? != en_us {
            return None;
        }
        let mut locales = translations.as_object()?.clone();
        if kind == "operator" {
            for locale in [
                "de-DE", "en-US", "es-ES", "es-MX", "fr-FR", "it-IT", "ja-JP", "ko-KR", "pl-PL",
                "pt-BR", "ru-RU", "th-TH", "tr-TR", "zh-CN", "zh-TW",
            ] {
                locales.insert(locale.to_string(), Value::String(id.to_string()));
            }
        }
        Some(vec![Candidate {
            key: key.to_string(),
            locales: locales
                .into_iter()
                .filter_map(|(locale, value)| {
                    value.as_str().map(|value| (locale, value.to_string()))
                })
                .collect(),
        }])
    }

    /// One matched corpus entry.
    #[derive(Debug, Clone)]
    struct Match {
        kind: String,
        id: String,
        en: String,
        locales: std::collections::BTreeMap<String, String>,
        locale_exclusions: std::collections::BTreeMap<String, String>,
        sources: Vec<String>,
    }

    /// One excluded catalog identity with its reason.
    #[derive(Debug, Clone)]
    struct Exclusion {
        kind: String,
        id: String,
        en: String,
        reason: String,
    }

    /// The declared settings surface projected from `settings::schema`.
    #[derive(Debug)]
    struct SettingsSurface {
        namespaces: Vec<(String, String)>,
        /// `(surface id, en-US name)` of every rendering label. The
        /// per-mode `enabled` members render no label (the mode header's
        /// `disabled` prefix instead) and are excluded here.
        labels: Vec<(String, String)>,
        modes: Vec<(String, String)>,
        maps: Vec<(String, String)>,
        heroes: Vec<(String, String)>,
        teams: Vec<(String, String)>,
        enums: Vec<(String, String)>,
        tokens: Vec<(String, String)>,
    }

    type SettingsSection<'a> = (&'a str, Vec<(String, String)>, &'a Index);

    struct Report<'a> {
        coverage: &'a [(String, usize, usize)],
        total_matched: usize,
        total_entries: usize,
        excluded: &'a [Exclusion],
        settings: &'a Value,
        catalog_file: &'a Path,
        manifest_path: &'a Path,
        settings_out: &'a Path,
    }

    fn settings_surface(catalog: &Catalog) -> SettingsSurface {
        // The declared label surface is the set of distinct rendered names;
        // per-mode repeats (enabled maps, Limit Roles, Competitive Rules)
        // share one label. The per-mode `enabled` members render no label
        // (the mode header's `disabled` prefix instead) and are excluded.
        let namespaces = [
            ("namespace.main", "main"),
            ("namespace.lobby", "lobby"),
            ("namespace.modes", "modes"),
            ("namespace.heroes", "heroes"),
            ("namespace.extensions", "extensions"),
            ("namespace.workshop", "workshop"),
        ]
        .into_iter()
        .map(|(id, name)| (id.to_string(), name.to_string()))
        .collect();
        let mut labels: Vec<(String, String)> = Vec::new();
        let mut seen = std::collections::HashSet::new();
        let definitions: Vec<_> = schema::definitions().collect();
        for definition in &definitions {
            if definition.path().ends_with(".enabled") {
                continue;
            }
            let name = definition.presentation().english_name;
            if seen.insert(name) {
                labels.push((definition.path().to_string(), name.to_string()));
            }
        }
        let locale = Locale::new("en-US");
        let mut mode_keys = std::collections::BTreeSet::new();
        for definition in &definitions {
            if let Some(mode) = definition
                .path()
                .strip_prefix("gamemodes.")
                .and_then(|path| path.split('.').next())
            {
                mode_keys.insert(mode.to_string());
            }
        }
        mode_keys.insert("tdm".to_string());
        mode_keys.insert("ctf".to_string());
        mode_keys.insert("general".to_string());
        let modes = mode_keys
            .into_iter()
            .map(|key| {
                let name = if key == "general" {
                    "General".to_string()
                } else {
                    enum_spelling(catalog, "Gamemode", &key, &locale).unwrap_or_else(|| key.clone())
                };
                (format!("mode.{}.name", key), name)
            })
            .collect();
        let mut maps = catalog_name_surface(catalog, "Map", "map", &locale);
        if !maps.iter().any(|(_, name)| name == "Workshop Island") {
            maps.push((
                "map.workshopIsland.name".to_string(),
                "Workshop Island".to_string(),
            ));
        }
        let heroes = catalog_name_surface(catalog, "Hero", "hero", &locale);
        let teams = vec![
            ("team.allTeams.name".to_string(), "General".to_string()),
            ("team.team1.name".to_string(), "Team 1".to_string()),
            ("team.team2.name".to_string(), "Team 2".to_string()),
        ];
        let enums = definitions
            .iter()
            .flat_map(|definition| {
                definition.enum_members().map(|member| {
                    (
                        format!("enum.{}.{}", member.domain(), member.id()),
                        member.english_name().to_string(),
                    )
                })
            })
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();
        let tokens = [
            ("token.on", "On"),
            ("token.off", "Off"),
            ("token.no", "No"),
            ("token.disabled", "disabled"),
        ]
        .iter()
        .map(|(id, name)| (id.to_string(), name.to_string()))
        .collect();
        SettingsSurface {
            namespaces,
            labels,
            modes,
            maps,
            heroes,
            teams,
            enums,
            tokens,
        }
    }

    fn catalog_name_surface(
        catalog: &Catalog,
        domain: &str,
        prefix: &str,
        locale: &Locale,
    ) -> Vec<(String, String)> {
        let Some(domain) = catalog.enum_domain(domain) else {
            return Vec::new();
        };
        domain
            .members
            .iter()
            .filter_map(|member| {
                let name = member.spelling(locale)?;
                Some((
                    format!("{prefix}.{}.name", lower_camel(&member.member)),
                    name.to_string(),
                ))
            })
            .collect()
    }

    fn enum_spelling(
        catalog: &Catalog,
        domain: &str,
        member: &str,
        locale: &Locale,
    ) -> Option<String> {
        let member = catalog
            .enum_domain(domain)?
            .members
            .iter()
            .find(|candidate| candidate.member.eq_ignore_ascii_case(member))?;
        member.spelling(locale).map(str::to_string)
    }

    fn lower_camel(value: &str) -> String {
        let mut parts = value.split('_');
        let mut result = parts.next().unwrap_or_default().to_ascii_lowercase();
        for part in parts {
            let mut chars = part.to_ascii_lowercase().chars().collect::<Vec<_>>();
            if let Some(first) = chars.first_mut() {
                first.make_ascii_uppercase();
            }
            result.extend(chars);
        }
        result
    }

    /// The corpus pipeline report lines.
    pub(crate) fn generate(
        catalog_data: &str,
        catalog_file: &Path,
        export_path: &Path,
        out_dir: &Path,
        settings_out: &Path,
    ) -> Result<Vec<String>, String> {
        // The base catalog must be valid before merging corpus data.
        let settings_catalog = Catalog::load_unverified(catalog_data)
            .map_err(|error| format!("catalog data: {error}"))?;
        let export_text = std::fs::read_to_string(export_path)
            .map_err(|error| format!("cannot read export {}: {error}", export_path.display()))?;
        let export: Value = serde_json::from_str(&export_text)
            .map_err(|error| format!("cannot parse export {}: {error}", export_path.display()))?;
        let meta = export.get("meta").cloned().unwrap_or(Value::Null);
        let locales = export_locales(&meta)?;
        let catalog: Value =
            serde_json::from_str(catalog_data).map_err(|error| format!("catalog data: {error}"))?;

        // --- builtin corpus -------------------------------------------------
        let actions = localized_index(&export, &["actions."]);
        let structural = {
            let mut index = localized_index(&export, &["other.rules."]);
            merge_index(&mut index, actions.clone());
            merge_index(
                &mut index,
                localized_index(&export, &["customGameSettings."]),
            );
            index
        };
        let values = localized_index(&export, &["values."]);
        let events = localized_index(&export, &["other.events."]);
        let event_teams = {
            let mut index = localized_index(&export, &["other.eventTeams."]);
            merge_index(
                &mut index,
                nested_data_index(&export, "other", "eventTeams"),
            );
            index
        };
        let event_players = {
            let mut index = localized_index(&export, &["other.eventPlayers.", "other.eventSlots."]);
            merge_index(
                &mut index,
                nested_data_index(&export, "other", "eventPlayers"),
            );
            merge_index(
                &mut index,
                nested_data_index(&export, "other", "eventSlots"),
            );
            index
        };
        let operators = localized_index(&export, &["values.", "constants.__Operation__."]);
        let maps = {
            let mut index = localized_index(&export, &["maps."]);
            merge_index(&mut index, data_index(&export, "maps"));
            index
        };
        let heroes = {
            let mut index = localized_index(&export, &["heroes."]);
            merge_index(&mut index, data_index(&export, "heroes"));
            index
        };
        let vector = localized_index(&export, &["values.Vector."]);
        let localized_strings = localized_index(&export, &["localizedStrings."]);
        let gamemodes = data_index(&export, "gamemodes");
        // Enum domains match the export's constants domain for their exact
        // en-US spellings; the export renames a few domains.
        let mut constants_by_domain: HashMap<String, Index> = HashMap::new();
        for domain in enum_domains(&catalog)? {
            let export_domain = match domain.as_str() {
                "EventTeam" => "__event_team__",
                "EventPlayer" => "__event_player__",
                "Color" => "ColorLiteral",
                "Team" => "TeamLiteral",
                "Button" => "ButtonLiteral",
                "Clipping" => "Clip",
                "InworldTextReeval" => "WorldTextReeval",
                "Operation" => "__Operation__",
                "Rounding" => "__Rounding__",
                other => other,
            };
            let prefix = format!("constants.{export_domain}.");
            let index = match domain.as_str() {
                "EventTeam" => event_teams.clone(),
                "EventPlayer" => event_players.clone(),
                _ => localized_index(&export, &[&prefix]),
            };
            constants_by_domain.insert(domain.to_string(), index);
        }

        let mut matches: Vec<Match> = Vec::new();
        let mut excluded: Vec<Exclusion> = Vec::new();
        let mut coverage: Vec<(String, usize, usize)> = Vec::new();

        for (kind, category, entries) in [
            ("structural", "structural", catalog.get("structural")),
            ("action", "actions", catalog.get("actions")),
            ("value", "values", catalog.get("values")),
            ("event", "events", catalog.get("events")),
            ("operator", "operators", catalog.get("operators")),
            (
                "localizedString",
                "localizedStrings",
                catalog.get("localizedStrings"),
            ),
        ] {
            let index = match category {
                "structural" => &structural,
                "actions" => &actions,
                "values" => &values,
                "events" => &events,
                "operators" => &operators,
                "localizedStrings" => &localized_strings,
                _ => unreachable!(),
            };
            let mut matched = 0;
            let mut total = 0;
            for entry in entries.and_then(Value::as_array).into_iter().flatten() {
                total += 1;
                let id = entry
                    .get("id")
                    .and_then(Value::as_str)
                    .ok_or_else(|| "catalog entry without id".to_string())?;
                let aliases = en_aliases(entry)?;
                let (en, candidates) = match match_en_aliases(index, &aliases) {
                    Ok((candidates, en)) => (en, Ok(candidates)),
                    Err(reason) => (
                        aliases[0],
                        confirmed_identity_match(&export, kind, id).ok_or(reason),
                    ),
                };
                match candidates {
                    Ok(candidates) => {
                        matched += 1;
                        let (locale_aliases, locale_exclusions) =
                            locale_aliases(&candidates, &locales);
                        matches.push(Match {
                            kind: kind.to_string(),
                            id: id.to_string(),
                            en: en.to_string(),
                            locales: locale_aliases,
                            locale_exclusions,
                            sources: candidates.iter().map(|c| c.key.clone()).collect(),
                        });
                    }
                    Err(reason) => excluded.push(Exclusion {
                        kind: kind.to_string(),
                        id: id.to_string(),
                        en: en.to_string(),
                        reason,
                    }),
                }
            }
            coverage.push((category.to_string(), matched, total));
        }

        // Enum members: Map/Hero/Vector domains match their own export
        // domains; other domains match their (renamed) constants domain.
        let mut members_matched = 0;
        let mut members_total = 0;
        let enum_domains = enum_domains(&catalog)?;
        for domain_name in &enum_domains {
            let index = match domain_name.as_str() {
                "Map" => &maps,
                "Hero" => &heroes,
                "Vector" => &vector,
                "Gamemode" => &gamemodes,
                _ => constants_by_domain.get(domain_name).ok_or_else(|| {
                    format!("missing constants index for enum domain '{domain_name}'")
                })?,
            };
            let Some(domain) = catalog
                .get("enums")
                .and_then(Value::as_array)
                .and_then(|domains| {
                    domains.iter().find(|d| {
                        d.get("domain").and_then(Value::as_str) == Some(domain_name.as_str())
                    })
                })
            else {
                continue;
            };
            for member in domain
                .get("members")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                members_total += 1;
                let id = member
                    .get("id")
                    .and_then(Value::as_str)
                    .ok_or_else(|| "enum member without id".to_string())?;
                let aliases = en_aliases(member)?;
                let (en, candidates) = match match_en_aliases(index, &aliases) {
                    Ok((candidates, en)) => (en, candidates),
                    Err(reason) => {
                        let full_id = format!("{domain_name}.{id}");
                        match confirmed_identity_match(&export, "enum member", &full_id) {
                            Some(candidates) => (aliases[0], candidates),
                            None => {
                                excluded.push(Exclusion {
                                    kind: "enum member".to_string(),
                                    id: full_id,
                                    en: aliases[0].to_string(),
                                    reason,
                                });
                                continue;
                            }
                        }
                    }
                };
                members_matched += 1;
                let (locale_aliases, locale_exclusions) = locale_aliases(&candidates, &locales);
                matches.push(Match {
                    kind: "enum member".to_string(),
                    id: format!("{domain_name}.{id}"),
                    en: en.to_string(),
                    locales: locale_aliases,
                    locale_exclusions,
                    sources: candidates.iter().map(|c| c.key.clone()).collect(),
                });
            }
        }
        coverage.push(("enums".to_string(), members_matched, members_total));

        let total_matched: usize = coverage.iter().map(|(_, m, _)| m).sum();
        let total_entries: usize = coverage.iter().map(|(_, _, t)| t).sum();

        // --- merge reviewed locale aliases into the catalog data -------------
        let merged = merge_localized_aliases(&catalog, &matches, &locales)?;
        let merged_text = canonical_json(&merged)?;
        Catalog::load_unverified(&merged_text)
            .map_err(|error| format!("merged catalog is invalid: {error}"))?;
        std::fs::write(catalog_file, &merged_text)
            .map_err(|error| format!("cannot write {}: {error}", catalog_file.display()))?;

        // --- corpus manifest -------------------------------------------------
        let manifest = manifest(
            &meta,
            &matches,
            &excluded,
            &coverage,
            total_matched,
            total_entries,
            &locales,
        );
        std::fs::create_dir_all(out_dir)
            .map_err(|error| format!("cannot create {}: {error}", out_dir.display()))?;
        let manifest_path = out_dir.join("zh-cn-corpus.json");
        std::fs::write(&manifest_path, canonical_json(&manifest)?)
            .map_err(|error| format!("cannot write {}: {error}", manifest_path.display()))?;

        // --- settings locale corpus ------------------------------------------
        let settings = settings_corpus(&export, settings_out, &settings_catalog)?;
        if let Some(parent) = settings_out.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|error| format!("cannot create {}: {error}", parent.display()))?;
        }
        std::fs::write(settings_out, canonical_json(&settings)?)
            .map_err(|error| format!("cannot write {}: {error}", settings_out.display()))?;

        Ok(format_report(Report {
            coverage: &coverage,
            total_matched,
            total_entries,
            excluded: &excluded,
            settings: &settings,
            catalog_file,
            manifest_path: &manifest_path,
            settings_out,
        }))
    }

    /// The export index for one catalog enum domain.
    fn enum_domains(catalog: &Value) -> Result<Vec<String>, String> {
        let mut domains = Vec::new();
        for domain in catalog
            .get("enums")
            .and_then(Value::as_array)
            .ok_or_else(|| "catalog without enums".to_string())?
        {
            domains.push(
                domain
                    .get("domain")
                    .and_then(Value::as_str)
                    .ok_or_else(|| "enum domain without name".to_string())?
                    .to_string(),
            );
        }
        Ok(domains)
    }

    fn merge_index(target: &mut Index, source: Index) {
        for (en, candidates) in source.by_en {
            target.by_en.entry(en).or_default().extend(candidates);
        }
    }

    fn export_locales(meta: &Value) -> Result<Vec<String>, String> {
        let locales = meta
            .get("locales")
            .and_then(Value::as_array)
            .ok_or_else(|| "export metadata has no locale list".to_string())?
            .iter()
            .map(|locale| {
                locale
                    .as_str()
                    .map(str::to_string)
                    .ok_or_else(|| "export metadata contains a non-string locale".to_string())
            })
            .collect::<Result<Vec<_>, _>>()?;
        if locales.is_empty() {
            return Err("export metadata declares no locales".to_string());
        }
        Ok(locales)
    }

    fn locale_aliases(
        candidates: &[Candidate],
        locales: &[String],
    ) -> (
        std::collections::BTreeMap<String, String>,
        std::collections::BTreeMap<String, String>,
    ) {
        let mut aliases = std::collections::BTreeMap::new();
        let mut exclusions = std::collections::BTreeMap::new();
        for locale in locales {
            match Index::locale_spelling(candidates, locale) {
                Ok(Some(spelling)) => {
                    aliases.insert(locale.clone(), spelling);
                }
                Ok(None) => {}
                Err(reason) => {
                    exclusions.insert(locale.clone(), reason);
                }
            }
        }
        (aliases, exclusions)
    }

    /// The en-US alias of a catalog entry/member.
    /// The en-US alias spellings of a catalog entry or enum member. A single
    /// string alias is a one-element list; an array lists alternative en-US
    /// spellings that are each eligible for exact export matching.
    fn en_aliases(entry: &Value) -> Result<Vec<&str>, String> {
        let alias = entry
            .get("aliases")
            .and_then(|aliases| aliases.get("en-US"))
            .ok_or_else(|| format!("catalog entry '{}' without en-US alias", entry))?;
        match alias {
            Value::String(spelling) => Ok(vec![spelling.as_str()]),
            Value::Array(spellings) => {
                let spellings = spellings
                    .iter()
                    .map(Value::as_str)
                    .collect::<Option<Vec<&str>>>()
                    .ok_or_else(|| {
                        format!("catalog entry '{}' with non-string en-US alias", entry)
                    })?;
                if spellings.is_empty() {
                    return Err(format!(
                        "catalog entry '{}' with empty en-US aliases",
                        entry
                    ));
                }
                Ok(spellings)
            }
            _ => Err(format!(
                "catalog entry '{}' with non-string en-US alias",
                entry
            )),
        }
    }

    /// Try every en-US alias spelling in order against the index; returns the
    /// first exact match with the alias that matched, or the last miss reason.
    fn match_en_aliases<'a>(
        index: &Index,
        aliases: &'a [&'a str],
    ) -> Result<(Vec<Candidate>, &'a str), String> {
        let mut reason = "no en-US alias spellings".to_string();
        for &en in aliases {
            match index.match_spelling(en) {
                Ok(candidates) => return Ok((candidates, en)),
                Err(miss) => reason = miss,
            }
        }
        Err(reason)
    }

    /// Canonical (sorted-key, pretty) JSON serialization, byte-idempotent.
    fn canonical_json(value: &Value) -> Result<String, String> {
        let mut out = serde_json::to_string_pretty(value)
            .map_err(|error| format!("cannot serialize JSON: {error}"))?;
        out.push('\n');
        Ok(out)
    }

    /// Merge reviewed locale aliases into the catalog data. Existing aliases
    /// remain accepted aliases; no spelling is fabricated (ADR-0001).
    fn merge_localized_aliases(
        catalog: &Value,
        matches: &[Match],
        locales: &[String],
    ) -> Result<Value, String> {
        let mut merged = catalog.clone();
        let conflicts = localized_alias_conflicts(&merged, matches);
        let Some(object) = merged.as_object_mut() else {
            return Err("catalog is not an object".to_string());
        };
        object.insert(
            "locales".to_string(),
            Value::Array(locales.iter().cloned().map(Value::String).collect()),
        );
        let mut by_identity: HashMap<(&str, &str), &Match> = HashMap::new();
        for matched in matches {
            by_identity.insert((matched.kind.as_str(), matched.id.as_str()), matched);
        }
        for category in [
            "structural",
            "actions",
            "values",
            "events",
            "operators",
            "localizedStrings",
        ] {
            let Some(list) = object.get_mut(category).and_then(Value::as_array_mut) else {
                continue;
            };
            for entry in list {
                let Some(id) = entry.get("id").and_then(Value::as_str) else {
                    continue;
                };
                let kind = match category {
                    "structural" => "structural",
                    "actions" => "action",
                    "values" => "value",
                    "events" => "event",
                    "operators" => "operator",
                    "localizedStrings" => "localizedString",
                    _ => unreachable!(),
                };
                if let Some(matched) = by_identity.get(&(kind, id)).copied() {
                    for (locale, spelling) in &matched.locales {
                        if conflicts.contains(&(kind.to_string(), locale.clone(), spelling.clone()))
                        {
                            continue;
                        }
                        set_alias(entry, locale, spelling)?;
                    }
                }
            }
        }
        if let Some(enums) = object.get_mut("enums").and_then(Value::as_array_mut) {
            for domain in enums {
                let domain_name = domain
                    .get("domain")
                    .and_then(Value::as_str)
                    .map(str::to_string);
                let Some(members) = domain.get_mut("members").and_then(Value::as_array_mut) else {
                    continue;
                };
                for member in members {
                    let Some(id) = member.get("id").and_then(Value::as_str) else {
                        continue;
                    };
                    let Some(domain_name) = &domain_name else {
                        continue;
                    };
                    let key = format!("{domain_name}.{id}");
                    if let Some(matched) = by_identity.get(&("enum member", key.as_str())).copied()
                    {
                        for (locale, spelling) in &matched.locales {
                            if conflicts.contains(&(
                                "enum member".to_string(),
                                locale.clone(),
                                spelling.clone(),
                            )) {
                                continue;
                            }
                            set_alias(member, locale, spelling)?;
                        }
                    }
                }
            }
        }
        Ok(merged)
    }

    fn localized_alias_conflicts(
        catalog: &Value,
        matches: &[Match],
    ) -> std::collections::HashSet<(String, String, String)> {
        let mut owners: HashMap<
            (String, String, String),
            std::collections::BTreeSet<(String, String)>,
        > = HashMap::new();
        for category in [
            "structural",
            "actions",
            "values",
            "events",
            "operators",
            "localizedStrings",
        ] {
            let kind = match category {
                "structural" => "structural",
                "actions" => "action",
                "values" => "value",
                "events" => "event",
                "operators" => "operator",
                "localizedStrings" => "localizedString",
                _ => unreachable!(),
            };
            for entry in catalog
                .get(category)
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                let Some(id) = entry.get("id").and_then(Value::as_str) else {
                    continue;
                };
                record_alias_owners(&mut owners, kind, id, entry);
            }
        }
        for domain in catalog
            .get("enums")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let Some(domain_name) = domain.get("domain").and_then(Value::as_str) else {
                continue;
            };
            for member in domain
                .get("members")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                let Some(id) = member.get("id").and_then(Value::as_str) else {
                    continue;
                };
                record_alias_owners(
                    &mut owners,
                    "enum member",
                    &format!("{domain_name}.{id}"),
                    member,
                );
            }
        }
        for matched in matches {
            for (locale, spelling) in &matched.locales {
                owners
                    .entry((matched.kind.clone(), locale.clone(), spelling.clone()))
                    .or_default()
                    .insert((matched.kind.clone(), matched.id.clone()));
            }
        }
        owners
            .into_iter()
            .filter_map(|(alias, owners)| (owners.len() > 1).then_some(alias))
            .collect()
    }

    type AliasOwners =
        HashMap<(String, String, String), std::collections::BTreeSet<(String, String)>>;

    fn record_alias_owners(owners: &mut AliasOwners, kind: &str, id: &str, entry: &Value) {
        let Some(aliases) = entry.get("aliases").and_then(Value::as_object) else {
            return;
        };
        for (locale, values) in aliases {
            match values {
                Value::String(spelling) => {
                    owners
                        .entry((kind.to_string(), locale.clone(), spelling.clone()))
                        .or_default()
                        .insert((kind.to_string(), id.to_string()));
                }
                Value::Array(values) => {
                    for spelling in values.iter().filter_map(Value::as_str) {
                        owners
                            .entry((kind.to_string(), locale.clone(), spelling.to_string()))
                            .or_default()
                            .insert((kind.to_string(), id.to_string()));
                    }
                }
                _ => {}
            }
        }
    }

    /// Set (or extend) the reviewed aliases of one catalog entry.
    fn set_alias(entry: &mut Value, locale: &str, spelling: &str) -> Result<(), String> {
        let Some(aliases) = entry.get_mut("aliases").and_then(Value::as_object_mut) else {
            return Err("catalog entry without aliases object".to_string());
        };
        match aliases.remove(locale) {
            Some(Value::String(existing)) if existing == spelling => {
                aliases.insert(locale.to_string(), Value::String(existing));
            }
            Some(Value::String(existing)) => {
                aliases.insert(
                    locale.to_string(),
                    Value::Array(vec![
                        Value::String(existing),
                        Value::String(spelling.to_string()),
                    ]),
                );
            }
            Some(Value::Array(existing)) => {
                let mut existing = existing;
                if !existing
                    .iter()
                    .any(|alias| alias.as_str() == Some(spelling))
                {
                    existing.push(Value::String(spelling.to_string()));
                }
                aliases.insert(locale.to_string(), Value::Array(existing));
            }
            Some(existing) => {
                return Err(format!(
                    "catalog declares invalid {locale} aliases value {existing}"
                ));
            }
            None => {
                aliases.insert(locale.to_string(), Value::String(spelling.to_string()));
            }
        }
        Ok(())
    }

    /// The machine-readable corpus manifest (ADR-0001 Decision 6).
    fn manifest(
        meta: &Value,
        matches: &[Match],
        excluded: &[Exclusion],
        coverage: &[(String, usize, usize)],
        total_matched: usize,
        total_entries: usize,
        locales: &[String],
    ) -> Value {
        let mut matches_json: Vec<Value> = matches
            .iter()
            .map(|m| {
                serde_json::json!({
                    "kind": m.kind,
                    "id": m.id,
                    "en-US": m.en,
                    "zh-CN": m.locales.get("zh-CN"),
                    "locales": m.locales,
                    "localeExclusions": m.locale_exclusions,
                    "sources": m.sources,
                })
            })
            .collect();
        matches_json.sort_by(|a, b| {
            (a["kind"].as_str(), a["id"].as_str()).cmp(&(b["kind"].as_str(), b["id"].as_str()))
        });
        let mut excluded_json: Vec<Value> = excluded
            .iter()
            .map(|e| {
                serde_json::json!({
                    "kind": e.kind,
                    "id": e.id,
                    "en-US": e.en,
                    "reason": e.reason,
                })
            })
            .collect();
        excluded_json.sort_by(|a, b| {
            (a["kind"].as_str(), a["id"].as_str()).cmp(&(b["kind"].as_str(), b["id"].as_str()))
        });
        let mut coverage_json = serde_json::Map::new();
        for (category, matched, total) in coverage {
            coverage_json.insert(
                category.clone(),
                serde_json::json!({ "matched": matched, "total": total }),
            );
        }
        let mut coverage_all = coverage_json;
        coverage_all.insert(
            "total".to_string(),
            serde_json::json!({ "matched": total_matched, "total": total_entries }),
        );
        serde_json::json!({
            "schemaVersion": 2,
            "locale": "zh-CN",
            "locales": locales,
            "generator": "workshop-catalog-gen corpus",
            "generatorVersion": env!("CARGO_PKG_VERSION"),
            "source": {
                "export": meta.get("commit").and_then(Value::as_str).map(|_| "workshop-data.json").unwrap_or("<unknown>"),
                "commit": meta.get("commit").and_then(Value::as_str).unwrap_or("<unknown>"),
                "commitDate": meta.get("commitDate").and_then(Value::as_str).unwrap_or("<unknown>"),
                "fetchedAt": meta.get("fetchedAt").and_then(Value::as_str).unwrap_or("<unknown>"),
            },
            "method": "exact en-US spelling match between the catalog aliases and the export's localized index (actions/values/events/operators/constants/event filters/maps/heroes), plus confirmed legacy identity/GUID mappings for Set Player Allowed Heroes, and bare comparison-symbol entries; every declared locale is retained only when the export provides an unambiguous spelling; entries without an accepted match, or whose export candidates disagree on zh-CN, are excluded with a recorded reason and keep fail-explicit behavior (ADR-0001 Decision 7)",
            "sourceReview": "reviewed: workshop-rs commits its own mapping data; the user-provided JSON is build input only and is not redistributed",
            "coverage": Value::Object(coverage_all),
            "matches": matches_json,
            "excluded": excluded_json,
        })
    }

    /// The settings locale corpus for the declared settings surface.
    fn settings_corpus(
        export: &Value,
        settings_out: &Path,
        catalog: &Catalog,
    ) -> Result<Value, String> {
        let custom_game = {
            let mut index = localized_index(
                export,
                &[
                    "customGameSettings.",
                    "heroes.",
                    "maps.",
                    "gamemodes.",
                    "constants.",
                ],
            );
            merge_index(
                &mut index,
                localized_index(export, &["other.customGameSettings."]),
            );
            index
        };
        let gamemodes = {
            let mut index =
                localized_index(export, &["gamemodes.", "customGameSettings.gamemodes."]);
            merge_index(&mut index, data_index(export, "gamemodes"));
            index
        };
        let maps = localized_index(export, &["maps."]);
        let heroes = localized_index(export, &["heroes."]);
        let teams = localized_index(
            export,
            &["heroes.teams.", "customGameSettings.heroes.teams."],
        );
        let tokens = localized_index(export, &["other.customGameSettings."]);
        let surface = settings_surface(catalog);
        let meta = export.get("meta").cloned().unwrap_or(Value::Null);
        let locales = export_locales(&meta)?;

        let mut sections: Vec<SettingsSection<'_>> = vec![
            ("namespaces", surface.namespaces, &custom_game),
            ("labels", surface.labels, &custom_game),
            ("modes", surface.modes, &gamemodes),
            ("maps", surface.maps, &maps),
            ("heroes", surface.heroes, &heroes),
            ("teams", surface.teams, &teams),
            ("enums", surface.enums, &custom_game),
            ("tokens", surface.tokens, &tokens),
        ];

        let mut entries: Map<String, Value> = Map::new();
        let mut excluded: Vec<Value> = Vec::new();
        let mut coverage: Map<String, Value> = Map::new();
        for (section, surface_entries, index) in &mut sections {
            let mut matched = 0;
            let mut total = 0;
            for (surface_id, en) in surface_entries {
                total += 1;
                // The mode-header `disabled` prefix (surface form) maps the
                // export's capitalized `Disabled` token (__disabled__).
                let export_en = if *section == "tokens" && en == "disabled" {
                    "Disabled"
                } else {
                    en.as_str()
                };
                match index.match_spelling(export_en) {
                    Ok(candidates) => {
                        matched += 1;
                        entries.insert(
                            en.clone(),
                            settings_entry(&candidates, export_en, &locales)?,
                        );
                    }
                    Err(reason) => {
                        match confirmed_settings_identity_match(export, surface_id, en) {
                            Some(candidates) => {
                                matched += 1;
                                entries.insert(
                                    en.clone(),
                                    settings_entry(&candidates, export_en, &locales)?,
                                );
                            }
                            None => excluded.push(serde_json::json!({
                                "surface": surface_id,
                                "en-US": en,
                                "reason": reason,
                            })),
                        }
                    }
                }
            }
            coverage.insert(
                section.to_string(),
                serde_json::json!({ "matched": matched, "total": total }),
            );
        }

        if let Ok(previous) = std::fs::read_to_string(settings_out) {
            if let Ok(previous) = serde_json::from_str::<Value>(&previous) {
                preserve_existing_settings(&mut entries, &previous);
            }
        }

        // Split the flat matched entries into per-section maps mirroring the
        // declared settings surface.
        let mut labels = Map::new();
        let mut namespaces = Map::new();
        let mut modes = Map::new();
        let mut maps_out = Map::new();
        let mut heroes_out = Map::new();
        let mut teams_out = Map::new();
        let mut enums_out = Map::new();
        let mut tokens_out = Map::new();
        for (section, surface_entries, _) in &sections {
            let target = match *section {
                "namespaces" => &mut namespaces,
                "labels" => &mut labels,
                "modes" => &mut modes,
                "maps" => &mut maps_out,
                "heroes" => &mut heroes_out,
                "teams" => &mut teams_out,
                "enums" => &mut enums_out,
                "tokens" => &mut tokens_out,
                _ => unreachable!(),
            };
            for (_, en) in surface_entries {
                if let Some(entry) = entries.get(en) {
                    target.insert(en.clone(), entry.clone());
                }
            }
        }

        Ok(serde_json::json!({
            "schemaVersion": 2,
            "locales": locales,
            "provenance": {
                "generator": "workshop-catalog-gen corpus",
                "generatorVersion": env!("CARGO_PKG_VERSION"),
                "source": "user-provided workshop-data export (workshop-data.json)",
                "commit": meta.get("commit").and_then(Value::as_str).unwrap_or("<unknown>"),
                "commitDate": meta.get("commitDate").and_then(Value::as_str).unwrap_or("<unknown>"),
                "fetchedAt": meta.get("fetchedAt").and_then(Value::as_str).unwrap_or("<unknown>"),
                "method": "exact en-US spelling match between the declared settings surface (settings::schema) and every locale translation carried by the export's customGameSettings/gamemodes/maps/heroes labels, direct data.gamemodes entries, and other.customGameSettings tokens; the two hero Ultimate Generation labels are composed per reviewed locale from exact export templates and hero identities; entries without an accepted match keep fail-explicit behavior (ADR-0001 Decision 7)",
            "sourceReview": "reviewed: workshop-rs commits its own settings mapping data; the user-provided JSON is build input only and is not redistributed",
            },
            "namespaces": namespaces,
            "labels": labels,
            "modes": modes,
            "maps": maps_out,
            "heroes": heroes_out,
            "teams": teams_out,
            "enums": enums_out,
            "tokens": tokens_out,
            "excluded": excluded,
            "coverage": coverage,
            "projection": "multi-locale-settings/v1",
        }))
    }

    fn settings_entry(
        candidates: &[Candidate],
        expected_en: &str,
        locales: &[String],
    ) -> Result<Value, String> {
        let mut aliases = Map::new();
        let mut locale_exclusions = Map::new();
        for candidate in candidates {
            if candidate.locales.get("en-US").map(String::as_str) != Some(expected_en) {
                continue;
            }
            for (locale, value) in &candidate.locales {
                if !locales.contains(locale) {
                    continue;
                }
                if let Some(previous) = aliases.get(locale).and_then(Value::as_str) {
                    if previous != value {
                        locale_exclusions.insert(
                            locale.clone(),
                            Value::String(format!("ambiguous: {previous:?} vs {value:?}")),
                        );
                        aliases.remove(locale);
                    }
                } else if !locale_exclusions.contains_key(locale) {
                    aliases.insert(locale.clone(), Value::String(value.clone()));
                }
            }
        }
        if aliases.get("en-US").and_then(Value::as_str).is_none() {
            return Err("settings match lacks the required primary en-US alias".to_string());
        }
        aliases.insert(
            "sources".to_string(),
            Value::Array(
                candidates
                    .iter()
                    .map(|candidate| Value::String(candidate.key.clone()))
                    .collect(),
            ),
        );
        if !locale_exclusions.is_empty() {
            aliases.insert(
                "localeExclusions".to_string(),
                Value::Object(locale_exclusions),
            );
        }
        Ok(Value::Object(aliases))
    }

    fn preserve_existing_settings(entries: &mut Map<String, Value>, previous: &Value) {
        let mut previous_entries = Map::new();
        for section in [
            "namespaces",
            "labels",
            "modes",
            "maps",
            "heroes",
            "teams",
            "enums",
            "tokens",
        ] {
            if let Some(section_entries) = previous.get(section).and_then(Value::as_object) {
                for (name, entry) in section_entries {
                    previous_entries.insert(name.clone(), entry.clone());
                }
            }
        }
        for (name, previous_entry) in previous_entries {
            let current_entry = entries.entry(name).or_insert(previous_entry.clone());
            let (Some(current), Some(previous)) =
                (current_entry.as_object_mut(), previous_entry.as_object())
            else {
                continue;
            };
            for locale in ["en-US", "zh-CN"] {
                if let Some(value) = previous.get(locale).and_then(Value::as_str) {
                    current.insert(locale.to_string(), Value::String(value.to_string()));
                }
            }
            if let Some(previous_sources) = previous.get("sources").and_then(Value::as_array) {
                let sources = current
                    .entry("sources")
                    .or_insert_with(|| Value::Array(Vec::new()));
                if let Some(sources) = sources.as_array_mut() {
                    for source in previous_sources {
                        if !sources.contains(source) {
                            sources.push(source.clone());
                        }
                    }
                }
            }
        }
    }

    /// Resolve the two hero settings labels whose English surface expands a
    /// reviewed `%1$s` export template with the reviewed `Blizzard` hero
    /// spelling. The template, hero identity, GUIDs, and both locale values
    /// are checked before composing the locale label.
    fn confirmed_settings_identity_match(
        export: &Value,
        surface: &str,
        en: &str,
    ) -> Option<Vec<Candidate>> {
        let (template_key, template_id, template_guid, template_en, hero_key, hero_guid) =
            match (surface, en) {
                (
                    "heroes.<team>.<hero>.passiveUltGen%",
                    "Ultimate Generation - Passive Blizzard",
                ) => (
                    "customGameSettings.heroes.values.__eachHero__.passiveUltGen%",
                    "heroes.values.__eachHero__.passiveUltGen%",
                    "00000000765E",
                    "Ultimate Generation - Passive %1$s",
                    "heroes.mei.ultimate",
                    "000000001789",
                ),
                ("heroes.<team>.<hero>.combatUltGen%", "Ultimate Generation - Combat Blizzard") => {
                    (
                        "customGameSettings.heroes.values.__eachHero__.combatUltGen%",
                        "heroes.values.__eachHero__.combatUltGen%",
                        "00000000765D",
                        "Ultimate Generation - Combat %1$s",
                        "heroes.mei.ultimate",
                        "000000001789",
                    )
                }
                _ => return None,
            };
        let template = export.get("localized")?.get(template_key)?;
        if template.get("category")?.as_str()? != "customGameSettings"
            || template.get("id")?.as_str()? != template_id
            || template.get("guid")?.as_str()? != template_guid
        {
            return None;
        }
        let template_translations = template.get("translations")?;
        let template_zh = template_translations.get("zh-CN")?.as_str()?;
        if template_translations.get("en-US")?.as_str()? != template_en
            || !template_zh.contains("%1$s")
        {
            return None;
        }
        let hero = export.get("localized")?.get(hero_key)?;
        if hero.get("category")?.as_str()? != "heroes"
            || hero.get("id")?.as_str()? != "mei.ultimate"
            || hero.get("guid")?.as_str()? != hero_guid
        {
            return None;
        }
        let hero_translations = hero.get("translations")?;
        if hero_translations.get("en-US")?.as_str()? != "Blizzard" {
            return None;
        }
        hero_translations.get("zh-CN")?.as_str()?;
        let mut template_locales = template_translations.as_object()?.clone();
        let hero_locales = hero_translations.as_object()?.clone();
        for (locale, value) in &mut template_locales {
            if let Some(template) = value.as_str() {
                if let Some(hero_name) = hero_locales.get(locale).and_then(Value::as_str) {
                    *value = Value::String(template.replace("%1$s", hero_name));
                }
            }
        }
        Some(vec![
            Candidate {
                key: template_key.to_string(),
                locales: template_locales
                    .into_iter()
                    .filter_map(|(locale, value)| {
                        value.as_str().map(|value| (locale, value.to_string()))
                    })
                    .collect(),
            },
            Candidate {
                key: hero_key.to_string(),
                locales: hero_locales
                    .into_iter()
                    .filter_map(|(locale, value)| {
                        value.as_str().map(|value| (locale, value.to_string()))
                    })
                    .collect(),
            },
        ])
    }

    fn format_report(report: Report<'_>) -> Vec<String> {
        let Report {
            coverage,
            total_matched,
            total_entries,
            excluded,
            settings,
            catalog_file,
            manifest_path,
            settings_out,
        } = report;
        let mut lines = vec![format!(
            "corpus: locale set matched {total_matched}/{total_entries} canonical entries and enum members"
        )];
        for (category, matched, total) in coverage {
            lines.push(format!("  {category}: {matched}/{total}"));
        }
        lines.push(format!("  excluded (fail-explicit): {}", excluded.len()));
        for exclusion in excluded {
            lines.push(format!(
                "    {} {} ({}): {}",
                exclusion.kind, exclusion.id, exclusion.en, exclusion.reason
            ));
        }
        let settings_coverage = settings
            .get("coverage")
            .and_then(Value::as_object)
            .map(|coverage| {
                coverage
                    .iter()
                    .map(|(section, counts)| {
                        format!(
                            "{} {}/{}",
                            section,
                            counts["matched"].as_u64().unwrap_or(0),
                            counts["total"].as_u64().unwrap_or(0)
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_default();
        lines.push(format!("  settings: {settings_coverage}"));
        lines.push(format!("wrote {}", catalog_file.display()));
        lines.push(format!("wrote {}", manifest_path.display()));
        lines.push(format!("wrote {}", settings_out.display()));
        lines.push(
            "next: run 'workshop-catalog-gen build' (fresh digest) then 'check' (verify)"
                .to_string(),
        );
        lines
    }

    #[cfg(test)]
    mod tests {
        use super::{
            Index, en_aliases, match_en_aliases, set_alias, settings_corpus, settings_surface,
        };
        use serde_json::json;
        use std::path::Path;
        use workshop_rs::catalog::Catalog;

        #[test]
        fn en_aliases_accepts_scalar_and_array_spellings() {
            let scalar = json!({"aliases": {"en-US": "Nearest"}});
            assert_eq!(
                en_aliases(&scalar).expect("scalar alias is accepted"),
                vec!["Nearest"]
            );

            let array = json!({"aliases": {"en-US": ["Jinyu", "Domina"]}});
            assert_eq!(
                en_aliases(&array).expect("alias array is accepted"),
                vec!["Jinyu", "Domina"]
            );

            assert!(en_aliases(&json!({"aliases": {"zh-CN": "至最近"}})).is_err());
            assert!(en_aliases(&json!({"aliases": {"en-US": []}})).is_err());
            assert!(en_aliases(&json!({"aliases": {"en-US": 7}})).is_err());
        }

        #[test]
        fn match_en_aliases_returns_the_first_exact_match() {
            let mut index = Index::default();
            index.add_translations(
                "heroes.domina",
                "Domina",
                serde_json::json!({"zh-CN": "多美娜"})
                    .as_object()
                    .cloned()
                    .unwrap(),
            );

            let (candidates, en) = match_en_aliases(&index, &["Jinyu", "Domina"])
                .expect("second alias matches the export spelling");
            assert_eq!(en, "Domina");
            assert_eq!(
                candidates[0].locales.get("zh-CN").map(String::as_str),
                Some("多美娜")
            );

            assert!(match_en_aliases(&index, &["Jinyu"]).is_err());
        }

        #[test]
        fn corpus_merge_preserves_and_extends_reviewed_alias_arrays() {
            let mut entry = json!({
                "aliases": {"zh-CN": ["中止", "中断"]}
            });

            set_alias(&mut entry, "zh-CN", "中断").expect("existing reviewed alias is accepted");
            set_alias(&mut entry, "zh-CN", "中止条件").expect("new corpus alias is appended");

            assert_eq!(
                entry["aliases"]["zh-CN"],
                json!(["中止", "中断", "中止条件"])
            );
        }

        #[test]
        fn corpus_merge_promotes_a_scalar_conflict_to_reviewed_aliases() {
            let mut entry = json!({
                "aliases": {"zh-CN": "中止"}
            });

            set_alias(&mut entry, "zh-CN", "中断").expect("conflicting corpus alias is retained");

            assert_eq!(entry["aliases"]["zh-CN"], json!(["中止", "中断"]));
        }

        #[test]
        fn settings_corpus_includes_direct_gamemode_data() {
            let export = json!({
                "meta": { "locales": ["en-US", "zh-CN"] },
                "data": {
                    "gamemodes": {
                        "ctf": {
                            "en-US": "Capture The Flag",
                            "zh-CN": "勇夺锦旗"
                        }
                    }
                }
            });

            let catalog = Catalog::builtin().expect("built-in catalog");
            let settings = settings_corpus(&export, Path::new("settings.json"), &catalog)
                .expect("settings corpus builds");
            assert_eq!(settings["modes"]["Capture The Flag"]["zh-CN"], "勇夺锦旗");
            assert_eq!(
                settings["modes"]["Capture The Flag"]["sources"],
                json!(["data.gamemodes.ctf"])
            );
        }

        #[test]
        fn settings_surface_uses_catalog_gamemode_ids() {
            let catalog = Catalog::builtin().expect("built-in catalog");
            let surface = settings_surface(&catalog);

            for (mode, expected) in [
                ("ffa", "Deathmatch"),
                ("tdm", "Team Deathmatch"),
                ("ctf", "Capture The Flag"),
            ] {
                let id = format!("mode.{mode}.name");
                let actual = surface
                    .modes
                    .iter()
                    .find(|(candidate, _)| candidate == &id)
                    .map(|(_, name)| name.as_str());
                assert_eq!(actual, Some(expected), "mode surface for {mode}");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::native_spellings;

    #[test]
    fn committed_catalog_uses_only_native_spellings() {
        let catalog = include_str!("../catalog/data/catalog.json");
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
