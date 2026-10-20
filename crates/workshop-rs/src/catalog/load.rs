use std::collections::{BTreeMap, HashMap};
use std::sync::OnceLock;

use serde::Deserialize;

use crate::core::error::{CatalogError, Result, WorkshopError};

use super::{
    CATALOG_DATA, Catalog, CatalogEntry, EnumDomain, EnumMember, Kind, Locale,
    LocalizedStringEntry, NormalizedIndex, ParamCoercions, ParamContent, Provenance, TargetMeta,
    normalize_spelling,
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CatalogFile {
    schema_version: u32,
    locales: Vec<String>,
    target: TargetMeta,
    provenance: Provenance,
    #[serde(default)]
    version: Option<String>,
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
    /// The declared parameter list. `None` means the entry declares no
    /// signature at all; `Some` — even empty, as on the zero-argument
    /// `else`/`end` constructs — declares a call signature.
    #[serde(default)]
    params: Option<Vec<String>>,
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
    /// `rule-content-v1` facts per parameter position (parallel to
    /// `params`), declared where the surface parameter list and the content
    /// argument list differ.
    #[serde(default)]
    param_content: Vec<Option<ParamContent>>,
    #[serde(default)]
    return_type: Option<String>,
    /// Reviewed reevaluation coverage keyed by enum member id (e.g.
    /// `COLOR`), listing the parameter positions each member keeps
    /// re-evaluating. Declared only on actions whose `*Reeval` parameter
    /// selects the reevaluation mode; validated against that domain's
    /// members and the parameter arity.
    #[serde(default)]
    reevaluation_coverage: Option<BTreeMap<String, Vec<usize>>>,
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
            detection_index: None,
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
            normalized: NormalizedIndex::default(),
            descriptions: Default::default(),
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
        catalog.build_normalized_index();
        catalog.validate_param_domains()?;
        catalog.validate_param_content()?;
        catalog.detection_index =
            Some(super::detect::AliasIndex::build(&catalog).map_err(|error| {
                CatalogError::validation(format!("locale detection index: {error}"))
            })?);
        Ok(catalog)
    }

    /// The built-in catalog data.
    pub fn builtin() -> Result<Catalog> {
        Self::builtin_ref().cloned().map_err(Clone::clone)
    }

    /// The built-in catalog, parsed and indexed once per process. The
    /// embedded description dataset (`descriptions.json`) is attached and
    /// verified with it.
    pub fn builtin_ref() -> std::result::Result<&'static Catalog, &'static WorkshopError> {
        static DATA: OnceLock<Result<Catalog>> = OnceLock::new();
        match DATA.get_or_init(|| {
            let mut catalog = Self::load(CATALOG_DATA)?;
            catalog.load_descriptions(super::descriptions::DESCRIPTIONS_DATA)?;
            Ok(catalog)
        }) {
            Ok(catalog) => Ok(catalog),
            Err(error) => Err(error),
        }
    }
    /// Mirror the verbatim spelling indexes under whitespace-collapsed
    /// keys (#448). A normalized key claimed by distinct targets is poisoned
    /// (`None`) so it resolves nothing rather than guessing; identical
    /// targets (two declared spellings that differ only in whitespace, such
    /// as zh-CN `66号公路`/`66 号公路`) keep their single meaning.
    fn build_normalized_index(&mut self) {
        fn insert<T: PartialEq + Clone>(
            map: &mut HashMap<String, Option<T>>,
            key: String,
            value: T,
        ) {
            match map.get_mut(&key) {
                Some(slot) if *slot != Some(value.clone()) => *slot = None,
                Some(_) => {}
                None => {
                    map.insert(key, Some(value));
                }
            }
        }

        let mut normalized = NormalizedIndex::default();
        for (locale, by_kind) in &self.alias_to_entry {
            let by_kind_norm = normalized
                .alias_to_entry
                .entry(locale.clone())
                .or_insert_with(|| std::array::from_fn(|_| HashMap::new()));
            for (kind_index, map) in by_kind.iter().enumerate() {
                for (spelling, &entry_index) in map {
                    let key = normalize_spelling(spelling);
                    if key != spelling.as_str() {
                        insert(&mut by_kind_norm[kind_index], key.into_owned(), entry_index);
                    }
                }
            }
        }
        for (locale, map) in &self.localized_string_alias {
            let norm = normalized
                .localized_string_alias
                .entry(locale.clone())
                .or_default();
            for (spelling, &string_index) in map {
                let key = normalize_spelling(spelling);
                if key != spelling.as_str() {
                    insert(norm, key.into_owned(), string_index);
                }
            }
        }
        for (locale, map) in &self.enum_alias_to_domain {
            let norm = normalized
                .enum_alias_to_domain
                .entry(locale.clone())
                .or_default();
            for (spelling, domain) in map {
                let key = normalize_spelling(spelling);
                if key != spelling.as_str() {
                    insert(norm, key.into_owned(), domain.clone());
                }
            }
        }
        for (locale, domains) in &self.enum_alias_to_member {
            let norm = normalized
                .enum_alias_to_member
                .entry(locale.clone())
                .or_default();
            for (domain, map) in domains {
                let norm_map = norm.entry(domain.clone()).or_default();
                for (spelling, &indexes) in map {
                    let key = normalize_spelling(spelling);
                    if key != spelling.as_str() {
                        insert(norm_map, key.into_owned(), indexes);
                    }
                }
            }
        }
        for (locale, map) in &self.bare_member_index {
            let norm = normalized.bare_member.entry(locale.clone()).or_default();
            for (spelling, matches) in map {
                let key = normalize_spelling(spelling);
                if key != spelling.as_str() {
                    let merged = norm.entry(key.into_owned()).or_default();
                    for pair in matches {
                        if !merged.contains(pair) {
                            merged.push(pair.clone());
                        }
                    }
                }
            }
        }
        self.normalized = normalized;
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
        // A signature-bearing entry is any action or value, or a structural
        // entry that declares its parameter list — `"params": []` marks a
        // zero-argument construct such as `else`/`end`, while section
        // keywords declare no `params` field and stay without a signature.
        let callable = matches!(kind, Kind::Action | Kind::Value)
            || (kind == Kind::Structural && item.params.is_some());
        let params = item.params.unwrap_or_default();
        let param_names = match item.param_names {
            names if names.is_empty() => None,
            names if names.len() != params.len() => {
                return Err(CatalogError::validation(format!(
                    "{} '{}' declares {} param names for {} params",
                    kind.as_str(),
                    item.id,
                    names.len(),
                    params.len()
                )));
            }
            names if names == params => None,
            names => Some(names),
        };
        self.by_id[kind.as_index()].insert(item.id.clone(), index);
        let item_id = item.id.clone();
        self.entries.push(CatalogEntry {
            id: item.id,
            kind,
            callable,
            params,
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
            param_content: item.param_content,
            return_type: item.return_type,
            reevaluation_coverage: item.reevaluation_coverage,
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

    fn validate_param_domains(&self) -> Result<()> {
        for entry in &self.entries {
            for (name, count) in [
                ("parameter alias sets", entry.param_aliases.len()),
                ("param domains", entry.param_domains.len()),
                ("param defaults", entry.param_defaults.len()),
                ("param types", entry.param_types.len()),
                ("param coercions", entry.param_coercions.len()),
                ("param content facts", entry.param_content.len()),
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
            if let Some(coverage) = &entry.reevaluation_coverage {
                let reeval_params: Vec<usize> = (0..entry.params.len())
                    .filter(|index| {
                        entry
                            .param_domains
                            .get(*index)
                            .and_then(Option::as_deref)
                            .is_some_and(|domain| domain.ends_with("Reeval"))
                    })
                    .collect();
                if reeval_params.len() != 1 {
                    return Err(CatalogError::validation(format!(
                        "{} '{}' declares reevaluation coverage but has {} *Reeval parameters",
                        entry.kind.as_str(),
                        entry.id,
                        reeval_params.len()
                    )));
                }
                let domain_name = entry.param_domains[reeval_params[0]]
                    .as_deref()
                    .expect("filtered above");
                let domain = self
                    .enum_by_domain
                    .get(domain_name)
                    .map(|index| &self.enums[*index])
                    .expect("param domain validated above");
                for (member, positions) in coverage {
                    if !domain.members.iter().any(|m| m.member == *member) {
                        return Err(CatalogError::validation(format!(
                            "{} '{}' declares reevaluation coverage for '{member}', which is not a member of '{domain_name}'",
                            entry.kind.as_str(),
                            entry.id
                        )));
                    }
                    for position in positions {
                        if *position >= entry.params.len() {
                            return Err(CatalogError::validation(format!(
                                "{} '{}' reevaluation coverage for '{member}' names parameter {position}, out of {} params",
                                entry.kind.as_str(),
                                entry.id,
                                entry.params.len()
                            )));
                        }
                        if *position == reeval_params[0] {
                            return Err(CatalogError::validation(format!(
                                "{} '{}' reevaluation coverage for '{member}' covers the reevaluation parameter itself",
                                entry.kind.as_str(),
                                entry.id
                            )));
                        }
                    }
                }
                for member in &domain.members {
                    if !coverage.contains_key(&member.member) {
                        return Err(CatalogError::validation(format!(
                            "{} '{}' reevaluation coverage does not review '{domain_name}' member '{}'",
                            entry.kind.as_str(),
                            entry.id,
                            member.member
                        )));
                    }
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

    /// The `rule-content-v1` facts declared per parameter position must
    /// describe a well-formed content layout: a declared `paramContent`
    /// element carries at least one fact, every `fold` names a reserved
    /// variable-target call at a valid argument, and the members of one
    /// fold group are contiguous parameters covering every argument of the
    /// folded call exactly once — a `Variable`-typed member at the tagged
    /// `{"variable"}` position.
    fn validate_param_content(&self) -> Result<()> {
        for entry in &self.entries {
            // `call` -> per-member (surface position, inner `arg`) pairs.
            let mut groups: BTreeMap<&str, Vec<(usize, usize)>> = BTreeMap::new();
            for (index, content) in entry.param_content.iter().enumerate() {
                let Some(content) = content else {
                    continue;
                };
                if content.fold.is_none() && content.operations.is_empty() {
                    return Err(CatalogError::validation(format!(
                        "{} '{}' parameter {} declares an empty paramContent fact",
                        entry.kind.as_str(),
                        entry.id,
                        index + 1
                    )));
                }
                for operation in &content.operations {
                    if !self.by_id[Kind::Operator.as_index()].contains_key(operation) {
                        return Err(CatalogError::validation(format!(
                            "{} '{}' parameter {} accepts undeclared operator '{operation}'",
                            entry.kind.as_str(),
                            entry.id,
                            index + 1
                        )));
                    }
                }
                let Some(fold) = &content.fold else {
                    continue;
                };
                // The reserved variable-target calls fix both their arity
                // and which argument is the tagged `{"variable"}` node: the
                // member supplying it must be `Variable`-typed.
                let (arity, variable_arg) = match fold.call.as_str() {
                    "globalVariable" => (1, 0),
                    "playerVariable" => (2, 1),
                    _ => {
                        return Err(CatalogError::validation(format!(
                            "{} '{}' parameter {} declares a fold into unknown \
                             call '{}'",
                            entry.kind.as_str(),
                            entry.id,
                            index + 1,
                            fold.call
                        )));
                    }
                };
                if fold.arg >= arity {
                    return Err(CatalogError::validation(format!(
                        "{} '{}' parameter {} declares a fold into '{}' \
                         argument {}, out of {}",
                        entry.kind.as_str(),
                        entry.id,
                        index + 1,
                        fold.call,
                        fold.arg,
                        arity
                    )));
                }
                let variable_typed = matches!(
                    entry.param_type(index),
                    Some("Variable") | Some("Global Variable")
                );
                if fold.arg == variable_arg && !variable_typed {
                    return Err(CatalogError::validation(format!(
                        "{} '{}' parameter {} supplies '{}' argument {}, the tagged \
                         variable node, but is not Variable-typed",
                        entry.kind.as_str(),
                        entry.id,
                        index + 1,
                        fold.call,
                        fold.arg
                    )));
                }
                groups
                    .entry(fold.call.as_str())
                    .or_default()
                    .push((index, fold.arg));
            }
            for (call, members) in groups {
                let arity = if call == "globalVariable" { 1 } else { 2 };
                let positions: Vec<usize> = members.iter().map(|(index, _)| *index).collect();
                let contiguous = positions.windows(2).all(|pair| pair[1] == pair[0] + 1);
                let mut args: Vec<usize> = members.iter().map(|(_, arg)| *arg).collect();
                args.sort_unstable();
                if !contiguous || args != (0..arity).collect::<Vec<_>>() {
                    return Err(CatalogError::validation(format!(
                        "{} '{}' declares an incomplete '{}' fold group: members must be \
                         contiguous parameters covering all {} argument(s)",
                        entry.kind.as_str(),
                        entry.id,
                        call,
                        arity
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

pub(crate) fn pretty_json(value: &serde_json::Value) -> Result<String> {
    serde_json::to_string_pretty(value)
        .map(|mut out| {
            out.push('\n');
            out
        })
        .map_err(|error| CatalogError::malformed(format!("cannot serialize catalog: {error}")))
}
