//! Reviewed reconciliation rules for overlapping raw settings projections.

use std::sync::OnceLock;

use serde::Deserialize;

const DATA: &str = include_str!("data/projection_reconciliation.json");

#[derive(Debug, Deserialize)]
pub(crate) struct Reconciliation {
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
