use crate::common::zh;
use workshop_rs::catalog::{Catalog, Locale};

const PINNED_CATALOG_DIGEST: &str =
    "bd6ef06fdc2ea233c0c4bcdfc91307a7dc5e3ae1c4ee08611cd8304c0993b1e4";

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
    assert_eq!(identity.catalog_version, "0.1.15");
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
    assert!(json.get("target-evidence").is_none());
    assert!(json.get("target").is_some());
    assert!(json.get("provenance").is_some());
}

#[test]
fn locale_coverage_is_exact_and_primary_is_complete() {
    let catalog = Catalog::builtin().expect("built-in catalog");
    let en = catalog.locale_coverage(&crate::common::en());
    assert_eq!(en.mapped, en.total, "the primary locale is complete");
    let zh = catalog.locale_coverage(&zh());
    assert!(zh.mapped > 0 && zh.mapped < zh.total);
    assert_eq!(zh.total, en.total);
    let all = catalog.locale_coverage_all();
    assert_eq!(all.len(), 15);
    assert_eq!(all[0].locale, crate::common::en());
    assert_eq!(all[14].locale, Locale::new("zh-TW"));
    assert!(all.iter().skip(1).all(|coverage| coverage.mapped > 0));
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
