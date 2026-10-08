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

pub mod detect;

use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Deserializer, Serialize};

use crate::core::signatures::ExpectedDomain;

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
    Structural = 0,
    /// An action function.
    Action = 1,
    /// A value function.
    Value = 2,
    /// An event.
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

/// One catalog builtin.
#[derive(Debug, Clone)]
pub struct CatalogEntry {
    pub id: String,
    pub kind: Kind,
    /// Parameter names, when the catalog documents them.
    pub(crate) params: Vec<String>,
    /// Reviewed semantic names when they differ from the catalog parameters.
    pub(crate) param_names: Option<Vec<String>>,
    /// Reviewed localized spellings for each parameter, parallel to `params`.
    pub(crate) param_aliases: Vec<HashMap<Locale, Vec<String>>>,
    /// The canonical enum domain expected at each parameter position, when
    /// the parameter takes an enumerated value (parallel to `params`).
    /// `None` for non-enum parameters and for parameters whose accepted
    /// values span multiple canonical domains. In particular, a filtered
    /// rule event's `Player` parameter accepts `EventPlayer` members or
    /// canonical `Hero` members; the WIR [`crate::wir::EventTarget`] carries
    /// that union explicitly.
    pub(crate) param_domains: Vec<Option<String>>,
    /// Default value per parameter position (parallel to `params`),
    /// resolved when a call omits the argument. See the catalog data
    /// provenance for the value syntax and source.
    pub(crate) param_defaults: Vec<Option<String>>,
    /// Source-backed semantic type per parameter position. `None` means
    /// the available sources do not establish a narrower type.
    pub(crate) param_types: Vec<Option<String>>,
    /// Contextual literal substitutions per parameter position.
    pub(crate) param_coercions: Vec<Option<ParamCoercions>>,
    /// Source-backed return type for Value entries. Actions must leave this
    /// unset; an absent value remains unresolved.
    pub(crate) return_type: Option<String>,
    /// Reevaluation coverage for the action's `*Reeval` parameter, keyed by
    /// enum member id: which parameter positions the selected reevaluation
    /// member keeps re-evaluating. `None` when the action has no `*Reeval`
    /// parameter or coverage is not reviewed. Members absent from the map
    /// are unknown, not empty — an empty vector is a reviewed "reevaluates
    /// nothing" claim.
    pub(crate) reevaluation_coverage: Option<BTreeMap<String, Vec<usize>>>,
    /// Whether the final declared parameter repeats for additional arguments.
    pub(crate) variadic: bool,
    pub(crate) aliases: HashMap<Locale, Vec<String>>,
}

/// A locale-independent identity for a preset used by the Workshop `String`
/// value. Unlike a custom `Value::String`, this identity must resolve through
/// reviewed client-locale aliases before it can be parsed or emitted.
#[derive(Debug, Clone)]
pub struct LocalizedStringEntry {
    pub id: String,
    pub(crate) aliases: HashMap<Locale, Vec<String>>,
}

fn spelling<'a>(aliases: &'a HashMap<Locale, Vec<String>>, locale: &Locale) -> Option<&'a str> {
    aliases
        .get(locale)
        .and_then(|spellings| spellings.first())
        .map(String::as_str)
}

fn spellings_for<'a>(aliases: &'a HashMap<Locale, Vec<String>>, locale: &Locale) -> &'a [String] {
    aliases.get(locale).map(Vec::as_slice).unwrap_or_default()
}

impl LocalizedStringEntry {
    /// The deterministic emitted spelling in `locale`, when mapped.
    pub fn spelling(&self, locale: &Locale) -> Option<&str> {
        spelling(&self.aliases, locale)
    }

    /// All reviewed spellings accepted for this locale.
    pub fn spellings(&self, locale: &Locale) -> &[String] {
        spellings_for(&self.aliases, locale)
    }
}

impl CatalogEntry {
    /// The localized spelling of this builtin in `locale`, when declared.
    pub fn spelling(&self, locale: &Locale) -> Option<&str> {
        spelling(&self.aliases, locale)
    }

    /// Every reviewed localized spelling of this builtin, with the first
    /// spelling reserved for deterministic emission.
    pub fn spellings(&self, locale: &Locale) -> &[String] {
        spellings_for(&self.aliases, locale)
    }

    /// Resolve a canonical or reviewed localized parameter spelling to its
    /// unambiguous declared position.
    pub fn resolve_param(&self, locale: &Locale, spelling: &str) -> Option<usize> {
        let matches = self
            .params
            .iter()
            .enumerate()
            .filter(|(index, canonical)| {
                canonical == &spelling
                    || self
                        .param_aliases
                        .get(*index)
                        .and_then(|aliases| aliases.get(locale))
                        .is_some_and(|aliases| aliases.iter().any(|alias| alias == spelling))
            })
            .map(|(index, _)| index)
            .collect::<Vec<_>>();
        (matches.len() == 1).then(|| matches[0])
    }

    /// The canonical parameter names in declaration order.
    pub fn params(&self) -> &[String] {
        &self.params
    }

    /// The number of declared arguments for this builtin.
    pub fn param_count(&self) -> usize {
        self.params.len()
    }

