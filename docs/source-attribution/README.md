# Workshop source attribution

This directory records source attribution for committed datasets and fixtures
in `workshop-rs`, including licensing and the information needed for
reproducible regeneration. It routes the records by data surface so a task can
load only the relevant source material.

Hero/gameplay data follows the separate identity and source-record contract in
[`ADR-0007`](../adr/0007-gameplay-domain-api.md); catalog identity does not
identify a gameplay dataset. The repository is MIT-licensed. Committed mapping
data is workshop-rs-owned, with its source and generation method recorded in
the relevant document.

- [Hero gameplay data](gameplay.md)
- [Catalog, locale, and update pipeline](catalog.md)
- [Test fixtures and code attribution](fixtures-and-code.md)

Current generated data and tests establish implementation reality; these
documents record source identity, licensing, and update constraints.

[`../provenance.md`](../provenance.md) remains available only for older links.
