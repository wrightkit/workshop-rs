use std::fs;
use std::path::{Path, PathBuf};
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

fn get_docs_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("docs")
}

fn get_all_support_doc_paths() -> Vec<PathBuf> {
    let docs_dir = get_docs_dir();
    let mut paths = vec![docs_dir.join("language-support.md")];

    let sub_dir = docs_dir.join("language-support");
    if sub_dir.exists() {
        for entry in fs::read_dir(&sub_dir).expect("failed to read docs/language-support directory")
        {
            let entry = entry.expect("valid entry");
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "md") {
                paths.push(path);
            }
        }
    }
    paths
}

#[test]
fn test_language_support_doc_exists_and_is_authoritative() {
    let doc_path = get_docs_dir().join("language-support.md");

    assert!(
        doc_path.exists(),
        "docs/language-support.md must exist at {}",
        doc_path.display()
    );

    let content = fs::read_to_string(&doc_path).expect("failed to read docs/language-support.md");
    assert!(
        content.contains("# Workshop Language Support Matrix"),
        "docs/language-support.md must contain the main title"
    );
    assert!(
        content.contains("single authoritative index"),
        "docs/language-support.md must declare itself as authoritative"
    );

    // Verify sub-documents exist
    let sub_dir = get_docs_dir().join("language-support");
    for sub in [
        "structure.md",
        "events.md",
        "control-flow.md",
        "operators.md",
        "actions.md",
        "values.md",
        "enums.md",
        "settings.md",
        "strings.md",
        "tooling.md",
    ] {
        let sub_path = sub_dir.join(sub);
        assert!(
            sub_path.exists(),
            "Sub-document {} must exist at {}",
            sub,
            sub_path.display()
        );
    }
}

#[test]
fn test_language_support_statuses_and_headers_are_valid() {
    for doc_path in get_all_support_doc_paths() {
        let content = fs::read_to_string(&doc_path)
            .unwrap_or_else(|_| panic!("failed to read {}", doc_path.display()));

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
                                "File {} table header '{header}' contains forbidden internal vocabulary '{forbidden}'",
                                doc_path.display()
                            );
                        }
                    }
                    continue;
                }

                if in_table && cells.len() >= 2 {
                    let status_cell = cells[1];
                    let matches_allowed = ALLOWED_STATUSES.contains(&status_cell);
                    assert!(
                        matches_allowed,
                        "Invalid status '{status_cell}' in file {} table row: '{line}'. Allowed: {:?}",
                        doc_path.display(),
                        ALLOWED_STATUSES
                    );
                }
            } else {
                in_table = false;
            }
        }
    }
}

