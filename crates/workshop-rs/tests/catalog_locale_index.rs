use crate::common::catalog;
use workshop_rs::catalog::{Catalog, Kind};

#[test]
fn embedded_catalog_passes_full_load_validation_for_every_locale() {
    // The runtime builds locale indexes lazily and unchecked; this load
    // validates the aliases of every declared locale of the embedded data.
    Catalog::load(workshop_rs::catalog::CATALOG_DATA).expect("embedded catalog validates");
}

#[test]
fn duplicate_alias_in_a_non_primary_locale_fails_load() {
    let bad = r#"{
        "schemaVersion": 1,
        "locales": ["en-US", "zh-CN"],
        "target": { "game": "g", "format": "f", "surface": "s" },
        "provenance": { "generator": "g", "generatorVersion": "0", "source": "s", "license": "l", "reviewed": true },
        "actions": [
            { "id": "a", "aliases": { "en-US": "A", "zh-CN": "同" } },
            { "id": "b", "aliases": { "en-US": "B", "zh-CN": "同" } }
        ]
    }"#;
    let error = Catalog::load(bad).expect_err("colliding zh-CN aliases must fail");
    assert!(error.to_string().contains("duplicate"), "{error}");
}

#[test]
fn every_declared_spelling_resolves_back_in_every_locale() {
    let catalog = catalog();
    for locale in catalog.locales() {
        for entry in catalog.entries_of(Kind::Action) {
            for spelling in entry.spellings(locale) {
                let resolved = catalog.resolve(Kind::Action, locale, spelling);
                assert_eq!(resolved.map(|e| e.id.as_str()), Some(entry.id.as_str()));
            }
        }
        for domain in catalog.enum_domains() {
            for member in &domain.members {
                for spelling in member.spellings(locale) {
                    assert_eq!(
                        catalog
                            .resolve_enum_member(&domain.domain, locale, spelling)
                            .map(|(_, member)| member),
                        Some(member.member.clone())
                    );
                }
            }
        }
    }
}

#[test]
fn enum_domain_name_collisions_are_rejected_in_either_order() {
    let load = |first: &str, second: &str| {
        Catalog::load(&format!(
            r#"{{
            "schemaVersion": 1,
            "locales": ["en-US"],
            "target": {{ "game": "g", "format": "f", "surface": "s" }},
            "provenance": {{ "generator": "g", "generatorVersion": "0", "source": "s", "license": "l", "reviewed": true }},
            "enums": [{first}, {second}]
        }}"#
        ))
    };
    let x_with_alias = r#"{ "domain": "X", "aliases": { "en-US": "Xa" }, "members": [] }"#;
    let x_plain = r#"{ "domain": "X", "members": [] }"#;
    let y_claims_x = r#"{ "domain": "Y", "aliases": { "en-US": "X" }, "members": [] }"#;
    for (first, second) in [(x_with_alias, y_claims_x), (y_claims_x, x_plain)] {
        let error = load(first, second).expect_err("a domain alias equal to another domain name");
        assert!(
            error.to_string().contains("duplicate enum domain alias"),
            "{error}"
        );
    }
}
