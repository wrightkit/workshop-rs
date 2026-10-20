//! Localized descriptions for catalog actions and values.
//!
//! Descriptions are catalog data, not Rust items
//! ([ADR-0016](https://github.com/wrightkit/workshop-rs/blob/main/docs/adr/0016-catalog-content-outside-rust-api.md)):
//! they live in `data/descriptions.json`, a separate dataset indexed by
//! catalog id next to `catalog.json`. An entry carries a description and
//! per-parameter descriptions as locale maps; a requested locale with no
//! text falls back to the primary locale (`en-US`) with the fallback
//! marked on the returned [`Description`].
//!
//! Coverage is explicit: for every shipped locale, every action, every
//! value, and every parameter position either carries text or appears on
//! the dataset's `gaps` list, enforced at load time. A locale declared in
//! `pendingLocales` is an announced future stage — it carries no text and
//! needs no per-entry gaps.

use std::collections::{BTreeMap, HashMap};

use serde::Deserialize;

use crate::core::error::{CatalogError, Result};

use super::load::pretty_json;
use super::{Catalog, Kind, Locale, Provenance, content_digest};

/// The embedded description data, next to [`super::CATALOG_DATA`].
pub const DESCRIPTIONS_DATA: &str = include_str!("data/descriptions.json");

/// A description text resolved for a requested locale.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Description {
    /// The description text.
    pub text: String,
    /// The locale the text is written in — the requested locale, or the
    /// primary locale when the request fell back.
    pub locale: Locale,
    /// Whether the requested locale had no text and the primary locale's
    /// text was returned instead.
    pub fallback: bool,
}

/// The description data of one catalog entry: the entry description plus
/// per-parameter descriptions, each a locale map.
#[derive(Debug, Clone)]
pub(crate) struct EntryDescriptions {
    pub(crate) description: HashMap<Locale, String>,
    /// Parallel to `CatalogEntry::params`; `None` where the position has
    /// no description.
    pub(crate) params: Vec<Option<HashMap<Locale, String>>>,
}

/// The validated description dataset attached to a [`Catalog`].
#[derive(Debug, Clone, Default)]
pub(crate) struct Descriptions {
    /// The dataset version; `"dev"` when the data declares none.
    pub(crate) version: String,
    /// The declared content digest (sha256 hex), when the data carries one.
    pub(crate) digest: Option<String>,
    /// The locales the dataset ships text for.
    pub(crate) locales: Vec<Locale>,
    /// Declared locales whose coverage is a pending future stage; they
    /// carry no text and need no gap entries.
    pub(crate) pending: Vec<Locale>,
    /// Entry index → description data.
    pub(crate) by_entry: HashMap<usize, EntryDescriptions>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DescriptionsFile {
    schema_version: u32,
    /// The locales this dataset ships text for.
    locales: Vec<String>,
    /// Declared catalog locales whose description coverage is a pending
    /// stage.
    #[serde(default)]
    pending_locales: Vec<String>,
    #[allow(dead_code)]
    provenance: Provenance,
    #[serde(default)]
    version: Option<String>,
    #[serde(default)]
    digest: Option<String>,
    #[serde(default)]
    entries: DescriptionSections,
    #[serde(default)]
    gaps: BTreeMap<String, Vec<String>>,
}

/// Per-kind sections of `entries`; kinds without reviewed descriptions
/// (structural constructs, events, operators, settings) are rejected so
/// the file cannot silently carry undescribed-scope data.
#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct DescriptionSections {
    #[serde(default)]
    actions: BTreeMap<String, EntryDescriptionsFile>,
    #[serde(default)]
    values: BTreeMap<String, EntryDescriptionsFile>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EntryDescriptionsFile {
    /// The entry's own description as a locale map.
    #[serde(default)]
    description: HashMap<String, String>,
    /// Per-parameter descriptions as locale maps, parallel to the
    /// entry's catalog parameter list; `null` marks a position with no
    /// description.
    #[serde(default)]
    params: Option<Vec<Option<HashMap<String, String>>>>,
}

/// One declared coverage gap: `<id>` for an entry description, or
/// `<id>.params[<index>]` for a parameter position.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Gap {
    Entry(String),
    Param(String, usize),
}

