//! Catalog data pipeline: the checked-in JSON schema, loading, and
//! deterministic digest. [`Catalog`] accessors live in `mod.rs`; this file
//! owns how catalog bytes become the validated structure.

use std::collections::HashMap;

use serde::Deserialize;

use crate::core::error::{CatalogError, Result};

use super::{
    CATALOG_DATA, Catalog, CatalogEntry, EnumDomain, EnumMember, Kind, Locale,
    LocalizedStringEntry, ParamCoercions, Provenance, TargetMeta,
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CatalogFile {
    schema_version: u32,
    locales: Vec<String>,
    target: TargetMeta,
    provenance: Provenance,
    /// The catalog dataset version; absent in ad-hoc test data.
    #[serde(default)]
    version: Option<String>,
    /// The declared content digest; absent in ad-hoc test data.
    #[serde(default)]
    digest: Option<String>,
    #[serde(default)]
    structural: Vec<EntryFile>,
    #[serde(default)]
    actions: Vec<EntryFile>,
    #[serde(default)]
    values: Vec<EntryFile>,
    #[serde(default)]
    events: Vec<EntryFile>,
    #[serde(default)]
    operators: Vec<EntryFile>,
    #[serde(default)]
    settings: Vec<EntryFile>,
    #[serde(default)]
    localized_strings: Vec<LocalizedStringFile>,
    #[serde(default)]
    enums: Vec<EnumFile>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct EntryFile {
    id: String,
    aliases: HashMap<String, AliasFile>,
    #[serde(default)]
    params: Vec<String>,
    /// Reviewed semantic parameter names, parallel to `params`.
    #[serde(default)]
    param_names: Vec<String>,
    #[serde(default)]
    param_aliases: Vec<HashMap<String, AliasFile>>,
    /// Canonical enum domain per parameter position (parallel to `params`);
    /// empty when no parameter domains are documented.
    #[serde(default)]
    param_domains: Vec<Option<String>>,
    /// Default value per parameter position (parallel to `params`),
    /// resolved when a call omits the argument. `None` means no default is
    /// declared. Default value syntax: `null`, a numeric literal, localized
    /// string text, `Domain.MEMBER` (builtin enum member), or a catalog value
    /// id resolved as a zero-argument call. Every default is pinned-reference
    /// probe evidence, never copied from upstream game data.
    #[serde(default)]
    param_defaults: Vec<Option<String>>,
    #[serde(default)]
    param_types: Vec<Option<String>>,
    #[serde(default)]
    param_coercions: Vec<Option<ParamCoercions>>,
    #[serde(default)]
    return_type: Option<String>,
    #[serde(default)]
    variadic: bool,
}

#[derive(Deserialize)]
struct LocalizedStringFile {
    id: String,
    aliases: HashMap<String, AliasFile>,
}

#[derive(Deserialize)]
struct EnumFile {
    domain: String,
    #[serde(default)]
    aliases: HashMap<String, AliasFile>,
    members: Vec<MemberFile>,
}

#[derive(Deserialize)]
struct MemberFile {
    id: String,
    aliases: HashMap<String, AliasFile>,
}

/// A locale may have one canonical emitter spelling or several reviewed
/// spellings observed across current Workshop producers. The string form is
/// retained for the common case; the array form makes conflicts explicit in
/// the data instead of forcing parser branches or silently choosing one.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum AliasFile {
    One(String),
    Many(Vec<String>),
}

impl AliasFile {
    fn into_spellings(self, id: &str, locale: &str) -> Result<Vec<String>> {
        let spellings = match self {
            AliasFile::One(spelling) => vec![spelling],
            AliasFile::Many(spellings) => spellings,
        };
        if spellings.is_empty() || spellings.iter().any(String::is_empty) {
            return Err(CatalogError::validation(format!(
                "catalog entry '{}' declares an empty alias for locale '{}'",
                id, locale
            )));
        }
        Ok(spellings)
    }
}
impl Catalog {
    /// Parse and validate catalog data, verifying the declared content
    /// digest when the data carries one.
    pub fn load(json: &str) -> Result<Catalog> {
        let catalog = Self::load_unverified(json)?;
        if let Some(declared) = &catalog.catalog_digest {
            let computed = content_digest(json)?;
            if declared != &computed {
                return Err(CatalogError::validation(format!(
                    "catalog digest mismatch: declared '{declared}', content '{computed}' — \
                     run the catalog pipeline (workshop-catalog-gen build)"
                )));
            }
        }
        Ok(catalog)
    }

    /// Parse and validate catalog data without digest verification. Used by
    /// the catalog pipeline so a stale digest can be repaired by `build`.
    pub fn load_unverified(json: &str) -> Result<Catalog> {
        let file: CatalogFile = serde_json::from_str(json)
            .map_err(|error| CatalogError::malformed(format!("catalog data: {error}")))?;
        if file.schema_version != 1 {
            return Err(CatalogError::malformed(format!(
                "unsupported catalog schemaVersion {}",
                file.schema_version
            )));
        }
        let locales: Vec<Locale> = file.locales.iter().map(|s| Locale::new(s)).collect();
        if locales.is_empty() {
            return Err(CatalogError::malformed(
                "catalog declares no locales".to_string(),
            ));
        }

        let mut catalog = Catalog {
            schema_version: file.schema_version,
            locales,
            target: file.target,
            provenance: file.provenance,
            catalog_version: file.version.unwrap_or_else(|| "dev".to_string()),
            catalog_digest: file.digest,
            entries: Vec::new(),
            localized_strings: Vec::new(),
            enums: Vec::new(),
            by_id: Default::default(),
            alias_to_entry: HashMap::new(),
            localized_string_by_id: HashMap::new(),
            localized_string_alias: HashMap::new(),
            enum_by_domain: HashMap::new(),
            enum_alias_to_domain: HashMap::new(),
            enum_alias_to_member: HashMap::new(),
            bare_member_index: HashMap::new(),
        };

        for (kind, items) in [
            (Kind::Structural, file.structural),
            (Kind::Action, file.actions),
            (Kind::Value, file.values),
            (Kind::Event, file.events),
            (Kind::Operator, file.operators),
            (Kind::Setting, file.settings),
        ] {
            for item in items {
                catalog.insert_entry(kind, item)?;
            }
        }
        for item in file.localized_strings {
            catalog.insert_localized_string(item)?;
        }
        for domain in file.enums {
            catalog.insert_enum(domain)?;
        }
        for domain in &catalog.enums {
            for member in &domain.members {
                for (locale, spellings) in &member.aliases {
                    for spelling in spellings {
                        catalog
                            .bare_member_index
                            .entry(locale.clone())
                            .or_default()
                            .entry(spelling.clone())
                            .or_default()
                            .push((domain.domain.clone(), member.member.clone()));
                    }
                }
            }
        }
        catalog.validate_param_domains()?;
        Ok(catalog)
    }

    /// The built-in catalog data.
    pub fn builtin() -> Result<Catalog> {
        Self::load(CATALOG_DATA)
    }
    fn insert_entry(&mut self, kind: Kind, item: EntryFile) -> Result<()> {
        let index = self.entries.len();
        let aliases = self.collect_aliases(
            &item.id,
            &format!("entry '{}'", item.id),
            item.aliases,
            true,
            |catalog, locale, spelling| {
                let locale_map = catalog.alias_to_entry.entry(locale.clone()).or_default();
                if locale_map[kind.as_index()].contains_key(spelling) {
                    return Err(CatalogError::validation(format!(
                        "duplicate {} alias '{spelling}' for locale '{locale}'",
                        kind.as_str()
                    )));
                }
                locale_map[kind.as_index()].insert(spelling.to_string(), index);
                Ok(())
            },
        )?;
        if self.by_id[kind.as_index()].contains_key(&item.id) {
            return Err(CatalogError::validation(format!(
                "duplicate {} id '{}'",
                kind.as_str(),
                item.id
            )));
        }
        // The primary locale's surface is complete: every builtin carries a
        // primary-locale alias. Additional declared locales may be partially
        // covered; missing target-locale mappings fail explicitly at
        // conversion/emission time (ADR-0001 Decision 7).
        let param_names = match item.param_names {
            names if names.is_empty() => None,
            names if names.len() != item.params.len() => {
                return Err(CatalogError::validation(format!(
                    "{} '{}' declares {} param names for {} params",
                    kind.as_str(),
                    item.id,
                    names.len(),
                    item.params.len()
                )));
            }
            names if names == item.params => None,
            names => Some(names),
        };
        self.by_id[kind.as_index()].insert(item.id.clone(), index);
        let item_id = item.id.clone();
        self.entries.push(CatalogEntry {
            id: item.id,
            kind,
            params: item.params,
            param_names,
            param_aliases: item
                .param_aliases
                .into_iter()
                .map(|aliases| {
                    aliases
                        .into_iter()
                        .map(|(locale, alias)| {
                            let locale_key = Locale::new(&locale);
                            let spellings = alias.into_spellings(&item_id, locale_key.as_str())?;
                            Ok((locale_key, spellings))
                        })
                        .collect::<Result<HashMap<_, _>>>()
                })
                .collect::<Result<Vec<_>>>()?,
            param_domains: item.param_domains,
            param_defaults: item.param_defaults,
            param_types: item.param_types,
            param_coercions: item.param_coercions,
            return_type: item.return_type,
            variadic: item.variadic,
            aliases,
        });
        Ok(())
    }

    fn insert_localized_string(&mut self, item: LocalizedStringFile) -> Result<()> {
        if self.localized_string_by_id.contains_key(&item.id) {
            return Err(CatalogError::validation(format!(
                "duplicate localized string id '{}'",
                item.id
            )));
        }
        let index = self.localized_strings.len();
        let aliases = self.collect_aliases(
            &item.id,
            &format!("localized string '{}'", item.id),
            item.aliases,
            true,
            |catalog, locale, spelling| {
                let locale_map = catalog
                    .localized_string_alias
                    .entry(locale.clone())
                    .or_default();
                if locale_map.contains_key(spelling) {
                    return Err(CatalogError::validation(format!(
                        "duplicate localized string alias '{spelling}' for locale '{locale}'"
                    )));
                }
                locale_map.insert(spelling.to_string(), index);
                Ok(())
            },
        )?;
        self.localized_string_by_id.insert(item.id.clone(), index);
        self.localized_strings.push(LocalizedStringEntry {
            id: item.id,
            aliases,
        });
        Ok(())
    }

    /// Every declared `paramDomains` domain must name a declared enum domain.
    fn validate_param_domains(&self) -> Result<()> {
        for entry in &self.entries {
            for (name, count) in [
                ("parameter alias sets", entry.param_aliases.len()),
                ("param domains", entry.param_domains.len()),
                ("param defaults", entry.param_defaults.len()),
                ("param types", entry.param_types.len()),
                ("param coercions", entry.param_coercions.len()),
            ] {
                if count > entry.params.len() {
                    return Err(CatalogError::validation(format!(
                        "{} '{}' declares more {name} than params",
                        entry.kind.as_str(),
                        entry.id
                    )));
                }
            }
            if entry.kind != Kind::Value && entry.return_type.is_some() {
                return Err(CatalogError::validation(format!(
                    "{} '{}' declares a return type but is not a value",
                    entry.kind.as_str(),
                    entry.id
                )));
            }
            for domain in entry.param_domains.iter().flatten() {
                if !self.enum_by_domain.contains_key(domain) {
                    return Err(CatalogError::validation(format!(
                        "{} '{}' declares undeclared enum domain '{domain}'",
                        entry.kind.as_str(),
                        entry.id
                    )));
                }
            }
            for aliases in &entry.param_aliases {
                for locale in aliases.keys() {
                    if !self.locales.contains(locale) {
                        return Err(CatalogError::validation(format!(
                            "{} '{}' declares parameter alias for undeclared locale '{}'",
                            entry.kind.as_str(),
                            entry.id,
                            locale
                        )));
                    }
                }
            }
        }
        Ok(())
    }

    fn insert_enum(&mut self, domain: EnumFile) -> Result<()> {
        let domain_index = self.enums.len();
        if self.enum_by_domain.contains_key(&domain.domain) {
            return Err(CatalogError::validation(format!(
                "duplicate enum domain '{}'",
                domain.domain
            )));
        }
        let primary = self.locales[0].clone();
        let mut domain_aliases = self.collect_aliases(
            &domain.domain,
            &format!("enum domain '{}'", domain.domain),
            domain.aliases,
            false,
            |catalog, locale, spelling| {
                let locale_map = catalog
                    .enum_alias_to_domain
                    .entry(locale.clone())
                    .or_default();
                if let Some(existing) = locale_map.get(spelling) {
                    return Err(CatalogError::validation(format!(
                        "duplicate enum domain alias '{spelling}' for '{existing}' and '{}' in locale '{locale}'",
                        domain.domain
                    )));
                }
                locale_map.insert(spelling.to_string(), domain.domain.clone());
                Ok(())
            },
        )?;
        domain_aliases
            .entry(primary.clone())
            .or_insert_with(|| vec![domain.domain.clone()]);
        self.enum_alias_to_domain
            .entry(primary.clone())
            .or_default()
            .entry(domain.domain.clone())
            .or_insert_with(|| domain.domain.clone());
        let mut members = Vec::new();
        for (member_index, member) in domain.members.into_iter().enumerate() {
            let aliases = self.collect_aliases(
                &member.id,
                &format!("enum {}::{}", domain.domain, member.id),
                member.aliases,
                true,
                |catalog, locale, spelling| {
                    let domain_map = catalog
                        .enum_alias_to_member
                        .entry(locale.clone())
                        .or_default()
                        .entry(domain.domain.clone())
                        .or_default();
                    if domain_map.contains_key(spelling) {
                        return Err(CatalogError::validation(format!(
                            "duplicate enum alias '{spelling}' in '{}' for locale '{locale}'",
                            domain.domain
                        )));
                    }
                    domain_map.insert(spelling.to_string(), (domain_index, member_index));
                    Ok(())
                },
            )?;
            members.push(EnumMember {
                member: member.id,
                aliases,
            });
        }
        self.enum_by_domain
            .insert(domain.domain.clone(), domain_index);
        self.enums.push(EnumDomain {
            domain: domain.domain,
            aliases: domain_aliases,
            members,
        });
        Ok(())
    }

    /// Collect per-locale alias spellings for one catalog item: reject
    /// undeclared locales, normalize each spelling list, hand every spelling
    /// to `register` for the caller's index/dup check, and — when
    /// `require_primary` — require coverage of the primary locale.
    fn collect_aliases(
        &mut self,
        id: &str,
        label: &str,
        aliases: HashMap<String, AliasFile>,
        require_primary: bool,
        mut register: impl FnMut(&mut Self, &Locale, &str) -> Result<()>,
    ) -> Result<HashMap<Locale, Vec<String>>> {
        let mut collected = HashMap::new();
        for (locale_str, alias_file) in aliases {
            let locale = Locale::new(&locale_str);
            if !self.locales.contains(&locale) {
                return Err(CatalogError::validation(format!(
                    "{label} declares alias for undeclared locale '{locale}'"
                )));
            }
            let spellings = alias_file.into_spellings(id, locale.as_str())?;
            for spelling in &spellings {
                register(self, &locale, spelling)?;
            }
            collected.insert(locale, spellings);
        }
        let primary = &self.locales[0];
        if require_primary && !collected.contains_key(primary) {
            return Err(CatalogError::validation(format!(
                "{label} is missing a '{primary}' alias"
            )));
        }
        Ok(collected)
    }
}

/// Canonicalize catalog data: parse, validate, and re-serialize
/// deterministically (object keys sorted, stable formatting). Re-running on
/// the same input produces byte-identical output, so the data pipeline is
/// reproducible. Validation intentionally skips digest verification so a
/// stale digest can be repaired by [`build_canonical`].
pub fn canonicalize(json: &str) -> Result<String> {
    // Validate the semantic content first.
    Catalog::load_unverified(json)?;
    let value: serde_json::Value = serde_json::from_str(json)
        .map_err(|error| CatalogError::malformed(format!("catalog data: {error}")))?;
    pretty_json(&value)
}

/// Rebuild the canonical catalog form with a fresh content digest: validate,
/// canonicalize, and (re)write the `digest` field. Byte-idempotent, so the
/// committed dataset and its digest are reproducible from the data file.
pub fn build_canonical(json: &str) -> Result<String> {
    let mut value: serde_json::Value = serde_json::from_str(json)
        .map_err(|error| CatalogError::malformed(format!("catalog data: {error}")))?;
    let digest = content_digest(json)?;
    if let Some(object) = value.as_object_mut() {
        object.insert("digest".to_string(), serde_json::Value::String(digest));
    }
    let output = pretty_json(&value)?;
    // Validate the semantic content (including the fresh digest) before
    // returning the rebuilt file.
    Catalog::load(&output)?;
    Ok(output)
}

/// The deterministic content digest of catalog data: sha256 of the canonical
/// (sorted-key, pretty) serialization of the parsed content with the
/// self-referential `digest` field removed. Independent of file formatting;
/// changes whenever any semantic content changes.
pub fn content_digest(json: &str) -> Result<String> {
    let mut value: serde_json::Value = serde_json::from_str(json)
        .map_err(|error| CatalogError::malformed(format!("catalog data: {error}")))?;
    if let Some(object) = value.as_object_mut() {
        object.remove("digest");
    }
    let canonical = pretty_json(&value)?;
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(canonical.trim_end().as_bytes());
    Ok(format!("{:x}", hasher.finalize()))
}

/// Sorted-key pretty JSON with a trailing newline — the canonical catalog
/// byte form all digest and rebuild paths share.
fn pretty_json(value: &serde_json::Value) -> Result<String> {
    serde_json::to_string_pretty(value)
        .map(|mut out| {
            out.push('\n');
            out
        })
        .map_err(|error| CatalogError::malformed(format!("cannot serialize catalog: {error}")))
}
