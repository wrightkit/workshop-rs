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
//! * `corpus` applies the zh-CN corpus evidence to the catalog data
//!   (ADR-0001 Decision 6): it reads the user-provided Workshop data export
//!   (`--export`, or the `WORKSHOP_DATA_EXPORT` environment variable),
//!   matches every catalog entry and enum member by its exact en-US spelling
//!   against the export's localized index, and writes
//!   - the merged catalog data file (zh-CN aliases added; data change only,
//!     the declared digest is left stale for `build` to recompute),
//!   - the machine-readable corpus manifest with every match, every exclusion
//!     and its reason, and per-category match statistics, and
//!   - the settings locale corpus for the declared settings surface.
//!     Unmatched entries are excluded from the corpus with a recorded reason
//!     and keep fail-explicit behavior (ADR-0001 Decision 7); no spelling is
//!     fabricated. Re-running on the merged data is byte-idempotent.
//!
//! Updating localization data is a bounded data change: edit the JSON and
//! re-run the pipeline; no parser or emitter code changes. The full zh-CN
//! corpus flow is: `corpus` (data merge) -> `build` (fresh digest) ->
//! `check` (verify); commit data and regenerated files together.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use workshop_rs::catalog::{Catalog, build_canonical};
use workshop_rs::settings::table;

/// The committed catalog data, relative to the workspace root (where CI and
/// the documented pipeline commands run); `--file` overrides it.
const DEFAULT_FILE: &str = "crates/workshop-rs/src/catalog/data/catalog.json";

/// The default corpus manifest output directory.
const DEFAULT_OUT_DIR: &str = "tools/corpus";

/// The default settings locale corpus output file.
const DEFAULT_SETTINGS_OUT: &str = "crates/workshop-rs/src/settings/data/zh-cn.json";

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

/// The zh-CN corpus pipeline (ADR-0001 Decision 6).
mod corpus {
    use super::*;
    use serde_json::{Map, Value};

    /// A match candidate inside the export: the export key (for provenance)
    /// and the zh-CN spelling.
    #[derive(Debug, Clone)]
    struct Candidate {
        key: String,
        zh_cn: String,
    }

    /// An exact-en-US-spelling index over a slice of the export.
    #[derive(Debug, Clone, Default)]
    struct Index {
        by_en: HashMap<String, Vec<Candidate>>,
    }

    impl Index {
        fn add(&mut self, key: &str, en: &str, zh: &str) {
            if !en.is_empty() && !zh.is_empty() {
                self.by_en
                    .entry(en.to_string())
                    .or_default()
                    .push(Candidate {
                        key: key.to_string(),
                        zh_cn: zh.to_string(),
                    });
            }
        }

        /// Match an exact en-US spelling. Returns `(sources, zh-CN)` when
        /// every candidate agrees on zh-CN; a `String` reason otherwise.
        fn match_spelling(&self, en: &str) -> Result<Vec<Candidate>, String> {
            let Some(candidates) = self.by_en.get(en) else {
                return Err("no exact en-US match in the export".to_string());
            };
            let zh = candidates[0].zh_cn.clone();
            for candidate in candidates {
                if candidate.zh_cn != zh {
                    let keys: Vec<&str> = candidates.iter().map(|c| c.key.as_str()).collect();
                    return Err(format!(
                        "ambiguous: export candidates disagree on zh-CN ({})",
                        keys.join(", ")
                    ));
                }
            }
            Ok(candidates.clone())
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
            let zh = translations.get("zh-CN").and_then(Value::as_str);
            if let (Some(en), Some(zh)) = (en, zh) {
                index.add(key, en, zh);
            }
        }
        index
    }

