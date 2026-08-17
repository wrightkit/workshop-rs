# Provenance and licensing record

This document records the source, evidence class, and license status of every
committed dataset and fixture in `workshop-rs`, per the workspace evidence
hierarchy (reproducible behavior > accepted contracts > tests and fixtures >
consumer projects > upstream references > documented community evidence >
assumptions) and ADR-0001 Decision 6 (provenance and the reproducible
catalog-update pipeline).

The repository is MIT-licensed. Committed data must be MIT-compatible with
recorded provenance. OverPy's translation tables are GPL-3.0 reference data
and are **not** a permissible source for catalog or locale data (Wright
ADR-0004, Wright `docs/licensing.md`). Observed reference behavior is an
interoperability input, not permission to copy an implementation.

## Catalog data (`src/catalog/data/catalog.json`)

The catalog dataset is WrightKit-authored data with recorded provenance,
transferred to `workshop-rs` on 2026-08-16 as the canonical Workshop catalog
(workspace ownership contract; Wright issue #136 / ADR-0009 direction).
Its machine-readable provenance record (generator, generator version, source,
license, reviewed) is embedded in the dataset itself and surfaced by
`workshop-rs-cli version --json`.

### Evidence classes per entry group

| Entry group | Evidence class |
| --- | --- |
| en-US spellings of the M5 P0 surface | Transcribed from the Wright compatibility corpus workshop snapshots (pinned OverPy 9.7.10 en-US reference emissions) and the Wright M5 support matrix — classes 1/5 (reproducible behavior; upstream reference emission). |
| `squareRoot`, receiver-call action/value spellings (`setMoveSpeed`, `isAlive`, …) | Pinned OverPy 9.7.10 en-US emission surface for the `.opy` forms (class 5). |
| Chase family spellings (`Chase Global Variable Over Time`, `Chase Player Variable At Rate`, …) and their expected enum domains (`ChaseTimeReeval`, `ChaseRateReeval`) | Wright-authored OPY semantic manifest probe data (#109/#110), migrated into the canonical catalog so the standalone core resolves ambiguous bare members without any Wright tooling dependency (classes 1/5; canonical signature data is catalog-owned per ADR-0001 Decision 1). |
| OSTW-exercised params/spellings and enum domains (CreateEffect, Workshop Setting, Hero/Map/Button/Icon/Operation/Rounding/InworldTextRev, …) | Pinned OSTW v3.4.0 reference probe emissions (P4/P5/P6/P6b) and the protect-ban entry-point reachable closure (class 5). |
| Parameter metadata (`paramDomains`, `paramDefaults`) | Pinned-reference probe evidence (classes 1/5), never copied from upstream game data. |
| Settings emission table (`src/settings/table.rs`) | Observed from pinned OverPy 9.7.10 en-US output of the oracle-success settings programs (classes 1/5); provenance is recorded in the table header itself. |

### Locale coverage

* `en-US` is the primary locale and is complete (344/344 canonical entries:
  168 builtins + 176 enum members). The committed catalog validates that the
  primary locale is complete.
* `zh-CN` has an evidence-backed corpus of **327/344** canonical entries:
  structural 11/11, actions 55/62, values 77/78, events 3/3, operators 8/14,
  and enum members 173/176. The reproducible manifest is
  `tools/corpus/zh-cn-corpus.json`; it records exact en-US spelling matches,
  every exclusion, and the export provenance. The source is the user-provided
  `workshop-data/workshop-data.json` export at commit
  `d854bf01fc7bbf3b2169f67408c07a8da8989ad6`, commit date 2026-08-12, fetched
  2026-08-17. The export is not committed to this repository.
* The generated settings corpus covers labels 17/19, modes 7/7, maps 2/2,
  heroes 10/10, enum values 2/2, tokens 3/3, and teams 1/1. Its exact-match
  exclusions are recorded in
  `crates/workshop-rs/src/settings/data/zh-cn.json`; settings without a
  mapping continue to fail explicitly. The data's license review remains
  marked pending until the Blizzard-content redistribution review is recorded.

All committed zh-CN spellings come from the export through the corpus
pipeline; no OverPy translation table is used. The complete catalog coverage
and settings gate remains open for the recorded exclusions.

## Test fixtures (`tests/fixtures/`)

The corpus Workshop texts and settings sections were extracted from the
Wright repository's compatibility fixtures (pinned OverPy 9.7.10 en-US
reference emissions) on 2026-08-16. The spellings are Blizzard game content
(functional/interoperability data); the texts are observed reference
behavior, not OverPy source. Full provenance, extraction method, and
per-file SHA-256 verification are recorded in `tests/fixtures/README.md`.
Final redistribution review of the migrated corpus is tracked with the
first-release gate.

## Code provenance

The implementation modules are Wright-authored code extracted from the
`wright-workshop`, `wright-ir` (the `wir`, `settings`, `source` subset), and
`wright-core` (`signatures`) crates, adapted to be standalone: the
`ExpectedDomain` contract was copied into `crate::signatures`, and
`wright_*` crate references were replaced with `workshop_rs` module paths.
No third-party implementation internals were copied or mechanically
translated.

## Catalog identity and pipeline

* `implementation-version`: `workshop-rs` package version (`Cargo.toml`).
* `catalog-version`: the dataset `version` field; bumped by any dataset
  change.
* `catalog-digest`: sha256 of the canonical (sorted-key) serialization of the
  dataset content excluding the self-referential `digest` field; recomputed
  by `workshop-catalog-gen build` and verified at load and by the pinned
  digest test (`tests/identity.rs`).
* `locale-coverage`: declared locales with per-locale mapped/total counts.
* `target-evidence`: recorded in the dataset `target` record.

Dataset changes are deliberate: edit the data, update this document and the
dataset `provenance` record, run `workshop-catalog-gen check` then `build`,
and commit data and regenerated file together (repo `AGENTS.md`).