fn parse_gap(key: &str) -> Option<Gap> {
    if let Some(without_bracket) = key.strip_suffix(']') {
        let (id, index) = without_bracket.split_once(".params[")?;
        return index
            .parse()
            .ok()
            .map(|index| Gap::Param(id.to_string(), index));
    }
    (!key.contains(['.', '[', ']'])).then(|| Gap::Entry(key.to_string()))
}

fn parse_file(json: &str) -> Result<DescriptionsFile> {
    serde_json::from_str(json)
        .map_err(|error| CatalogError::malformed(format!("descriptions data: {error}")))
}

fn verify_digest(json: &str, file: &DescriptionsFile) -> Result<()> {
    if let Some(declared) = &file.digest {
        let computed = content_digest(json)?;
        if declared != &computed {
            return Err(CatalogError::validation(format!(
                "descriptions digest mismatch: declared '{declared}', content '{computed}' — \
                 run the catalog pipeline (workshop-catalog-gen build)"
            )));
        }
    }
    Ok(())
}

fn check_text(owner: &str, locale: &str, text: &str) -> Result<()> {
    if text.is_empty() {
        return Err(CatalogError::validation(format!(
            "descriptions entry {owner} declares an empty '{locale}' text"
        )));
    }
    if text.trim() != text || text.contains('\r') {
        return Err(CatalogError::validation(format!(
            "descriptions entry {owner} declares an untrimmed '{locale}' text"
        )));
    }
    Ok(())
}

