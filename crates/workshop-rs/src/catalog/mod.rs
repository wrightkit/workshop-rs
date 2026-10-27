//! The canonical Workshop catalog.
//!
//! The catalog is the locale-independent semantic identity layer between
//! textual Workshop spellings and WIR. Every builtin has a canonical `id` and
//! a [`Kind`]; locale tables map canonical identities to client spellings and
//! back, so parser, emitter, analyzer, and tooling never embed
//! locale-specific strings as identity.
//!
//! Locale coverage is data ([ADR-0001](https://github.com/wrightkit/workshop-rs/blob/main/docs/adr/0001-catalog-boundaries.md)):
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

pub mod descriptions;
pub mod detect;
mod entry;

use std::borrow::Cow;
use std::collections::HashMap;

use serde::{Deserialize, Deserializer, Serialize};

use crate::core::signatures::ExpectedDomain;

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
    Structural = 0,
    Action = 1,
    Value = 2,
    Event = 3,
    /// An operator token (comparison operators).
    Operator = 4,
    /// An enumerated value domain.
    Enum = 5,
    /// A settings entry.
    Setting = 6,
}

impl Kind {
    pub const NUM_KINDS: usize = 7;

    pub const fn as_index(self) -> usize {
        self as usize
    }

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

/// Literal substitutions accepted at one parameter position. The authored
/// literal is kept in WIR; these facts only decide acceptance.
///
/// These are deliberately per-parameter facts. They do not establish a
/// global relationship between Workshop booleans, numbers, arrays, strings,
/// vectors, or null.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ParamCoercions {
    /// Accept `False` as numeric zero.
    #[serde(default)]
    pub false_as_number: bool,
    /// Accept `True` as numeric one.
    #[serde(default)]
    pub true_as_number: bool,
    /// Accept numeric zero as `Null`.
    #[serde(default)]
    pub zero_as_null: bool,
    /// Accept `Vector(0, 0, 0)` as `Null`.
    #[serde(default)]
    pub null_vector_as_null: bool,
    /// Accept `Empty Array` as an empty string.
    #[serde(default)]
    pub empty_array_as_string: bool,
}

/// `rule-content-v1` facts for one parameter position, declared when the
/// surface parameter list and the structured document's argument list
/// differ. Absent elements and entries without the array map their
/// parameters one-to-one to plain content arguments.
///
/// These are deliberately per-parameter facts describing the emitted
/// document's layout; they do not change the surface signature, which
/// follows the Workshop call syntax.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ParamContent {
    /// The folded variable-target call this parameter contributes to, when
    /// the call's variable target folds surface parameters into one
    /// `globalVariable`/`playerVariable` content argument.
    #[serde(default)]
    pub fold: Option<ParamContentFold>,
    /// The modify-operator call ids accepted at this position (`add`,
    /// `removeFromArrayByIndex`, …). In `rule-content-v1` the operand is a
    /// zero-argument call node named by one of these ids — not an enum
    /// member, so the position declares no enum domain.
    #[serde(default)]
    pub operations: Vec<String>,
}

/// Membership in a folded variable-target call: this surface parameter
/// supplies one argument of the `globalVariable`/`playerVariable` call the
/// `rule-content-v1` target renders as. All members of a fold group share
/// the same `call` and are contiguous surface parameters; the folded call
/// occupies the content-argument position of the group's first member.
///
/// The member's node form follows its declared parameter type: a
/// `Variable`-typed parameter supplies the tagged `{"variable": "<name>"}`
/// node bound to the declared variable name, and any other parameter
/// supplies a normal value node.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ParamContentFold {
    /// The reserved target call id (`globalVariable` or `playerVariable`).
    pub call: String,
    /// The argument index inside the folded call this parameter supplies.
    pub arg: usize,
}

pub use entry::{CatalogEntry, LocalizedStringEntry};
use entry::{spelling, spellings_for};

