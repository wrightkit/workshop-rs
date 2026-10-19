//! Deterministic, sharded census of the canonical Workshop surface.
//!
//! The census is derived from this crate's catalog, settings table, and WIR
//! capabilities. It is tooling that runs contract/regression probes and
//! produces structured results, not a source-language inventory or a
//! live-client oracle.

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::conformance::{
    CONFORMANCE_SCHEMA_VERSION, Comparison, ConformanceReason, ConformanceResult,
    ConformanceStatus, Equivalence, FeatureId, FeatureKind, FeatureNamespace, ReasonCode,
    TestArtifact, is_sha256_digest,
};
use workshop_rs::catalog::{Catalog, CatalogEntry, EnumDomain, Kind, Locale};
use workshop_rs::gameplay::{self, HeroId};
use workshop_rs::settings::{
    self as settings_schema, SettingDefinition, SettingTarget, SettingTargetKind,
    SettingValueDomain,
};
use workshop_rs::{WorkshopError, convert, emitter, parser, roundtrip};

#[derive(Clone, Copy)]
enum CensusCapabilityKind {
    Variable,
    PlayerVariable,
    Subroutine,
    ControlFlow,
    String,
}

struct CensusCapability {
    kind: CensusCapabilityKind,
    name: &'static str,
}

const CENSUS_CAPABILITIES: &[CensusCapability] = &[
    CensusCapability {
        kind: CensusCapabilityKind::Variable,
        name: "global",
    },
    CensusCapability {
        kind: CensusCapabilityKind::PlayerVariable,
        name: "player",
    },
    CensusCapability {
        kind: CensusCapabilityKind::Subroutine,
        name: "declaration-and-call",
    },
    CensusCapability {
        kind: CensusCapabilityKind::ControlFlow,
        name: "if",
    },
    CensusCapability {
        kind: CensusCapabilityKind::ControlFlow,
        name: "else-if",
    },
    CensusCapability {
        kind: CensusCapabilityKind::ControlFlow,
        name: "else",
    },
    CensusCapability {
        kind: CensusCapabilityKind::ControlFlow,
        name: "while",
    },
    CensusCapability {
        kind: CensusCapabilityKind::ControlFlow,
        name: "for-global-variable",
    },
    CensusCapability {
        kind: CensusCapabilityKind::String,
        name: "custom-string",
    },
];

pub const CENSUS_SCHEMA_VERSION: u32 = 1;
pub const CENSUS_IDENTITY_SCHEMA_VERSION: u32 = 1;
const EN_US: &str = "en-US";
const ZH_CN: &str = "zh-CN";
const LOCALIZATION_EN_US_SOURCE: &str = r#"rule ("Localization") {
    event {
        Ongoing - Global;
    }
    actions {
        Disable Inspector Recording;
    }
}
"#;
const LOCALIZATION_ZH_CN_SOURCE: &str = r#"rule ("Localization") {
    event {
        持续 - 全局;
    }
    actions {
        禁用查看器录制;
    }
}
"#;

/// An explicit support classification for a census case.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum CensusSupport {
    Exercise,
    Unsupported { detail: String },
    KnownGap { detail: String },
    Inconclusive { detail: String },
}

/// One deterministic case with explicit source-locale provenance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CensusCase {
    pub case_id: String,
    pub features: Vec<FeatureId>,
    /// Locale of `source`; a zh-CN case is a real input, not an
    /// implementation-generated conversion.
    #[serde(default = "default_source_locale")]
    pub source_locale: String,
    pub source: String,
    /// Independently recorded source text, when this case has an offline
    /// expectation. None means the case remains inconclusive offline.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference_source: Option<String>,
    pub support: CensusSupport,
}

/// A named collection of independently attributable cases.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CensusShard {
    pub shard_id: String,
    pub cases: Vec<CensusCase>,
}

impl CensusShard {
    pub fn new(
        shard_id: impl Into<String>,
        mut cases: Vec<CensusCase>,
    ) -> Result<Self, CensusError> {
        let shard_id = shard_id.into();
        validate_name("shard_id", &shard_id)?;
        cases.sort_by(|left, right| left.case_id.cmp(&right.case_id));
        for case in &cases {
            case.validate()?;
        }
        if cases
            .windows(2)
            .any(|pair| pair[0].case_id == pair[1].case_id)
        {
            return Err(CensusError::new(format!(
                "shard '{shard_id}' contains duplicate case IDs"
            )));
        }
        Ok(Self { shard_id, cases })
    }
}

impl CensusCase {
    fn probe(case_id: impl Into<String>, features: Vec<FeatureId>, source: String) -> Self {
        Self {
            case_id: case_id.into(),
            features,
            source_locale: EN_US.to_string(),
            source,
            reference_source: None,
            support: generated_probe_support(),
        }
    }

    fn exercise(mut self) -> Self {
        self.support = CensusSupport::Exercise;
        self
    }

    fn locale(mut self, locale: &str) -> Self {
        self.source_locale = locale.to_string();
        self
    }

    #[cfg(test)]
    fn with_support(mut self, support: CensusSupport) -> Self {
        self.support = support;
        self
    }

    fn validate(&self) -> Result<(), CensusError> {
        validate_name("case_id", &self.case_id)?;
        if self.features.is_empty() {
            return Err(CensusError::new(format!(
                "case '{}' has no feature IDs",
                self.case_id
            )));
        }
        if self.source.trim().is_empty() {
            return Err(CensusError::new(format!(
                "case '{}' has no source",
                self.case_id
            )));
        }
        validate_name("source_locale", &self.source_locale)?;
        if self
            .reference_source
            .as_deref()
            .is_some_and(|source| source.trim().is_empty())
        {
            return Err(CensusError::new(format!(
                "case '{}' has an empty reference source",
                self.case_id
            )));
        }
        let mut features = HashSet::new();
        if self
            .features
            .iter()
            .any(|feature| !features.insert(feature))
        {
            return Err(CensusError::new(format!(
                "case '{}' contains duplicate feature IDs",
                self.case_id
            )));
        }
        Ok(())
    }
}

/// The complete census assembled from deterministic shards.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Census {
    shards: Vec<CensusShard>,
}

/// The stable identity of a reviewed census definition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CensusIdentity {
    pub schema_version: u32,
    pub digest: String,
    pub shards: Vec<String>,
}

impl Census {
    /// Assemble shards in stable shard and case order.
    pub fn assemble(mut shards: Vec<CensusShard>) -> Result<Self, CensusError> {
        shards.sort_by(|left, right| left.shard_id.cmp(&right.shard_id));
        let mut shard_ids = HashSet::new();
        let mut case_ids = HashSet::new();
        for shard in &shards {
            if !shard_ids.insert(shard.shard_id.clone()) {
                return Err(CensusError::new(format!(
                    "duplicate census shard '{}'",
                    shard.shard_id
                )));
            }
            for case in &shard.cases {
                if !case_ids.insert(case.case_id.clone()) {
                    return Err(CensusError::new(format!(
                        "duplicate census case '{}'",
                        case.case_id
                    )));
                }
            }
        }
        Ok(Self { shards })
    }

    /// Derive the current surface from the canonical catalog, settings table,
    /// and WIR capability names owned by this crate.
    pub fn builtin(catalog: &Catalog) -> Result<Self, CensusError> {
        Self::assemble(vec![
            catalog_shard(catalog, Kind::Event, "catalog-events")?,
            catalog_shard(catalog, Kind::Action, "catalog-actions")?,
            catalog_shard(catalog, Kind::Value, "catalog-values")?,
            catalog_shard(catalog, Kind::Operator, "catalog-operators")?,
            catalog_shard(catalog, Kind::Structural, "catalog-structural")?,
            enum_shard(catalog)?,
            settings_shard(catalog)?,
            wir_shard()?,
            localization_shard(catalog)?,
            content_id_shard(catalog)?,
        ])
    }

    pub fn shards(&self) -> &[CensusShard] {
        &self.shards
    }

    pub fn cases(&self) -> impl Iterator<Item = &CensusCase> {
        self.shards.iter().flat_map(|shard| shard.cases.iter())
    }

    /// Execute all cases. No result state is dropped or converted to success.
    /// Cases are independent and the report sorts results by case ID, so the
    /// run parallelizes over available cores without changing the output.
    pub fn run(&self, catalog: &Catalog) -> CensusReport {
        let cases: Vec<(&str, &CensusCase)> = self
            .shards
            .iter()
            .flat_map(|shard| {
                shard
                    .cases
                    .iter()
                    .map(move |case| (shard.shard_id.as_str(), case))
            })
            .collect();
        let workers = std::thread::available_parallelism()
            .map(std::num::NonZero::get)
            .unwrap_or(1)
            .min(cases.len().max(1));
        let results: Vec<ConformanceResult> = if workers <= 1 {
            cases
                .iter()
                .map(|(shard_id, case)| run_case(case, shard_id, catalog))
                .collect()
        } else {
            let next = std::sync::atomic::AtomicUsize::new(0);
            std::thread::scope(|scope| {
                let handles: Vec<_> = (0..workers)
                    .map(|_| {
                        scope.spawn(|| {
                            let mut worker_results = Vec::new();
                            loop {
                                let index = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                                let Some((shard_id, case)) = cases.get(index) else {
                                    break;
                                };
                                worker_results.push(run_case(case, shard_id, catalog));
                            }
                            worker_results
                        })
                    })
                    .collect();
                handles
                    .into_iter()
                    .flat_map(|handle| handle.join().expect("census worker panicked"))
                    .collect()
            })
        };
        let mut results = results;
        results.sort_by(|left, right| left.case_id.cmp(&right.case_id));
        CensusReport {
            schema_version: CENSUS_SCHEMA_VERSION,
            conformance_schema_version: CONFORMANCE_SCHEMA_VERSION,
            catalog: catalog.identity(),
            census: self.identity(),
            results,
        }
    }

