//! Deterministic, sharded census of the canonical Workshop surface.
//!
//! The census is derived from this crate's catalog, settings table, and WIR
//! capabilities. It is a runner and evidence assembler, not a source-language
//! inventory or a live-client oracle.

use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::catalog::{Catalog, CatalogEntry, EnumDomain, Kind, Locale};
use crate::conformance::{
    CONFORMANCE_SCHEMA_VERSION, Comparison, ConformanceReason, ConformanceResult,
    ConformanceStatus, Equivalence, Evidence, EvidenceArtifact, EvidenceBasis, EvidenceClass,
    ExpectationSource, FeatureId, FeatureKind, FeatureNamespace, ImplementationIdentity,
    ReasonCode,
};
use crate::convert;
use crate::emitter;
use crate::error::WorkshopError;
use crate::parser;
use crate::roundtrip;
use crate::settings::table::{self, KeyKind, PathPart, TableEntry};
use crate::wir::{CENSUS_CAPABILITIES, CensusCapabilityKind};

pub const CENSUS_SCHEMA_VERSION: u32 = 1;
pub const CENSUS_IDENTITY_SCHEMA_VERSION: u32 = 1;
const EN_US: &str = "en-US";
const ZH_CN: &str = "zh-CN";
const CENSUS_TRACKING_REF: &str = "#19";

/// An explicit support classification for a census case.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum CensusSupport {
    Exercise,
    Unsupported {
        detail: String,
    },
    KnownGap {
        detail: String,
        tracking_ref: String,
    },
    Inconclusive {
        detail: String,
    },
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
            settings_shard()?,
            wir_shard()?,
            localization_shard()?,
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
    pub fn run(&self, catalog: &Catalog) -> CensusReport {
        let mut results: Vec<_> = self
            .shards
            .iter()
            .flat_map(|shard| {
                shard
                    .cases
                    .iter()
                    .map(move |case| run_case(case, &shard.shard_id, catalog))
            })
            .collect();
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
    pub catalog: crate::catalog::CatalogIdentity,
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
        if self.census.digest.len() != 64
            || !self
                .census
                .digest
                .chars()
                .all(|character| character.is_ascii_hexdigit())
        {
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
    FeatureId::owned(namespace, kind, name).expect("canonical census feature ID")
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
            CensusCase {
                case_id: format!("{shard_id}/{}", entry.id),
                features: vec![catalog_feature(kind, &entry.id)],
                source_locale: EN_US.to_string(),
                source,
                reference_source: None,
                support: generated_probe_support(),
            }
        })
        .collect();
    CensusShard::new(shard_id, cases)
}

fn enum_shard(catalog: &Catalog) -> Result<CensusShard, CensusError> {
    let mut cases = Vec::new();
    for domain in catalog.enum_domains() {
        for member in &domain.members {
            let features = vec![
                catalog_feature(Kind::Enum, &domain.domain),
                FeatureId::from_enum_member(&domain.domain, &member.member)
                    .expect("canonical enum member ID"),
            ];
            cases.push(CensusCase {
                case_id: format!("catalog-enums/{}/{}", domain.domain, member.member),
                features,
                source_locale: EN_US.to_string(),
                source: enum_probe(catalog, domain, &member.member),
                reference_source: None,
                support: generated_probe_support(),
            });
        }
    }
    CensusShard::new("catalog-enums", cases)
}

fn content_id_shard(catalog: &Catalog) -> Result<CensusShard, CensusError> {
    let mut cases = Vec::new();
    for domain in catalog.enum_domains() {
        if !matches!(domain.domain.as_str(), "Hero" | "Map") {
            continue;
        }
        for member in &domain.members {
            cases.push(CensusCase {
                case_id: format!("content-ids/{}/{}", domain.domain, member.member),
                features: vec![
                    FeatureId::from_enum_member(&domain.domain, &member.member)
                        .expect("canonical content enum-member ID"),
                ],
                source_locale: EN_US.to_string(),
                source: enum_probe(catalog, domain, &member.member),
                reference_source: None,
                support: generated_probe_support(),
            });
        }
    }
    CensusShard::new("content-ids", cases)
}

