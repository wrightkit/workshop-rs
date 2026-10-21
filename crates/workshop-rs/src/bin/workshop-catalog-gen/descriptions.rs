use serde_json::{Map, Value};

use super::*;

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

const GAPPED_SOURCE: &[(&str, Option<usize>)] = &[("customString", None)];

const ZH_REVIEWED_TEXT: &[(&str, Option<usize>, &str)] = &[
    (
        "isObjectiveComplete",
        None,
        "指定目标是否已经完成。如果游戏模式不是攻防作战、护送或攻击/护送，则结果为“假”。",
    ),
    (
        "isTeamOnDefense",
        None,
        "指定队伍是否是防守方。如果游戏模式不是攻防作战、护送或攻击/护送，则结果为“假”。",
    ),
    (
        "mappedArray",
        Some(1),
        "映射表达式将此数组中的每个元素赋值为被复制的数组中的元素值。使用“当前数组元素值”可以指定当前考察的数组中元素的值。",
    ),
];

const ZH_USE_SECONDARY: &[(&str, Option<usize>)] = &[
    // The primary record describes the kill feed ("消灭提示") here instead of the
    // scoreboard.
    ("disableScoreboard", None),
    // The primary record reduces the comparison result to equality only.
    ("compare", None),
    // The primary record's text ends with a stray "." artifact.
    ("and", None),
    // The primary record's "返回输入的布尔的相反值" drops the false-equivalence clause
    // the en-US text carries.
    ("not", None),
    // The primary record says "向左" (left) where the en-US text says "upward".
    ("__yComponentOf__", None),
    // The primary record says "向左" (left) where the en-US text says "forward".
    ("__zComponentOf__", None),
    // The primary record's "同进" is a typo of "同时" (simultaneously).
    ("getFacingDirection", None),
    // The primary record describes horizontal speed where the en-US text describes
    // vertical speed.
    ("getVerticalSpeed", None),
    // The primary record describes the player closest to the reticle where the en-US
    // text describes players inside the view angle.
    ("getPlayersInViewAngle", None),
    // The primary record says "输出英雄" (damage heroes) where the en-US text says
    // support heroes.
    ("allSupportHeroes", None),
    // The primary record's text is truncated mid-sentence before the "-1" result.
    ("indexOfArrayValue", None),
    // The primary record uses "数列" (sequence) where every other zh-CN text and the
    // catalog's zh-CN names use "数组" (array).
    ("getPlayersOnHero", None),
    ("removeFromArray", Some(0)),
    ("removeFromArray", Some(1)),
    // The primary record's "赋值" clause reads as assignment where the en-US text
    // describes the value being evaluated.
    ("currentArrayElement", None),
    // The primary record mixes 弧度/度 where the en-US text fixes radians.
    ("tan", None),
    ("tan", Some(0)),
    ("sin", Some(0)),
    // The primary record describes the healing amount where the en-US text describes
    // the damage amount.
    ("damage", Some(2)),
    // The primary record describes choosing heroes from a list where the en-US text
    // describes preloading heroes.
    ("preloadHero", Some(1)),
    // The primary record drops "最大" (max) from the ammo amount the en-US text names.
    ("setMaxAmmo", Some(1)),
    // The primary record narrows spectators to teamless spectators where the en-US
    // text says spectators.
    ("createHudText", Some(10)),
    // The primary record says "向后" (backward) where the en-US text says sideways.
    ("startForcingThrottle", Some(5)),
    // The primary record's "将家" is a typo of "玩家" (player).
    ("playEffect", Some(0)),
    ("createEffect", Some(0)),
    // The primary record's text belongs to input-direction/eye-position respectively —
    // the two records' parameter texts are transposed.
    ("nearestWalkablePosition", Some(0)),
    ("getEyePosition", Some(0)),
    // The primary record says "最近" (nearest) where the en-US text says farthest.
    ("getFarthestPlayer", Some(1)),
    // The primary record swaps the horizontal/vertical angle directions.
    ("horizontalAngleFromDirection", Some(0)),
    ("verticalAngleFromDirection", Some(0)),
    // The primary record reads the angle endpoint generically where the en-US text
    // names the angle's ending position.
    ("verticalAngleTowards", Some(1)),
    // The primary record drops "水平" (horizontal) from the facing angle.
    ("getHorizontalFacingAngle", Some(0)),
];

