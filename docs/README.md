# workshop-rs Documentation

This directory is the canonical documentation index for `workshop-rs`. The root
[`README.md`](../README.md) is the user-facing project overview and quick start;
detailed architecture specifications, ADRs, catalog and gameplay domain contracts,
conformance methodology, provenance records, and release runbooks live here.

## Documentation Model

```text
GitHub Issues (Product scope, roadmap, issue sequencing)
  └─ docs/adr/ (Accepted point-in-time architecture decision records)
      └─ docs/ (Living reference contracts: catalog, gameplay, provenance, release)
          └─ Implementation & Executable Evidence (Crates, datasets, tests, census, corpus)
```

## Index

### Architecture and Semantic Ownership

- [ADR-0001: Workshop Catalog, Locale, Provenance, and Version Boundaries](adr/0001-catalog-boundaries.md):
  Contract between semantic code and data allowlists, locale-independent canonical
  identities, missing-mapping behavior, and four-tier version identification.
- [ADR-0002 (Gameplay): Hero Gameplay Domain API and Provenance Boundary](adr/0002-gameplay-domain-api.md):
  Architecture for typed hero/ability kit topologies, variant representation, open
  identities (`AbilityRef`, `LogicalSlot`, `Fact`, `Quantity`), and decoupled dataset lifecycles.
- [Repository Ownership Rules](../AGENTS.md):
  Ownership boundaries within WrightKit, clean-room MIT licensing, and development workflows.

### Workshop Catalog, Localization, and Provenance

- [Provenance Record](provenance.md):
  Complete provenance ledger for catalog entries, rule event identities/filters,
  locale coverage (`en-US`, `zh-CN`), test fixtures, code provenance, and deterministic
  dataset content digests.
- [Test Fixture Provenance](../crates/workshop-rs/tests/fixtures/README.md):
  Source origin, extraction methods, and SHA-256 verification hashes for raw Workshop
  and settings test fixtures.

### Hero Gameplay Data and Query APIs

- [Hero Gameplay Dataset](gameplay-data.md):
  Embedded hero dataset (`src/data/gameplay.json`), schema specification, role facts,
  ability keywords, and strict validation boundaries.
- [Gameplay Query API](gameplay-query.md):
  Read-only semantic query interface (`GameplayCatalog::query()`), deterministic lookups,
  Custom Game cooldown percentage calculations, and locale-aware ability name resolution.
- [Hero Gameplay Topology Survey](gameplay-roster-survey.md):
  Pinned 53-hero roster survey, logical slot topology categorization, and form/configuration
  variant evidence.

### Conformance, Census, and Compatibility Evidence

- [ADR-0002: Workshop Conformance Result and Feature Identity Contract](adr/0002-conformance-contract.md):
  Schema and classification rules for `FeatureId`, `ConformanceResult`, and `Evidence`
  records across all evidence tracks.
- [ADR-0003: Canonical Sharded Workshop Feature Census](adr/0003-sharded-census.md):
  Deterministic offline census runner (`workshop-rs-cli census`), WIR capability shards,
  and report validation.
- [ADR-0004: Provenance-Linked Real-Project Evidence](adr/0004-real-project-evidence.md):
  Offline execution of preserved real-world Workshop scripts (`overpy-cake.ws`) and
  minimized regression fixtures.
- [ADR-0005: Seasonal Workshop Client Validation Workflow](adr/0005-seasonal-client-validation.md):
  Maintainer workflow for live Overwatch client capture verification, structured drift
  diffing (`seasonal-diff`), and manual verification boundaries.

### Release and Operations

- [Release Automation](release.md):
  Automated dual-crate release pipeline (`release-plz`), GitHub Actions configuration,
  multi-platform CLI artifact distribution, checksum verification, and maintainer runbook.

## Authority and Contract Map

| Contract Class | Primary Document | Normative Scope |
| --- | --- | --- |
| **Catalog & Semantic Boundaries** | [`adr/0001-catalog-boundaries.md`](adr/0001-catalog-boundaries.md) | Separation of code vs catalog data, allowlist model, missing-locale fail-explicit policy. |
| **Provenance & Evidence** | [`provenance.md`](provenance.md) | Evidence hierarchy, entry group sources, locale corpus provenance, dataset digests. |
| **Hero Gameplay Model** | [`adr/0002-gameplay-domain-api.md`](adr/0002-gameplay-domain-api.md), [`gameplay-data.md`](gameplay-data.md) | Hero kit schemas, `AbilityRef` identities, open slots, keyword labels, evidence references. |
| **Gameplay Query API** | [`gameplay-query.md`](gameplay-query.md) | Deterministic lookup semantics, cooldown percentage math, localized ability resolution. |
| **Conformance & Evidence Framework** | [`adr/0002-conformance-contract.md`](adr/0002-conformance-contract.md) | Stable feature IDs, result states (`matched`, `known-gap`, etc.), evidence provenance. |
| **Feature Census** | [`adr/0003-sharded-census.md`](adr/0003-sharded-census.md) | Shard definitions, offline execution, semantic round-trip validation. |
| **Real-Project Evidence** | [`adr/0004-real-project-evidence.md`](adr/0004-real-project-evidence.md) | Preserved project manifest admission, regression detection, offline CLI runner. |
| **Seasonal Validation** | [`adr/0005-seasonal-client-validation.md`](adr/0005-seasonal-client-validation.md) | Live-capture schema, structured drift classification, maintainer import/export procedure. |
| **Release & Publishing** | [`release.md`](release.md) | Version grouping, `release-plz` workflow, crate publish order, artifact packaging. |

> [!NOTE]
> GitHub issues and pull requests define active execution scope and acceptance criteria.
> Documents in this directory record durable architecture, interfaces, contracts, and evidence.
