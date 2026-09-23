# Test fixtures and code provenance

[← Provenance index](README.md)

## Test fixtures (`crates/workshop-rs/tests/fixtures/`)

The corpus Workshop texts and settings sections were extracted from the
Wright repository's compatibility fixtures (pinned OverPy 9.7.10 en-US
reference emissions) on 2026-08-16. The spellings are Blizzard game content
(functional/interoperability data); the texts are observed reference
behavior, not OverPy source. Full provenance, extraction method, and
per-file SHA-256 verification are recorded in
[`crates/workshop-rs/tests/fixtures/README.md`](../../crates/workshop-rs/tests/fixtures/README.md).
The committed fixtures are reference-emission inputs with per-file hashes;
the JSON evidence used to generate locale mappings is not committed.

## Code provenance

The implementation modules are Wright-authored code extracted from the
`wright-workshop`, `wright-ir` (the `wir`, `settings`, `source` subset), and
`wright-core` (`signatures`) crates, adapted to be standalone: the
`ExpectedDomain` contract was copied into `crate::signatures`, and
`wright_*` crate references were replaced with `workshop_rs` module paths.
No third-party implementation internals were copied or mechanically
translated.