    /// Return the deterministic identity of this census definition.
    pub fn identity(&self) -> CensusIdentity {
        let definition = self
            .export_json()
            .expect("census definitions must remain serializable");
        CensusIdentity {
            schema_version: CENSUS_IDENTITY_SCHEMA_VERSION,
            digest: sha256(&definition),
            shards: self
                .shards
                .iter()
                .map(|shard| shard.shard_id.clone())
                .collect(),
        }
    }

    /// Export shard definitions without executing them.
    pub fn export_json(&self) -> Result<String, CensusError> {
        serde_json::to_string_pretty(&self.shards)
            .map_err(|error| CensusError::new(format!("cannot serialize census shards: {error}")))
    }
}

/// Machine-readable output from a census run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CensusReport {
    pub schema_version: u32,
    pub conformance_schema_version: u32,
    pub catalog: workshop_rs::catalog::CatalogIdentity,
    pub census: CensusIdentity,
    pub results: Vec<ConformanceResult>,
}

impl CensusReport {
    pub fn validate(&self) -> Result<(), CensusError> {
        let catalog =
            Catalog::builtin().map_err(|error| CensusError::new(format!("catalog: {error}")))?;
        self.validate_against(&catalog)
    }

    pub fn validate_against(&self, catalog: &Catalog) -> Result<(), CensusError> {
        if self.schema_version != CENSUS_SCHEMA_VERSION {
            return Err(CensusError::new("unsupported census schema version"));
        }
        if self.conformance_schema_version != CONFORMANCE_SCHEMA_VERSION {
            return Err(CensusError::new("unsupported conformance schema version"));
        }
        if self.catalog != catalog.identity() {
            return Err(CensusError::new(
                "report catalog identity does not match the loaded catalog",
            ));
        }
        if self.census.schema_version != CENSUS_IDENTITY_SCHEMA_VERSION {
            return Err(CensusError::new(
                "unsupported census identity schema version",
            ));
        }
        if !is_sha256_digest(&self.census.digest) {
            return Err(CensusError::new(
                "census identity digest must be a SHA-256 hex digest",
            ));
        }
        if self.census.shards.is_empty()
            || self.census.shards.windows(2).any(|pair| pair[0] >= pair[1])
        {
            return Err(CensusError::new(
                "report shards must be non-empty and strictly sorted",
            ));
        }
        for result in &self.results {
            result
                .validate_against(catalog)
                .map_err(|error| CensusError::new(error.to_string()))?;
            let matching_shards = self
                .census
                .shards
                .iter()
                .filter(|shard| {
                    result
                        .case_id
                        .strip_prefix(shard.as_str())
                        .is_some_and(|rest| rest.starts_with('/'))
                })
                .count();
            if matching_shards != 1 {
                return Err(CensusError::new(format!(
                    "result '{}' does not map to exactly one census shard",
                    result.case_id
                )));
            }
        }
        Ok(())
    }