#[test]
fn test_language_support_covers_complete_catalog_inventory() {
    let sub_dir = get_docs_dir().join("language-support");
    let catalog = Catalog::builtin().expect("failed to load builtin catalog");
    let en_us = Locale::new("en-US");

    // 1. Actions
    let actions_content = fs::read_to_string(sub_dir.join("actions.md"))
        .expect("failed to read docs/language-support/actions.md");
    for entry in catalog.entries_of(Kind::Action) {
        let id_needle = format!("`{}`", entry.id);
        let spelling = entry.spelling(&en_us).unwrap_or(&entry.id);
        let spelling_needle = format!("`{spelling}`");
        assert!(
            actions_content.contains(&id_needle) || actions_content.contains(&spelling_needle),
            "Action '{}' ({}) missing from actions.md",
            entry.id,
            spelling
        );
    }

    // 2. Values
    let values_content = fs::read_to_string(sub_dir.join("values.md"))
        .expect("failed to read docs/language-support/values.md");
    for entry in catalog.entries_of(Kind::Value) {
        let id_needle = format!("`{}`", entry.id);
        let spelling = entry.spelling(&en_us).unwrap_or(&entry.id);
        let spelling_needle = format!("`{spelling}`");
        assert!(
            values_content.contains(&id_needle) || values_content.contains(&spelling_needle),
            "Value '{}' ({}) missing from values.md",
            entry.id,
            spelling
        );
    }

    // 3. Events
    let events_content = fs::read_to_string(sub_dir.join("events.md"))
        .expect("failed to read docs/language-support/events.md");
    for entry in catalog.entries_of(Kind::Event) {
        let id_needle = format!("`{}`", entry.id);
        let spelling = entry.spelling(&en_us).unwrap_or(&entry.id);
        let spelling_needle = format!("`{spelling}`");
        assert!(
            events_content.contains(&id_needle) || events_content.contains(&spelling_needle),
            "Event '{}' ({}) missing from events.md",
            entry.id,
            spelling
        );
    }

    // 4. Operators
    let operators_content = fs::read_to_string(sub_dir.join("operators.md"))
        .expect("failed to read docs/language-support/operators.md");
    for entry in catalog.entries_of(Kind::Operator) {
        let id_needle = format!("`{}`", entry.id);
        let spelling = entry.spelling(&en_us).unwrap_or(&entry.id);
        let spelling_needle = format!("`{spelling}`");
        assert!(
            operators_content.contains(&id_needle) || operators_content.contains(&spelling_needle),
            "Operator '{}' ({}) missing from operators.md",
            entry.id,
            spelling
        );
    }

    // 5. Structural
    let structure_content = fs::read_to_string(sub_dir.join("structure.md"))
        .expect("failed to read docs/language-support/structure.md");
    let control_flow_content = fs::read_to_string(sub_dir.join("control-flow.md"))
        .expect("failed to read docs/language-support/control-flow.md");
    let settings_content = fs::read_to_string(sub_dir.join("settings.md"))
        .expect("failed to read docs/language-support/settings.md");
    let combined_structural =
        format!("{structure_content}\n{control_flow_content}\n{settings_content}");
    for entry in catalog.entries_of(Kind::Structural) {
        let id_needle = format!("`{}`", entry.id);
        let spelling = entry.spelling(&en_us).unwrap_or(&entry.id);
        let spelling_needle = format!("`{spelling}`");
        assert!(
            combined_structural.contains(&id_needle)
                || combined_structural.contains(&spelling_needle),
            "Structural entry '{}' ({}) missing from structure.md / control-flow.md / settings.md",
            entry.id,
            spelling
        );
    }

    // 6. Enum domains
    let enums_content = fs::read_to_string(sub_dir.join("enums.md"))
        .expect("failed to read docs/language-support/enums.md");
    for domain in catalog.enum_domains() {
        let domain_needle = format!("`{}`", domain.domain);
        assert!(
            enums_content.contains(&domain_needle),
            "Enum domain '{}' missing from enums.md",
            domain.domain
        );
    }

    // 7. Settings sections
    let settings_content = fs::read_to_string(sub_dir.join("settings.md"))
        .expect("failed to read docs/language-support/settings.md");
    for section in ["main", "lobby", "modes", "heroes", "extensions", "workshop"] {
        assert!(
            settings_content.contains(&format!("`{section}`")),
            "Settings section '{section}' missing from settings.md"
        );
    }

    // 8. Locales
    let strings_content = fs::read_to_string(sub_dir.join("strings.md"))
        .expect("failed to read docs/language-support/strings.md");
    assert!(
        strings_content.contains("`en-US`"),
        "Locale `en-US` missing from strings.md"
    );
    assert!(
        strings_content.contains("`zh-CN`"),
        "Locale `zh-CN` missing from strings.md"
    );

    // 9. Specific highlighted capabilities from issue requirements and native audit (#86)
    assert!(
        operators_content.contains("`Raise To Power`"),
        "Raise To Power must be explicitly present and identifiable in operators.md"
    );
    assert!(
        values_content.contains("`Raise To Power` (Value)"),
        "Raise To Power (Value) coming soon contract missing from values.md"
    );
    assert!(
        values_content.contains("`Randomized Array`"),
        "Randomized Array coming soon contract missing from values.md"
    );
    assert!(
        values_content.contains("`String` | 🚧 Coming soon"),
        "String value coming soon contract missing from values.md"
    );
}

#[test]
fn test_language_support_distinguishes_unsupported_scope() {
    let doc_path = get_docs_dir().join("language-support.md");
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