    /// Build an index from a `data.*` section with direct en-US/zh-CN fields
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
            let zh = entry.get("zh-CN").and_then(Value::as_str);
            if let (Some(en), Some(zh)) = (en, zh) {
                index.add(&format!("data.{section}.{id}"), en, zh);
            }
        }
        index
    }

    /// Build an index from a nested `data.<parent>.<section>` table with
    /// direct en-US/zh-CN fields.
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
            let zh = entry.get("zh-CN").and_then(Value::as_str);
            if let (Some(en), Some(zh)) = (en, zh) {
                index.add(&format!("data.{parent}.{section}.{id}"), en, zh);
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
            ("action", "stopChasingVariable") => (
                "actions.__stopChasingGlobalVariable__",
                "actions",
                "__stopChasingGlobalVariable__",
                "00000000B83E",
                "Stop Chasing Global Variable",
            ),
            ("action", "forcePlayerHero") => (
                "actions..startForcingHero",
                "actions",
                ".startForcingHero",
                "00000000ABFB",
                "Start Forcing Player To Be Hero",
            ),
            ("action", "stopForcingHero") => (
                "actions..stopForcingCurrentHero",
                "actions",
                ".stopForcingCurrentHero",
                "00000000AC1B",
                "Stop Forcing Player To Be Hero",
            ),
            ("action", "forceThrottle") => (
                "actions..startForcingThrottle",
                "actions",
                ".startForcingThrottle",
                "00000000BB0F",
                "Start Forcing Throttle",
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
        let export_zh_cn = translations.get("zh-CN")?.as_str()?;
        if translations.get("en-US")?.as_str()? != en_us || export_zh_cn.is_empty() {
            return None;
        }
        let zh_cn = if kind == "operator" {
            // The export's localized-string entry is a formatted display
            // template; the catalog operator token is the bare symbol.
            id
        } else {
            export_zh_cn
        };
        Some(vec![Candidate {
            key: key.to_string(),
            zh_cn: zh_cn.to_string(),
        }])
    }

    /// One matched corpus entry.
    #[derive(Debug, Clone)]
    struct Match {
        kind: String,
        id: String,
        en: String,
        zh: String,
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

    /// The declared settings surface (mirrors `settings::table`).
    #[derive(Debug)]
    struct SettingsSurface {
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

    fn settings_surface() -> SettingsSurface {
        // The declared label surface is the set of distinct rendered names;
        // per-mode repeats (enabled maps, Limit Roles, Competitive Rules)
        // share one label. The per-mode `enabled` members render no label
        // (the mode header's `disabled` prefix instead) and are excluded.
        let mut labels: Vec<(String, String)> = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for entry in table::ENTRIES {
            if matches!(entry.path.last(), Some(table::PathPart::Part("enabled"))) {
                continue;
            }
            if seen.insert(entry.workshop_name) {
                labels.push((
                    table::path_string(entry.path),
                    entry.workshop_name.to_string(),
                ));
            }
        }
        let modes = table::MODE_NAMES
            .iter()
            .map(|m| (format!("mode.{}.name", m.key), m.name.to_string()))
            .collect();
        let maps = table::MAP_NAMES
            .iter()
            .map(|m| (format!("map.{}.name", m.key), m.name.to_string()))
            .collect();
        let heroes = table::HERO_NAMES
            .iter()
            .map(|m| (format!("hero.{}.name", m.key), m.name.to_string()))
            .collect();
        let teams = table::TEAM_NAMES
            .iter()
            .map(|m| (format!("team.{}.name", m.key), m.name.to_string()))
            .collect();
        let enums = table::ENUM_MEMBERS
            .iter()
            .map(|m| {
                (
                    format!("enum.{}.{}", m.domain, m.member),
                    m.name.to_string(),
                )
            })
            .collect();
        let tokens = [
            ("token.on", "On"),
            ("token.off", "Off"),
            ("token.disabled", "disabled"),
        ]
        .iter()
        .map(|(id, name)| (id.to_string(), name.to_string()))
        .collect();
        SettingsSurface {
            labels,
            modes,
            maps,
            heroes,
            teams,
            enums,
            tokens,
        }
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
        Catalog::load_unverified(catalog_data).map_err(|error| format!("catalog data: {error}"))?;
        let export_text = std::fs::read_to_string(export_path)
            .map_err(|error| format!("cannot read export {}: {error}", export_path.display()))?;
        let export: Value = serde_json::from_str(&export_text)
            .map_err(|error| format!("cannot parse export {}: {error}", export_path.display()))?;
        let meta = export.get("meta").cloned().unwrap_or(Value::Null);
        let catalog: Value =
            serde_json::from_str(catalog_data).map_err(|error| format!("catalog data: {error}"))?;

        // --- builtin corpus -------------------------------------------------
        let actions = localized_index(&export, &["actions."]);
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
        ] {
            let index = match category {
                "structural" | "actions" => &actions,
                "values" => &values,
                "events" => &events,
                "operators" => &operators,
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
                let en = en_alias(entry)?;
                let candidates = match index.match_spelling(en) {
                    Ok(candidates) => Ok(candidates),
                    Err(reason) => confirmed_identity_match(&export, kind, id).ok_or(reason),
                };
                match candidates {
                    Ok(candidates) => {
                        matched += 1;
                        let zh = candidates[0].zh_cn.clone();
                        matches.push(Match {
                            kind: kind.to_string(),
                            id: id.to_string(),
                            en: en.to_string(),
                            zh: zh.clone(),
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
                let en = en_alias(member)?;
                match index.match_spelling(en) {
                    Ok(candidates) => {
                        members_matched += 1;
                        let zh = candidates[0].zh_cn.clone();
                        matches.push(Match {
                            kind: "enum member".to_string(),
                            id: format!("{domain_name}.{id}"),
                            en: en.to_string(),
                            zh: zh.clone(),
                            sources: candidates.iter().map(|c| c.key.clone()).collect(),
                        });
                    }
                    Err(reason) => {
                        let full_id = format!("{domain_name}.{id}");
                        match confirmed_identity_match(&export, "enum member", &full_id) {
                            Some(candidates) => {
                                members_matched += 1;
                                let zh = candidates[0].zh_cn.clone();
                                matches.push(Match {
                                    kind: "enum member".to_string(),
                                    id: full_id,
                                    en: en.to_string(),
                                    zh,
                                    sources: candidates.iter().map(|c| c.key.clone()).collect(),
                                });
                            }
                            None => excluded.push(Exclusion {
                                kind: "enum member".to_string(),
                                id: full_id,
                                en: en.to_string(),
                                reason,
                            }),
                        }
                    }
                }
            }
        }
        coverage.push(("enums".to_string(), members_matched, members_total));

        let total_matched: usize = coverage.iter().map(|(_, m, _)| m).sum();
        let total_entries: usize = coverage.iter().map(|(_, _, t)| t).sum();

        // --- merge zh-CN aliases into the catalog data -----------------------
        let merged = merge_zh_aliases(&catalog, &matches)?;
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
        );
        std::fs::create_dir_all(out_dir)
            .map_err(|error| format!("cannot create {}: {error}", out_dir.display()))?;
        let manifest_path = out_dir.join("zh-cn-corpus.json");
        std::fs::write(&manifest_path, canonical_json(&manifest)?)
            .map_err(|error| format!("cannot write {}: {error}", manifest_path.display()))?;

        // --- settings locale corpus ------------------------------------------
        let settings = settings_corpus(&export)?;
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

    /// The en-US alias of a catalog entry/member.
    fn en_alias(entry: &Value) -> Result<&str, String> {
        entry
            .get("aliases")
            .and_then(|aliases| aliases.get("en-US"))
            .and_then(Value::as_str)
            .ok_or_else(|| format!("catalog entry '{}' without en-US alias", entry))
    }

    /// Canonical (sorted-key, pretty) JSON serialization, byte-idempotent.
    fn canonical_json(value: &Value) -> Result<String, String> {
        let mut out = serde_json::to_string_pretty(value)
            .map_err(|error| format!("cannot serialize JSON: {error}"))?;
        out.push('\n');
        Ok(out)
    }

    /// Merge `zh-CN` aliases into the catalog data. Existing zh-CN aliases
    /// must match the corpus; nothing else changes (data-only, ADR-0001).
    fn merge_zh_aliases(catalog: &Value, matches: &[Match]) -> Result<Value, String> {
        let mut merged = catalog.clone();
        let Some(object) = merged.as_object_mut() else {
            return Err("catalog is not an object".to_string());
        };
        let mut by_identity: HashMap<(&str, &str), &Match> = HashMap::new();
        for matched in matches {
            by_identity.insert((matched.kind.as_str(), matched.id.as_str()), matched);
        }
        for category in ["structural", "actions", "values", "events", "operators"] {
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
                    _ => unreachable!(),
                };
                if let Some(matched) = by_identity.get(&(kind, id)).copied() {
                    set_zh_alias(entry, &matched.zh)?;
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
                        set_zh_alias(member, &matched.zh)?;
                    }
                }
            }
        }
        Ok(merged)
    }

    /// Set (or extend) the reviewed zh-CN aliases of one catalog entry.
    fn set_zh_alias(entry: &mut Value, zh: &str) -> Result<(), String> {
        let Some(aliases) = entry.get_mut("aliases").and_then(Value::as_object_mut) else {
            return Err("catalog entry without aliases object".to_string());
        };
        match aliases.remove("zh-CN") {
            Some(Value::String(existing)) if existing == zh => {
                aliases.insert("zh-CN".to_string(), Value::String(existing));
            }
            Some(Value::String(existing)) => {
                aliases.insert(
                    "zh-CN".to_string(),
                    Value::Array(vec![Value::String(existing), Value::String(zh.to_string())]),
                );
            }
            Some(Value::Array(existing)) => {
                let mut existing = existing;
                if !existing.iter().any(|alias| alias.as_str() == Some(zh)) {
                    existing.push(Value::String(zh.to_string()));
                }
                aliases.insert("zh-CN".to_string(), Value::Array(existing));
            }
            Some(existing) => {
                return Err(format!(
                    "catalog declares invalid zh-CN aliases value {existing}"
                ));
            }
            None => {
                aliases.insert("zh-CN".to_string(), Value::String(zh.to_string()));
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
    ) -> Value {
        let mut matches_json: Vec<Value> = matches
            .iter()
            .map(|m| {
                serde_json::json!({
                    "kind": m.kind,
                    "id": m.id,
                    "en-US": m.en,
                    "zh-CN": m.zh,
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
            "schemaVersion": 1,
            "locale": "zh-CN",
            "generator": "workshop-catalog-gen corpus",
            "generatorVersion": env!("CARGO_PKG_VERSION"),
            "source": {
                "export": meta.get("commit").and_then(Value::as_str).map(|_| "workshop-data.json").unwrap_or("<unknown>"),
                "commit": meta.get("commit").and_then(Value::as_str).unwrap_or("<unknown>"),
                "commitDate": meta.get("commitDate").and_then(Value::as_str).unwrap_or("<unknown>"),
                "fetchedAt": meta.get("fetchedAt").and_then(Value::as_str).unwrap_or("<unknown>"),
            },
            "method": "exact en-US spelling match between the catalog aliases and the export's localized index (actions/values/events/operators/constants/event filters/maps/heroes), plus confirmed legacy identity/GUID mappings for global stop-chasing, force hero/throttle, Set Player Allowed Heroes, and bare comparison-symbol entries; zh-CN is taken from the same export entry; entries without an accepted match, or whose export candidates disagree on zh-CN, are excluded with a recorded reason and keep fail-explicit behavior (ADR-0001 Decision 7)",
            "sourceReview": "reviewed: workshop-rs commits its own mapping data; the user-provided JSON is build input only and is not redistributed",
            "coverage": Value::Object(coverage_all),
            "matches": matches_json,
            "excluded": excluded_json,
        })
    }

    /// The settings locale corpus for the declared settings surface.
    fn settings_corpus(export: &Value) -> Result<Value, String> {
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
        let gamemodes = localized_index(export, &["gamemodes.", "customGameSettings.gamemodes."]);
        let maps = localized_index(export, &["maps."]);
        let heroes = localized_index(export, &["heroes."]);
        let teams = localized_index(
            export,
            &["heroes.teams.", "customGameSettings.heroes.teams."],
        );
        let tokens = localized_index(export, &["other.customGameSettings."]);
        let surface = settings_surface();

        let mut sections: Vec<SettingsSection<'_>> = vec![
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
                            serde_json::json!({
                                "en-US": en,
                                "zh-CN": candidates[0].zh_cn,
                                "sources": candidates.iter().map(|c| c.key.clone()).collect::<Vec<_>>(),
                            }),
                        );
                    }
                    Err(reason) => {
                        match confirmed_settings_identity_match(export, surface_id, en) {
                            Some(candidates) => {
                                matched += 1;
                                entries.insert(
                                    en.clone(),
                                    serde_json::json!({
                                        "en-US": en,
                                        "zh-CN": candidates[0].zh_cn,
                                        "sources": candidates.iter().map(|c| c.key.clone()).collect::<Vec<_>>(),
                                    }),
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

        let meta = export.get("meta").cloned().unwrap_or(Value::Null);
        // Split the flat matched entries into per-section maps mirroring the
        // declared settings surface.
        let mut labels = Map::new();
        let mut modes = Map::new();
        let mut maps_out = Map::new();
        let mut heroes_out = Map::new();
        let mut teams_out = Map::new();
        let mut enums_out = Map::new();
        let mut tokens_out = Map::new();
        for (section, surface_entries, _) in &sections {
            let target = match *section {
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
            "schemaVersion": 1,
            "locale": "zh-CN",
            "provenance": {
                "generator": "workshop-catalog-gen corpus",
                "generatorVersion": env!("CARGO_PKG_VERSION"),
                "source": "user-provided workshop-data export (workshop-data.json)",
                "commit": meta.get("commit").and_then(Value::as_str).unwrap_or("<unknown>"),
                "commitDate": meta.get("commitDate").and_then(Value::as_str).unwrap_or("<unknown>"),
                "fetchedAt": meta.get("fetchedAt").and_then(Value::as_str).unwrap_or("<unknown>"),
                "method": "exact en-US spelling match between the declared settings surface (settings::table) and the export's customGameSettings/gamemodes/maps/heroes labels and other.customGameSettings tokens; the two hero Ultimate Generation labels are composed only from exact export template and Blizzard hero identity/GUID matches; entries without an accepted match keep fail-explicit behavior (ADR-0001 Decision 7); the mode-header 'disabled' prefix maps the export's __disabled__ token and follows the fixture-evidenced en-US emission format",
                "sourceReview": "reviewed: workshop-rs commits its own settings mapping data; the user-provided JSON is build input only and is not redistributed",
            },
            "labels": labels,
            "modes": modes,
            "maps": maps_out,
            "heroes": heroes_out,
            "teams": teams_out,
            "enums": enums_out,
            "tokens": tokens_out,
            "excluded": excluded,
            "coverage": coverage,
        }))
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
        let hero_zh = hero_translations.get("zh-CN")?.as_str()?;
        let zh = template_zh.replace("%1$s", hero_zh);
        Some(vec![
            Candidate {
                key: template_key.to_string(),
                zh_cn: zh.clone(),
            },
            Candidate {
                key: hero_key.to_string(),
                zh_cn: hero_zh.to_string(),
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
            "corpus: zh-CN matched {total_matched}/{total_entries} canonical entries and enum members"
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
        use super::set_zh_alias;
        use serde_json::json;

        #[test]
        fn corpus_merge_preserves_and_extends_reviewed_alias_arrays() {
            let mut entry = json!({
                "aliases": {"zh-CN": ["中止", "中断"]}
            });

            set_zh_alias(&mut entry, "中断").expect("existing reviewed alias is accepted");
            set_zh_alias(&mut entry, "中止条件").expect("new corpus alias is appended");

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

            set_zh_alias(&mut entry, "中断").expect("conflicting corpus alias is retained");

            assert_eq!(entry["aliases"]["zh-CN"], json!(["中止", "中断"]));
        }
    }
}