/// One enum member within a domain.
#[derive(Debug, Clone)]
pub struct EnumMember {
    pub member: String,
    pub(crate) aliases: HashMap<Locale, Vec<String>>,
}

impl EnumMember {
    /// The localized spelling of this member in `locale`, when declared.
    pub fn spelling(&self, locale: &Locale) -> Option<&str> {
        spelling(&self.aliases, locale)
    }

    /// Every reviewed localized spelling of this enum member, with the first
    /// spelling reserved for deterministic emission.
    pub fn spellings(&self, locale: &Locale) -> &[String] {
        spellings_for(&self.aliases, locale)
    }
}

/// One enum value domain (e.g. `Color`, `Beam`).
#[derive(Debug, Clone)]
pub struct EnumDomain {
    pub domain: String,
    pub(crate) aliases: HashMap<Locale, Vec<String>>,
    pub members: Vec<EnumMember>,
}

impl EnumDomain {
    pub fn spelling(&self, locale: &Locale) -> Option<&str> {
        spelling(&self.aliases, locale)
    }
}

/// Target-format metadata recorded in the catalog.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub struct TargetMeta {
    pub game: String,
    pub format: String,
    pub surface: String,
}

/// Provenance of the catalog data.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
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

/// Per-locale mapping coverage: how many canonical entries (builtins,
/// localized preset identities, and enum members) carry a mapping for the
/// locale out of the declared total.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
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
#[non_exhaustive]
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

pub(crate) type MemberIndexMap = HashMap<String, HashMap<String, (usize, usize)>>;

type EnumMemberAliasMap = HashMap<String, HashMap<String, Option<(usize, usize)>>>;

#[derive(Debug, Clone, Default)]
pub(crate) struct NormalizedIndex {
    pub(crate) alias_to_entry: HashMap<Locale, [HashMap<String, Option<usize>>; Kind::NUM_KINDS]>,
    pub(crate) localized_string_alias: HashMap<Locale, HashMap<String, Option<usize>>>,
    pub(crate) enum_alias_to_domain: HashMap<Locale, HashMap<String, Option<String>>>,
    pub(crate) enum_alias_to_member: HashMap<Locale, EnumMemberAliasMap>,
    pub(crate) bare_member: HashMap<Locale, HashMap<String, Vec<(String, String)>>>,
}

pub(crate) fn normalize_spelling(spelling: &str) -> Cow<'_, str> {
    if spelling.chars().any(char::is_whitespace) {
        Cow::Owned(spelling.chars().filter(|c| !c.is_whitespace()).collect())
    } else {
        Cow::Borrowed(spelling)
    }
}

fn lookup_normalized<'a, V>(
    index: &'a HashMap<String, Option<V>>,
    spelling: &str,
) -> Option<&'a V> {
    index.get(normalize_spelling(spelling).as_ref())?.as_ref()
}

/// The validated canonical Workshop catalog.
#[derive(Debug, Clone)]
pub struct Catalog {
    detection_index: Option<detect::AliasIndex>,
    pub(crate) schema_version: u32,
    pub(crate) locales: Vec<Locale>,
    pub(crate) target: TargetMeta,
    pub(crate) provenance: Provenance,
    pub(crate) catalog_version: String,
    pub(crate) catalog_digest: Option<String>,
    pub(crate) entries: Vec<CatalogEntry>,
    pub(crate) localized_strings: Vec<LocalizedStringEntry>,
    pub(crate) enums: Vec<EnumDomain>,
    pub(crate) by_id: [HashMap<String, usize>; Kind::NUM_KINDS],
    pub(crate) alias_to_entry: HashMap<Locale, [HashMap<String, usize>; Kind::NUM_KINDS]>,
    pub(crate) localized_string_by_id: HashMap<String, usize>,
    pub(crate) localized_string_alias: HashMap<Locale, HashMap<String, usize>>,
    pub(crate) enum_by_domain: HashMap<String, usize>,
    pub(crate) enum_alias_to_domain: HashMap<Locale, HashMap<String, String>>,
    pub(crate) enum_alias_to_member: HashMap<Locale, MemberIndexMap>,
    pub(crate) bare_member_index: HashMap<Locale, HashMap<String, Vec<(String, String)>>>,
    pub(crate) normalized: NormalizedIndex,
    pub(crate) descriptions: descriptions::Descriptions,
}