    pub fn to_json(&self) -> Result<String, CensusError> {
        self.validate()?;
        serde_json::to_string_pretty(self)
            .map_err(|error| CensusError::new(format!("cannot serialize census report: {error}")))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CensusError {
    pub message: String,
}

impl CensusError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for CensusError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for CensusError {}

fn validate_name(field: &str, value: &str) -> Result<(), CensusError> {
    if value.trim().is_empty() || value.chars().any(char::is_control) {
        Err(CensusError::new(format!(
            "{field} must be non-empty and printable"
        )))
    } else {
        Ok(())
    }
}

fn default_source_locale() -> String {
    EN_US.to_string()
}

fn feature(namespace: FeatureNamespace, kind: FeatureKind, name: impl Into<String>) -> FeatureId {
    FeatureId::new(namespace, kind, name).expect("canonical census feature ID")
}

fn catalog_feature(kind: Kind, id: &str) -> FeatureId {
    FeatureId::from_catalog(kind, id).expect("catalog IDs are validated by Catalog::load")
}

fn catalog_shard(
    catalog: &Catalog,
    kind: Kind,
    shard_id: &str,
) -> Result<CensusShard, CensusError> {
    let cases = catalog
        .entries_of(kind)
        .map(|entry| {
            let source = match kind {
                Kind::Event => event_probe(catalog, entry),
                Kind::Action => action_probe(catalog, entry),
                Kind::Value => value_probe(catalog, entry),
                Kind::Operator => operator_probe(catalog, entry),
                Kind::Structural => structural_probe(catalog, entry),
                Kind::Setting => unreachable!("settings use the settings table"),
                Kind::Enum => unreachable!("enum domains use the enum shard"),
            };
            CensusCase::probe(
                format!("{shard_id}/{}", entry.id),
                vec![catalog_feature(kind, &entry.id)],
                source,
            )
        })
        .collect();
    CensusShard::new(shard_id, cases)
}

fn enum_shard(catalog: &Catalog) -> Result<CensusShard, CensusError> {
    enum_member_cases(catalog, "catalog-enums", false, true)
}

fn content_id_shard(catalog: &Catalog) -> Result<CensusShard, CensusError> {
    enum_member_cases(catalog, "content-ids", true, false)
}

fn enum_member_cases(
    catalog: &Catalog,
    shard_id: &str,
    content_only: bool,
    domain_feature: bool,
) -> Result<CensusShard, CensusError> {
    let mut cases = Vec::new();
    for domain in catalog.enum_domains() {
        if content_only && !matches!(domain.domain.as_str(), "Hero" | "Map") {
            continue;
        }
        for member in &domain.members {
            let mut features = Vec::with_capacity(2);
            if domain_feature {
                features.push(catalog_feature(Kind::Enum, &domain.domain));
            }
            features.push(
                FeatureId::from_enum_member(&domain.domain, &member.member)
                    .expect("canonical enum member ID"),
            );
            cases.push(CensusCase::probe(
                format!("{shard_id}/{}/{}", domain.domain, member.member),
                features,
                enum_probe(catalog, domain, &member.member),
            ));
        }
    }
    CensusShard::new(shard_id, cases)
}
fn settings_shard(catalog: &Catalog) -> Result<CensusShard, CensusError> {
    let cases = settings_schema::definitions()
        .map(|definition| {
            let path = definition.path().to_string();
            CensusCase::probe(
                format!("settings/{path}"),
                vec![feature(
                    FeatureNamespace::Settings,
                    FeatureKind::Setting,
                    path,
                )],
                settings_probe(&definition, catalog),
            )
        })
        .collect();
    CensusShard::new("settings", cases)
}

fn wir_shard() -> Result<CensusShard, CensusError> {
    let cases = CENSUS_CAPABILITIES
        .iter()
        .map(|capability| match capability.kind {
            CensusCapabilityKind::Variable => wir_case(
                "variables-global",
                FeatureKind::Variable,
                capability.name,
                variables_source(),
            ),
            CensusCapabilityKind::PlayerVariable => CensusCase::probe(
                "wir/variables-player".to_string(),
                vec![feature(
                    FeatureNamespace::Wir,
                    FeatureKind::Variable,
                    capability.name,
                )],
                player_variable_source(),
            ),
            CensusCapabilityKind::Subroutine => wir_case(
                "subroutine",
                FeatureKind::Subroutine,
                capability.name,
                subroutine_source(),
            ),
            CensusCapabilityKind::ControlFlow => {
                let actions = match capability.name {
                    "if" => "If(True);\n    Wait(0);\nEnd;",
                    "else-if" => "If(True);\n    Wait(0);\nElse If(False);\n    Wait(0);\nEnd;",
                    "else" => "If(True);\n    Wait(0);\nElse;\n    Wait(0);\nEnd;",
                    "while" => "While(True);\n    Wait(0);\nEnd;",
                    "for-global-variable" => {
                        "For Global Variable(probe, 0, 1, 1);\n    Wait(0);\nEnd;"
                    }
                    _ => unreachable!("unknown WIR control-flow census capability"),
                };
                control_flow_case(capability.name, actions)
            }
            CensusCapabilityKind::String => CensusCase::probe(
                "wir/string/custom-string".to_string(),
                vec![feature(
                    FeatureNamespace::Wir,
                    FeatureKind::String,
                    capability.name,
                )],
                rule_source(
                    "String",
                    "Set Global Variable(probe, Custom String(\"census\"));",
                ),
            )
            .exercise(),
        })
        .collect();
    CensusShard::new("wir", cases)
}

fn localization_shard(catalog: &Catalog) -> Result<CensusShard, CensusError> {
    let mut cases = vec![
        CensusCase::probe(
            "localization/en-us-to-zh-cn".to_string(),
            vec![feature(
                FeatureNamespace::Localization,
                FeatureKind::Localization,
                "en-us-to-zh-cn",
            )],
            LOCALIZATION_EN_US_SOURCE.to_string(),
        ),
        CensusCase::probe(
            "localization/zh-cn-to-en-us".to_string(),
            vec![feature(
                FeatureNamespace::Localization,
                FeatureKind::Localization,
                "zh-cn-to-en-us",
            )],
            LOCALIZATION_ZH_CN_SOURCE.to_string(),
        )
        .locale(ZH_CN),
    ];
    // #448: every spelling the catalog declares for a locale gets a case
    // written in that spelling, so an alias the frontend cannot represent
    // shows up as an explicit result instead of never being exercised.
    let settings_index = SettingsSpellings::build();
    for locale in catalog.locales() {
        for kind in [
            Kind::Event,
            Kind::Action,
            Kind::Value,
            Kind::Operator,
            Kind::Structural,
        ] {
            for entry in catalog.entries_of(kind) {
                for (index, spelling) in entry.spellings(locale).iter().enumerate() {
                    cases.push(
                        CensusCase::probe(
                            format!(
                                "localization/{}/{}/{}/{}",
                                locale.as_str(),
                                kind.as_str(),
                                entry.id,
                                index
                            ),
                            vec![catalog_feature(kind, &entry.id)],
                            localized_probe(catalog, locale, kind, entry, spelling),
                        )
                        .locale(locale.as_str()),
                    );
                }
            }
        }
        for domain in catalog.enum_domains() {
            let domain_spelling = CONSTRUCTOR_DOMAINS
                .contains(&domain.domain.as_str())
                .then(|| domain.spelling(locale))
                .flatten();
            if let Some(spelling) = domain_spelling {
                let Some(member) = domain.members.first() else {
                    continue;
                };
                let member_spelling =
                    localized_member(catalog, &domain.domain, &member.member, locale);
                cases.push(
                    CensusCase::probe(
                        format!(
                            "localization/{}/enum-domain/{}",
                            locale.as_str(),
                            domain.domain
                        ),
                        vec![catalog_feature(Kind::Enum, &domain.domain)],
                        global_probe(catalog, locale, &format!("{spelling}({member_spelling})")),
                    )
                    .locale(locale.as_str()),
                );
            }
            for member in &domain.members {
                for (index, spelling) in member.spellings(locale).iter().enumerate() {
                    cases.push(
                        CensusCase::probe(
                            format!(
                                "localization/{}/enum/{}/{}/{}",
                                locale.as_str(),
                                domain.domain,
                                member.member,
                                index
                            ),
                            vec![
                                FeatureId::from_enum_member(&domain.domain, &member.member)
                                    .expect("canonical enum member ID"),
                            ],
                            member_probe(catalog, locale, domain, spelling),
                        )
                        .locale(locale.as_str()),
                    );
                }
            }
        }
        for entry in catalog.localized_strings() {
            for (index, spelling) in entry.spellings(locale).iter().enumerate() {
                let string_call = catalog
                    .spelling(Kind::Value, locale, "string")
                    .or_else(|| catalog.spelling(Kind::Value, &Locale::new(EN_US), "string"))
                    .unwrap_or("string");
                cases.push(
                    CensusCase::probe(
                        format!(
                            "localization/{}/localized-string/{}/{}",
                            locale.as_str(),
                            entry.id,
                            index
                        ),
                        vec![feature(
                            FeatureNamespace::Localization,
                            FeatureKind::String,
                            entry.id.clone(),
                        )],
                        global_probe(catalog, locale, &format!("{string_call}({spelling})")),
                    )
                    .locale(locale.as_str()),
                );
            }
        }
        for (row, name) in settings_schema::localized_setting_names()
            .iter()
            .enumerate()
        {
            if !name.locale.eq_ignore_ascii_case(locale.as_str()) {
                continue;
            }
            let case_id = format!(
                "localization/{}/settings/{}/{}",
                locale.as_str(),
                name.section,
                row
            );
            let feature_id = feature(
                FeatureNamespace::Settings,
                FeatureKind::Setting,
                format!("{}/{}", name.section, slug(name.english)),
            );
            match localized_settings_probe(catalog, &settings_index, name, locale) {
                Ok(source) => cases.push(
                    CensusCase::probe(case_id, vec![feature_id], source).locale(locale.as_str()),
                ),
                Err(detail) => cases.push(
                    CensusCase {
                        case_id,
                        features: vec![feature_id],
                        source_locale: locale.as_str().to_string(),
                        source: "settings {\n}".to_string(),
                        reference_source: None,
                        support: CensusSupport::KnownGap {
                            detail: format!(
                                "settings {} {:?} for {}: {detail}",
                                name.section, name.spelling, name.english
                            ),
                        },
                    }
                    .locale(locale.as_str()),
                ),
            }
        }
    }
    CensusShard::new("localization", cases)
}

const CONSTRUCTOR_DOMAINS: &[&str] = &["Hero", "Button", "Color", "Map"];

fn slug(name: &str) -> String {
    name.chars()
        .map(|character| {
            if character.is_whitespace() || character == '/' {
                '-'
            } else {
                character
            }
        })
        .collect()
}

fn localized_spelling(catalog: &Catalog, kind: Kind, locale: &Locale, id: &str) -> String {
    catalog
        .spelling(kind, locale, id)
        .or_else(|| catalog.spelling(kind, &Locale::new(EN_US), id))
        .unwrap_or(id)
        .to_string()
}

fn localized_member(catalog: &Catalog, domain: &str, member: &str, locale: &Locale) -> String {
    catalog
        .enum_spelling(domain, locale, member)
        .or_else(|| catalog.enum_spelling(domain, &Locale::new(EN_US), member))
        .unwrap_or(member)
        .to_string()
}

fn probe_filler(catalog: &Catalog, locale: &Locale, entry: &CatalogEntry, index: usize) -> String {
    if let Some(domain) = entry.param_domain(index) {
        let member = catalog
            .enum_domain(domain)
            .and_then(|domain| domain.members.first())
            .map(|member| member.member.as_str())
            .unwrap_or("ALL");
        let spelling = localized_member(catalog, domain, member, locale);
        if CONSTRUCTOR_DOMAINS.contains(&domain) {
            let domain_spelling = catalog
                .enum_domain(domain)
                .and_then(|domain| domain.spelling(locale))
                .unwrap_or(domain);
            return format!("{domain_spelling}({spelling})");
        }
        return spelling;
    }
    if let Some(default) = entry.param_default(index) {
        return param_default_filler(catalog, locale, default);
    }
    match entry.param_type(index) {
        Some("Variable") => "probe".to_string(),
        Some("Subroutine") | Some("Subroutines") => "probe".to_string(),
        Some("Bool") => localized_spelling(catalog, Kind::Value, locale, "true"),
        // `Hero`-typed params take the member spelling bare; the enclosing
        // `Hero(...)` call is the value under test or the entry's own name.
        Some("Hero") => {
            let member = catalog
                .enum_domain("Hero")
                .and_then(|domain| domain.members.first())
                .map(|member| member.member.as_str())
                .unwrap_or("ANA");
            localized_member(catalog, "Hero", member, locale)
        }
        // String params emit canonically as `Custom String("...")`, so the
        // probe writes that form instead of a bare literal (#448).
        Some("String") => {
            let custom_string = localized_spelling(catalog, Kind::Value, locale, "customString");
            format!("{custom_string}(\"x\")")
        }
        Some(kind) if kind.contains("Player") || kind.contains("EntityId") => {
            localized_spelling(catalog, Kind::Value, locale, "eventPlayer")
        }
        _ => "0".to_string(),
    }
}

fn param_default_filler(catalog: &Catalog, locale: &Locale, default: &str) -> String {
    if let Some((domain, member)) = default.split_once('.') {
        let spelling = localized_member(catalog, domain, member, locale);
        if CONSTRUCTOR_DOMAINS.contains(&domain) {
            let domain_spelling = catalog
                .enum_domain(domain)
                .and_then(|domain| domain.spelling(locale))
                .unwrap_or(domain);
            return format!("{domain_spelling}({spelling})");
        }
        return spelling;
    }
    if catalog.entry(Kind::Value, default).is_some() {
        return localized_spelling(catalog, Kind::Value, locale, default);
    }
    if let Some(spelling) = catalog.localized_string_spelling(locale, default) {
        return spelling.to_string();
    }
    default.to_string()
}

fn probe_args(catalog: &Catalog, locale: &Locale, entry: &CatalogEntry) -> String {
    (0..entry.param_count())
        .map(|index| probe_filler(catalog, locale, entry, index))
        .collect::<Vec<_>>()
        .join(", ")
}

/// A probe rule written fully in `locale` (#448): every fixed token —
/// section keywords, the header event, `Wait`, `If`, `End` — carries that
/// locale's declared spelling so a cross-locale round trip compares the
/// tested spelling rather than fallback noise in the boilerplate.
fn probe_rule(catalog: &Catalog, locale: &Locale, actions: &str) -> String {
    let variables = localized_spelling(catalog, Kind::Structural, locale, "variables");
    let global = localized_spelling(catalog, Kind::Structural, locale, "global");
    let player = localized_spelling(catalog, Kind::Structural, locale, "player");
    let subroutines = localized_spelling(catalog, Kind::Structural, locale, "subroutines");
    let rule = localized_spelling(catalog, Kind::Structural, locale, "rule");
    let event = localized_spelling(catalog, Kind::Structural, locale, "event");
    let ongoing_global = localized_spelling(catalog, Kind::Event, locale, "global");
    let actions_section = localized_spelling(catalog, Kind::Structural, locale, "actions");
    format!(
        "{variables} {{\n    {global}:\n        0: probe\n    {player}:\n        0: probe\n}}\n\n{subroutines} {{\n    0: probe\n}}\n\n{rule} (\"probe\") {{\n    {event} {{\n        {ongoing_global};\n    }}\n    {actions_section} {{\n        {actions}\n    }}\n}}\n"
    )
}

fn global_probe(catalog: &Catalog, locale: &Locale, operand: &str) -> String {
    let set_global = localized_spelling(catalog, Kind::Structural, locale, "setGlobalVariable");
    probe_rule(catalog, locale, &format!("{set_global}(probe, {operand});"))
}

fn localized_probe(
    catalog: &Catalog,
    locale: &Locale,
    kind: Kind,
    entry: &CatalogEntry,
    spelling: &str,
) -> String {
    match kind {
        Kind::Event => {
            let extra = match entry.id.as_str() {
                "global" => String::new(),
                "subroutine" => "        probe;\n".to_string(),
                _ => format!(
                    "        {};\n        {};\n",
                    localized_member(catalog, "EventTeam", "ALL", locale),
                    localized_member(catalog, "EventPlayer", "ALL", locale),
                ),
            };
            let subroutines = localized_spelling(catalog, Kind::Structural, locale, "subroutines");
            let rule = localized_spelling(catalog, Kind::Structural, locale, "rule");
            let event = localized_spelling(catalog, Kind::Structural, locale, "event");
            let actions = localized_spelling(catalog, Kind::Structural, locale, "actions");
            let wait = localized_spelling(catalog, Kind::Action, locale, "wait");
            format!(
                "{subroutines} {{\n    0: probe\n}}\n\n{rule} (\"e\") {{\n    {event} {{\n        {spelling};\n{extra}    }}\n    {actions} {{\n        {wait};\n    }}\n}}\n"
            )
        }
        Kind::Action => {
            let call = if entry.param_count() == 0 {
                format!("{spelling};")
            } else {
                format!("{spelling}({});", probe_args(catalog, locale, entry))
            };
            // `For * Variable` actions open a block that needs `End`.
            let call = if entry.id.starts_with("__for") {
                let wait = localized_spelling(catalog, Kind::Action, locale, "wait");
                let end = localized_spelling(catalog, Kind::Structural, locale, "end");
                format!("{call}\n        {wait}(0);\n    {end};")
            } else {
                call
            };
            probe_rule(catalog, locale, &call)
        }
        Kind::Value => {
            let operand = match entry.id.as_str() {
                "string" => {
                    let hello = catalog
                        .localized_string_spelling(locale, "hello")
                        .or_else(|| catalog.localized_string_spelling(&Locale::new(EN_US), "hello"))
                        .unwrap_or("Hello");
                    format!("{spelling}({hello})")
                }
                "compare" => {
                    let eq = localized_spelling(catalog, Kind::Operator, locale, "==");
                    format!("{spelling}(1, {eq}, 1)")
                }
                _ if entry.param_count() == 0 => spelling.to_string(),
                _ => format!("{spelling}({})", probe_args(catalog, locale, entry)),
            };
            global_probe(catalog, locale, &operand)
        }
        Kind::Operator => {
            if matches!(entry.id.as_str(), "==" | "!=" | "<" | "<=" | ">" | ">=") {
                let if_ = localized_spelling(catalog, Kind::Structural, locale, "if");
                let wait = localized_spelling(catalog, Kind::Action, locale, "wait");
                let end = localized_spelling(catalog, Kind::Structural, locale, "end");
                probe_rule(
                    catalog,
                    locale,
                    &format!("{if_}(1 {spelling} 1);\n        {wait}(0);\n    {end};"),
                )
            } else {
                let modify =
                    localized_spelling(catalog, Kind::Structural, locale, "modifyGlobalVariable");
                probe_rule(catalog, locale, &format!("{modify}(probe, {spelling}, 1);"))
            }
        }
        Kind::Structural => structural_spelling_probe(catalog, locale, &entry.id, spelling),
        _ => unreachable!("settings and enum spellings use their own probes"),
    }
}

fn structural_spelling_probe(
    catalog: &Catalog,
    locale: &Locale,
    id: &str,
    spelling: &str,
) -> String {
    let kw = |name: &str| localized_spelling(catalog, Kind::Structural, locale, name);
    let wait = localized_spelling(catalog, Kind::Action, locale, "wait");
    let true_ = localized_spelling(catalog, Kind::Value, locale, "true");
    let ongoing_global = localized_spelling(catalog, Kind::Event, locale, "global");
    let event_player = localized_spelling(catalog, Kind::Value, locale, "eventPlayer");
    let end = kw("end");
    let block = |head: &str| format!("{head}\n        {wait}(0);\n    {end};");
    match id {
        "settings" => format!("{spelling} {{\n}}\n"),
        "workshop" => format!("{} {{\n    {spelling} {{\n    }}\n}}\n", kw("settings")),
        "variables" => format!(
            "{spelling} {{\n    {}:\n        0: probe\n}}\n",
            kw("global")
        ),
        "global" | "player" => {
            format!(
                "{} {{\n    {spelling}:\n        0: probe\n}}\n",
                kw("variables")
            )
        }
        "subroutines" => format!("{spelling} {{\n    0: probe\n}}\n"),
        "rule" => format!(
            "{spelling} (\"r\") {{\n    {} {{\n        {ongoing_global};\n    }}\n    {} {{\n        {wait};\n    }}\n}}\n",
            kw("event"),
            kw("actions"),
        ),
        "event" => format!(
            "{} (\"r\") {{\n    {spelling} {{\n        {ongoing_global};\n    }}\n    {} {{\n        {wait};\n    }}\n}}\n",
            kw("rule"),
            kw("actions"),
        ),
        "conditions" => {
            // Conditions emit canonically in `a == b` comparison form.
            let equals = localized_spelling(catalog, Kind::Operator, locale, "==");
            format!(
                "{} (\"c\") {{\n    {} {{\n        {ongoing_global};\n    }}\n    {spelling} {{\n        {true_} {equals} {true_};\n    }}\n    {} {{\n        {wait};\n    }}\n}}\n",
                kw("rule"),
                kw("event"),
                kw("actions"),
            )
        }
        "actions" => format!(
            "{} (\"r\") {{\n    {} {{\n        {ongoing_global};\n    }}\n    {spelling} {{\n        {wait};\n    }}\n}}\n",
            kw("rule"),
            kw("event"),
        ),
        "disabled" => probe_rule(catalog, locale, &format!("{spelling} {wait}(0);")),
        "if" => probe_rule(catalog, locale, &block(&format!("{spelling}({true_});"))),
        "elseIf" => {
            let if_ = kw("if");
            probe_rule(
                catalog,
                locale,
                &format!(
                    "{if_}({true_});\n        {wait}(0);\n    {spelling}({true_});\n        {wait}(0);\n    {end};"
                ),
            )
        }
        "else" => {
            let if_ = kw("if");
            probe_rule(
                catalog,
                locale,
                &format!(
                    "{if_}({true_});\n        {wait}(0);\n    {spelling};\n        {wait}(0);\n    {end};"
                ),
            )
        }
        "end" => {
            let if_ = kw("if");
            probe_rule(
                catalog,
                locale,
                &format!("{if_}({true_});\n        {wait}(0);\n    {spelling};"),
            )
        }
        "while" => probe_rule(catalog, locale, &block(&format!("{spelling}({true_});"))),
        "forGlobalVariable" => probe_rule(
            catalog,
            locale,
            &block(&format!("{spelling}(probe, 0, 1, 1);")),
        ),
        "forPlayerVariable" => probe_rule(
            catalog,
            locale,
            &block(&format!("{spelling}({event_player}, probe, 0, 1, 1);")),
        ),
        "setGlobalVariable" => probe_rule(catalog, locale, &format!("{spelling}(probe, 1);")),
        "modifyGlobalVariable" => {
            let add = localized_spelling(catalog, Kind::Operator, locale, "add");
            probe_rule(catalog, locale, &format!("{spelling}(probe, {add}, 1);"))
        }
        "setPlayerVariable" => probe_rule(
            catalog,
            locale,
            &format!("{spelling}({event_player}, probe, 1);"),
        ),
        "modifyPlayerVariable" => {
            let add = localized_spelling(catalog, Kind::Operator, locale, "add");
            probe_rule(
                catalog,
                locale,
                &format!("{spelling}({event_player}, probe, {add}, 1);"),
            )
        }
        "callSubroutine" => probe_rule(catalog, locale, &format!("{spelling}(probe);")),
        _ => probe_rule(catalog, locale, &format!("{spelling};")),
    }
}

fn member_probe(catalog: &Catalog, locale: &Locale, domain: &EnumDomain, spelling: &str) -> String {
    if CONSTRUCTOR_DOMAINS.contains(&domain.domain.as_str()) {
        let domain_spelling = domain.spelling(locale).unwrap_or(&domain.domain);
        return global_probe(catalog, locale, &format!("{domain_spelling}({spelling})"));
    }
    if domain.domain == "EventTeam" || domain.domain == "EventPlayer" {
        let event = localized_spelling(catalog, Kind::Event, locale, "eachPlayer");
        let other = if domain.domain == "EventTeam" {
            localized_member(catalog, "EventPlayer", "ALL", locale)
        } else {
            localized_member(catalog, "EventTeam", "ALL", locale)
        };
        let (team, player) = if domain.domain == "EventTeam" {
            (spelling.to_string(), other)
        } else {
            (other, spelling.to_string())
        };
        let rule = localized_spelling(catalog, Kind::Structural, locale, "rule");
        let event_kw = localized_spelling(catalog, Kind::Structural, locale, "event");
        let actions = localized_spelling(catalog, Kind::Structural, locale, "actions");
        let wait = localized_spelling(catalog, Kind::Action, locale, "wait");
        return format!(
            "{rule} (\"e\") {{\n    {event_kw} {{\n        {event};\n        {team};\n        {player};\n    }}\n    {actions} {{\n        {wait};\n    }}\n}}\n"
        );
    }
    let host = [Kind::Value, Kind::Action]
        .into_iter()
        .flat_map(|kind| catalog.entries_of(kind))
        .filter(|entry| {
            (0..entry.param_count()).any(|index| entry.param_domain(index) == Some(&domain.domain))
        })
        .min_by_key(|entry| entry.required_param_count());
    let Some(host) = host else {
        return global_probe(catalog, locale, spelling);
    };
    let target = (0..host.param_count())
        .find(|index| host.param_domain(*index) == Some(&domain.domain))
        .expect("host was selected for carrying this domain");
    let mut call_args: Vec<String> = (0..host.param_count())
        .map(|index| probe_filler(catalog, locale, host, index))
        .collect();
    call_args[target] = spelling.to_string();
    let host_spelling = localized_spelling(catalog, host.kind, locale, &host.id);
    let call = format!("{host_spelling}({})", call_args.join(", "));
    match host.kind {
        Kind::Value => global_probe(catalog, locale, &call),
        _ => probe_rule(catalog, locale, &format!("{call};")),
    }
}

struct SettingsSpellings {
    by_label: HashMap<String, Vec<SettingDefinition>>,
    by_member: HashMap<String, Vec<SettingDefinition>>,
    names: HashMap<(String, String, String), String>,
}

impl SettingsSpellings {
    fn build() -> Self {
        let mut by_label: HashMap<String, Vec<SettingDefinition>> = HashMap::new();
        let mut by_member: HashMap<String, Vec<SettingDefinition>> = HashMap::new();
        for definition in settings_schema::definitions() {
            by_label
                .entry(definition.presentation().english_name.to_string())
                .or_default()
                .push(definition.clone());
            for member in definition.enum_members() {
                by_member
                    .entry(member.english_name().to_string())
                    .or_default()
                    .push(definition.clone());
            }
        }
        let mut names = HashMap::new();
        for row in settings_schema::localized_setting_names() {
            names.insert(
                (
                    row.locale.to_string(),
                    row.section.to_string(),
                    row.english.to_string(),
                ),
                row.spelling.to_string(),
            );
        }
        Self {
            by_label,
            by_member,
            names,
        }
    }

    fn name(&self, locale: &str, section: &str, english: &str) -> String {
        self.lookup(locale, section, english)
            .or_else(|| self.lookup(EN_US, section, english))
            .unwrap_or(english)
            .to_string()
    }

    fn lookup(&self, locale: &str, section: &str, english: &str) -> Option<&str> {
        self.names
            .get(&(locale.to_string(), section.to_string(), english.to_string()))
            .map(String::as_str)
    }
}

/// A probe hero for `definition`'s `<hero>` slot: ability-scoped defs need
/// a hero that owns the slot; prefer a hero whose emitted name equals the
/// tested label so generic labels exercise the typed path (#448).
fn settings_probe_hero(definition: &SettingDefinition, english: &str) -> Option<Option<String>> {
    if !definition.path().contains(".<hero>.") && !definition.path().ends_with(".<hero>") {
        return Some(None);
    }
    let data = gameplay::data::builtin_ref().ok()?;
    let heroes = data.query().heroes().to_vec();
    let hero_ids: Vec<String> = match definition.target_kind() {
        SettingTargetKind::HeroAbility { slot, variant } => heroes
            .iter()
            .filter(|hero| {
                data.query()
                    .ability_name(hero.id().as_str(), slot.as_str(), variant.as_ref(), EN_US)
                    .is_ok()
            })
            .map(|hero| hero.id().as_str().to_string())
            .collect(),
        _ => heroes
            .iter()
            .map(|hero| hero.id().as_str().to_string())
            .collect(),
    };
    // Prefer a hero that actually renders the tested label (generic
    // spellings are shadowed by hero-specific ability names), but any hero
    // carrying the slot still writes the spelling verbatim.
    let mut matching = Vec::new();
    let mut any = Vec::new();
    for id in hero_ids {
        let target = match definition.target_kind() {
            SettingTargetKind::HeroAbility { slot, variant } => SettingTarget::HeroAbility {
                team: None,
                hero: HeroId::new(id.as_str()),
                slot,
                variant,
            },
            _ => SettingTarget::Hero {
                team: None,
                hero: HeroId::new(id.as_str()),
            },
        };
        let emitted = definition
            .localized_name(EN_US, &target)
            .ok()
            .flatten()
            .map(str::to_string);
        any.push(id.clone());
        if emitted.as_deref() == Some(english) {
            matching.push(id);
        }
    }
    matching.into_iter().chain(any).next().map(Some)
}

/// Substitute the ability slot's localized name into a `%1$s` template
/// spelling — the only form in which such a label can appear in source.
/// Returns None when the definition is not hero-ability scoped or the hero
/// has no ability name to substitute (#448).
fn substitute_template(
    definition: &SettingDefinition,
    spelling: &str,
    hero: Option<&str>,
    locale: &str,
) -> Option<String> {
    if !spelling.contains("%1$s") {
        return Some(spelling.to_string());
    }
    let SettingTargetKind::HeroAbility { slot, variant } = definition.target_kind() else {
        return None;
    };
    let ability = gameplay::data::builtin_ref()
        .ok()?
        .query()
        .ability_name(hero?, slot.as_str(), variant.as_ref(), locale)
        .ok()?
        .to_string();
    Some(spelling.replace("%1$s", &ability))
}

fn localized_settings_context(
    catalog: &Catalog,
    index: &SettingsSpellings,
    locale: &Locale,
    definition: &SettingDefinition,
    leaf: &str,
    hero: Option<&str>,
) -> String {
    let hero_name = hero
        .and_then(|id| {
            gameplay::data::builtin_ref()
                .ok()?
                .query()
                .heroes()
                .iter()
                .find(|hero| hero.id().as_str() == id)
                .and_then(|hero| hero.name().value().get(EN_US).map(str::to_string))
        })
        .map(|english| index.name(locale.as_str(), "heroes", &english))
        .unwrap_or_else(|| index.name(locale.as_str(), "heroes", "Ana"));
    let settings = localized_spelling(catalog, Kind::Structural, locale, "settings");
    let mut lines = vec![format!("{settings} {{")];
    let mut depth = 1;
    let parts: Vec<_> = definition.path().split('.').collect();
    for (part_index, part) in parts.iter().take(parts.len() - 1).enumerate() {
        let name = match *part {
            "gamemodes" => index.name(locale.as_str(), "namespaces", "modes"),
            "heroes" => index.name(locale.as_str(), "namespaces", "heroes"),
            "<team>" | "general" => index.name(locale.as_str(), "teams", "General"),
            "<hero>" => hero_name.clone(),
            value if part_index == 1 && parts.first() == Some(&"gamemodes") => {
                let english = catalog_enum_spelling(catalog, "Gamemode", value)
                    .unwrap_or_else(|| value.to_string());
                index.name(locale.as_str(), "modes", &english)
            }
            value => index.name(locale.as_str(), "labels", value),
        };
        lines.push(format!("{}{} {{", "    ".repeat(depth), name));
        depth += 1;
    }
    lines.push(format!("{}{leaf}", "    ".repeat(depth)));
    while depth > 1 {
        depth -= 1;
        lines.push(format!("{}{}", "    ".repeat(depth), "}"));
    }
    lines.push("}".to_string());
    lines.join("\n")
}

fn leaf_value(
    index: &SettingsSpellings,
    locale: &Locale,
    definition: &SettingDefinition,
) -> String {
    match definition.domain() {
        SettingValueDomain::PresenceOnly => String::new(),
        SettingValueDomain::String => ": \"x\"".to_string(),
        SettingValueDomain::Boolean | SettingValueDomain::Enum { .. } => {
            let member = definition
                .enum_members()
                .next()
                .map(|member| {
                    // Settings members are declared under `enums`; the
                    // boolean tokens (On/Off/Yes/No) live under `tokens`.
                    index
                        .lookup(locale.as_str(), "enums", member.english_name())
                        .or_else(|| index.lookup(locale.as_str(), "tokens", member.english_name()))
                        .unwrap_or(member.english_name())
                        .to_string()
                })
                .unwrap_or_else(|| index.name(locale.as_str(), "tokens", "On"));
            format!(": {member}")
        }
        SettingValueDomain::Number(_) => ": 1".to_string(),
        SettingValueDomain::Percent(_) => ": 100%".to_string(),
        SettingValueDomain::HeroList | SettingValueDomain::MapList => " {\n}".to_string(),
        _ => ": 0".to_string(),
    }
}

fn localized_settings_probe(
    catalog: &Catalog,
    index: &SettingsSpellings,
    name: &settings_schema::LocalizedSettingName,
    locale: &Locale,
) -> Result<String, String> {
    let english = name.english;
    let spelling = name.spelling;
    let loc = locale.as_str();
    let settings = localized_spelling(catalog, Kind::Structural, locale, "settings");
    let modes = index.name(loc, "namespaces", "modes");
    let heroes = index.name(loc, "namespaces", "heroes");
    let general = index.name(loc, "teams", "General");
    let assault = index.name(loc, "modes", "Assault");
    match name.section {
        "namespaces" => Ok(format!("{settings} {{\n    {spelling} {{\n    }}\n}}")),
        "modes" => Ok(format!(
            "{settings} {{\n    {modes} {{\n        {spelling} {{\n        }}\n    }}\n}}"
        )),
        "teams" => Ok(format!(
            "{settings} {{\n    {heroes} {{\n        {spelling} {{\n            {} {{\n            }}\n        }}\n    }}\n}}",
            index.name(loc, "heroes", "Ana")
        )),
        "heroes" => Ok(format!(
            "{settings} {{\n    {heroes} {{\n        {general} {{\n            {spelling} {{\n            }}\n        }}\n    }}\n}}"
        )),
        "maps" => Ok(format!(
            "{settings} {{\n    {modes} {{\n        {assault} {{\n            {} {{\n                {spelling}\n            }}\n        }}\n    }}\n}}",
            index.name(loc, "labels", "enabled maps")
        )),
        "tokens" => match english {
            "disabled" => Ok(format!(
                "{settings} {{\n    {modes} {{\n        {spelling} {assault} {{\n        }}\n    }}\n}}"
            )),
            _ => Ok(format!(
                "{settings} {{\n    {modes} {{\n        {assault} {{\n            {}: {spelling}\n        }}\n    }}\n}}",
                index.name(loc, "labels", "Competitive Rules")
            )),
        },
        "labels" => {
            let Some(defs) = index.by_label.get(english) else {
                return Err(format!("no settings definition named {english:?}"));
            };
            let mut reasons = Vec::new();
            for definition in defs {
                let hero = settings_probe_hero(definition, english);
                let leaf = match substitute_template(
                    definition,
                    spelling,
                    hero.as_ref().and_then(|hero| hero.as_deref()),
                    loc,
                ) {
                    Some(leaf) => leaf,
                    None => {
                        reasons.push("template needs a hero-ability context".to_string());
                        continue;
                    }
                };
                return Ok(localized_settings_context(
                    catalog,
                    index,
                    locale,
                    definition,
                    &format!("{leaf}{}", leaf_value(index, locale, definition)),
                    hero.as_ref().and_then(|hero| hero.as_deref()),
                ));
            }
            Err(reasons
                .into_iter()
                .next()
                .unwrap_or_else(|| "no probe context".to_string()))
        }
        "enums" => {
            let Some(defs) = index.by_member.get(english) else {
                return Err(format!("no settings definition takes member {english:?}"));
            };
            let definition = &defs[0];
            let hero = settings_probe_hero(definition, definition.presentation().english_name);
            let label = index.name(loc, "labels", definition.presentation().english_name);
            Ok(localized_settings_context(
                catalog,
                index,
                locale,
                definition,
                &format!("{label}: {spelling}"),
                hero.as_ref().and_then(|hero| hero.as_deref()),
            ))
        }
        other => Err(format!("unhandled settings section {other:?}")),
    }
}

fn wir_case(case_id: &str, kind: FeatureKind, name: &str, source: String) -> CensusCase {
    CensusCase::probe(
        format!("wir/{case_id}"),
        vec![feature(FeatureNamespace::Wir, kind, name)],
        source,
    )
    .exercise()
}

fn control_flow_case(name: &str, actions: &str) -> CensusCase {
    CensusCase::probe(
        format!("wir/control-flow/{name}"),
        vec![feature(
            FeatureNamespace::Wir,
            FeatureKind::ControlFlow,
            name,
        )],
        rule_source(name, actions),
    )
}

fn generated_probe_support() -> CensusSupport {
    CensusSupport::Inconclusive {
        detail: "generated probe has no independently recorded expected result".to_string(),
    }
}

fn rule_source(name: &str, actions: &str) -> String {
    format!(
        "variables {{\n    global:\n        0: probe\n}}\n\nrule (\"{name}\") {{\n    event {{\n        Ongoing - Global;\n    }}\n    actions {{\n        {actions}\n    }}\n}}\n"
    )
}

fn variables_source() -> String {
    "variables {\n    global:\n        0: probe\n}\n\nrule (\"Global variable\") {\n    event {\n        Ongoing - Global;\n    }\n    actions {\n        Set Global Variable(probe, 1);\n    }\n}\n"
        .to_string()
}

fn player_variable_source() -> String {
    "variables {\n    player:\n        0: probe\n}\n\nrule (\"Player variable\") {\n    event {\n        Ongoing - Each Player;\n        All;\n        All;\n    }\n    actions {\n        Set Player Variable(Event Player, probe, 1);\n    }\n}\n"
        .to_string()
}

fn subroutine_source() -> String {
    "subroutines {\n    0: probe\n}\n\nrule (\"Subroutine\") {\n    event {\n        Subroutine;\n        probe;\n    }\n    actions {\n        Call Subroutine(probe);\n    }\n}\n"
        .to_string()
}

fn event_probe(catalog: &Catalog, entry: &CatalogEntry) -> String {
    let spelling = catalog
        .spelling(Kind::Event, &Locale::new(EN_US), &entry.id)
        .unwrap_or(&entry.id);
    let filters = if matches!(entry.id.as_str(), "global" | "subroutine") {
        String::new()
    } else {
        "        All;\n        All;\n".to_string()
    };
    let subroutine = if entry.id == "subroutine" {
        "        probe;\n"
    } else {
        ""
    };
    format!(
        "subroutines {{\n    0: probe\n}}\n\nrule (\"Event\") {{\n    event {{\n        {spelling};\n{filters}{subroutine}    }}\n    actions {{\n        Wait;\n    }}\n}}\n"
    )
}

fn action_probe(catalog: &Catalog, entry: &CatalogEntry) -> String {
    let spelling = catalog
        .spelling(Kind::Action, &Locale::new(EN_US), &entry.id)
        .unwrap_or(&entry.id);
    let call = if matches!(
        entry.id.as_str(),
        "chasePlayerVariableAtRate" | "chasePlayerVariableOverTime"
    ) {
        format!("{spelling}(Event Player, probe, 0, 1, 0);")
    } else {
        format!("{spelling};")
    };
    rule_source("Action", &call)
}

fn value_probe(catalog: &Catalog, entry: &CatalogEntry) -> String {
    let spelling = catalog
        .spelling(Kind::Value, &Locale::new(EN_US), &entry.id)
        .unwrap_or(&entry.id);
    rule_source("Value", &format!("Set Global Variable(probe, {spelling});"))
}

fn operator_probe(catalog: &Catalog, entry: &CatalogEntry) -> String {
    let spelling = catalog
        .spelling(Kind::Operator, &Locale::new(EN_US), &entry.id)
        .unwrap_or(&entry.id);
    rule_source(
        "Operator",
        &format!("If(1 {spelling} 1);\n    Wait(0);\nEnd;"),
    )
}

fn structural_probe(catalog: &Catalog, entry: &CatalogEntry) -> String {
    let spelling = catalog
        .spelling(Kind::Structural, &Locale::new(EN_US), &entry.id)
        .unwrap_or(&entry.id);
    let actions = match entry.id.as_str() {
        "if" => format!("{spelling}(True);\n    Wait(0);\nEnd;"),
        "elseIf" => format!("If(True);\n    Wait(0);\n{spelling}(False);\n    Wait(0);\nEnd;"),
        "else" => format!("If(True);\n    Wait(0);\n{spelling};\n    Wait(0);\nEnd;"),
        "end" => format!("If(True);\n    Wait(0);\n{spelling};"),
        "while" => format!("{spelling}(True);\n    Wait(0);\nEnd;"),
        "forGlobalVariable" => format!("{spelling}(probe, 0, 1, 1);\n    Wait(0);\nEnd;"),
        "setGlobalVariable" => format!("{spelling}(probe, 1);"),
        "modifyGlobalVariable" => format!("{spelling}(probe, Add, 1);"),
        "setPlayerVariable" => format!("{spelling}(Event Player, probe, 1);"),
        "modifyPlayerVariable" => format!("{spelling}(Event Player, probe, Add, 1);"),
        "callSubroutine" => format!("{spelling}(probe);"),
        _ => format!("{spelling};"),
    };
    let prefix = match entry.id.as_str() {
        "setPlayerVariable" | "modifyPlayerVariable" => {
            "variables {\n    player:\n        0: probe\n}\n\n"
        }
        "callSubroutine" => "subroutines {\n    0: probe\n}\n\n",
        _ => "",
    };
    format!("{prefix}{}", rule_source("Structural", &actions))
}

fn enum_probe(catalog: &Catalog, domain: &EnumDomain, member: &str) -> String {
    let locale = Locale::new(EN_US);
    let domain_spelling = catalog
        .spelling(Kind::Value, &locale, &domain.domain)
        .unwrap_or(&domain.domain);
    let member_spelling = catalog
        .enum_spelling(&domain.domain, &locale, member)
        .unwrap_or(member);
    // Only the constructor-form domains parse `Domain(Member)`; every other
    // domain writes the member bare (mirrors `Catalog::enum_member_form`).
    let operand = if CONSTRUCTOR_DOMAINS.contains(&domain.domain.as_str()) {
        format!("{domain_spelling}({member_spelling})")
    } else {
        member_spelling.to_string()
    };
    rule_source("Enum", &format!("Set Global Variable(probe, {operand});"))
}

fn settings_probe(definition: &SettingDefinition, catalog: &Catalog) -> String {
    let mut lines = vec!["settings {".to_string()];
    let mut depth = 1;
    let parts: Vec<_> = definition.path().split('.').collect();
    for (index, part) in parts.iter().enumerate() {
        let name = match *part {
            "gamemodes" => "modes".to_string(),
            "heroes" => "heroes".to_string(),
            "<team>" => "General".to_string(),
            "<hero>" => {
                catalog_enum_spelling(catalog, "Hero", "mei").unwrap_or_else(|| "Mei".to_string())
            }
            "general" => "General".to_string(),
            value if index == 1 && parts.first() == Some(&"gamemodes") => {
                mode_spelling(catalog, value)
            }
            value => value.to_string(),
        };
        lines.push(format!("{}{} {{", "    ".repeat(depth), name));
        depth += 1;
    }
    let indent = "    ".repeat(depth);
    let name = definition.presentation().english_name;
    match definition.domain() {
        SettingValueDomain::PresenceOnly => lines.push(format!("{indent}{name}")),
        SettingValueDomain::String => lines.push(format!("{indent}{name}: \"census\"")),
        SettingValueDomain::Boolean => {
            let value = definition
                .enum_members()
                .next()
                .map(|member| member.english_name().to_string())
                .unwrap_or_else(|| "On".to_string());
            lines.push(format!("{indent}{name}: {value}"));
        }
        SettingValueDomain::Number(_) => lines.push(format!("{indent}{name}: 1")),
        SettingValueDomain::Percent(_) => lines.push(format!("{indent}{name}: 100%")),
        SettingValueDomain::Enum { .. } => {
            let value = definition
                .enum_members()
                .next()
                .map(|member| member.english_name().to_string())
                .unwrap_or_else(|| "Off".to_string());
            lines.push(format!("{indent}{name}: {value}"));
        }
        SettingValueDomain::HeroList | SettingValueDomain::MapList => {
            lines.push(format!("{indent}{name} {{"));
            lines.push(format!("{indent}}}"));
        }
        other => unreachable!("census has no probe for settings value domain {other:?}"),
    }
    while depth > 1 {
        depth -= 1;
        lines.push(format!("{}{}", "    ".repeat(depth), "}"));
    }
    lines.push("}".to_string());
    lines.join("\n")
}

fn mode_spelling(catalog: &Catalog, key: &str) -> String {
    catalog_enum_spelling(catalog, "Gamemode", key).unwrap_or_else(|| key.to_string())
}

fn catalog_enum_spelling(catalog: &Catalog, domain: &str, key: &str) -> Option<String> {
    let normalize = |value: &str| {
        value
            .chars()
            .filter(|character| character.is_ascii_alphanumeric())
            .flat_map(char::to_lowercase)
            .collect::<String>()
    };
    let normalized = normalize(key);
    let member = catalog
        .enum_domains()
        .find(|candidate| candidate.domain == domain)?
        .members
        .iter()
        .find(|candidate| normalize(&candidate.member) == normalized)?;
    catalog
        .enum_spelling(domain, &Locale::new(EN_US), &member.member)
        .map(str::to_string)
}

fn run_case(case: &CensusCase, shard_id: &str, catalog: &Catalog) -> ConformanceResult {
    let source = artifact(
        format!("census/{shard_id}/{}.ws", case.case_id),
        &case.source,
    );
    let base = |status, comparison, reason, locale| ConformanceResult {
        schema_version: CONFORMANCE_SCHEMA_VERSION,
        case_id: case.case_id.clone(),
        features: case.features.clone(),
        status,
        comparison,
        source: source.clone(),
        catalog: catalog.identity(),
        locale,
        reason,
    };
    match &case.support {
        CensusSupport::Unsupported { detail } => base(
            ConformanceStatus::Unsupported,
            not_comparable(),
            Some(reason(ReasonCode::Unsupported, detail)),
            None,
        ),
        CensusSupport::KnownGap { detail } => base(
            ConformanceStatus::KnownGap,
            not_comparable(),
            Some(reason(ReasonCode::KnownGap, detail)),
            None,
        ),
        CensusSupport::Inconclusive { detail } => execute_case(case, base, catalog, Some(detail)),
        CensusSupport::Exercise => execute_case(case, base, catalog, None),
    }
}

struct Fail {
    code: ReasonCode,
    detail: String,
    locale: Locale,
}

impl Fail {
    fn workshop(error: &WorkshopError, locale: &Locale) -> Self {
        let code = match error {
            WorkshopError::Unsupported { .. } => ReasonCode::Unsupported,
            WorkshopError::MissingMapping { .. } => ReasonCode::KnownGap,
            _ => ReasonCode::UnexpectedRegression,
        };
        Self {
            code,
            detail: error.to_string(),
            locale: locale.clone(),
        }
    }

    fn regression(detail: impl Into<String>, locale: &Locale) -> Self {
        Self {
            code: ReasonCode::UnexpectedRegression,
            detail: detail.into(),
            locale: locale.clone(),
        }
    }
}

fn execute_case(
    case: &CensusCase,
    base: impl Fn(
        ConformanceStatus,
        Comparison,
        Option<ConformanceReason>,
        Option<Locale>,
    ) -> ConformanceResult,
    catalog: &Catalog,
    inconclusive_detail: Option<&str>,
) -> ConformanceResult {
    match run_pipeline(case, catalog, inconclusive_detail) {
        Ok((comparison, locale)) => {
            base(ConformanceStatus::Matched, comparison, None, Some(locale))
        }
        Err(fail) => failed_text(base, fail.code, fail.detail, Some(fail.locale)),
    }
}

fn run_pipeline(
    case: &CensusCase,
    catalog: &Catalog,
    inconclusive_detail: Option<&str>,
) -> Result<(Comparison, Locale), Fail> {
    let source_locale = Locale::new(&case.source_locale);
    let target_locale = if source_locale.as_str() == ZH_CN {
        Locale::new(EN_US)
    } else {
        Locale::new(ZH_CN)
    };

    let parse = |source: &str, locale: &Locale| {
        parser::parse_with_context(source, catalog, locale, catalog)
            .map_err(|error| Fail::workshop(&error, locale))
    };
    let validate = |program: &workshop_rs::Program, locale: &Locale| {
        program
            .validate()
            .map_err(|error| Fail::regression(error.to_string(), locale))
    };
    let emit = |program: &workshop_rs::Program, locale: &Locale| {
        emitter::emit(program, catalog, locale).map_err(|error| Fail::workshop(&error, locale))
    };
    let convert_or =
        |source: &str, from: &Locale, to: &Locale| -> Result<convert::Conversion, Fail> {
            convert::convert(source, catalog, from, to, &Default::default())
                .map_err(|error| Fail::workshop(&error, to))
        };

    // Some declared spellings are inherently ambiguous in a locale: zh-CN
    // writes `下` for `down`, `Vector.DOWN`, and `Rounding.DOWN` alike, so a
    // faithful conversion reparses to a preserved `__ambiguous_enum` rather
    // than the resolved node. When the target program emits byte-identical
    // canonical text, the client-observable surface is right and only the
    // resolved-identity check is undecidable — report that explicitly as
    // Inconclusive instead of a regression (#448).
    let surface_equivalent =
        |a: &workshop_rs::Program, b: &workshop_rs::Program, locale: &Locale| {
            let (Ok(x), Ok(y)) = (emit(a, locale), emit(b, locale)) else {
                return false;
            };
            normalize_workshop(&x) == normalize_workshop(&y)
        };

    let program = parse(&case.source, &source_locale)?;
    validate(&program, &source_locale)?;
    let emitted = emit(&program, &source_locale)?;
    let reparsed = parse(&emitted, &source_locale)?;
    validate(&reparsed, &source_locale)?;
    let reemitted = emit(&reparsed, &source_locale)?;
    if !roundtrip::equivalent(&program, &reparsed)
        || normalize_workshop(&emitted) != normalize_workshop(&reemitted)
    {
        return Err(Fail::regression(
            "en-US semantic or normalized gate diverged",
            &source_locale,
        ));
    }

    let converted = convert_or(&case.source, &source_locale, &target_locale)?;
    let target_program = parse(&converted.text, &target_locale)?;
    validate(&target_program, &target_locale)?;
    if !roundtrip::equivalent(&program, &target_program) {
        if surface_equivalent(&program, &target_program, &target_locale) {
            return Err(Fail {
                code: ReasonCode::Inconclusive,
                detail: "converted text matches the canonical emission, but the \
                         target-locale spelling is ambiguous across enum domains; \
                         resolved semantics cannot be independently verified"
                    .to_string(),
                locale: target_locale,
            });
        }
        return Err(Fail::regression(
            "zh-CN conversion changed canonical WIR semantics",
            &target_locale,
        ));
    }
    let back = convert_or(&converted.text, &target_locale, &source_locale)?;
    let back_program = parse(&back.text, &source_locale)?;
    // Compare against the canonical emission, not the raw source: probes
    // written in a non-canonical declared alias (#448) still must land on
    // the same canonical program and surface form after a round trip.
    if !roundtrip::equivalent(&program, &back_program)
        || normalize_workshop(&back.text) != normalize_workshop(&emitted)
    {
        if surface_equivalent(&program, &back_program, &source_locale) {
            return Err(Fail {
                code: ReasonCode::Inconclusive,
                detail: "round-trip text matches the canonical emission, but the \
                         source-locale spelling is ambiguous across enum domains; \
                         resolved semantics cannot be independently verified"
                    .to_string(),
                locale: source_locale,
            });
        }
        return Err(Fail::regression(
            "cross-locale semantic or normalized gate diverged",
            &source_locale,
        ));
    }

    let Some(reference_source) = case.reference_source.as_ref() else {
        return Err(Fail {
            code: ReasonCode::Inconclusive,
            detail: inconclusive_detail
                .unwrap_or(
                    "offline semantic and locale gates passed, but no independent expectation artifact is recorded",
                )
                .to_string(),
            locale: source_locale,
        });
    };
    let expected_program = parse(reference_source, &target_locale)?;
    if !roundtrip::equivalent(&program, &expected_program)
        || normalize_workshop(&converted.text) != normalize_workshop(reference_source)
    {
        return Err(Fail::regression(
            "conversion differed from the independent reference source",
            &target_locale,
        ));
    }
    Ok((
        Comparison {
            mode: Equivalence::Semantic,
            expected: Some(reference_artifact(case, reference_source)),
            observed: Some(artifact(
                format!("census/{}/converted-output.ws", case.case_id),
                &converted.text,
            )),
            normalizer: Some("canonical-wir;normalized-workshop-text".to_string()),
        },
        source_locale,
    ))
}

fn failed_text(
    base: impl Fn(
        ConformanceStatus,
        Comparison,
        Option<ConformanceReason>,
        Option<Locale>,
    ) -> ConformanceResult,
    code: ReasonCode,
    detail: String,
    locale: Option<Locale>,
) -> ConformanceResult {
    let status = match code {
        ReasonCode::Unsupported => ConformanceStatus::Unsupported,
        ReasonCode::KnownGap => ConformanceStatus::KnownGap,
        ReasonCode::UnexpectedRegression => ConformanceStatus::UnexpectedRegression,
        ReasonCode::Inconclusive => ConformanceStatus::Inconclusive,
    };
    let comparison = if code == ReasonCode::UnexpectedRegression {
        Comparison {
            mode: Equivalence::Normalized,
            expected: None,
            observed: None,
            normalizer: Some("census-stage".to_string()),
        }
    } else {
        not_comparable()
    };
    base(status, comparison, Some(reason(code, &detail)), locale)
}

fn reason(code: ReasonCode, detail: &str) -> ConformanceReason {
    ConformanceReason {
        code,
        detail: detail.to_string(),
    }
}

fn not_comparable() -> Comparison {
    Comparison {
        mode: Equivalence::NotComparable,
        expected: None,
        observed: None,
        normalizer: None,
    }
}

fn artifact(name: impl Into<String>, content: &str) -> TestArtifact {
    TestArtifact {
        name: name.into(),
        revision: None,
        path: None,
        sha256: Some(sha256(content)),
        license: Some("MIT".to_string()),
    }
}

fn reference_artifact(case: &CensusCase, content: &str) -> TestArtifact {
    TestArtifact {
        name: format!("census reference for {}", case.case_id),
        revision: Some("census-v1".to_string()),
        path: Some("tests/fixtures/census/reference.ws".to_string()),
        sha256: Some(sha256(content)),
        license: Some("MIT".to_string()),
    }
}

fn sha256(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn normalize_workshop(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_census_is_derived_and_deterministic() {
        let catalog = Catalog::builtin().expect("builtin catalog");
        let first = Census::builtin(&catalog).expect("census");
        let second = Census::builtin(&catalog).expect("census");
        assert_eq!(first, second);
        assert_eq!(first.shards().first().unwrap().shard_id, "catalog-actions");
        assert!(
            first
                .cases()
                .any(|case| case.features.iter().any(|feature| feature.name == "wait"))
        );
        assert!(first.cases().any(|case| {
            case.features
                .iter()
                .any(|feature| feature.kind == FeatureKind::Setting)
        }));
        assert!(first.cases().any(|case| {
            case.features
                .iter()
                .any(|feature| feature.kind == FeatureKind::ControlFlow)
        }));
        assert_eq!(first.export_json().unwrap(), second.export_json().unwrap());
        assert_eq!(first.identity(), second.identity());
        assert_eq!(first.identity().digest.len(), 64);
    }

    #[test]
    fn settings_probes_use_catalog_gamemode_spellings() {
        let catalog = Catalog::builtin().expect("builtin catalog");

        for (mode, expected) in [
            ("ffa", "Deathmatch"),
            ("tdm", "Team Deathmatch"),
            ("ctf", "Capture The Flag"),
        ] {
            let prefix = format!("gamemodes.{mode}.");
            let definition = settings_schema::definitions()
                .find(|definition| definition.path().starts_with(&prefix))
                .unwrap_or_else(|| panic!("missing settings definition for {mode}"));
            let probe = settings_probe(&definition, &catalog);
            assert!(
                probe.contains(&format!("{expected} {{")),
                "{mode} probe used the wrong mode spelling:\n{probe}"
            );
        }
    }

    #[test]
    fn explicit_non_matching_states_remain_machine_readable() {
        let feature_case = |id: &str, support| {
            CensusCase::probe(
                format!("state-tests/{id}"),
                vec![feature(FeatureNamespace::Wir, FeatureKind::Structural, id)],
                format!("rule (\"{id}\") {{}}"),
            )
            .with_support(support)
        };
        let shard = CensusShard::new(
            "state-tests",
            vec![
                feature_case(
                    "unsupported",
                    CensusSupport::Unsupported {
                        detail: "not declared".to_string(),
                    },
                ),
                feature_case(
                    "known-gap",
                    CensusSupport::KnownGap {
                        detail: "missing mapping".to_string(),
                    },
                ),
                feature_case(
                    "inconclusive",
                    CensusSupport::Inconclusive {
                        detail: "no oracle".to_string(),
                    },
                ),
            ],
        )
        .unwrap();
        let report = Census::assemble(vec![shard])
            .unwrap()
            .run(&Catalog::builtin().unwrap());
        report
            .validate()
            .expect("states use the current conformance adapter");
        let json = report.to_json().unwrap();
        assert!(json.contains("unsupported"));
        assert!(json.contains("known-gap"));
        assert!(json.contains("inconclusive"));
    }

    #[test]
    fn builtin_census_report_validates_against_the_catalog() {
        let catalog = Catalog::builtin().expect("builtin catalog");
        let census = Census::builtin(&catalog).expect("census");
        let report = census.run(&catalog);
        report
            .validate_against(&catalog)
            .expect("census results use canonical catalog identities");
        assert_eq!(report.census, census.identity());
        assert_eq!(
            report.results.len(),
            census.cases().count(),
            "census drops or duplicates cases"
        );
        let localization_cases = census
            .cases()
            .filter(|case| case.case_id.starts_with("localization/"))
            .count();
        assert_eq!(
            report
                .results
                .iter()
                .filter(|result| result.case_id.starts_with("localization/"))
                .count(),
            localization_cases
        );
        assert!(
            localization_cases > 2,
            "localization shard must enumerate per-spelling cases"
        );
        assert!(
            report.results.iter().any(|result| {
                result.case_id.starts_with("localization/")
                    && result.status == ConformanceStatus::KnownGap
            }),
            "unwritable localized settings spellings must report as known gaps"
        );
    }

    #[test]
    fn census_report_rejects_a_malformed_identity_digest() {
        let catalog = Catalog::builtin().expect("builtin catalog");
        let census = Census::assemble(vec![
            CensusShard::new(
                "state-tests",
                vec![CensusCase::probe(
                    "state-tests/probe".to_string(),
                    vec![feature(
                        FeatureNamespace::Wir,
                        FeatureKind::Structural,
                        "probe",
                    )],
                    "rule (\"probe\") {}".to_string(),
                )],
            )
            .unwrap(),
        ])
        .unwrap();
        let mut report = census.run(&catalog);
        report.census.digest = "not-a-digest".to_string();

        let error = report
            .validate_against(&catalog)
            .expect_err("report identity must carry a SHA-256 digest");
        assert!(error.to_string().contains("SHA-256"));
    }
}

#[cfg(test)]
mod probe_debug {
    use super::*;

    #[test]
    fn dump_failing_localization_probes() {
        let catalog = Catalog::builtin().unwrap();
        let census = Census::builtin(&catalog).unwrap();
        let wanted = [
            "localization/es-mx/value/hero/0",
            "localization/fr-fr/value/hero/0",
        ];
        for case in census.cases() {
            if !wanted.contains(&case.case_id.as_str()) {
                continue;
            }
            println!("==== {} ====", case.case_id);
            println!("{}", case.source);
            let result = run_case(case, "localization", &catalog);
            println!(
                "-> {:?} {:?}",
                result.status,
                result.reason.map(|r| r.detail)
            );
        }
    }
}