    /// The reviewed semantic name for an argument position, when declared.
    pub fn param_name(&self, index: usize) -> Option<&str> {
        let names = self.param_names.as_deref().unwrap_or(&self.params);
        param_at(names, index, self.variadic).map(String::as_str)
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

    /// Whether any declared parameter has a default value.
    pub fn has_param_defaults(&self) -> bool {
        self.param_defaults.iter().any(Option::is_some)
    }

    /// The default value for an argument position, when declared.
    pub fn param_default(&self, index: usize) -> Option<&str> {
        param_at(&self.param_defaults, index, self.variadic).and_then(Option::as_deref)
    }

    /// The declared enum domain for an argument position, when one exists.
    pub fn param_domain(&self, index: usize) -> Option<&str> {
        param_at(&self.param_domains, index, self.variadic).and_then(Option::as_deref)
    }

    /// The parameter positions the given `*Reeval` enum member keeps
    /// re-evaluating on this action, when the action declares reviewed
    /// reevaluation coverage. `Some(&[])` means the member re-evaluates no
    /// parameter of this action; `None` means the member's coverage is
    /// unknown for this action and callers must stay conservative.
    ///
    /// `member` is the canonical enum member id (e.g. `COLOR`), the same
    /// spelling a parsed `Value::Enum` carries.
    pub fn reevaluation_coverage(&self, member: &str) -> Option<&[usize]> {
        self.reevaluation_coverage
            .as_ref()
            .and_then(|coverage| coverage.get(member).map(Vec::as_slice))
    }

    /// Whether this action declares reviewed per-parameter reevaluation
    /// coverage for its `*Reeval` parameter.
    pub fn has_reevaluation_coverage(&self) -> bool {
        self.reevaluation_coverage.is_some()
    }

    /// The source-backed semantic type for an argument position, when
    /// available. Enum domains remain exposed separately by `param_domain`.
    pub fn param_type(&self, index: usize) -> Option<&str> {
        param_at(&self.param_types, index, self.variadic).and_then(Option::as_deref)
    }

    /// The contextual literal substitutions for an argument position.
    pub fn param_coercions(&self, index: usize) -> Option<&ParamCoercions> {
        param_at(&self.param_coercions, index, self.variadic).and_then(Option::as_ref)
    }

    /// The source-backed return type of a Value, when available.
    pub fn return_type(&self) -> Option<&str> {
        self.return_type.as_deref()
    }

    /// Whether the final declared parameter repeats for additional arguments.
    pub fn is_variadic(&self) -> bool {
        self.variadic
    }
}

fn param_at<T>(values: &[T], index: usize, variadic: bool) -> Option<&T> {
    values
        .get(index)
        .or_else(|| variadic.then(|| values.last()).flatten())
}

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

/// The validated canonical Workshop catalog.
#[derive(Debug, Clone)]
pub struct Catalog {
    detection_index: Option<detect::AliasIndex>,
    pub(crate) schema_version: u32,
    /// The declared locales, normalized; the first one is the primary
    /// locale and must be fully covered.
    pub(crate) locales: Vec<Locale>,
    pub(crate) target: TargetMeta,
    pub(crate) provenance: Provenance,
    /// The catalog dataset version (ADR-0001 `catalog-version`).
    pub(crate) catalog_version: String,
    /// The declared content digest (sha256 hex), verified at load when
    /// present (ADR-0001 `catalog-version`).
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
}

/// Catalog loading and digest machinery lives in `load.rs`.
mod load;

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

    /// Resolve a localized spelling to its canonical builtin.
    pub fn resolve(&self, kind: Kind, locale: &Locale, spelling: &str) -> Option<&CatalogEntry> {
        self.alias_to_entry
            .get(locale)
            .and_then(|by_kind| by_kind[kind.as_index()].get(spelling))
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
                    .get(locale)
                    .and_then(|map| map.get(spelling))
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
        let &(domain_index, member_index) = self
            .enum_alias_to_member
            .get(locale)?
            .get(domain)?
            .get(spelling)?;
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
    /// a given spelling.
    pub fn bare_member_matches(&self, locale: &Locale, spelling: &str) -> Vec<(String, String)> {
        self.bare_member_index
            .get(locale)
            .and_then(|map| map.get(spelling))
            .cloned()
            .unwrap_or_default()
    }

    /// The canonical ids and `locale` spellings of `kind` entries, for
    /// nearest-candidate diagnostics that reject in the canonical space
    /// (validation and emission, where the rejected name is an id or an
    /// authored spelling).
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

    /// The emitted form of `member_spelling` in `domain` under `locale`:
    /// constructor-form domains (`Hero`, `Button`, `Color`, `Map`) write
    /// `Domain(Member)` in every position and locale; every other domain
    /// writes the member bare (docs/wrapper-forms.md).
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

    /// The canonical ids and `locale` spellings of `domain` members, for
    /// nearest-candidate diagnostics that reject in the canonical space.
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
                if let Some(domain) = entry.param_domain(arg_index) {
                    return Some(domain);
                }
            }
        }
        None
    }
}
