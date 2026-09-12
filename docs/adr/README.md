# Architecture Decision Records

ADRs preserve point-in-time architecture decisions and their rationale. They are history, not a database of current implementation reality.

Current durable architecture contracts are routed from [`../architecture/README.md`](../architecture/README.md). Source, tests, Cargo metadata, datasets, and integrations establish current implementation reality. An `Accepted` ADR records an approved decision; it does not by itself prove that later implementation still conforms to it.

## Conventions

- ADR numbers are unique, zero-padded, and never reused for a different decision.
- `Proposed` means the decision was recorded but not yet accepted.
- `Accepted` means the decision was approved at that point in project history.
- `Superseded` decisions remain in place and link to the replacing decision or current contract.
- Do not rewrite an accepted ADR to make old implementation details look current. Put the current invariant in `docs/architecture/` and record a new decision when the architecture materially changes.
- Versions, support counts, Issue progress, migration state, and other mutable reality do not belong in ADR status prose unless they are part of the historical context being recorded.

## Index

- [ADR-0001: Workshop catalog, locale, provenance, and version boundaries](0001-catalog-boundaries.md)
- [ADR-0002: Workshop conformance result and feature identity contract](0002-conformance-contract.md)
- [ADR-0003: Canonical sharded Workshop feature census](0003-sharded-census.md)
- [ADR-0004: Provenance-linked real-project and minimized evidence](0004-real-project-evidence.md)
- [ADR-0005: Seasonal Workshop client validation workflow](0005-seasonal-client-validation.md)
- [ADR-0006: Canonical typed Workshop settings semantics](0006-settings-semantic-schema.md)
- [ADR-0007: Hero gameplay domain API and provenance boundary](0007-gameplay-domain-api.md)
- [ADR-0008: Canonical public Workshop `Program` boundary](0008-canonical-public-program-boundary.md)
- [ADR-0009: Domain-local Workshop ownership and verification placement](0009-domain-local-ownership.md)
- [ADR-0010: Canonical Workshop target-layout and resource analysis](0010-target-layout-and-resource-analysis.md)
- [ADR-0011: Contextual Workshop semantics at the catalog/code boundary](0011-contextual-semantic-placement.md)

ADR-0007 was originally committed with a duplicate `ADR-0002` identifier. The 0007 number is a registry correction only; it does not change the recorded gameplay decision.

## Post-registry audit (#192)

The 2026-09-12 audit compared the ADR registry with the historical Issue/PR
record and the current architecture contracts. It classified material choices
as follows:

| Historical choice | Classification | Decision record |
| --- | --- | --- |
| #32, #112, #129, #177–#179, #187, and their implementation PRs | Backfill required: canonical public model, provider-carrier boundary, and optional source/provenance | [ADR-0008](0008-canonical-public-program-boundary.md) |
| #89 and #102 | Backfill required: Workshop-owned structured resource and target-layout APIs | [ADR-0010](0010-target-layout-and-resource-analysis.md) |
| #135 | Backfill required: positional catalog facts interpreted by typed contextual semantics | [ADR-0011](0011-contextual-semantic-placement.md) |
| #149, #150, #152 | Backfill required: domain-first ownership, phase contexts as orchestration, and removal of `evidence` as a semantic domain | [ADR-0009](0009-domain-local-ownership.md) |
| #109–#111, #117, #144, and #174 | Existing-ADR-covered settings identity, applicability, and typed edit decisions; source-preservation aspects remain covered by ADR-0008 | [ADR-0006](0006-settings-semantic-schema.md) |
| #22–#25, #35, and #36 | Existing-ADR-covered gameplay identity, topology, dataset, and query boundary | [ADR-0007](0007-gameplay-domain-api.md) |
| #1, #2, #65, #86, #87, and #176 | Existing-ADR-covered catalog, conformance, provenance, and locale-mapping boundaries | [ADR-0001](0001-catalog-boundaries.md), [ADR-0002](0002-conformance-contract.md), [ADR-0003](0003-sharded-census.md), [ADR-0004](0004-real-project-evidence.md), [ADR-0005](0005-seasonal-client-validation.md) |
| Release tags, documentation polish, CI/build policy, and focused formatting fixes (#164, #165, #168, #171, #172, #190, #191) | Non-ADR detail or consequence; no durable ownership, public-boundary, representation, or semantic/data-placement decision found | — |

No unresolved architectural decision was found in this audit. Current
architecture documents remain the authority for present implementation
boundaries; these ADRs preserve the historical rationale without turning Issue
status, versions, counts, or migration state into architecture facts.
