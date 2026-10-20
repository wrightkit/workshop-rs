//! `descriptions` — imports en-US action/value descriptions from the pinned
//! `workshop-data` export into `descriptions.json`.
//!
//! The export's `data.actions`/`data.values` entries carry an en-US
//! `description` map on the entry and on each `args` position — the
//! in-game Workshop documentation text — matched to catalog entries by
//! their exact en-US spelling through the export's `localized` index.
//! Source text that violates the content rules (gameplay advice,
//! source-language names, markup) is reduced to the reviewed text in
//! [`REVIEWED_TEXT`]; positions with no usable source text become
//! explicit `gaps` entries.

use serde_json::{Map, Value};

use super::*;

/// Reviewed replacements for source text that violates the description
/// content rules — `(catalog id, parameter position or entry-level, text)`.
/// Each entry names the canonical Workshop surface form instead of the
/// source's source-language spelling, or drops gameplay advice and
/// markup from an otherwise factual text. Every rule must apply at
/// generation time; an unused rule reports as stale.
const REVIEWED_TEXT: &[(&str, Option<usize>, &str)] = &[
    (
        "createHudText",
        None,
        "Creates hud text visible to specific players at a specific location on the \
         screen. This text will persist until destroyed. To obtain a reference to this \
         text, use the Last Text ID value. This action will fail if too many text \
         elements have been created.",
    ),
    (
        "inputBindingString",
        None,
        "Converts a button parameter into a string that shows up based on the \
         player's input bindings. This value cannot be stored in variables.",
    ),
    (
        "getPlayersInRadius",
        None,
        "An array containing all players within a certain distance of a position, \
         optionally restricted by team and line of sight. This function picks up \
         dead players.",
    ),
    (
        "getPlayersInViewAngle",
        None,
        "The players who are within a specific view angle of a specific player's \
         reticle, optionally restricted by team. This function picks up dead and \
         unspawned players.",
    ),
    (
        "preloadHero",
        None,
        "Preemptively loads the specified hero or heroes into memory using the \
         skins of the specified player or players, available memory permitting.",
    ),
    (
        "entityExists",
        None,
        "Whether the specified player, icon entity, or effect entity still exists.",
    ),
    (
        "evaluateOnce",
        None,
        "Makes a copy of the provided value that is evaluated only once.",
    ),
    (
        "updateEveryFrame",
        None,
        "Increases the update frequency of the provided value to once per tick. \
         Applies to rule conditions as well as reevaluating action parameters. The \
         value is interpolated client-side if the framerate is higher than the tick \
         rate. May increase server load and/or lower frame rate.",
    ),
    ("max", None, "The maximum of two or more numbers."),
    ("min", None, "The minimum of two or more numbers."),
    (
        "isInAlternateForm",
        None,
        "Whether the specified player is currently in an alternate form: Wrecking \
         Ball's ball form, Baby Dva, Bastion's turret and tank forms, Lucio's \
         speed song, Mercy's pistol, or Torbjorn's hammer. For Echo duplication, \
         use the Is Duplicating value instead.",
    ),
    (
        "stringReplace",
        None,
        "Results in a String Value. This String Value will be built from the \
         specified String Value, where all occurrences of the pattern String are \
         replaced with the replacement String. The result is clamped to 511 bytes \
         (in UTF-8).",
    ),
    (
        "indexOfArrayValue",
        None,
        "The index of a value within the array or -1 if no such value can be found. \
         Does not support nested arrays. A True in the array matches any truthy \
         value, and a True search value matches any truthy value in the array.",
    ),
    (
        "setHealth",
        None,
        "Sets the health of a player or players without affecting stats or granting \
         damage/healing credit. This action only has an effect on living players. \
         For dead players, use the Resurrect action instead.",
    ),
    (
        "createIcon",
        None,
        "Creates an in-world icon entity. This icon entity will persist until \
         destroyed. To obtain a reference to this entity, use the Last Created \
         Entity value. This action will fail if too many entities have been created.",
    ),
    (
        "stopForcingPosition",
        None,
        "Cancels the behavior of the Start Forcing Player Position action for the \
         specified player or players. Regular movement will resume from the last \
         forced position or positions.",
    ),
    (
        "stopForcingName",
        None,
        "Cancels the behavior of the Start Forcing Dummy Bot Name action for the \
         specified player or players.",
    ),
    (
        "countOf",
        None,
        "The number of elements in the specified array. For strings, use String \
         Length.",
    ),
    ("break", None, "Skips to the end of the innermost loop."),
    (
        "slice",
        None,
        "A copy of the specified array containing only values from a specified \
         index range. Does not support nested arrays.",
    ),
    (
        "attachTo",
        Some(2),
        "The coordinates of the child relative to the parent. For example, \
         Vector(1, 2, 0) would be above and to the left of the parent's head.",
    ),
];

