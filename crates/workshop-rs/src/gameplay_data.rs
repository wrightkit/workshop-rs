//! Embedded, provenance-pinned hero gameplay data.
//!
//! The data file is a checked-in projection of the user-provided
//! `workshop-data` export. This module owns parsing, schema validation, and
//! dataset-digest verification; query and calculation APIs remain separate.

use sha2::{Digest, Sha256};
use std::sync::OnceLock;

use crate::gameplay::{GameplayCatalog, GameplayDataError, GameplayDatasetIdentity, Hero};

/// The canonical gameplay dataset embedded in the crate.
pub const GAMEPLAY_DATA: &str = include_str!("data/gameplay.json");

const SCHEMA_VERSION: u32 = 1;

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct GameplayFile {
    schema_version: u32,
    identity: GameplayDatasetIdentity,
    heroes: Vec<Hero>,
}

/// Load and validate a gameplay dataset from its JSON representation.
pub fn load(json: &str) -> Result<GameplayCatalog, GameplayDataError> {
    let file: GameplayFile = serde_json::from_str(json)
        .map_err(|error| GameplayDataError::Malformed(error.to_string()))?;
    if file.schema_version != SCHEMA_VERSION {
        return Err(GameplayDataError::UnsupportedSchema(file.schema_version));
    }

    let computed = content_digest(json)?;
    if file.identity.digest != computed {
        return Err(GameplayDataError::DigestMismatch {
            declared: file.identity.digest,
            computed,
        });
    }

    GameplayCatalog::new(file.identity, file.heroes)
}

/// Load the checked-in gameplay dataset.
pub fn builtin() -> Result<GameplayCatalog, GameplayDataError> {
    builtin_ref().cloned().map_err(Clone::clone)
}

/// Borrow the cached checked-in gameplay dataset without cloning it.
pub fn builtin_ref() -> Result<&'static GameplayCatalog, &'static GameplayDataError> {
    static DATA: OnceLock<Result<GameplayCatalog, GameplayDataError>> = OnceLock::new();
    match DATA.get_or_init(|| load(GAMEPLAY_DATA)) {
        Ok(catalog) => Ok(catalog),
        Err(error) => Err(error),
    }
}

/// Compute the deterministic SHA-256 identity of a gameplay dataset.
///
/// The self-referential `identity.digest` field is excluded before the JSON
/// value is serialized with sorted object keys. Whitespace and source key
/// ordering therefore do not change the dataset identity.
pub fn content_digest(json: &str) -> Result<String, GameplayDataError> {
    let mut value: serde_json::Value = serde_json::from_str(json)
        .map_err(|error| GameplayDataError::Malformed(error.to_string()))?;
    let identity = value
        .as_object_mut()
        .and_then(|root| root.get_mut("identity"))
        .and_then(serde_json::Value::as_object_mut)
        .ok_or_else(|| GameplayDataError::Malformed("missing identity object".to_string()))?;
    identity
        .remove("digest")
        .ok_or_else(|| GameplayDataError::Malformed("missing identity.digest".to_string()))?;

    let mut canonical = String::new();
    write_canonical_json(&value, &mut canonical).map_err(GameplayDataError::Malformed)?;
    Ok(format!("sha256:{:x}", Sha256::digest(canonical.as_bytes())))
}

fn write_canonical_json(value: &serde_json::Value, output: &mut String) -> Result<(), String> {
    match value {
        serde_json::Value::Null => output.push_str("null"),
        serde_json::Value::Bool(value) => output.push_str(if *value { "true" } else { "false" }),
        serde_json::Value::Number(value) => output.push_str(&value.to_string()),
        serde_json::Value::String(value) => {
            output.push_str(&serde_json::to_string(value).map_err(|error| error.to_string())?)
        }
        serde_json::Value::Array(values) => {
            output.push('[');
            for (index, value) in values.iter().enumerate() {
                if index > 0 {
                    output.push(',');
                }
                write_canonical_json(value, output)?;
            }
            output.push(']');
        }
        serde_json::Value::Object(values) => {
            output.push('{');
            let mut keys: Vec<_> = values.keys().collect();
            keys.sort_unstable();
            for (index, key) in keys.into_iter().enumerate() {
                if index > 0 {
                    output.push(',');
                }
                output.push_str(&serde_json::to_string(key).map_err(|error| error.to_string())?);
                output.push(':');
                write_canonical_json(&values[key], output)?;
            }
            output.push('}');
        }
    }
    Ok(())
}