fn settings_shard() -> Result<CensusShard, CensusError> {
    let cases = table::ENTRIES
        .iter()
        .map(|entry| {
            let path = table::path_string(entry.path);
            CensusCase {
                case_id: format!("settings/{path}"),
                features: vec![feature(
                    FeatureNamespace::Settings,
                    FeatureKind::Setting,
                    path,
                )],
                source_locale: EN_US.to_string(),
                source: settings_probe(entry),
                reference_source: None,
                support: generated_probe_support(),
            }
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
            CensusCapabilityKind::PlayerVariable => CensusCase {
                case_id: "wir/variables-player".to_string(),
                features: vec![feature(
                    FeatureNamespace::Wir,
                    FeatureKind::Variable,
                    capability.name,
                )],
                source_locale: EN_US.to_string(),
                source: player_variable_source(),
                reference_source: None,
                support: generated_probe_support(),
            },
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
            CensusCapabilityKind::String => CensusCase {
                case_id: "wir/string/custom-string".to_string(),
                features: vec![feature(
                    FeatureNamespace::Wir,
                    FeatureKind::String,
                    capability.name,
                )],
                source_locale: EN_US.to_string(),
                source: rule_source(
                    "String",
                    "Set Global Variable(probe, Custom String(\"census\"));",
                ),
                reference_source: None,
                support: CensusSupport::Exercise,
            },
        })
        .collect();
    CensusShard::new("wir", cases)
}

fn localization_shard() -> Result<CensusShard, CensusError> {
    let en_source = include_str!("../tests/fixtures/census/localization-en-us.ws").to_string();
    let zh_source = include_str!("../tests/fixtures/census/localization-zh-cn.ws").to_string();
    CensusShard::new(
        "localization",
        vec![
            CensusCase {
                case_id: "localization/en-us-to-zh-cn".to_string(),
                features: vec![feature(
                    FeatureNamespace::Localization,
                    FeatureKind::Localization,
                    "en-us-to-zh-cn",
                )],
                source_locale: EN_US.to_string(),
                source: en_source,
                reference_source: None,
                support: generated_probe_support(),
            },
            CensusCase {
                case_id: "localization/zh-cn-to-en-us".to_string(),
                features: vec![feature(
                    FeatureNamespace::Localization,
                    FeatureKind::Localization,
                    "zh-cn-to-en-us",
                )],
                source_locale: ZH_CN.to_string(),
                source: zh_source,
                reference_source: None,
                support: generated_probe_support(),
            },
        ],
    )
}

fn wir_case(case_id: &str, kind: FeatureKind, name: &str, source: String) -> CensusCase {
    CensusCase {
        case_id: format!("wir/{case_id}"),
        features: vec![feature(FeatureNamespace::Wir, kind, name)],
        source_locale: EN_US.to_string(),
        source,
        reference_source: None,
        support: CensusSupport::Exercise,
    }
}

fn control_flow_case(name: &str, actions: &str) -> CensusCase {
    CensusCase {
        case_id: format!("wir/control-flow/{name}"),
        features: vec![feature(
            FeatureNamespace::Wir,
            FeatureKind::ControlFlow,
            name,
        )],
        source_locale: EN_US.to_string(),
        source: rule_source(name, actions),
        reference_source: None,
        support: generated_probe_support(),
    }
}

