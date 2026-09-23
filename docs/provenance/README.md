# Workshop source attribution

This directory records the source attribution of committed datasets and fixtures
in `workshop-rs`, as required for attribution, licensing, and reproducible
regeneration. It routes those contracts by owned data surface so a task can load
only the relevant source class.

Hero/gameplay data follows the separate identity and source-record contract in
[`ADR-0007`](../adr/0007-gameplay-domain-api.md); catalog identity does not
identify a gameplay dataset. The repository is MIT-licensed. Committed mapping
data is workshop-rs-owned, with source and generation method recorded in the
relevant document.

- [Hero gameplay data](gameplay.md)
- [Catalog and locale data](catalog.md)
- [Test fixtures and code provenance](fixtures-and-code.md)
- [Catalog identity and update pipeline](catalog-pipeline.md)

The repository's source-attribution requirement remains unchanged by this split.
Current generated data and tests establish implementation reality; these
documents record durable source identity, licensing, and update constraints.

[`../provenance.md`](../provenance.md) is retained only as a compatibility
pointer for older links.
