# Provenance record

This document records the source and evidence class of every committed dataset
and fixture in `workshop-rs`, per the workspace evidence
hierarchy (reproducible behavior > accepted contracts > tests and fixtures >
consumer projects > upstream references > documented community evidence >
assumptions) and ADR-0001 Decision 6 (provenance and the reproducible
catalog-update pipeline).

The repository is MIT-licensed. Committed mapping data is workshop-rs-owned,
with the source evidence and generation method recorded here. The input JSON
is a build-time evidence artifact and is not redistributed by workshop-rs.

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

* `en-US` is the primary locale and is complete (341/341 canonical entries:
  165 builtins + 176 enum members). The committed catalog validates that the
  primary locale is complete.
* `zh-CN` has an evidence-backed corpus of **341/341** canonical entries:
  structural 11/11, actions 60/60, values 77/77, events 3/3, operators 14/14,
  and enum members 176/176. The reproducible manifest is
  `tools/corpus/zh-cn-corpus.json`; it records exact en-US spelling matches,
  every exclusion, and the export provenance. The source is the user-provided
  `workshop-data/workshop-data.json` export at commit
  `d854bf01fc7bbf3b2169f67408c07a8da8989ad6`, commit date 2026-08-12, fetched
  2026-08-17. The export is not committed to this repository.
* The generated settings corpus covers labels 19/19, modes 7/7, maps 2/2,
  heroes 10/10, enum values 2/2, tokens 3/3, and teams 1/1. Its exact-match
  exclusions are recorded in
  `crates/workshop-rs/src/settings/data/zh-cn.json`; settings without a
  mapping continue to fail explicitly.

All committed zh-CN spellings come from the JSON evidence through the corpus
pipeline. The confirmed legacy mappings use the export identities/GUIDs for
global stop-chasing, force hero/throttle, `Set Player Allowed Heroes`, and
the four bare comparison symbols. The three enum aliases use exact export
identity/GUID matches: Lijiang Tower Lunar New Year, Visible To and Values,
and To Nearest. The two hero settings labels are composed only after exact
template and Blizzard hero identity/GUID checks. Following the explicit
product decision, `Delete All Classes`, `Chase Variable At Rate`, and `Array
Element` are not declared Workshop identities: they are legacy/provider syntax
sugar represented by the corresponding canonical Workshop identities. The
declared corpus is therefore complete and contains no silent exclusions.

## Test fixtures (`tests/fixtures/`)

The corpus Workshop texts and settings sections were extracted from the
Wright repository's compatibility fixtures (pinned OverPy 9.7.10 en-US
reference emissions) on 2026-08-16. The spellings are Blizzard game content
(functional/interoperability data); the texts are observed reference
behavior, not OverPy source. Full provenance, extraction method, and
per-file SHA-256 verification are recorded in `tests/fixtures/README.md`.
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