fn generated_probe_support() -> CensusSupport {
    CensusSupport::Inconclusive {
        detail: "generated probe is exportable for independent Workshop/client evidence but has no independently recorded expected result".to_string(),
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
    rule_source(
        "Enum",
        &format!("Set Global Variable(probe, {domain_spelling}({member_spelling}));"),
    )
}

fn settings_probe(entry: &TableEntry) -> String {
    let mut lines = vec!["settings {".to_string()];
    let mut depth = 1;
    for part in entry.path {
        let name = match part {
            PathPart::Part("gamemodes") => "modes",
            PathPart::Part("heroes") => "heroes",
            PathPart::Part("main") => "main",
            PathPart::Part("lobby") => "lobby",
            PathPart::Part(value) => table::mode_name(value).unwrap_or(value),
            PathPart::Team => "General",
            PathPart::Hero => "Mei",
        };
        lines.push(format!("{}{} {{", "    ".repeat(depth), name));
        depth += 1;
    }
    let indent = "    ".repeat(depth);
    match entry.kind {
        KeyKind::Flag => lines.push(format!("{indent}{}", entry.workshop_name)),
        KeyKind::String => lines.push(format!("{indent}{}: \"census\"", entry.workshop_name)),
        KeyKind::Bool => lines.push(format!("{indent}{}: On", entry.workshop_name)),
        KeyKind::Number => lines.push(format!("{indent}{}: 1", entry.workshop_name)),
        KeyKind::Percent => lines.push(format!("{indent}{}: 100%", entry.workshop_name)),
        KeyKind::Enum(domain) => {
            let member = if domain == "roleLimit" {
                "2OfEachRolePerTeam"
            } else {
                "off"
            };
            let value = table::enum_name(domain, member).unwrap_or("Off");
            lines.push(format!("{indent}{}: {value}", entry.workshop_name));
        }
        KeyKind::ListMap | KeyKind::ListHero => {
            lines.push(format!("{indent}{} {{", entry.workshop_name));
            lines.push(format!("{indent}}}"));
        }
    }
    while depth > 1 {
        depth -= 1;
        lines.push(format!("{}{}", "    ".repeat(depth), "}"));
    }
    lines.push("}".to_string());
    lines.join("\n")
}

fn run_case(case: &CensusCase, shard_id: &str, catalog: &Catalog) -> ConformanceResult {
    let fixture = artifact(
        format!("census/{shard_id}/{}.ws", case.case_id),
        &case.source,
    );
    let expectation = case
        .reference_source
        .as_ref()
        .map(|source| ExpectationSource {
            basis: EvidenceBasis::PreservedRegression,
            artifact: reference_artifact(case, source),
            tracking_ref: Some(CENSUS_TRACKING_REF.to_string()),
        })
        .unwrap_or_else(|| ExpectationSource {
            basis: EvidenceBasis::SemanticContract,
            artifact: EvidenceArtifact {
                name: "docs/adr/0002-conformance-contract.md".to_string(),
                revision: Some("ADR-0002".to_string()),
                path: Some("docs/adr/0002-conformance-contract.md".to_string()),
                sha256: None,
                license: Some("MIT".to_string()),
            },
            tracking_ref: None,
        });
    let evidence = |locale: Option<Locale>| Evidence {
        class: EvidenceClass::Synthetic,
        fixture: fixture.clone(),
        expectation: expectation.clone(),
        catalog: catalog.identity(),
        locale,
        client: None,
        implementation: Some(ImplementationIdentity {
            name: "workshop-rs".to_string(),
            version: Catalog::implementation_version().to_string(),
            revision: None,
            artifact: None,
        }),
    };
    let base = |status, comparison, reason, locale| ConformanceResult {
        schema_version: CONFORMANCE_SCHEMA_VERSION,
        case_id: case.case_id.clone(),
        features: case.features.clone(),
        status,
        comparison,
        evidence: evidence(locale),
        reason,
    };
    match &case.support {
        CensusSupport::Unsupported { detail } => base(
            ConformanceStatus::Unsupported,
            not_comparable(),
            Some(reason(ReasonCode::Unsupported, detail, None)),
            None,
        ),
        CensusSupport::KnownGap {
            detail,
            tracking_ref,
        } => base(
            ConformanceStatus::KnownGap,
            not_comparable(),
            Some(reason(
                ReasonCode::KnownGap,
                detail,
                Some(tracking_ref.clone()),
            )),
            None,
        ),
        CensusSupport::Inconclusive { detail } => execute_case(case, base, catalog, Some(detail)),
        CensusSupport::Exercise => execute_case(case, base, catalog, None),
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
    let source_locale = Locale::new(&case.source_locale);
    let target_locale = if source_locale.as_str() == Locale::new(ZH_CN).as_str() {
        Locale::new(EN_US)
    } else {
        Locale::new(ZH_CN)
    };
    let program = match parser::parse_with_context(&case.source, catalog, &source_locale, catalog) {
        Ok(program) => program,
        Err(error) => return failed(base, &error, &source_locale),
    };
    if let Err(error) = program.validate() {
        return failed_text(
            base,
            ReasonCode::UnexpectedRegression,
            error.to_string(),
            Some(source_locale.clone()),
        );
    }
    let emitted_source = match emitter::emit(&program, catalog, &source_locale) {
        Ok(output) => output,
        Err(error) => return failed(base, &error, &source_locale),
    };
    let reparsed_source =
        match parser::parse_with_context(&emitted_source, catalog, &source_locale, catalog) {
            Ok(program) => program,
            Err(error) => return failed(base, &error, &source_locale),
        };
    if let Err(error) = reparsed_source.validate() {
        return failed_text(
            base,
            ReasonCode::UnexpectedRegression,
            error.to_string(),
            Some(source_locale.clone()),
        );
    }
    let emitted_source_again = match emitter::emit(&reparsed_source, catalog, &source_locale) {
        Ok(output) => output,
        Err(error) => return failed(base, &error, &source_locale),
    };
    if !roundtrip::equivalent(&program, &reparsed_source)
        || normalize_workshop(&emitted_source) != normalize_workshop(&emitted_source_again)
    {
        return failed_text(
            base,
            ReasonCode::UnexpectedRegression,
            "en-US semantic or normalized gate diverged".to_string(),
            Some(source_locale.clone()),
        );
    }
    let converted = match convert::convert(
        &case.source,
        catalog,
        &source_locale,
        &target_locale,
        &Default::default(),
    ) {
        Ok(output) => output,
        Err(error) => return failed(base, &error, &target_locale),
    };
    let program_target =
        match parser::parse_with_context(&converted.text, catalog, &target_locale, catalog) {
            Ok(program) => program,
            Err(error) => return failed(base, &error, &target_locale),
        };
    if let Err(error) = program_target.validate() {
        return failed_text(
            base,
            ReasonCode::UnexpectedRegression,
            error.to_string(),
            Some(target_locale.clone()),
        );
    }
    if !roundtrip::equivalent(&program, &program_target) {
        return failed_text(
            base,
            ReasonCode::UnexpectedRegression,
            "zh-CN conversion changed canonical WIR semantics".to_string(),
            Some(target_locale.clone()),
        );
    }
    let back_to_source = match convert::convert(
        &converted.text,
        catalog,
        &target_locale,
        &source_locale,
        &Default::default(),
    ) {
        Ok(output) => output,
        Err(error) => return failed(base, &error, &source_locale),
    };
    let reparsed_back =
        match parser::parse_with_context(&back_to_source.text, catalog, &source_locale, catalog) {
            Ok(program) => program,
            Err(error) => return failed(base, &error, &source_locale),
        };
    if !roundtrip::equivalent(&program, &reparsed_back)
        || normalize_workshop(&back_to_source.text) != normalize_workshop(&case.source)
    {
        return failed_text(
            base,
            ReasonCode::UnexpectedRegression,
            "cross-locale semantic or normalized gate diverged".to_string(),
            Some(source_locale.clone()),
        );
    }
    let Some(reference_source) = case.reference_source.as_ref() else {
        return failed_text(
            base,
            ReasonCode::Inconclusive,
            inconclusive_detail.unwrap_or(
                "offline semantic and locale gates passed, but no independent expectation artifact is recorded",
            )
            .to_string(),
            Some(source_locale),
        );
    };
    let expected_program =
        match parser::parse_with_context(reference_source, catalog, &target_locale, catalog) {
            Ok(program) => program,
            Err(error) => return failed(base, &error, &target_locale),
        };
    if !roundtrip::equivalent(&program, &expected_program)
        || normalize_workshop(&converted.text) != normalize_workshop(reference_source)
    {
        return failed_text(
            base,
            ReasonCode::UnexpectedRegression,
            "conversion differed from the independent reference source".to_string(),
            Some(target_locale),
        );
    }
    base(
        ConformanceStatus::Matched,
        Comparison {
            mode: Equivalence::Semantic,
            expected: Some(reference_artifact(case, reference_source)),
            observed: Some(artifact(
                format!("census/{}/converted-output.ws", case.case_id),
                &converted.text,
            )),
            normalizer: Some("canonical-wir;normalized-workshop-text".to_string()),
        },
        None,
        Some(source_locale),
    )
}

fn failed(
    base: impl Fn(
        ConformanceStatus,
        Comparison,
        Option<ConformanceReason>,
        Option<Locale>,
    ) -> ConformanceResult,
    error: &WorkshopError,
    locale: &Locale,
) -> ConformanceResult {
    let code = match error {
        WorkshopError::Unsupported { .. } => ReasonCode::Unsupported,
        WorkshopError::MissingMapping { .. } => ReasonCode::KnownGap,
        _ => ReasonCode::UnexpectedRegression,
    };
    failed_text(base, code, error.to_string(), Some(locale.clone()))
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
    let tracking = (code == ReasonCode::KnownGap).then(|| CENSUS_TRACKING_REF.to_string());
    base(
        status,
        comparison,
        Some(reason(code, &detail, tracking)),
        locale,
    )
}

fn reason(code: ReasonCode, detail: &str, tracking_ref: Option<String>) -> ConformanceReason {
    ConformanceReason {
        code,
        detail: detail.to_string(),
        tracking_ref,
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

fn artifact(name: impl Into<String>, content: &str) -> EvidenceArtifact {
    EvidenceArtifact {
        name: name.into(),
        revision: None,
        path: None,
        sha256: Some(sha256(content)),
        license: Some("MIT".to_string()),
    }
}

fn reference_artifact(case: &CensusCase, content: &str) -> EvidenceArtifact {
    EvidenceArtifact {
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
    fn explicit_non_matching_states_remain_machine_readable() {
        let feature_case = |id: &str, support| CensusCase {
            case_id: format!("state-tests/{id}"),
            features: vec![feature(FeatureNamespace::Wir, FeatureKind::Structural, id)],
            source_locale: EN_US.to_string(),
            source: format!("rule (\"{id}\") {{}}"),
            reference_source: None,
            support,
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
                        tracking_ref: "#19".to_string(),
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
            .expect("states use the current #18 adapter");
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
        assert!(
            report
                .results
                .iter()
                .filter(|result| result.case_id.starts_with("localization/"))
                .all(|result| result.status == ConformanceStatus::Inconclusive)
        );
    }

    #[test]
    fn census_report_rejects_a_malformed_identity_digest() {
        let catalog = Catalog::builtin().expect("builtin catalog");
        let census = Census::builtin(&catalog).expect("census");
        let mut report = census.run(&catalog);
        report.census.digest = "not-a-digest".to_string();

        let error = report
            .validate_against(&catalog)
            .expect_err("report identity must carry a SHA-256 digest");
        assert!(error.to_string().contains("SHA-256"));
    }
}
