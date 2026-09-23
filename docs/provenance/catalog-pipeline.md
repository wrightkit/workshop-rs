# Catalog identity and update pipeline

[← Provenance index](README.md)

## Catalog identity and pipeline

* `implementation-version`: `workshop-rs` package version (`Cargo.toml`).
* `catalog-version`: the dataset `version` field; bumped by any dataset
  change.
* `catalog-digest`: sha256 of the canonical (sorted-key) serialization of the
  dataset content excluding the self-referential `digest` field; recomputed
  by `workshop-catalog-gen build` and verified at load and by the pinned
  digest test (`tests/identity.rs`).
* `locale-coverage`: declared locales with per-locale mapped/total counts.
* `target-source`: recorded in the dataset `target` record.

The source-bound settings inventory is regenerated with
`tools/settings/generate_inventory.py`; its committed output is
`crates/workshop-rs/src/settings/data/inventory.json` and records the input
SHA-256 plus per-entry source paths. The generated Rust settings projection
and catalog surface are then checked with `workshop-catalog-gen check`.

Dataset changes are deliberate: edit the data, update this document and the
dataset `provenance` record, run `workshop-catalog-gen check` then `build`,
and commit data and regenerated file together (repo `AGENTS.md`).

The catalog's `localized_enum_spelling` boundary may expose reviewed partial
locale spellings needed by a consumer while the broader locale remains outside
the complete catalog locale set. Consumer-specific source-language mappings
remain outside `workshop-rs`.
