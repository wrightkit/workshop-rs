use workshop_rs::catalog::{Catalog, Locale};

const ORACLE_TABLE: &str = include_str!("fixtures/map-variants/overpy-9.7.17.json");

fn oracle_rows() -> Vec<serde_json::Value> {
    serde_json::from_str(ORACLE_TABLE).expect("oracle table")
}

#[test]
fn every_oracle_map_variant_resolves_to_its_id() {
    let catalog = Catalog::builtin().expect("built-in catalog");
    let en = Locale::new("en-US");
    let rows = oracle_rows();
    assert_eq!(rows.len(), 82);
    let mut pairs = 0;
    for row in &rows {
        let name = row["enUS"].as_str().expect("map name");
        let map = catalog
            .resolve_enum_member("Map", &en, name)
            .unwrap_or_else(|| panic!("oracle map {name} is not a catalog Map member"))
            .1;
        for (variant, id) in row["variants"].as_object().expect("variants") {
            let id = id.as_str().expect("variant id");
            assert_eq!(
                catalog.map_variant_id(&map, variant),
                Some(id),
                "{map} {variant}"
            );
            pairs += 1;
        }
    }
    assert_eq!(pairs, 109);
}

#[test]
fn catalog_declares_no_variant_the_oracle_lacks() {
    let catalog = Catalog::builtin().expect("built-in catalog");
    let en = Locale::new("en-US");
    let rows = oracle_rows();
    let oracle_maps: Vec<String> = rows
        .iter()
        .map(|row| {
            catalog
                .resolve_enum_member("Map", &en, row["enUS"].as_str().unwrap())
                .unwrap()
                .1
        })
        .collect();
    let expected: usize = rows
        .iter()
        .map(|row| row["variants"].as_object().unwrap().len())
        .sum();
    let mut declared = 0;
    for member in &catalog.enum_domain("Map").unwrap().members {
        for variant in [
            "default", "morning", "night", "evening", "sunset", "overcast", "dawn",
        ] {
            if catalog.map_variant_id(&member.member, variant).is_some() {
                assert!(oracle_maps.contains(&member.member), "{}", member.member);
                declared += 1;
            }
        }
    }
    assert_eq!(declared, expected);
}

#[test]
fn unknown_map_or_variant_has_no_id() {
    let catalog = Catalog::builtin().expect("built-in catalog");
    assert_eq!(catalog.map_variant_id("NOT_A_MAP", "default"), None);
    assert_eq!(catalog.map_variant_id("ANTARCTIC_PENINSULA", "noon"), None);
    assert_eq!(
        catalog.map_variant_id("ANTARCTIC_PENINSULA", "default"),
        None
    );
    assert_eq!(catalog.map_variant_id("GRIMSVOTN", "default"), None);
    assert_eq!(
        catalog.map_variant_id("ANTARCTIC_PENINSULA", "morning"),
        Some("972777519512068154")
    );
}