/// Source texts that are not descriptions at all and must become explicit
/// gaps rather than imported text — `(catalog id, parameter position or
/// entry-level)`. `customString`'s source description is a contributor
/// credit (`"ty magzie for adding that"`), not a description of the value.
const GAPPED_SOURCE: &[(&str, Option<usize>)] = &[("customString", None)];

/// Import en-US descriptions from the pinned `workshop-data` export and
/// write the canonical descriptions dataset (without a digest; `build`
/// records it). Returns the report lines.
pub(crate) fn generate(
    catalog_data: &str,
    out_path: &Path,
    export_path: &Path,
) -> Result<Vec<String>, String> {
    let catalog = Catalog::load(catalog_data).map_err(|error| format!("catalog data: {error}"))?;
    let catalog_json: Value =
        serde_json::from_str(catalog_data).map_err(|error| format!("catalog data: {error}"))?;
    let export_text = std::fs::read_to_string(export_path)
        .map_err(|error| format!("cannot read export {}: {error}", export_path.display()))?;
    let export: Value = serde_json::from_str(&export_text)
        .map_err(|error| format!("cannot parse export {}: {error}", export_path.display()))?;
    let commit = export
        .get("meta")
        .and_then(|meta| meta.get("commit"))
        .and_then(Value::as_str)
        .unwrap_or("<unknown>")
        .to_string();

    // en-US spelling -> "section\0export id", over the actions/values
    // localized rows.
    let mut by_en: HashMap<String, Vec<String>> = HashMap::new();
    for (key, entry) in export
        .get("localized")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default()
    {
        let Some(section) = key
            .strip_prefix("actions.")
            .map(|_| "actions")
            .or_else(|| key.strip_prefix("values.").map(|_| "values"))
        else {
            continue;
        };
        let Some(en) = entry
            .get("translations")
            .and_then(|t| t.get("en-US"))
            .and_then(Value::as_str)
        else {
            continue;
        };
        let id = entry
            .get("id")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        by_en
            .entry(en.to_string())
            .or_default()
            .push(format!("{section}\u{0}{id}"));
    }

    let mut used_rules = std::collections::HashSet::new();
    let mut reviewed = |id: &str, position: Option<usize>| -> Option<&'static str> {
        REVIEWED_TEXT
            .iter()
            .find(|(rule_id, rule_position, _)| rule_id == &id && rule_position == &position)
            .map(|(rule_id, rule_position, text)| {
                used_rules.insert((*rule_id, *rule_position));
                *text
            })
    };

    let mut entries_actions = Map::new();
    let mut entries_values = Map::new();
    let mut gaps: Vec<String> = Vec::new();
    let mut report = Vec::new();
    let mut described = 0usize;
    let mut param_described = 0usize;
    let mut unmatched: Vec<String> = Vec::new();

    for (section_key, out) in [
        ("actions", &mut entries_actions),
        ("values", &mut entries_values),
    ] {
        let Some(entries) = catalog_json.get(section_key).and_then(Value::as_array) else {
            continue;
        };
        for entry in entries {
            let id = entry
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            let en = entry
                .get("aliases")
                .and_then(|a| a.get("en-US"))
                .map(|alias| match alias {
                    Value::String(s) => s.clone(),
                    Value::Array(list) => list
                        .first()
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_string(),
                    _ => String::new(),
                })
                .unwrap_or_default();
            let candidates = by_en.get(&en).cloned().unwrap_or_default();
            if candidates.len() > 1 {
                return Err(format!(
                    "ambiguous: export gives {} candidates for '{en}' ({id})",
                    candidates.len()
                ));
            }
            let source = candidates.first().and_then(|candidate| {
                let (export_section, export_id) = candidate.split_once('\u{0}')?;
                export
                    .get("data")
                    .and_then(|data| data.get(export_section))
                    .and_then(|entries| entries.get(export_id))
            });
            let params_len = entry
                .get("params")
                .and_then(Value::as_array)
                .map_or(0, Vec::len);
            let Some(source) = source else {
                unmatched.push(format!("{id} ({en})"));
                gaps.push(id.clone());
                gaps.extend((0..params_len).map(|position| format!("{id}.params[{position}]")));
                continue;
            };
            let gapped = |position: Option<usize>| {
                GAPPED_SOURCE
                    .iter()
                    .any(|(rule_id, rule_position)| rule_id == &id && rule_position == &position)
            };
            let description = if gapped(None) {
                None
            } else {
                reviewed(&id, None).map(str::to_string).or_else(|| {
                    source
                        .get("description")
                        .and_then(|d| d.get("en-US"))
                        .and_then(Value::as_str)
                        .map(str::trim)
                        .filter(|text| !text.is_empty())
                        .map(str::to_string)
                })
            };
            let mut record = Map::new();
            let mut has_text = false;
            if let Some(description) = description {
                has_text = true;
                described += 1;
                record.insert(
                    "description".to_string(),
                    serde_json::json!({ "en-US": description }),
                );
            } else {
                gaps.push(id.clone());
            }
            let args: Vec<&Value> = source
                .get("args")
                .and_then(Value::as_array)
                .map(|args| args.iter().collect())
                .unwrap_or_default();
            if params_len > 0 {
                if args.len() != params_len {
                    return Err(format!(
                        "{id}: catalog declares {params_len} params but the export has {} args",
                        args.len()
                    ));
                }
                let params: Vec<Value> = (0..params_len)
                    .map(|position| {
                        let text = if gapped(Some(position)) {
                            None
                        } else {
                            reviewed(&id, Some(position))
                                .map(str::to_string)
                                .or_else(|| {
                                    args[position]
                                        .get("description")
                                        .and_then(|d| d.get("en-US"))
                                        .and_then(Value::as_str)
                                        .map(str::trim)
                                        .filter(|text| !text.is_empty())
                                        .map(str::to_string)
                                })
                        };
                        match text {
                            Some(text) => {
                                has_text = true;
                                param_described += 1;
                                serde_json::json!({ "en-US": text })
                            }
                            None => {
                                gaps.push(format!("{id}.params[{position}]"));
                                Value::Null
                            }
                        }
                    })
                    .collect();
                record.insert("params".to_string(), Value::Array(params));
            }
            if has_text {
                out.insert(id, Value::Object(record));
            }
        }
    }

    for (rule_id, position, _) in REVIEWED_TEXT {
        if !used_rules.contains(&(*rule_id, *position)) {
            return Err(format!(
                "stale reviewed text for '{rule_id}'{}: no source text was replaced",
                position.map_or_else(String::new, |p| format!(" params[{p}]"))
            ));
        }
    }
    gaps.sort();
    gaps.dedup();
    for (rule_id, position) in GAPPED_SOURCE {
        let key = position.map_or_else(
            || (*rule_id).to_string(),
            |p| format!("{rule_id}.params[{p}]"),
        );
        if !gaps.iter().any(|gap| gap == &key) {
            return Err(format!(
                "stale gapped source for '{key}': no gap was produced"
            ));
        }
    }

    let mut entries = Map::new();
    entries.insert("actions".to_string(), Value::Object(entries_actions));
    entries.insert("values".to_string(), Value::Object(entries_values));
    let document = serde_json::json!({
        "schemaVersion": 1,
        "version": dataset_version(out_path),
        "locales": ["en-US"],
        "pendingLocales": ["zh-CN"],
        "provenance": {
            "generator": "workshop-catalog-gen",
            "generatorVersion": env!("CARGO_PKG_VERSION"),
            "source": format!(
                "en-US action/value and parameter descriptions imported from the pinned \
                 workshop-data export at commit {commit} (in-game Workshop documentation \
                 text); reviewed reductions for source text violating the content rules are \
                 recorded in the importer's REVIEWED_TEXT table; zh-CN is a pending stage",
            ),
            "license": "MIT (in-game Workshop documentation text; provenance in docs/source-attribution/catalog.md)",
            "reviewed": true,
        },
        "entries": Value::Object(entries),
        "gaps": { "en-US": gaps },
    });

    let output = workshop_rs::catalog::descriptions::canonicalize_descriptions(
        &catalog,
        &serde_json::to_string_pretty(&document)
            .map_err(|error| format!("cannot serialize descriptions: {error}"))?,
    )
    .map_err(|error| format!("descriptions data: {error}"))?;
    std::fs::write(out_path, output)
        .map_err(|error| format!("cannot write {}: {error}", out_path.display()))?;

    report.push(format!(
        "wrote {}: {described} entry descriptions, {param_described} parameter descriptions, {} gaps",
        out_path.display(),
        gaps.len()
    ));
    for gap in &gaps {
        report.push(format!("gap: {gap}"));
    }
    for missing in &unmatched {
        report.push(format!("unmatched in export: {missing}"));
    }
    Ok(report)
}

/// The dataset version to write: keep the existing file's version across
/// regenerations so a deliberate version bump survives the pipeline;
/// `"0.1.0"` when the file does not declare one yet.
fn dataset_version(out_path: &Path) -> String {
    std::fs::read_to_string(out_path)
        .ok()
        .and_then(|text| serde_json::from_str::<Value>(&text).ok())
        .and_then(|document| {
            document
                .get("version")
                .and_then(Value::as_str)
                .map(str::to_string)
        })
        .unwrap_or_else(|| "0.1.0".to_string())
}
