# Architecture Decision Records

ADRs preserve point-in-time architecture decisions and their rationale. They are history, not a database of current implementation reality.

Current durable architecture contracts are routed from [`../architecture/README.md`](../architecture/README.md). Source, tests, Cargo metadata, datasets, and integrations establish current implementation reality. An `Accepted` ADR records an approved decision; it does not by itself prove that later implementation still conforms to it.

Issue and pull-request links in an ADR identify the historical evidence and
origin of a decision. Their current state is maintained by GitHub, not by the
ADR.

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

ADR-0007 was originally committed with a duplicate `ADR-0002` identifier. The
number was corrected to ADR-0007; the recorded gameplay decision is unchanged.