/// Validate `file` against `catalog` into a [`Descriptions`] dataset.
fn validated(catalog: &Catalog, file: DescriptionsFile) -> Result<Descriptions> {
    if file.schema_version != 1 {
        return Err(CatalogError::malformed(format!(
            "unsupported descriptions schemaVersion {}",
            file.schema_version
        )));
    }
    let locales: Vec<Locale> = file.locales.iter().map(|s| Locale::new(s)).collect();
    for locale in &locales {
        if !catalog.locales.contains(locale) {
            return Err(CatalogError::validation(format!(
                "descriptions data ships undeclared catalog locale '{locale}'"
            )));
        }
    }
    let mut pending = Vec::new();
    for pending_locale in &file.pending_locales {
        let locale = Locale::new(pending_locale);
        if !catalog.locales.contains(&locale) {
            return Err(CatalogError::validation(format!(
                "descriptions data pends undeclared catalog locale '{locale}'"
            )));
        }
        if locales.contains(&locale) || pending.contains(&locale) {
            return Err(CatalogError::validation(format!(
                "descriptions data declares locale '{locale}' as both shipped and pending"
            )));
        }
        pending.push(locale);
    }
    let shipped = |locale: &Locale| locales.contains(locale);

    // Parse the gap list: every key must name a real entry or parameter
    // position, and every gap locale must be a shipped locale.
    let mut gaps: HashMap<Locale, Vec<Gap>> = HashMap::new();
    for (locale_str, keys) in &file.gaps {
        let locale = Locale::new(locale_str);
        if !shipped(&locale) {
            return Err(CatalogError::validation(format!(
                "descriptions gaps list locale '{locale}', which the dataset does not ship"
            )));
        }
        let parsed = gaps.entry(locale.clone()).or_default();
        for key in keys {
            let gap = parse_gap(key).ok_or_else(|| {
                CatalogError::validation(format!(
                    "descriptions gap '{key}' is not '<id>' or '<id>.params[<index>]'"
                ))
            })?;
            let id = match &gap {
                Gap::Entry(id) | Gap::Param(id, _) => id,
            };
            let index = catalog.by_id[Kind::Action.as_index()]
                .get(id)
                .or_else(|| catalog.by_id[Kind::Value.as_index()].get(id))
                .copied()
                .ok_or_else(|| {
                    CatalogError::validation(format!(
                        "descriptions gap '{key}' names no catalog action or value"
                    ))
                })?;
            if let Gap::Param(_, position) = &gap {
                if *position >= catalog.entries[index].params.len() {
                    return Err(CatalogError::validation(format!(
                        "descriptions gap '{key}' names parameter {position}, \
                         out of {} params",
                        catalog.entries[index].params.len()
                    )));
                }
            }
            parsed.push(gap);
        }
    }

    // Attach entry descriptions: ids must exist, texts must be non-empty
    // shipped-locale maps, and a declared params list must be parallel to
    // the catalog parameter list.
    let mut by_entry: HashMap<usize, EntryDescriptions> = HashMap::new();
    for (kind, section) in [
        (Kind::Action, &file.entries.actions),
        (Kind::Value, &file.entries.values),
    ] {
        for (id, entry_file) in section {
            let index = *catalog.by_id[kind.as_index()].get(id).ok_or_else(|| {
                CatalogError::validation(format!(
                    "descriptions entry '{id}' is not a catalog {} id",
                    kind.as_str()
                ))
            })?;
            let entry = &catalog.entries[index];
            let mut description = HashMap::new();
            for (locale_str, text) in &entry_file.description {
                let locale = Locale::new(locale_str);
                if !shipped(&locale) {
                    return Err(CatalogError::validation(format!(
                        "descriptions entry '{id}' declares '{locale}' text, \
                         which the dataset does not ship"
                    )));
                }
                check_text(&format!("'{id}'"), locale_str, text)?;
                description.insert(locale, text.clone());
            }
            let mut params: Vec<Option<HashMap<Locale, String>>> = Vec::new();
            if let Some(param_files) = &entry_file.params {
                if param_files.len() != entry.params.len() {
                    return Err(CatalogError::validation(format!(
                        "{} '{id}' declares {} parameter descriptions for {} params",
                        kind.as_str(),
                        param_files.len(),
                        entry.params.len()
                    )));
                }
                for (position, param_file) in param_files.iter().enumerate() {
                    let mut texts = HashMap::new();
                    if let Some(param_file) = param_file {
                        if param_file.is_empty() {
                            return Err(CatalogError::validation(format!(
                                "{} '{id}' declares an empty locale map at parameter {position}",
                                kind.as_str()
                            )));
                        }
                        for (locale_str, text) in param_file {
                            let locale = Locale::new(locale_str);
                            if !shipped(&locale) {
                                return Err(CatalogError::validation(format!(
                                    "{} '{id}' parameter {position} declares '{locale}' text, \
                                     which the dataset does not ship",
                                    kind.as_str()
                                )));
                            }
                            check_text(&format!("'{id}' parameter {position}"), locale_str, text)?;
                            texts.insert(locale, text.clone());
                        }
                    }
                    params.push(if texts.is_empty() { None } else { Some(texts) });
                }
            }
            if description.is_empty() && params.iter().all(Option::is_none) {
                return Err(CatalogError::validation(format!(
                    "descriptions entry '{id}' carries no description text"
                )));
            }
            by_entry.insert(
                index,
                EntryDescriptions {
                    description,
                    params,
                },
            );
        }
    }

    // Coverage is explicit: for every shipped locale each action, each
    // value, and every parameter position either carries text or names a
    // gap — never both, never neither.
    for locale in &locales {
        let declared: Vec<&Gap> = gaps
            .get(locale)
            .map(|gaps| gaps.iter().collect())
            .unwrap_or_default();
        for entry in catalog
            .entries
            .iter()
            .filter(|entry| matches!(entry.kind, Kind::Action | Kind::Value))
        {
            let index = catalog.by_id[entry.kind.as_index()][&entry.id];
            let has_text = by_entry
                .get(&index)
                .is_some_and(|data| data.description.contains_key(locale));
            if has_text == declared.contains(&&Gap::Entry(entry.id.clone())) {
                return Err(CatalogError::validation(format!(
                    "descriptions coverage for '{}' '{locale}' must be text or a gap, \
                     not both or neither",
                    entry.id
                )));
            }
            for position in 0..entry.params.len() {
                let has_text = by_entry
                    .get(&index)
                    .and_then(|data| data.params.get(position))
                    .and_then(Option::as_ref)
                    .is_some_and(|texts| texts.contains_key(locale));
                if has_text == declared.contains(&&Gap::Param(entry.id.clone(), position)) {
                    return Err(CatalogError::validation(format!(
                        "descriptions coverage for '{}' parameter {position} '{locale}' \
                         must be text or a gap, not both or neither",
                        entry.id
                    )));
                }
            }
        }
    }

    Ok(Descriptions {
        version: file.version.unwrap_or_else(|| "dev".to_string()),
        digest: file.digest,
        locales,
        pending,
        by_entry,
    })
}

