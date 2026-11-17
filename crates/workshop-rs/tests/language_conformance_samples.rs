//! Sample arguments that satisfy a documented parameter type, for the
//! language-conformance cases.

use workshop_rs::catalog::{Catalog, Kind, Locale};

use super::EN_US;

pub(super) fn sample_argument(catalog: &Catalog, parameter_type: &str) -> String {
    if let Some(enum_domain) = catalog.enum_domain(parameter_type) {
        if parameter_type == "Vector" {
            return "Vector(0, 0, 0)".to_string();
        }
        if let Some(member) = enum_domain.members.first() {
            return catalog
                .enum_spelling(parameter_type, &Locale::new(EN_US), &member.member)
                .unwrap_or(&member.member)
                .to_string();
        }
    }
    match parameter_type {
        "Boolean" => "True".to_string(),
        "Number"
        | "Number|Player"
        | "Number|Boolean"
        | "Boolean|Number"
        | "Boolean|Number|Vector" => "1".to_string(),
        "Vector" | "Vector|Player" | "Vector|Player|Array" => "Vector(0, 0, 0)".to_string(),
        "String"
        | "String|Player"
        | "String|Array"
        | "String|Array|EntityId"
        | "String|Array|EntityId|Color"
        | "Object|String" => "Custom String(\"probe\")".to_string(),
        "Array" | "Array|EntityId" => "All Players(All Teams)".to_string(),
        "Player" | "Player|Array" | "Player|Array|EntityId" | "Player|EntityId" => {
            "Event Player".to_string()
        }
        "Object" | "Object|Array" | "Boolean|Number|Object|Array" => "Event Player".to_string(),
        "Hero|Array" => "D.Va".to_string(),
        "Operation" => "Add".to_string(),
        "Wait" => "Ignore Condition".to_string(),
        "__Operator__" => "==".to_string(),
        "Variable" | "Global Variable" | "Subroutine" => "probe".to_string(),
        "Player Variable" => "(Event Player).probe".to_string(),
        "EntityId" => value_spelling(catalog, "lastCreatedEntity", "Last Created Entity"),
        "TextId" => value_spelling(catalog, "lastTextId", "Last Text ID"),
        "AssistId" => value_spelling(catalog, "getLastAssistId", "Last Assist ID"),
        "DotId" => value_spelling(
            catalog,
            "getLastDamageOverTimeId",
            "Last Damage Over Time ID",
        ),
        "HotId" => value_spelling(
            catalog,
            "getLastHealingOverTimeId",
            "Last Heal Over Time ID",
        ),
        "HealthPoolId" => value_spelling(
            catalog,
            "getLastCreatedHealthPool",
            "Last Created Health Pool",
        ),
        "DamageModificationId" => value_spelling(
            catalog,
            "getLastDamageModification",
            "Last Damage Modification ID",
        ),
        "HealingModificationId" => value_spelling(
            catalog,
            "getLastHealingModification",
            "Last Healing Modification ID",
        ),
        _ => "1".to_string(),
    }
}

fn value_spelling(catalog: &Catalog, id: &str, fallback: &str) -> String {
    catalog
        .spelling(Kind::Value, &Locale::new(EN_US), id)
        .unwrap_or(fallback)
        .to_string()
}
