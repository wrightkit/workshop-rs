# workshop-rs Documentation

This directory is the canonical documentation index for `workshop-rs`. The root
[`README.md`](../README.md) is the user-facing project overview. `workshop-rs`
is both an independently usable raw Workshop implementation and WrightKit's
canonical Workshop semantic core.

## Documentation model

```text
implementation-role.md       standalone implementation and consumer relationship
  └─ docs/adr/               accepted point-in-time architecture decisions
      └─ living references   catalog, gameplay, provenance, conformance, release
          └─ executable evidence   crates, datasets, tests, census, corpus
```

GitHub issues/PRs own active execution scope and sequencing; durable contracts
belong in documentation.

## Architecture and ownership

- [Implementation role](implementation-role.md): standalone Workshop
  implementation identity, canonical ownership, dependency direction from
  `opy-rs` / `del-rs` / Wright, and consumer-driven evolution rules.
- [ADR-0001: Workshop Catalog, Locale, Provenance, and Version Boundaries](adr/0001-catalog-boundaries.md):
  semantic-code/catalog separation, locale-independent identities,
  missing-mapping behavior, and version identity.
- [ADR-0002 (Gameplay): Hero Gameplay Domain API and Provenance Boundary](adr/0002-gameplay-domain-api.md):
  typed hero/ability topology, variants, open identities, and dataset lifecycle.
- [Repository ownership rules](../AGENTS.md): cross-repository routing,
  provenance, validation, and delivery rules.

## Workshop catalog, localization, and provenance

- [Provenance record](provenance.md): catalog, event/filter, locale, gameplay,
  fixture, and dataset provenance.
- [Test fixture provenance](../crates/workshop-rs/tests/fixtures/README.md):
  source origin and verification for raw Workshop/settings fixtures.

## Hero gameplay data and query APIs

- [Hero gameplay dataset](gameplay-data.md): embedded dataset schema, role facts,
  ability keywords, and validation boundaries.
- [Gameplay query API](gameplay-query.md): read-only semantic queries, kit
  lookups, Custom Game calculations, and locale-aware ability resolution.
- [Hero gameplay topology survey](gameplay-roster-survey.md): roster/slot/variant
  evidence used by the domain model.

## Conformance, census, and compatibility evidence

- [ADR-0002: Workshop Conformance Result and Feature Identity Contract](adr/0002-conformance-contract.md):
  stable feature/result/evidence schema.
- [ADR-0003: Canonical Sharded Workshop Feature Census](adr/0003-sharded-census.md):
  deterministic offline census and WIR capability shards.
- [ADR-0004: Provenance-Linked Real-Project Evidence](adr/0004-real-project-evidence.md):
  preserved real-project and minimized regression evidence.
- [ADR-0005: Seasonal Workshop Client Validation Workflow](adr/0005-seasonal-client-validation.md):
  live-client capture and structured drift review.

## Release and operations

- [Release automation](release.md): release/publish workflow, artifacts,
  checksums, and maintainer runbook.

## Authority map

| Contract | Primary document |
| --- | --- |
| Repository/consumer role | [`implementation-role.md`](implementation-role.md) |
| Catalog & semantic boundaries | [`adr/0001-catalog-boundaries.md`](adr/0001-catalog-boundaries.md) |
| Provenance & evidence | [`provenance.md`](provenance.md) |
| Hero gameplay model | [`adr/0002-gameplay-domain-api.md`](adr/0002-gameplay-domain-api.md), [`gameplay-data.md`](gameplay-data.md) |
| Gameplay queries | [`gameplay-query.md`](gameplay-query.md) |
| Conformance | [`adr/0002-conformance-contract.md`](adr/0002-conformance-contract.md) |
| Feature census | [`adr/0003-sharded-census.md`](adr/0003-sharded-census.md) |
| Real-project evidence | [`adr/0004-real-project-evidence.md`](adr/0004-real-project-evidence.md) |
| Seasonal validation | [`adr/0005-seasonal-client-validation.md`](adr/0005-seasonal-client-validation.md) |
| Release & publishing | [`release.md`](release.md) |

> [!NOTE]
> Source-language-specific syntax, runtime lowering, compiler quirks, or
> reconstruction belong to `opy-rs` / `del-rs`, even when those implementations
> consume canonical Workshop contracts from this repository.