mod load;
mod validation;

pub use load::{build_canonical, canonicalize, content_digest};

impl Catalog {
    pub(crate) fn schema_version(&self) -> u32 {
        self.schema_version
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
    /// declared total (builtins, localized preset identities, and enum members).
    /// The primary locale is
    /// always complete; other locales may be partially covered.
    pub fn locale_coverage(&self, locale: &Locale) -> LocaleCoverage {
        let member_total: usize = self.enums.iter().map(|domain| domain.members.len()).sum();
        let total = self.entries.len() + self.localized_strings.len() + member_total;
        let mapped = self
            .entries
            .iter()
            .filter(|entry| entry.aliases.contains_key(locale))
            .count()
            + self
                .localized_strings
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
        self.by_id[kind.as_index()]
            .get(id)
            .map(|i| &self.entries[*i])
    }

    /// Resolve a localized spelling to its canonical builtin. The verbatim
    /// index answers first; on a miss the whitespace-collapsed index
    /// answers a spelling whose declared form carries unwritable
    /// whitespace (`ヒーロー: `, `Élévation à une puissance `, #448).
    pub fn resolve(&self, kind: Kind, locale: &Locale, spelling: &str) -> Option<&CatalogEntry> {
        self.alias_to_entry
            .get(locale)
            .and_then(|by_kind| by_kind[kind.as_index()].get(spelling))
            .map(|i| &self.entries[*i])
            .or_else(|| {
                let index = &self.normalized.alias_to_entry.get(locale)?[kind.as_index()];
                lookup_normalized(index, spelling).map(|i| &self.entries[*i])
            })
    }

    /// The localized spelling of a canonical builtin id.
    pub fn spelling(&self, kind: Kind, locale: &Locale, id: &str) -> Option<&str> {
        self.entry(kind, id)?.spelling(locale)
    }

    /// Every entry of a kind, in catalog order.
    pub fn entries_of(&self, kind: Kind) -> impl Iterator<Item = &CatalogEntry> {
        self.entries.iter().filter(move |entry| entry.kind == kind)
    }

    /// Resolve a localized preset spelling to its stable identity.
    pub fn resolve_localized_string(
        &self,
        locale: &Locale,
        spelling: &str,
    ) -> Option<&LocalizedStringEntry> {
        self.localized_string_alias
            .get(locale)
            .and_then(|map| map.get(spelling))
            .map(|index| &self.localized_strings[*index])
            .or_else(|| {
                let index = self.normalized.localized_string_alias.get(locale)?;
                lookup_normalized(index, spelling).map(|i| &self.localized_strings[*i])
            })
    }

    /// Resolve the emitted spelling of a localized preset identity.
    pub fn localized_string_spelling(&self, locale: &Locale, id: &str) -> Option<&str> {
        self.localized_string_by_id
            .get(id)
            .and_then(|i| self.localized_strings.get(*i))
            .and_then(|entry| entry.spelling(locale))
    }

    /// Every reviewed localized preset identity, in catalog order.
    pub fn localized_strings(&self) -> impl Iterator<Item = &LocalizedStringEntry> {
        self.localized_strings.iter()
    }

    /// The total number of builtin entries.
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

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
                    .get(locale)
                    .and_then(|map| map.get(spelling))
                    .map(String::as_str)
            })
            .or_else(|| {
                let index = self.normalized.enum_alias_to_domain.get(locale)?;
                lookup_normalized(index, spelling).map(String::as_str)
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
        let indexes = self
            .enum_alias_to_member
            .get(locale)?
            .get(domain)?
            .get(spelling)
            .copied()
            .or_else(|| {
                let index = self
                    .normalized
                    .enum_alias_to_member
                    .get(locale)?
                    .get(domain)?;
                lookup_normalized(index, spelling).copied()
            })?;
        let (domain_index, member_index) = indexes;
        Some((
            domain.to_string(),
            self.enums[domain_index].members[member_index]
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

    /// Resolve a canonical enum member through the locale boundary, including
    /// reviewed partial locale spellings that are not yet part of the full
    /// catalog locale set.
    pub fn localized_enum_spelling(
        &self,
        domain: &str,
        locale: &Locale,
        member: &str,
    ) -> Option<&str> {
        self.enum_spelling(domain, locale, member).or_else(|| {
            (domain == "Color" && member == "WHITE").then_some(match locale.as_str() {
                "de-de" => "Weiß",
                "es-es" | "es-mx" => "Blanco",
                "fr-fr" => "Blanc",
                "it-it" => "Bianco",
                "ja-jp" => "白",
                "ko-kr" => "흰색",
                "pl-pl" => "Biały",
                "pt-br" => "Branco",
                "ru-ru" => "Белый",
                "th-th" => "สีขาว",
                "tr-tr" => "Beyaz",
                "zh-tw" => "白色",
                _ => return None,
            })
        })
    }

    /// Every `(domain, canonical member)` match for a bare (domain-less)
    /// localized member spelling. Returns all matches so callers can report
    /// ambiguity; a well-formed catalog has at most one meaningful match for
    /// a given spelling. The verbatim index answers first; a spelling that
    /// differs only in whitespace falls back to the collapsed index (#448).
    pub fn bare_member_matches(&self, locale: &Locale, spelling: &str) -> Vec<(String, String)> {
        if let Some(matches) = self
            .bare_member_index
            .get(locale)
            .and_then(|map| map.get(spelling))
        {
            return matches.clone();
        }
        self.normalized
            .bare_member
            .get(locale)
            .and_then(|map| map.get(normalize_spelling(spelling).as_ref()))
            .cloned()
            .unwrap_or_default()
    }

    pub(crate) fn canonical_spellings<'a>(
        &'a self,
        kind: Kind,
        locale: &'a Locale,
    ) -> impl Iterator<Item = &'a str> {
        self.entries_of(kind).flat_map(move |entry| {
            std::iter::once(entry.id.as_str())
                .chain(entry.spellings(locale).iter().map(String::as_str))
        })
    }

    pub(crate) fn enum_member_form(
        &self,
        domain: &str,
        member_spelling: &str,
        locale: &Locale,
    ) -> String {
        if matches!(domain, "Hero" | "Button" | "Color" | "Map") {
            let domain_display = self
                .enum_domain(domain)
                .and_then(|entry| entry.spelling(locale))
                .unwrap_or(domain);
            format!("{domain_display}({member_spelling})")
        } else {
            member_spelling.to_string()
        }
    }

    pub(crate) fn canonical_member_spellings<'a>(
        &'a self,
        domain: &'a str,
        locale: &'a Locale,
    ) -> impl Iterator<Item = &'a str> {
        self.enum_domain(domain)
            .into_iter()
            .flat_map(move |domain| {
                domain.members.iter().flat_map(move |member| {
                    std::iter::once(member.member.as_str())
                        .chain(member.spellings(locale).iter().map(String::as_str))
                })
            })
    }
}

impl ExpectedDomain for Catalog {
    fn expected_domain(&self, catalog_id: &str, arg_index: usize) -> Option<&str> {
        for kind in [Kind::Action, Kind::Value] {
            if let Some(entry) = self.entry(kind, catalog_id)
                && let Some(domain) = entry.param_domain(arg_index)
            {
                return Some(domain);
            }
        }
        None
    }
}
