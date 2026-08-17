//! Catalog identity tests (ADR-0001 Decision 5): the machine-readable
//! identities — implementation version, catalog version + content digest,
//! locale coverage, target evidence — and the deliberate-change pinning of
//! the committed dataset digest.

use workshop_rs::catalog::{Catalog, Locale};

/// The pinned digest of the committed catalog dataset (version 0.1.0).
///
/// Any dataset change (entries, aliases, locale tables, provenance, target)
/// changes the content digest, and this test fails until the pipeline
/// (`workshop-catalog-gen build`) recomputes it and the pin is updated
/// deliberately together with the data.
const PINNED_CATALOG_DIGEST: &str =
    "f88b1a99a8e8a5d613b2f850353b144120d8952cdf2303532134ef9ccf92fca9";

#[test]
fn committed_catalog_digest_is_pinned() {
    let catalog = Catalog::builtin().expect("built-in catalog");
    assert_eq!(
        catalog.catalog_digest(),
        Some(PINNED_CATALOG_DIGEST),
        "the committed catalog dataset digest changed; run \
         `cargo run -p workshop-rs --bin workshop-catalog-gen -- build` and \
         update this pin deliberately with the data change"
    );
    // The digest is deterministic: recomputing from the data yields the pin.
    let data = include_str!("../src/catalog/data/catalog.json");
    assert_eq!(
        workshop_rs::catalog::content_digest(data).expect("digest computes"),
        PINNED_CATALOG_DIGEST
    );
}

#[test]
fn identity_reports_all_four_machine_readable_identities() {
    let catalog = Catalog::builtin().expect("built-in catalog");
    let identity = catalog.identity();
    assert_eq!(identity.implementation_version, env!("CARGO_PKG_VERSION"));
    assert_eq!(identity.catalog_version, "0.1.0");
    assert_eq!(
        identity.catalog_digest.as_deref(),
        Some(PINNED_CATALOG_DIGEST)
    );
    assert_eq!(identity.target.game, "Overwatch 2");
    assert!(identity.provenance.reviewed);
    assert_eq!(identity.provenance.generator, "workshop-catalog-gen");
    assert!(
        identity.provenance.license.starts_with("MIT"),
        "the migrated catalog data is MIT: {}",
        identity.provenance.license
    );
}

#[test]
fn identity_serializes_with_the_adr_kebab_case_names() {
    let catalog = Catalog::builtin().expect("built-in catalog");
    let json = serde_json::to_value(catalog.identity()).expect("identity serializes");
    assert!(json.get("implementation-version").is_some());
    assert!(json.get("catalog-version").is_some());
    assert!(json.get("catalog-digest").is_some());
    assert!(json.get("locale-coverage").is_some());
    assert!(json.get("target-evidence").is_none()); // target is the target record
    assert!(json.get("target").is_some());
    assert!(json.get("provenance").is_some());
}

#[test]
fn locale_coverage_is_exact_and_primary_is_complete() {
    let catalog = Catalog::builtin().expect("built-in catalog");
    let en = catalog.locale_coverage(&Locale::new("en-US"));
    assert_eq!(en.mapped, en.total, "the primary locale is complete");
    assert_eq!(
        en.mapped, 344,
        "declared en-US surface (168 entries + 176 members)"
    );
    let zh = catalog.locale_coverage(&Locale::new("zh-CN"));
    assert_eq!(zh.mapped, 328, "zh-CN corpus coverage is pinned");
    assert_eq!(zh.total, en.total);
    let all = catalog.locale_coverage_all();
    assert_eq!(all.len(), 2);
    assert_eq!(all[0].locale, Locale::new("en-US"));
    assert_eq!(all[1].locale, Locale::new("zh-CN"));
}

#[test]
fn load_rejects_a_stale_digest() {
    let mut tampered = include_str!("../src/catalog/data/catalog.json").to_string();
    let needle = "\"en-US\": \"Wait\"";
    let pos = tampered.find(needle).expect("wait alias present");
    tampered.replace_range(pos..pos + needle.len(), "\"en-US\": \"Wait Modified\"");
    let error = Catalog::load(&tampered).expect_err("stale digest must be rejected");
    assert!(error.to_string().contains("digest mismatch"), "{error}");
}