const ZH_PRIMARY_PARAM_MAP: &[(&str, &[Option<usize>])] = &[
    // The primary record lists the two operand descriptions under the shared 值
    // label; the middle `comparison` position has no source text.
    ("compare", &[Some(0), None, Some(1)]),
    // The primary record's single parameter record carries the communication-type
    // text under a 玩家 label; the player position has no source text.
    ("isCommunicating", &[None, Some(0)]),
];

const ZH_SECONDARY_PARAM_MAP: &[(&str, &[Option<usize>])] = &[
    ("compare", &[Some(0), None, Some(1)]),
    // The secondary record's single parameter text is a generic "function input" note
    // that cannot be attributed to numerator or denominator specifically.
    ("atan2", &[None, None]),
    ("atan2Deg", &[None, None]),
    // The secondary record omits the trailing reevaluation position.
    ("addHealthPool", &[Some(0), Some(1), Some(2), Some(3), None]),
    (
        "createBeamEffect",
        &[Some(0), Some(1), Some(2), None, Some(3), Some(4)],
    ),
    (
        "createWorkshopSettingFloat",
        &[Some(0), Some(1), Some(2), Some(3), Some(4), None],
    ),
    (
        "workshopSettingInteger",
        &[Some(0), Some(1), Some(2), Some(3), Some(4), None],
    ),
    ("workshopSettingToggle", &[Some(0), Some(1), Some(2), None]),
    (
        "startForcingOutlineFor",
        &[Some(0), Some(1), Some(2), Some(3), None],
    ),
    // The secondary record omits the in-world text color position and names the
    // spectator position last.
    (
        "createInWorldText",
        &[
            Some(0),
            Some(1),
            Some(2),
            Some(3),
            Some(4),
            Some(5),
            None,
            Some(6),
        ],
    ),
    // The secondary record documents only the team and view-angle positions.
    ("getPlayersInViewAngle", &[None, Some(0), Some(1)]),
    // The secondary record omits the barriers position.
    ("isInLoS", &[Some(0), Some(1), None]),
    // The secondary record documents only the start and end positions.
    ("raycastHitNormal", &[Some(0), Some(1), None, None, None]),
    ("raycastHitPlayer", &[Some(0), Some(1), None, None, None]),
    ("raycastHitPosition", &[Some(0), Some(1), None, None, None]),
    // The secondary record documents only the rounding-direction position.
    ("roundToInteger", &[None, Some(0)]),
];

pub(crate) const DEFAULT_ZH_SOURCE: &str = "tools/descriptions/zh-source.json";

