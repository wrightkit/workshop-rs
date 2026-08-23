//! The canonical Workshop catalog.
//!
//! The catalog is the locale-independent semantic identity layer between
//! textual Workshop spellings and WIR. Every builtin has a canonical `id` and
//! a [`Kind`]; locale tables map canonical identities to client spellings and
//! back, so parser, emitter, analyzer, and tooling never embed
//! locale-specific strings as identity.
//!
//! Locale coverage is data ([`docs/adr/0001-catalog-boundaries.md`]):
//! the primary locale (the first declared one, `en-US`) is complete — every
//! entry and enum member carries a primary-locale alias — while additional
//! declared locales may be partially covered. Missing target-locale mappings
//! fail explicitly at conversion/emission time; the catalog reports exact
//! per-locale coverage machine-readably ([`Catalog::locale_coverage`],
//! [`Catalog::identity`]).
//!
//! The catalog dataset declares its own `version` and a deterministic content
//! `digest` (sha256) recomputed by the catalog pipeline
//! (`workshop-catalog-gen build`); [`Catalog::load`] rejects a digest
//! mismatch, so dataset changes are deliberate and reproducible.

use std::collections::HashMap;

use serde::{Deserialize, Deserializer, Serialize};

use crate::signatures::ExpectedDomain;

use crate::error::{CatalogError, Result};

/// The embedded catalog data.
pub const CATALOG_DATA: &str = include_str!("data/catalog.json");

/// A normalized Workshop client locale, e.g. `en-US`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
pub struct Locale(String);

impl Locale {
    /// Build a locale from a client spelling, normalized to lowercase.
    pub fn new(value: &str) -> Locale {
        Locale(value.trim().to_ascii_lowercase())
    }

    /// The normalized locale string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for Locale {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Ok(Self::new(&value))
    }
}

impl std::fmt::Display for Locale {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// The kind of a catalog builtin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind {
    /// A structural keyword (If, End, Set Global Variable, …).
    Structural,
    /// An action function.
    Action,
    /// A value function.
    Value,
    /// An event.
    Event,
    /// An operator token (comparison operators).
    Operator,
    /// An enumerated value domain.
    Enum,
    /// A settings entry.
    Setting,
}

impl Kind {
    pub fn as_str(self) -> &'static str {
        match self {
            Kind::Structural => "structural",
            Kind::Action => "action",
            Kind::Value => "value",
            Kind::Event => "event",
            Kind::Operator => "operator",
            Kind::Enum => "enum",
            Kind::Setting => "setting",
        }
    }
}

/// One catalog builtin.
#[derive(Debug, Clone)]
pub struct CatalogEntry {
    pub id: String,
    pub kind: Kind,
    /// Parameter names, when the catalog documents them.
    pub params: Vec<String>,
    /// The canonical enum domain expected at each parameter position, when
    /// the parameter takes an enumerated value (parallel to `params`).
    /// `None` for non-enum parameters and for parameters whose accepted
    /// values span multiple canonical domains. In particular, a filtered
    /// rule event's `Player` parameter accepts `EventPlayer` members or
    /// canonical `Hero` members; the WIR [`crate::wir::EventTarget`] carries
    /// that union explicitly.
    pub param_domains: Vec<Option<String>>,
    /// Default value per parameter position (parallel to `params`),
    /// resolved when a call omits the argument. See the catalog data
    /// provenance for the value syntax and evidence.
    pub param_defaults: Vec<Option<String>>,
    /// Evidence-backed semantic type per parameter position. `None` means
    /// the available sources do not prove a narrower type.
    pub param_types: Vec<Option<String>>,
    /// Evidence-backed return type for Value entries. Actions must leave this
    /// unset; an absent value is intentionally evidence-insufficient.
    pub return_type: Option<String>,
    /// Whether the final declared parameter repeats for additional arguments.
    pub variadic: bool,
    aliases: HashMap<Locale, Vec<String>>,
}

impl CatalogEntry {
    /// The localized spelling of this builtin in `locale`, when declared.
    pub fn spelling(&self, locale: &Locale) -> Option<&str> {
        self.aliases
            .get(locale)
            .and_then(|spellings| spellings.first())
            .map(String::as_str)
    }

    /// Every reviewed localized spelling of this builtin, with the first
    /// spelling reserved for deterministic emission.
    pub fn spellings(&self, locale: &Locale) -> &[String] {
        self.aliases.get(locale).map(Vec::as_slice).unwrap_or(&[])
    }

    /// The number of declared arguments for this builtin.
    pub fn param_count(&self) -> usize {
        self.params.len()
    }

