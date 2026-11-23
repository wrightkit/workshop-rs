use std::collections::HashSet;
use std::sync::OnceLock;

use serde::Deserialize;

use super::table::{self, KeyKind};
use super::{NumericBounds, SettingDefinition};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct InputBound {
    path: String,
    min: Option<f64>,
    max: Option<f64>,
}

fn records() -> &'static [InputBound] {
    static BOUNDS: OnceLock<Vec<InputBound>> = OnceLock::new();
    BOUNDS.get_or_init(|| {
        serde_json::from_str(include_str!("data/input_bounds.json"))
            .expect("settings input bounds are valid JSON")
    })
}

impl SettingDefinition {
    /// Inclusive client import bounds, when established by a reviewed source.
    pub fn input_bounds(&self) -> Option<NumericBounds> {
        input_bounds(self.path())
    }
}

fn input_bounds(path: &str) -> Option<NumericBounds> {
    records()
        .iter()
        .find(|record| record.path == path)
        .map(|record| {
            NumericBounds::new(record.min, record.max).expect("reviewed input bounds are valid")
        })
}

pub(super) fn violation(path: &str, value: f64) -> Option<String> {
    bound_violation(path, value, input_bounds(path)?)
}

fn bound_violation(path: &str, value: f64, bounds: NumericBounds) -> Option<String> {
    if let Some(min) = bounds.min().filter(|min| value < *min) {
        return Some(format!(
            "setting '{path}' value {value} is below minimum {min}"
        ));
    }
    if let Some(max) = bounds.max().filter(|max| value > *max) {
        return Some(format!(
            "setting '{path}' value {value} is above maximum {max}"
        ));
    }
    None
}

pub(super) fn validate() -> Vec<String> {
    validate_records(records())
}

fn validate_records(records: &[InputBound]) -> Vec<String> {
    let mut errors = Vec::new();
    let mut paths = HashSet::new();
    for record in records {
        if !paths.insert(&record.path) {
            errors.push(format!("duplicate settings input bounds: {}", record.path));
        }
        match table::entries().find(|entry| table::path_string(entry.path) == record.path) {
            Some(entry) if matches!(entry.kind, KeyKind::Number | KeyKind::Percent) => {}
            Some(_) => errors.push(format!("nonnumeric settings input bounds: {}", record.path)),
            None => errors.push(format!(
                "unknown settings input bounds path: {}",
                record.path
            )),
        }
        if record.min.is_none() && record.max.is_none()
            || NumericBounds::new(record.min, record.max).is_err()
        {
            errors.push(format!("invalid settings input bounds: {}", record.path));
        }
    }
    errors
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_bounds_include_both_endpoints() {
        let bounds = NumericBounds::new(Some(50.0), Some(200.0)).unwrap();
        assert!(
            bound_violation("example", 49.9, bounds)
                .unwrap()
                .contains("50")
        );
        assert!(bound_violation("example", 50.0, bounds).is_none());
        assert!(bound_violation("example", 200.0, bounds).is_none());
        assert!(
            bound_violation("example", 200.1, bounds)
                .unwrap()
                .contains("200")
        );
    }

    #[test]
    fn input_bounds_require_unique_numeric_paths_and_valid_ranges() {
        let record = |path: &str, min, max| InputBound {
            path: path.to_string(),
            min,
            max,
        };
        let errors = validate_records(&[
            record("heroes.<team>.<hero>.movementSpeed%", Some(50.0), None),
            record("heroes.<team>.<hero>.movementSpeed%", Some(60.0), None),
            record("lobby.mapRotation", Some(0.0), None),
            record("lobby.missing", Some(0.0), None),
            record("lobby.spectatorSlots", Some(10.0), Some(5.0)),
        ]);
        for message in ["duplicate", "nonnumeric", "unknown", "invalid"] {
            assert!(
                errors.iter().any(|error| error.contains(message)),
                "{errors:?}"
            );
        }
    }
}
