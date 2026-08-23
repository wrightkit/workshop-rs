use std::fs;
use std::path::Path;
use workshop_rs::catalog::{Catalog, Kind, Locale};

const ALLOWED_STATUSES: &[&str] = &["✅ Supported", "🚧 Coming soon", "❌ Unsupported"];
const FORBIDDEN_HEADER_TERMS: &[&str] = &[
    "hir",
    "wir",
    "frontend",
    "lowering",
    "conformance state",
    "evidence class",
    "corpus state",
    "agent state",
];

#[test]
fn test_language_support_doc_exists_and_is_authoritative() {
    let doc_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("docs/language-support.md");

    assert!(
        doc_path.exists(),
        "docs/language-support.md must exist at {}",
        doc_path.display()
    );

    let content = fs::read_to_string(&doc_path).expect("failed to read docs/language-support.md");
    assert!(
        content.contains("# Workshop Language Support"),
        "docs/language-support.md must contain the main title"
    );
    assert!(
        content.contains("single authoritative source of truth"),
        "docs/language-support.md must declare itself as authoritative"
    );
}

#[test]
fn test_language_support_statuses_and_headers_are_valid() {
    let doc_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("docs/language-support.md");

    let content = fs::read_to_string(&doc_path).expect("failed to read docs/language-support.md");

    let mut in_table = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('|') && trimmed.ends_with('|') {
            let cells: Vec<&str> = trimmed
                .split('|')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .collect();

            if cells.is_empty() {
                continue;
            }

            // Check if this is a header separator line (e.g. | --- | --- |)
            if cells
                .iter()
                .all(|c| c.chars().all(|ch| ch == '-' || ch == ' '))
            {
                continue;
            }

            // If this is a header line (e.g. | Feature | Status | Notes |)
            if cells.iter().any(|c| c.eq_ignore_ascii_case("status")) {
                let table_headers: Vec<String> = cells.iter().map(|s| s.to_string()).collect();
                in_table = true;

                // Validate header terms
                for header in &table_headers {
                    let lower = header.to_lowercase();
                    for forbidden in FORBIDDEN_HEADER_TERMS {
                        assert!(
                            !lower.contains(forbidden),
                            "Table header '{header}' contains forbidden internal vocabulary '{forbidden}'"
                        );
                    }
                }
                continue;
            }

            if in_table && cells.len() >= 2 {
                // The status column is typically the second column
                let status_cell = cells[1];
                let matches_allowed = ALLOWED_STATUSES.contains(&status_cell);
                assert!(
                    matches_allowed,
                    "Invalid status '{status_cell}' in table row: '{line}'. Allowed: {:?}",
                    ALLOWED_STATUSES
                );
            }
        } else {
            in_table = false;
        }
    }
}

#[test]
fn test_language_support_covers_complete_catalog_inventory() {
    let doc_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("docs/language-support.md");

    let content = fs::read_to_string(&doc_path).expect("failed to read docs/language-support.md");
    let catalog = Catalog::builtin().expect("failed to load builtin catalog");
    let en_us = Locale::new("en-US");

    // 1. Actions
    for entry in catalog.entries_of(Kind::Action) {
        let id_needle = format!("`{}`", entry.id);
        let spelling = entry.spelling(&en_us).unwrap_or(&entry.id);
        let spelling_needle = format!("`{spelling}`");
        assert!(
            content.contains(&id_needle) || content.contains(&spelling_needle),
            "Action '{}' ({}) missing from docs/language-support.md",
            entry.id,
            spelling
        );
    }

    // 2. Values
    for entry in catalog.entries_of(Kind::Value) {
        let id_needle = format!("`{}`", entry.id);
        let spelling = entry.spelling(&en_us).unwrap_or(&entry.id);
        let spelling_needle = format!("`{spelling}`");
        assert!(
            content.contains(&id_needle) || content.contains(&spelling_needle),
            "Value '{}' ({}) missing from docs/language-support.md",
            entry.id,
            spelling
        );
    }

    // 3. Events
    for entry in catalog.entries_of(Kind::Event) {
        let id_needle = format!("`{}`", entry.id);
        let spelling = entry.spelling(&en_us).unwrap_or(&entry.id);
        let spelling_needle = format!("`{spelling}`");
        assert!(
            content.contains(&id_needle) || content.contains(&spelling_needle),
            "Event '{}' ({}) missing from docs/language-support.md",
            entry.id,
            spelling
        );
    }

    // 4. Operators
    for entry in catalog.entries_of(Kind::Operator) {
        let id_needle = format!("`{}`", entry.id);
        let spelling = entry.spelling(&en_us).unwrap_or(&entry.id);
        let spelling_needle = format!("`{spelling}`");
        assert!(
            content.contains(&id_needle) || content.contains(&spelling_needle),
            "Operator '{}' ({}) missing from docs/language-support.md",
            entry.id,
            spelling
        );
    }

    // 5. Structural
    for entry in catalog.entries_of(Kind::Structural) {
        let id_needle = format!("`{}`", entry.id);
        let spelling = entry.spelling(&en_us).unwrap_or(&entry.id);
        let spelling_needle = format!("`{spelling}`");
        assert!(
            content.contains(&id_needle) || content.contains(&spelling_needle),
            "Structural entry '{}' ({}) missing from docs/language-support.md",
            entry.id,
            spelling
        );
    }

    // 6. Enum domains
    for domain in catalog.enum_domains() {
        let domain_needle = format!("`{}`", domain.domain);
        assert!(
            content.contains(&domain_needle),
            "Enum domain '{}' missing from docs/language-support.md",
            domain.domain
        );
    }

    // 7. Settings sections
    for section in ["main", "lobby", "modes", "heroes", "extensions", "workshop"] {
        assert!(
            content.contains(&format!("`{section}`")),
            "Settings section '{section}' missing from docs/language-support.md"
        );
    }

    // 8. Locales
    assert!(
        content.contains("`en-US`"),
        "Locale `en-US` missing from docs/language-support.md"
    );
    assert!(
        content.contains("`zh-CN`"),
        "Locale `zh-CN` missing from docs/language-support.md"
    );

    // 9. Specific highlighted capabilities from issue requirements
    assert!(
        content.contains("`Raise To Power`"),
        "Raise To Power must be explicitly present and identifiable in docs/language-support.md"
    );
}

#[test]
fn test_language_support_distinguishes_unsupported_scope() {
    let doc_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("docs/language-support.md");

    let content = fs::read_to_string(&doc_path).expect("failed to read docs/language-support.md");

    assert!(
        content.contains("Live Workshop runtime / VM simulation"),
        "Must document live runtime execution as unsupported"
    );
    assert!(
        content.contains("Source-language syntax"),
        "Must document source-language specific syntax as unsupported in workshop-rs"
    );
    assert!(
        content.contains("Dynamic script evaluation"),
        "Must document dynamic eval as unsupported"
    );
}