    /// The number of arguments that must be present when trailing defaults
    /// are applied. A missing default in the middle of a signature remains a
    /// required position; defaults only make the suffix optional.
    pub fn required_param_count(&self) -> usize {
        (0..self.params.len())
            .rev()
            .find(|index| {
                self.param_defaults
                    .get(*index)
                    .and_then(Option::as_ref)
                    .is_none()
            })
            .map_or(0, |index| index + 1)
    }

    /// The declared enum domain for an argument position, when one exists.
    pub fn param_domain(&self, index: usize) -> Option<&str> {
        self.param_domains
            .get(index)
            .or_else(|| self.variadic.then(|| self.param_domains.last()).flatten())
            .and_then(Option::as_deref)
    }

    /// The evidence-backed semantic type for an argument position, when
    /// available. Enum domains remain exposed separately by `param_domain`.
    pub fn param_type(&self, index: usize) -> Option<&str> {
        self.param_types
            .get(index)
            .or_else(|| self.variadic.then(|| self.param_types.last()).flatten())
            .and_then(Option::as_deref)
    }

    /// The evidence-backed return type of a Value, when available.
    pub fn return_type(&self) -> Option<&str> {
        self.return_type.as_deref()
    }
}

/// One enum member within a domain.
#[derive(Debug, Clone)]
pub struct EnumMember {
    pub member: String,
    aliases: HashMap<Locale, Vec<String>>,
}

impl EnumMember {
    /// The localized spelling of this member in `locale`, when declared.
    pub fn spelling(&self, locale: &Locale) -> Option<&str> {
        self.aliases
            .get(locale)
            .and_then(|spellings| spellings.first())
            .map(String::as_str)
    }

    /// Every reviewed localized spelling of this enum member, with the first
    /// spelling reserved for deterministic emission.
    pub fn spellings(&self, locale: &Locale) -> &[String] {
        self.aliases.get(locale).map(Vec::as_slice).unwrap_or(&[])
    }
}

/// One enum value domain (e.g. `Color`, `Beam`).
#[derive(Debug, Clone)]
pub struct EnumDomain {
    pub domain: String,
    aliases: HashMap<Locale, Vec<String>>,
    pub members: Vec<EnumMember>,
}

impl EnumDomain {
    pub fn spelling(&self, locale: &Locale) -> Option<&str> {
        self.aliases
            .get(locale)
            .and_then(|spellings| spellings.first())
            .map(String::as_str)
    }
}

/// Target-format metadata recorded in the catalog.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct TargetMeta {
    pub game: String,
    pub format: String,
    pub surface: String,
}

/// Provenance of the catalog data.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Provenance {
    pub generator: String,
    pub generator_version: String,
    pub source: String,
    pub license: String,
    pub reviewed: bool,
    /// Additional immutable observations that qualify the dataset source,
    /// including reviewed spelling conflicts retained as parse aliases.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_notes: Vec<String>,
}

/// Per-locale mapping coverage: how many canonical entries (builtins and
/// enum members) carry a mapping for the locale out of the declared total.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct LocaleCoverage {
    pub locale: Locale,
    /// Canonical entries with a declared mapping in this locale.
    pub mapped: usize,
    /// Canonical entries (builtins and enum members) declared by the catalog.
    pub total: usize,
}

/// The machine-readable catalog identity (ADR-0001 Decision 5): the four
/// identities that evolve independently — implementation version, catalog
/// dataset version plus content digest, locale coverage, and target evidence
/// — plus the data provenance record. Serialized with the ADR's kebab-case
/// identity names.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct CatalogIdentity {
    /// The `workshop-rs` package version (semver); bumped by code changes.
    pub implementation_version: String,
    /// The catalog dataset version; bumped by any dataset change.
    pub catalog_version: String,
    /// The deterministic content digest (sha256 hex) computed by the
    /// pipeline; `None` when the data does not declare one.
    pub catalog_digest: Option<String>,
    /// Declared locales with per-locale mapping counts.
    pub locale_coverage: Vec<LocaleCoverage>,
    /// The declared target surface.
    pub target: TargetMeta,
    /// The provenance record of the catalog data.
    pub provenance: Provenance,
}

