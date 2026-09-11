# workshop-rs Documentation

This directory is the canonical documentation index for `workshop-rs`. The root
[`README.md`](../README.md) is the user-facing overview.

## Documentation model

```text
architecture/README.md       current architecture routing
  └─ current domain contracts
language-support.md          current declared support, backed by executable tests
domain/provenance docs       durable public contracts and fixture provenance
adr/                         point-in-time decisions and rationale
Issues / PRs / releases      mutable execution state
source / tests / data        current implementation reality
```

An accepted ADR is not current-reality evidence. For implementation work, start
from [`architecture/README.md`](architecture/README.md), then inspect the
relevant code/tests/data and current Issue contract.

## Current architecture

- [Architecture routing](architecture/README.md): current contract index and
  authority model.
- [Workshop core boundaries](architecture/core-boundaries.md): repository
  ownership, dependency direction, canonical WIR, source-language separation,
  and semantic-code versus declarative-data boundary.
- [Workshop source layout](architecture/source-layout.md): domain entry points,
  shared implementation boundaries, CLI verification tooling, and test-owned
  fixtures.
- [Repository agent guidance](../AGENTS.md): implementation routing,
  verification, provenance, and delivery rules.

[`implementation-role.md`](implementation-role.md) is retained only as a
compatibility pointer for older links.

## Workshop language support

- [Language support matrix](language-support.md): current declared Workshop
  capability surface, with component inventories in [`language-support/`](language-support/):
  - [Program Structure & Variables](language-support/structure.md)
  - [Events & Event Filters](language-support/events.md)
  - [Conditions & Control Flow](language-support/control-flow.md)
  - [Operators & Variable Modifications](language-support/operators.md)
  - [Actions Inventory](language-support/actions.md)
  - [Values Inventory](language-support/values.md)
  - [Enumerated Domains](language-support/enums.md)
  - [Custom-Game Settings](language-support/settings.md)
  - [Strings & Localization](language-support/strings.md)
  - [Tooling & Semantic Capabilities](language-support/tooling.md)

Support prose is not a substitute for current executable evidence.

## Domain contracts and fixture provenance

- [Provenance record](provenance.md): catalog, locale, gameplay, fixture, and
  dataset provenance.
- [Source preservation contract](source-preservation.md): optional authored
  source, comment attachment, checked edits, and mixed-source boundaries.
- [Hero gameplay dataset](gameplay-data.md): embedded data model and validation
  boundaries.
- [Gameplay query API](gameplay-query.md): read-only semantic queries and
  locale-aware ability resolution.
- [Hero gameplay topology survey](gameplay-roster-survey.md): evidence used by
  the gameplay domain.
- [Canonical action layout](action-layout.md): validated WIR action-width and
  structured action expansion behavior.
- [Test fixture provenance](../crates/workshop-rs/tests/fixtures/README.md):
  source origin and verification for raw Workshop/settings fixtures.

## Architecture decision history

See [ADR registry](adr/README.md). ADRs explain why decisions were made; current
architecture is routed from `docs/architecture/`.

Current registry:

- [ADR-0001: Workshop catalog, locale, provenance, and version boundaries](adr/0001-catalog-boundaries.md)
- [ADR-0002: Workshop conformance result and feature identity contract](adr/0002-conformance-contract.md)
- [ADR-0003: Canonical sharded Workshop feature census](adr/0003-sharded-census.md)
- [ADR-0004: Provenance-linked real-project evidence](adr/0004-real-project-evidence.md)
- [ADR-0005: Seasonal Workshop client validation workflow](adr/0005-seasonal-client-validation.md)
- [ADR-0006: Canonical typed Workshop settings semantics](adr/0006-settings-semantic-schema.md)
- [ADR-0007: Hero gameplay domain API and provenance boundary](adr/0007-gameplay-domain-api.md)

## Release and operations

- [Release automation](release.md): release/publish workflow, artifacts,
  checksums, and maintainer runbook.

> [!NOTE]
> Source-language syntax, runtime lowering, compiler quirks, and reconstruction
> remain owned by `opy-rs` / `deltin-rs`, even when those implementations
> consume canonical Workshop contracts from this repository.
