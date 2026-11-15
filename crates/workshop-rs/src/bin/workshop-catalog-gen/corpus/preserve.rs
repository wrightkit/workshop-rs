use serde_json::{Map, Value};

pub(super) fn preserve_existing_settings(entries: &mut Map<String, Value>, previous: &Value) {
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
        let Some(current_entry) = entries.get_mut(&name) else {
            continue;
        };
        let (Some(current), Some(previous)) =
            (current_entry.as_object_mut(), previous_entry.as_object())
        else {
            continue;
        };
        fill_locale_gaps(current, previous);
    }
}

fn fill_locale_gaps(
    current: &mut serde_json::Map<String, Value>,
    previous: &serde_json::Map<String, Value>,
) {
    for locale in ["en-US", "zh-CN"] {
        if let Some(value) = previous.get(locale).and_then(Value::as_str) {
            current
                .entry(locale.to_string())
                .or_insert_with(|| Value::String(value.to_string()));
        }
    }
}