/// The validated canonical Workshop catalog.
#[derive(Debug, Clone)]
pub struct Catalog {
    pub schema_version: u32,
    /// The declared locales, normalized; the first one is the primary
    /// locale and must be fully covered.
    pub locales: Vec<Locale>,
    pub target: TargetMeta,
    pub provenance: Provenance,
    /// The catalog dataset version (ADR-0001 `catalog-version`).
    catalog_version: String,
    /// The declared content digest (sha256 hex), verified at load when
    /// present (ADR-0001 `catalog-version`).
    catalog_digest: Option<String>,
    entries: Vec<CatalogEntry>,
    enums: Vec<EnumDomain>,
    by_id: HashMap<(Kind, String), usize>,
    alias_to_entry: HashMap<(Kind, Locale, String), usize>,
    enum_by_domain: HashMap<String, usize>,
    enum_alias_to_domain: HashMap<(Locale, String), String>,
    enum_alias_to_member: HashMap<(String, Locale, String), (usize, usize)>,
}

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
    enums: Vec<EnumFile>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct EntryFile {
    id: String,
    aliases: HashMap<String, AliasFile>,
    #[serde(default)]
    params: Vec<String>,
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
    return_type: Option<String>,
    #[serde(default)]
    variadic: bool,
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
            enums: Vec::new(),
            by_id: HashMap::new(),
            alias_to_entry: HashMap::new(),
            enum_by_domain: HashMap::new(),
            enum_alias_to_domain: HashMap::new(),
            enum_alias_to_member: HashMap::new(),
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
        for domain in file.enums {
            catalog.insert_enum(domain)?;
        }
        catalog.validate_param_domains()?;
        Ok(catalog)
    }

    /// The built-in catalog data.
    pub fn builtin() -> Result<Catalog> {
        Self::load(CATALOG_DATA)
    }

    /// The declared locales, normalized; the first one is the primary locale.
    pub fn locales(&self) -> &[Locale] {
        &self.locales
    }

    /// The primary locale: the first declared one, whose mapping surface is
    /// complete (`en-US` in the committed catalog).
    pub fn primary_locale(&self) -> &Locale {
        &self.locales[0]
    }

    /// Whether a locale is declared by the catalog.
    pub fn supports(&self, locale: &Locale) -> bool {
        self.locales.contains(locale)
    }

    /// The catalog dataset version (ADR-0001 `catalog-version`).
    pub fn catalog_version(&self) -> &str {
        &self.catalog_version
    }

    /// The declared content digest (sha256 hex) of the catalog dataset,
    /// verified at load; `None` for data that declares none.
    pub fn catalog_digest(&self) -> Option<&str> {
        self.catalog_digest.as_deref()
    }

    /// The `workshop-rs` package version (ADR-0001 `implementation-version`).
    pub fn implementation_version() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    /// The machine-readable catalog identity: implementation version, catalog
    /// version + digest, locale coverage, target evidence, and provenance.
    pub fn identity(&self) -> CatalogIdentity {
        CatalogIdentity {
            implementation_version: Self::implementation_version().to_string(),
            catalog_version: self.catalog_version.clone(),
            catalog_digest: self.catalog_digest.clone(),
            locale_coverage: self
                .locales
                .iter()
                .map(|locale| self.locale_coverage(locale))
                .collect(),
            target: self.target.clone(),
            provenance: self.provenance.clone(),
        }
    }

    /// The mapping coverage of one declared locale: mapped entries out of the
    /// declared total (builtins and enum members). The primary locale is
    /// always complete; other locales may be partially covered.
    pub fn locale_coverage(&self, locale: &Locale) -> LocaleCoverage {
        let member_total: usize = self.enums.iter().map(|domain| domain.members.len()).sum();
        let total = self.entries.len() + member_total;
        let mapped = self
            .entries
            .iter()
            .filter(|entry| entry.aliases.contains_key(locale))
            .count()
            + self
                .enums
                .iter()
                .flat_map(|domain| &domain.members)
                .filter(|member| member.aliases.contains_key(locale))
                .count();
        LocaleCoverage {
            locale: locale.clone(),
            mapped,
            total,
        }
    }

    /// The mapping coverage of every declared locale, in declaration order.
    pub fn locale_coverage_all(&self) -> Vec<LocaleCoverage> {
        self.locales
            .iter()
            .map(|locale| self.locale_coverage(locale))
            .collect()
    }

    /// The builtin with the given canonical id and kind.
    pub fn entry(&self, kind: Kind, id: &str) -> Option<&CatalogEntry> {
        self.by_id
            .get(&(kind, id.to_string()))
            .map(|i| &self.entries[*i])
    }

    /// Resolve a localized spelling to its canonical builtin.
    pub fn resolve(&self, kind: Kind, locale: &Locale, spelling: &str) -> Option<&CatalogEntry> {
        self.alias_to_entry
            .get(&(kind, locale.clone(), spelling.to_string()))
            .map(|i| &self.entries[*i])
    }

    /// The localized spelling of a canonical builtin id.
    pub fn spelling(&self, kind: Kind, locale: &Locale, id: &str) -> Option<&str> {
        self.entry(kind, id)?.spelling(locale)
    }

    /// Every entry of a kind, in catalog order.
    pub fn entries_of(&self, kind: Kind) -> impl Iterator<Item = &CatalogEntry> {
        self.entries.iter().filter(move |entry| entry.kind == kind)
    }

    /// The total number of builtin entries.
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    /// The number of enum domains.
    pub fn enum_domains_count(&self) -> usize {
        self.enums.len()
    }

    /// The enum domain with the given name.
    pub fn enum_domain(&self, domain: &str) -> Option<&EnumDomain> {
        self.enum_by_domain.get(domain).map(|i| &self.enums[*i])
    }

    /// Resolve a localized enum-domain spelling to its canonical domain id.
    pub fn resolve_enum_domain(&self, locale: &Locale, spelling: &str) -> Option<&str> {
        self.enum_by_domain
            .get_key_value(spelling)
            .map(|(domain, _)| domain.as_str())
            .or_else(|| {
                self.enum_alias_to_domain
                    .get(&(locale.clone(), spelling.to_string()))
                    .map(String::as_str)
            })
    }

    /// Every enum domain, in catalog order.
    pub fn enum_domains(&self) -> impl Iterator<Item = &EnumDomain> {
        self.enums.iter()
    }

    /// Resolve a localized enum member spelling to `(domain, canonical member)`.
    pub fn resolve_enum_member(
        &self,
        domain: &str,
        locale: &Locale,
        spelling: &str,
    ) -> Option<(String, String)> {
        let (domain_index, member_index) = self.enum_alias_to_member.get(&(
            domain.to_string(),
            locale.clone(),
            spelling.to_string(),
        ))?;
        Some((
            domain.to_string(),
            self.enums[*domain_index].members[*member_index]
                .member
                .clone(),
        ))
    }

    /// The localized spelling of a canonical enum member.
    pub fn enum_spelling(&self, domain: &str, locale: &Locale, member: &str) -> Option<&str> {
        let domain_index = self.enum_by_domain.get(domain)?;
        let domain = &self.enums[*domain_index];
        domain
            .members
            .iter()
            .find(|candidate| candidate.member == member)?
            .spelling(locale)
    }

    /// Every `(domain, canonical member)` match for a bare (domain-less)
    /// localized member spelling. Returns all matches so callers can report
    /// ambiguity; a well-formed catalog has at most one meaningful match for
    /// a given spelling.
    pub fn bare_member_matches(&self, locale: &Locale, spelling: &str) -> Vec<(String, String)> {
        let mut matches = Vec::new();
        for domain in &self.enums {
            for member in &domain.members {
                if member
                    .spellings(locale)
                    .iter()
                    .any(|alias| alias == spelling)
                {
                    matches.push((domain.domain.clone(), member.member.clone()));
                }
            }
        }
        matches
    }

    fn insert_entry(&mut self, kind: Kind, item: EntryFile) -> Result<()> {
        let index = self.entries.len();
        let mut aliases = HashMap::new();
        for (locale_str, alias_file) in item.aliases {
            let locale = Locale::new(&locale_str);
            if !self.locales.contains(&locale) {
                return Err(CatalogError::validation(format!(
                    "entry '{}' declares alias for undeclared locale '{}'",
                    item.id, locale
                )));
            }
            let spellings = alias_file.into_spellings(&item.id, locale.as_str())?;
            for spelling in &spellings {
                let key = (kind, locale.clone(), spelling.clone());
                if self.alias_to_entry.contains_key(&key) {
                    return Err(CatalogError::validation(format!(
                        "duplicate {} alias '{spelling}' for locale '{}'",
                        kind.as_str(),
                        locale
                    )));
                }
                self.alias_to_entry.insert(key, index);
            }
            aliases.insert(locale, spellings);
        }
        let id_key = (kind, item.id.clone());
        if self.by_id.contains_key(&id_key) {
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
        let primary = self.locales[0].clone();
        if !aliases.contains_key(&primary) {
            return Err(CatalogError::validation(format!(
                "{} '{}' is missing a '{}' alias",
                kind.as_str(),
                item.id,
                primary
            )));
        }
        self.by_id.insert(id_key, index);
        self.entries.push(CatalogEntry {
            id: item.id,
            kind,
            params: item.params,
            param_domains: item.param_domains,
            param_defaults: item.param_defaults,
            param_types: item.param_types,
            return_type: item.return_type,
            variadic: item.variadic,
            aliases,
        });
        Ok(())
    }

    /// Every declared `paramDomains` domain must name a declared enum domain.
    fn validate_param_domains(&self) -> Result<()> {
        for entry in &self.entries {
            if entry.param_domains.len() > entry.params.len() {
                return Err(CatalogError::validation(format!(
                    "{} '{}' declares more param domains than params",
                    entry.kind.as_str(),
                    entry.id
                )));
            }
            if entry.param_defaults.len() > entry.params.len() {
                return Err(CatalogError::validation(format!(
                    "{} '{}' declares more param defaults than params",
                    entry.kind.as_str(),
                    entry.id
                )));
            }
            if entry.param_types.len() > entry.params.len() {
                return Err(CatalogError::validation(format!(
                    "{} '{}' declares more param types than params",
                    entry.kind.as_str(),
                    entry.id
                )));
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
        let mut domain_aliases = HashMap::new();
        for (locale_str, alias_file) in domain.aliases {
            let locale = Locale::new(&locale_str);
            if !self.locales.contains(&locale) {
                return Err(CatalogError::validation(format!(
                    "enum domain '{}' declares alias for undeclared locale '{}'",
                    domain.domain, locale
                )));
            }
            let spellings = alias_file.into_spellings(&domain.domain, locale.as_str())?;
            for spelling in &spellings {
                let key = (locale.clone(), spelling.clone());
                if let Some(existing) = self.enum_alias_to_domain.get(&key) {
                    return Err(CatalogError::validation(format!(
                        "duplicate enum domain alias '{spelling}' for '{}' and '{}' in locale '{}'",
                        existing, domain.domain, locale
                    )));
                }
                self.enum_alias_to_domain.insert(key, domain.domain.clone());
            }
            domain_aliases.insert(locale, spellings);
        }
        domain_aliases
            .entry(primary.clone())
            .or_insert_with(|| vec![domain.domain.clone()]);
        self.enum_alias_to_domain
            .entry((primary.clone(), domain.domain.clone()))
            .or_insert_with(|| domain.domain.clone());
        let mut members = Vec::new();
        for (member_index, member) in domain.members.into_iter().enumerate() {
            let mut aliases = HashMap::new();
            for (locale_str, alias_file) in member.aliases {
                let locale = Locale::new(&locale_str);
                if !self.locales.contains(&locale) {
                    return Err(CatalogError::validation(format!(
                        "enum {}::{} declares alias for undeclared locale '{}'",
                        domain.domain, member.id, locale
                    )));
                }
                let spellings = alias_file.into_spellings(&member.id, locale.as_str())?;
                for spelling in &spellings {
                    let key = (domain.domain.clone(), locale.clone(), spelling.clone());
                    if self.enum_alias_to_member.contains_key(&key) {
                        return Err(CatalogError::validation(format!(
                            "duplicate enum alias '{spelling}' in '{}' for locale '{}'",
                            domain.domain, locale
                        )));
                    }
                    self.enum_alias_to_member
                        .insert(key, (domain_index, member_index));
                }
                aliases.insert(locale, spellings);
            }
            if !aliases.contains_key(&primary) {
                return Err(CatalogError::validation(format!(
                    "enum {}::{} is missing a '{}' alias",
                    domain.domain, member.id, primary
                )));
            }
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
}

/// The catalog is the canonical source of expected enum domains for the
/// Workshop surface it documents: `expected_domain(catalog_id, arg_index)`
/// answers the domain declared for that parameter position (e.g. `createHudText`
/// argument 9 is `HudReeval`), so the Workshop parser can resolve bare enum
/// members that are ambiguous across domains (e.g. `Visible To and String`).
/// Positions without a documented domain answer `None`.
impl ExpectedDomain for Catalog {
    fn expected_domain(&self, catalog_id: &str, arg_index: usize) -> Option<&str> {
        for kind in [Kind::Action, Kind::Value] {
            if let Some(entry) = self.entry(kind, catalog_id) {
                if let Some(domain) = entry
                    .param_domains
                    .get(arg_index)
                    .and_then(Option::as_deref)
                {
                    return Some(domain);
                }
            }
        }
        None
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
    serde_json::to_string_pretty(&value)
        .map(|mut out| {
            out.push('\n');
            out
        })
        .map_err(|error| CatalogError::malformed(format!("cannot serialize catalog: {error}")))
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
    let output = serde_json::to_string_pretty(&value)
        .map(|mut out| {
            out.push('\n');
            out
        })
        .map_err(|error| CatalogError::malformed(format!("cannot serialize catalog: {error}")))?;
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
    let canonical = serde_json::to_string_pretty(&value)
        .map_err(|error| CatalogError::malformed(format!("cannot serialize catalog: {error}")))?;
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(canonical.as_bytes());
    Ok(format!("{:x}", hasher.finalize()))
}
