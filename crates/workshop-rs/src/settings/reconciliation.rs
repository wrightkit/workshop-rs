//! Reviewed reconciliation rules for overlapping raw settings projections.

use std::sync::OnceLock;

use serde::Deserialize;

const DATA: &str = include_str!("data/projection_reconciliation.json");

#[derive(Debug, Deserialize)]
pub(crate) struct Reconciliation {
    #[serde(rename = "schemaVersion")]
    pub(crate) schema_version: u32,
    #[serde(rename = "entryOverrides")]
    pub(crate) entry_overrides: Vec<EntryOverride>,
    #[serde(rename = "enumMemberMappings")]
    pub(crate) enum_member_mappings: Vec<EnumMemberMapping>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct EntryOverride {
    pub(crate) path: String,
    pub(crate) fixture: EntryContract,
    pub(crate) generated: EntryContract,
}

#[derive(Debug, Deserialize)]
pub(crate) struct EntryContract {
    pub(crate) name: String,
    pub(crate) kind: String,
    pub(crate) domain: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct EnumMemberMapping {
    #[serde(rename = "sourceDomain")]
    pub(crate) source_domain: String,
    #[serde(rename = "sourceMember")]
    pub(crate) source_member: String,
    #[serde(rename = "targetDomain")]
    pub(crate) target_domain: String,
    #[serde(rename = "targetMember")]
    pub(crate) target_member: String,
}

pub(crate) fn data() -> &'static Reconciliation {
    static RECONCILIATION: OnceLock<Reconciliation> = OnceLock::new();
    RECONCILIATION.get_or_init(|| {
        serde_json::from_str(DATA).expect("settings projection reconciliation data is valid JSON")
    })
}

/// Reject ambiguous reconciliation rules before callers can use their
/// first-match lookup behavior.
pub(crate) fn validate() -> Vec<String> {
    validate_manifest(data())
}

fn validate_manifest(reconciliation: &Reconciliation) -> Vec<String> {
    use std::collections::HashSet;

    let mut errors = Vec::new();
    if reconciliation.schema_version != 1 {
        errors.push(format!(
            "unsupported settings reconciliation schema version: {}",
            reconciliation.schema_version
        ));
    }

    let mut entry_paths = HashSet::new();
    for override_ in &reconciliation.entry_overrides {
        if !entry_paths.insert(override_.path.as_str()) {
            errors.push(format!(
                "duplicate settings reconciliation entry override: {}",
                override_.path
            ));
        }
    }

    let mut member_sources = HashSet::new();
    for mapping in &reconciliation.enum_member_mappings {
        let source = (
            mapping.source_domain.as_str(),
            mapping.source_member.as_str(),
        );
        if !member_sources.insert(source) {
            errors.push(format!(
                "duplicate settings enum reconciliation source: {}.{}",
                mapping.source_domain, mapping.source_member
            ));
        }
    }
    errors
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry_override(path: &str) -> EntryOverride {
        EntryOverride {
            path: path.to_string(),
            fixture: EntryContract {
                name: "Fixture".to_string(),
                kind: "bool".to_string(),
                domain: None,
            },
            generated: EntryContract {
                name: "Generated".to_string(),
                kind: "bool".to_string(),
                domain: None,
            },
        }
    }

    fn mapping(source_member: &str) -> EnumMemberMapping {
        EnumMemberMapping {
            source_domain: "exportDomain".to_string(),
            source_member: source_member.to_string(),
            target_domain: "canonicalDomain".to_string(),
            target_member: "canonicalMember".to_string(),
        }
    }

    fn manifest() -> Reconciliation {
        Reconciliation {
            schema_version: 1,
            entry_overrides: Vec::new(),
            enum_member_mappings: Vec::new(),
        }
    }

    #[test]
    fn manifest_rejects_duplicate_entry_override_paths() {
        let mut reconciliation = manifest();
        reconciliation.entry_overrides = vec![
            entry_override("lobby.mapRotation"),
            entry_override("lobby.mapRotation"),
        ];
        let errors = validate_manifest(&reconciliation);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("duplicate settings reconciliation entry override"));
    }

    #[test]
    fn manifest_rejects_duplicate_enum_member_sources() {
        let mut reconciliation = manifest();
        reconciliation.enum_member_mappings = vec![mapping("afterGame"), mapping("afterGame")];
        let errors = validate_manifest(&reconciliation);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("duplicate settings enum reconciliation source"));
    }

    #[test]
    fn manifest_rejects_unsupported_schema_versions() {
        let mut reconciliation = manifest();
        reconciliation.schema_version = 2;
        let errors = validate_manifest(&reconciliation);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("unsupported settings reconciliation schema version"));
    }
}