impl Catalog {
    /// Validate and attach localized description data (`descriptions.json`),
    /// verifying the declared content digest when present.
    ///
    /// The dataset must cover the catalog it attaches to: for every shipped
    /// locale each action, each value, and every parameter position either
    /// carries text or names an explicit `gaps` entry. `pendingLocales`
    /// declares catalog locales whose coverage is a planned stage; they
    /// carry no text.
    pub fn load_descriptions(&mut self, json: &str) -> Result<()> {
        let file = parse_file(json)?;
        verify_digest(json, &file)?;
        self.descriptions = validated(self, file)?;
        Ok(())
    }

    /// Parse and validate description data without digest verification, so
    /// the catalog pipeline can repair a stale digest (`build`).
    pub fn load_descriptions_unverified(&mut self, json: &str) -> Result<()> {
        self.descriptions = validated(self, parse_file(json)?)?;
        Ok(())
    }

    /// The description dataset version; `"dev"` when none is declared or
    /// no descriptions are attached.
    pub fn descriptions_version(&self) -> &str {
        &self.descriptions.version
    }

    /// The declared content digest (sha256 hex) of the attached description
    /// dataset, verified at load; `None` when none is declared or no
    /// descriptions are attached.
    pub fn descriptions_digest(&self) -> Option<&str> {
        self.descriptions.digest.as_deref()
    }

    /// The locales the description dataset ships text for.
    pub fn description_locales(&self) -> &[Locale] {
        &self.descriptions.locales
    }

    /// Declared catalog locales whose description coverage is a pending
    /// future stage; they carry no text.
    pub fn pending_description_locales(&self) -> &[Locale] {
        &self.descriptions.pending
    }

    /// The description of an action or value in `locale` — the locale's own
    /// text when declared, or the primary locale's text with `fallback`
    /// marked, or `None` when neither exists.
    pub fn description(&self, kind: Kind, id: &str, locale: &Locale) -> Option<Description> {
        let index = *self.by_id[kind.as_index()].get(id)?;
        self.descriptions
            .by_entry
            .get(&index)
            .and_then(|data| self.resolve_text(&data.description, locale))
    }

    /// The description of one declared parameter position of an action or
    /// value in `locale`, with the same marked primary-locale fallback as
    /// [`Catalog::description`].
    pub fn param_description(
        &self,
        kind: Kind,
        id: &str,
        position: usize,
        locale: &Locale,
    ) -> Option<Description> {
        let index = *self.by_id[kind.as_index()].get(id)?;
        self.descriptions
            .by_entry
            .get(&index)
            .and_then(|data| data.params.get(position))
            .and_then(Option::as_ref)
            .and_then(|texts| self.resolve_text(texts, locale))
    }

    fn resolve_text(
        &self,
        texts: &HashMap<Locale, String>,
        locale: &Locale,
    ) -> Option<Description> {
        if let Some(text) = texts.get(locale) {
            return Some(Description {
                text: text.clone(),
                locale: locale.clone(),
                fallback: false,
            });
        }
        let primary = self.primary_locale();
        texts.get(primary).map(|text| Description {
            text: text.clone(),
            locale: primary.clone(),
            fallback: true,
        })
    }
}

/// Canonicalize description data against `catalog`: validate and re-serialize
/// deterministically (sorted keys, stable formatting). Byte-idempotent.
pub fn canonicalize_descriptions(catalog: &Catalog, json: &str) -> Result<String> {
    validated(catalog, parse_file(json)?)?;
    let value: serde_json::Value = serde_json::from_str(json)
        .map_err(|error| CatalogError::malformed(format!("descriptions data: {error}")))?;
    pretty_json(&value)
}

/// Rebuild the canonical description form with a fresh content digest:
/// validate against `catalog`, canonicalize, and (re)write the `digest`
/// field. Byte-idempotent, mirroring [`super::build_canonical`].
pub fn build_canonical_descriptions(catalog: &Catalog, json: &str) -> Result<String> {
    let mut value: serde_json::Value = serde_json::from_str(json)
        .map_err(|error| CatalogError::malformed(format!("descriptions data: {error}")))?;
    let digest = content_digest(json)?;
    if let Some(object) = value.as_object_mut() {
        object.insert("digest".to_string(), serde_json::Value::String(digest));
    }
    let output = pretty_json(&value)?;
    let file = parse_file(&output)?;
    verify_digest(&output, &file)?;
    validated(catalog, file)?;
    Ok(output)
}