pub(crate) fn generate(
    catalog_data: &str,
    out_path: &Path,
    export_path: &Path,
    zh_path: &Path,
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
    let zh_source = load_zh_source(zh_path)?;

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
    let mut used_zh_rules = std::collections::HashSet::new();
    let mut zh_reviewed = |id: &str, position: Option<usize>| -> Option<&'static str> {
        ZH_REVIEWED_TEXT
            .iter()
            .find(|(rule_id, rule_position, _)| rule_id == &id && rule_position == &position)
            .map(|(rule_id, rule_position, text)| {
                used_zh_rules.insert((*rule_id, *rule_position));
                *text
            })
    };
    let mut used_secondary_rules: std::collections::HashSet<(&'static str, Option<usize>)> =
        std::collections::HashSet::new();
    let mut zh_disagreements: Vec<String> = Vec::new();
    let mut zh_resolve = |id: &str,
                          position: Option<usize>,
                          primary: Option<&str>,
                          secondary: Option<&str>|
     -> Option<String> {
        if let Some(text) = zh_reviewed(id, position) {
            return Some(text.to_string());
        }
        let primary_text = primary.map(clean_zh_source).filter(|text| !text.is_empty());
        let secondary_text = secondary
            .map(clean_zh_source)
            .filter(|text| !text.is_empty());
        match (primary_text, secondary_text) {
            (Some(primary), Some(secondary)) => {
                if canon_zh_text(&primary) != canon_zh_text(&secondary) {
                    if let Some(rule) = ZH_USE_SECONDARY.iter().find(|(rule_id, rule_position)| {
                        rule_id == &id && rule_position == &position
                    }) {
                        used_secondary_rules.insert((rule.0, rule.1));
                        zh_disagreements.push(format!(
                            "{id}{}: sources disagree, the secondary record arbitrates per ZH_USE_SECONDARY",
                            position.map_or_else(String::new, |p| format!(".params[{p}]"))
                        ));
                        return Some(secondary);
                    }
                    zh_disagreements.push(format!(
                        "{id}{}: sources disagree, the primary record arbitrates (default)",
                        position.map_or_else(String::new, |p| format!(".params[{p}]"))
                    ));
                }
                Some(primary)
            }
            (Some(primary), None) => Some(primary),
            (None, Some(secondary)) => Some(secondary),
            (None, None) => None,
        }
    };

    let mut entries_actions = Map::new();
    let mut entries_values = Map::new();
    let mut gaps: Vec<String> = Vec::new();
    let mut zh_gaps: Vec<String> = Vec::new();
    let mut report = Vec::new();
    let mut described = 0usize;
    let mut param_described = 0usize;
    let mut zh_described = 0usize;
    let mut zh_param_described = 0usize;
    let mut unmatched: Vec<String> = Vec::new();
    let mut zh_unmatched: Vec<String> = Vec::new();

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
            let en = first_alias(entry, "en-US");
            let zh_name = first_alias(entry, "zh-CN");
            let params_len = entry
                .get("params")
                .and_then(Value::as_array)
                .map_or(0, Vec::len);

            let primary_record = zh_source.primary_by_name.get(&normalize_zh_name(&zh_name));
            let secondary_record = zh_source.secondary_by_name.get(&normalize_en_name(&en));
            if primary_record.is_none() && secondary_record.is_none() {
                zh_unmatched.push(format!("{id} ({zh_name})"));
                zh_gaps.push(id.clone());
                zh_gaps.extend((0..params_len).map(|position| format!("{id}.params[{position}]")));
            }
            let primary_params = match primary_record {
                Some(record) => zh_param_indexes(
                    &id,
                    params_len,
                    record.params.len(),
                    ZH_PRIMARY_PARAM_MAP,
                    "primary",
                )?,
                None => vec![None; params_len],
            };
            let secondary_params = match secondary_record {
                Some(record) => zh_param_indexes(
                    &id,
                    params_len,
                    record.params.len(),
                    ZH_SECONDARY_PARAM_MAP,
                    "secondary",
                )?,
                None => vec![None; params_len],
            };

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
            let gapped = |position: Option<usize>| {
                GAPPED_SOURCE
                    .iter()
                    .any(|(rule_id, rule_position)| rule_id == &id && rule_position == &position)
            };
            let mut record = Map::new();
            let mut has_text = false;

            let en_description = source.and_then(|source| {
                if gapped(None) {
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
                }
            });
            let zh_description = zh_resolve(
                &id,
                None,
                primary_record.and_then(|record| record.description.as_deref()),
                secondary_record.and_then(|record| record.description.as_deref()),
            );
            let mut description = Map::new();
            match en_description {
                Some(text) => {
                    has_text = true;
                    described += 1;
                    description.insert("en-US".to_string(), Value::String(text));
                }
                None => gaps.push(id.clone()),
            }
            match zh_description {
                Some(text) => {
                    has_text = true;
                    zh_described += 1;
                    description.insert("zh-CN".to_string(), Value::String(text));
                }
                None if primary_record.is_some() || secondary_record.is_some() => {
                    zh_gaps.push(id.clone())
                }
                None => {}
            }
            if !description.is_empty() {
                record.insert("description".to_string(), Value::Object(description));
            }
            if source.is_none() {
                unmatched.push(format!("{id} ({en})"));
            }

            let args: Vec<&Value> = source
                .and_then(|source| source.get("args"))
                .and_then(Value::as_array)
                .map(|args| args.iter().collect())
                .unwrap_or_default();
            if params_len > 0 {
                if source.is_some() && args.len() != params_len {
                    return Err(format!(
                        "{id}: catalog declares {params_len} params but the export has {} args",
                        args.len()
                    ));
                }
                let params: Vec<Value> = (0..params_len)
                    .map(|position| {
                        let mut texts = Map::new();
                        let en_text = source.and_then(|_| {
                            if gapped(Some(position)) {
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
                            }
                        });
                        match en_text {
                            Some(text) => {
                                has_text = true;
                                param_described += 1;
                                texts.insert("en-US".to_string(), Value::String(text));
                            }
                            None => gaps.push(format!("{id}.params[{position}]")),
                        }
                        let primary_text = primary_params[position].map(|index| {
                            primary_record
                                .expect("mapped indexes imply a record")
                                .params[index]
                                .description
                                .as_str()
                        });
                        let secondary_text = secondary_params[position].map(|index| {
                            secondary_record
                                .expect("mapped indexes imply a record")
                                .params[index]
                                .description
                                .as_str()
                        });
                        match zh_resolve(&id, Some(position), primary_text, secondary_text) {
                            Some(text) => {
                                has_text = true;
                                zh_param_described += 1;
                                texts.insert("zh-CN".to_string(), Value::String(text));
                            }
                            None if primary_record.is_some() || secondary_record.is_some() => {
                                zh_gaps.push(format!("{id}.params[{position}]"))
                            }
                            None => {}
                        }
                        if texts.is_empty() {
                            Value::Null
                        } else {
                            Value::Object(texts)
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
    for (rule_id, position, _) in ZH_REVIEWED_TEXT {
        if !used_zh_rules.contains(&(*rule_id, *position)) {
            return Err(format!(
                "stale zh-CN reviewed text for '{rule_id}'{}: no source text was replaced",
                position.map_or_else(String::new, |p| format!(" params[{p}]"))
            ));
        }
    }
    for (rule_id, position) in ZH_USE_SECONDARY {
        if !used_secondary_rules.contains(&(*rule_id, *position)) {
            return Err(format!(
                "stale secondary arbitration for '{rule_id}'{}: \
                 no disagreement was arbitrated",
                position.map_or_else(String::new, |p| format!(" params[{p}]"))
            ));
        }
    }
    gaps.sort();
    gaps.dedup();
    zh_gaps.sort();
    zh_gaps.dedup();
    check_stale_param_maps(&zh_source, &catalog_json)?;
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
        "locales": ["en-US", "zh-CN"],
        "pendingLocales": [],
        "provenance": {
            "generator": "workshop-catalog-gen",
            "generatorVersion": env!("CARGO_PKG_VERSION"),
            "source": format!(
                "en-US action/value and parameter descriptions imported from the pinned \
                 workshop-data export at commit {commit} (in-game Workshop documentation \
                 text); reviewed reductions for source text violating the content rules are \
                 recorded in the importer's REVIEWED_TEXT table; zh-CN action/value and \
                 parameter descriptions cross-checked from the two records of the zh-CN \
                 source artifact; source disagreements arbitrate to the zh-name-keyed \
                 record except the importer's ZH_USE_SECONDARY table, cleaned texts are \
                 recorded verbatim and uncovered positions are explicit gaps"
            ),
            "license": "MIT (in-game Workshop documentation text and community zh-CN reference text; provenance in docs/source-attribution/catalog.md)",
            "reviewed": true,
        },
        "entries": Value::Object(entries),
        "gaps": { "en-US": gaps, "zh-CN": zh_gaps },
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
        "wrote {}: {described} en-US entry descriptions, {param_described} en-US parameter descriptions, {} en-US gaps; \
         {zh_described} zh-CN entry descriptions, {zh_param_described} zh-CN parameter descriptions, {} zh-CN gaps",
        out_path.display(),
        gaps.len(),
        zh_gaps.len()
    ));
    for gap in &gaps {
        report.push(format!("gap: {gap}"));
    }
    for gap in &zh_gaps {
        report.push(format!("zh-CN gap: {gap}"));
    }
    for line in &zh_disagreements {
        report.push(format!("zh-CN source {line}"));
    }
    for missing in &unmatched {
        report.push(format!("unmatched in export: {missing}"));
    }
    for missing in &zh_unmatched {
        report.push(format!("unmatched in zh-CN sources: {missing}"));
    }
    Ok(report)
}

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
struct ZhSource {
    primary_by_name: HashMap<String, ZhRecord>,
    secondary_by_name: HashMap<String, ZhRecord>,
}
struct ZhRecord {
    description: Option<String>,
    params: Vec<ZhParam>,
}

struct ZhParam {
    description: String,
}
fn load_zh_source(path: &Path) -> Result<ZhSource, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|error| format!("cannot read zh-CN source {}: {error}", path.display()))?;
    let document: Value = serde_json::from_str(&text)
        .map_err(|error| format!("cannot parse zh-CN source {}: {error}", path.display()))?;
    let records =
        |name: &str, normalize: fn(&str) -> String| -> Result<HashMap<String, ZhRecord>, String> {
            let mut by_name = HashMap::new();
            let Some(table) = document.get(name).and_then(Value::as_object) else {
                return Err(format!(
                    "zh-CN source {} lacks a '{name}' table",
                    path.display()
                ));
            };
            for (key, record) in table {
                let description = record
                    .get("description")
                    .and_then(Value::as_str)
                    .map(str::to_string);
                let params = record
                    .get("params")
                    .and_then(Value::as_array)
                    .map(|params| {
                        params
                            .iter()
                            .map(|param| ZhParam {
                                description: param
                                    .get("description")
                                    .and_then(Value::as_str)
                                    .unwrap_or_default()
                                    .to_string(),
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                if by_name
                    .insert(
                        normalize(key),
                        ZhRecord {
                            description,
                            params,
                        },
                    )
                    .is_some()
                {
                    return Err(format!(
                        "zh-CN source '{name}' key '{key}' collides after normalization"
                    ));
                }
            }
            Ok(by_name)
        };
    Ok(ZhSource {
        primary_by_name: records("byZhName", normalize_zh_name)?,
        secondary_by_name: records("byEnName", normalize_en_name)?,
    })
}
fn zh_param_indexes(
    id: &str,
    params_len: usize,
    source_params_len: usize,
    map: &[(&str, &[Option<usize>])],
    source_name: &str,
) -> Result<Vec<Option<usize>>, String> {
    if source_params_len == 0 {
        return Ok(vec![None; params_len]);
    }
    if let Some((_, indexes)) = map.iter().find(|(rule_id, _)| rule_id == &id) {
        if indexes.len() != params_len {
            return Err(format!(
                "{id}: {source_name} parameter map lists {} positions for {params_len} catalog params",
                indexes.len()
            ));
        }
        for index in indexes.iter().flatten() {
            if *index >= source_params_len {
                return Err(format!(
                    "{id}: {source_name} parameter map names index {index} for {source_params_len} source params"
                ));
            }
        }
        return Ok(indexes.to_vec());
    }
    if source_params_len != params_len {
        return Err(format!(
            "{id}: catalog declares {params_len} params but {source_name} lists {source_params_len} — \
             add an explicit ZH_*_PARAM_MAP entry"
        ));
    }
    Ok((0..params_len).map(Some).collect())
}
fn check_stale_param_maps(source: &ZhSource, catalog_json: &Value) -> Result<(), String> {
    let catalog_entry = |id: &str| -> Option<&Value> {
        ["actions", "values"].iter().find_map(|section| {
            catalog_json
                .get(*section)
                .and_then(Value::as_array)?
                .iter()
                .find(|entry| entry.get("id").and_then(Value::as_str) == Some(id))
        })
    };
    let check = |by_name: &HashMap<String, ZhRecord>,
                 key_of: fn(&Value) -> String,
                 map: &[(&str, &[Option<usize>])],
                 source_name: &str|
     -> Result<(), String> {
        for (id, _) in map {
            let entry = catalog_entry(id).ok_or_else(|| {
                format!("stale {source_name} parameter map for '{id}': no such catalog entry")
            })?;
            let params_len = entry
                .get("params")
                .and_then(Value::as_array)
                .map_or(0, Vec::len);
            let Some(record) = by_name.get(&key_of(entry)) else {
                return Err(format!(
                    "stale {source_name} parameter map for '{id}': source record is absent"
                ));
            };
            if record.params.len() == params_len {
                return Err(format!(
                    "stale {source_name} parameter map for '{id}': source now aligns positionally"
                ));
            }
        }
        Ok(())
    };
    check(
        &source.primary_by_name,
        |entry| normalize_zh_name(&first_alias(entry, "zh-CN")),
        ZH_PRIMARY_PARAM_MAP,
        "primary",
    )?;
    check(
        &source.secondary_by_name,
        |entry| normalize_en_name(&first_alias(entry, "en-US")),
        ZH_SECONDARY_PARAM_MAP,
        "secondary",
    )?;
    Ok(())
}
fn first_alias(entry: &Value, locale: &str) -> String {
    entry
        .get("aliases")
        .and_then(|aliases| aliases.get(locale))
        .map(|alias| match alias {
            Value::String(spelling) => spelling.clone(),
            Value::Array(list) => list
                .first()
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
            _ => String::new(),
        })
        .unwrap_or_default()
}
fn normalize_zh_name(name: &str) -> String {
    name.chars().filter(|c| !c.is_whitespace()).collect()
}
fn normalize_en_name(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}
fn canon_zh_text(text: &str) -> String {
    text.chars()
        .flat_map(char::to_lowercase)
        .filter(|c| c.is_alphanumeric())
        .collect()
}
fn clean_zh_source(text: &str) -> String {
    let mut unstruck = String::with_capacity(text.len());
    let mut keep = true;
    for part in text.split("~~") {
        if keep {
            unstruck.push_str(part);
        }
        keep = !keep;
    }
    let mut repaired = String::with_capacity(unstruck.len());
    let mut inside = false;
    for c in unstruck.chars() {
        match c {
            '“' => {
                if inside {
                    // A '“' inside a span closes a swapped-open `”`.
                    repaired.push('”');
                    inside = false;
                } else {
                    repaired.push('“');
                    inside = true;
                }
            }
            '”' => {
                if inside {
                    repaired.push('”');
                    inside = false;
                } else {
                    // A '”' outside a span is a swapped-open quote.
                    repaired.push('“');
                    inside = true;
                }
            }
            _ => repaired.push(c),
        }
    }
    let repaired = repaired
        .replace('*', "")
        .replace("(黑人问号???)", "")
        .replace("（黑人问号???）", "");
    let collapsed = repaired.split_whitespace().collect::<Vec<_>>().join(" ");
    let collapsed = collapsed.trim_start_matches([':', '：']).trim().to_string();
    if matches!(collapsed.as_str(), "" | "无" | "无描述") {
        String::new()
    } else {
        collapsed
    }
}
