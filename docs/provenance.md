# Provenance record

This document records the source and evidence class of every committed dataset
and fixture in `workshop-rs`, per the workspace evidence
hierarchy (reproducible behavior > accepted contracts > tests and fixtures >
consumer projects > upstream references > documented community evidence >
assumptions) and ADR-0001 Decision 6 (provenance and the reproducible
catalog-update pipeline).

Hero/gameplay data follows the separate identity and evidence contract in
[`docs/adr/0002-gameplay-domain-api.md`](adr/0002-gameplay-domain-api.md). The
Workshop catalog identity below does not identify a gameplay dataset.

The repository is MIT-licensed. Committed mapping data is workshop-rs-owned,
with the source evidence and generation method recorded here. The input JSON
is a build-time evidence artifact and is not redistributed by workshop-rs.

## Hero gameplay data (`crates/workshop-rs/src/data/gameplay.json`)

The gameplay dataset is a workshop-rs-owned, MIT-compatible projection of the
user-provided `workshop-data/workshop-data.json` export. Its source is pinned
to commit `d854bf01fc7bbf3b2169f67408c07a8da8989ad6` (commit date
2026-08-12). The export is used only for hero identity, localized naming, and
declared named ability-slot topology; no OverPy or OSTW data is copied.

The committed projection contains hero identities, role facts, and
export-declared named ability slots, plus official-detail variant records for
Bastion, D.Va, and Ramattra. Each hero/ability name fact and export record
carries the export path as an `EvidenceRef`. Each role fact carries its
official Blizzard hero-detail URL and access date 2026-08-18 as separate
evidence.

The current identity digest is
`sha256:5c01599839834f3599a524c7307d3ceaa493e6a1e845d9884dc9617f2af4068a`.

Representative ability keywords are semantic labels, not Blizzard or Workshop
enum values. Their labels and the six variant names/shapes are evidenced by
the official Blizzard hero-detail URLs for Ana, Brigitte, Ramattra, D.Va,
Bastion, and Venture, accessed 2026-08-18;
the variants intentionally carry no fabricated Workshop-export provenance.
Venture base health and Drill Dash cooldown/damage are intentionally absent
because the cited June 30, 2026 Community Crafted patch source is scoped to a
limited mode and that scope is not modeled. Other base stats, armor/shields, cooldowns, damage,
healing, ammo, durations, ranges, projectile speeds, resources, and balance
values remain absent unless supported by a current explicit source. The
loader verifies the separate gameplay dataset identity and deterministic
SHA-256 digest; it does not alter the Workshop catalog identity.

## Catalog data (`crates/workshop-rs/src/catalog/data/catalog.json`)

The catalog dataset is WrightKit-authored data with recorded provenance,
transferred to `workshop-rs` on 2026-08-16 as the canonical Workshop catalog
(workspace ownership contract; Wright issue #136 / ADR-0009 direction).
Its machine-readable provenance record (generator, generator version, source,
license, reviewed) is embedded in the dataset itself and surfaced by
`workshop-rs-cli version --json`.

### Evidence classes per entry group

| Entry group | Evidence class |
| --- | --- |
| en-US spellings of the supported Workshop surface | Transcribed from the Wright compatibility corpus workshop snapshots (pinned OverPy 9.7.10 en-US reference emissions) and the Wright support matrix — classes 1/5 (reproducible behavior; upstream reference emission). |
| `squareRoot`, receiver-call action/value spellings (`setMoveSpeed`, `isAlive`, …) | Pinned OverPy 9.7.10 en-US emission surface for the `.opy` forms (class 5). |
| Chase family spellings (`Chase Global Variable Over Time`, `Chase Player Variable At Rate`, …) and their expected enum domains (`ChaseTimeReeval`, `ChaseRateReeval`) | Wright-authored OPY semantic manifest probe data (#109/#110), migrated into the canonical catalog so the standalone core resolves ambiguous bare members without any Wright tooling dependency (classes 1/5; canonical signature data is catalog-owned per ADR-0001 Decision 1). |
| Rule event identities and filters (`global`, `eachPlayer`, player events, `subroutine`, `EventTeam`, and `EventPlayer`) | User-provided Workshop export at commit `d854bf01fc7bbf3b2169f67408c07a8da8989ad6` (`other.events`, `other.eventTeams`, `other.eventPlayers`, and computed `other.eventSlots`), cross-checked against documented raw Workshop event blocks; the canonical WIR keeps existing parameterless `eachPlayer` input and requires the evidenced team/player filters for other filtered events. The `Player` filter's accepted union (`EventPlayer` slot/all or a canonical `Hero`) is represented explicitly by `EventTarget` (classes 1/2). |
| OSTW-exercised params/spellings and enum domains (CreateEffect, Workshop Setting, Hero/Map/Button/Icon/Operation/Rounding/InworldTextRev, …) | Pinned OSTW v3.4.0 reference probe emissions (P4/P5/P6/P6b) and the protect-ban entry-point reachable closure (class 5). |
| Parameter metadata (`paramDomains`, `paramDefaults`, `paramCoercions`) | Pinned-reference probe evidence (classes 1/5), never copied from upstream game data. Contextual literal substitutions are recorded per parameter position and cross-checked against [OverPy's replacement metadata](https://github.com/Zezombye/overpy/blob/master/src/types.d.ts), [Workshop emission path](https://github.com/Zezombye/overpy/blob/master/src/compiler/astToWorkshop.ts), and [Wait Until handling](https://github.com/Zezombye/overpy/blob/master/src/compiler/functions/waitUntil.ts); they do not establish global type coercions. |
| Action/Value parameter and return signatures | Workshop.codes structured article properties (Returns, Parameters, Type, and Default), cross-checked against the pinned OverPy metadata and static OSTW data; entries without convergent evidence remain explicitly evidence-insufficient. Representative article links are embedded in the catalog provenance. |
| Action/Value signature cross-check | Representative Workshop.codes article links remain recorded in the catalog provenance; fetched snapshots and acceptance results are CI evidence, not generator or runtime inputs. |
| Settings emission table (`src/settings/table.rs` and generated data files) | Hand-written fixture surface plus the reviewed `workshop-data` export at commit `d854bf01fc7bbf3b2169f67408c07a8da8989ad6`; generated entries, names, locale mappings, and source paths are committed together in the declared multi-locale projection, while pinned OverPy 9.7.10 output remains the behavioral check (classes 1/5). |

### Native display action boundary (`workshop-rs#129`)

The independent [Workshop.codes action inventory](https://workshop.codes/wiki/categories/actions)
contains no native `Debug` or `Print` action, while its [`Create HUD Text`]
article documents the native display action, its eleven parameters, and its
persistent HUD behavior. The catalog therefore represents HUD output through
the canonical `createHudText` action call. OverPy `debug(...)` and `print(...)`
remain source-language helpers whose lowering belongs to their provider, not to
canonical WIR.

[`Create HUD Text`]: https://workshop.codes/wiki/articles/create-hud-text

### Locale coverage

* `en-US` is the primary locale and is complete. The committed catalog
  validates that the primary locale is complete.
* `zh-CN` is an open, evidence-backed locale. Unmapped spellings remain
  explicit and fail closed; `workshop-catalog-gen check` is the authoritative
  coverage report. The source for the reconciled additions is the user-provided
  `workshop-data/workshop-data.json` export at commit
  `d854bf01fc7bbf3b2169f67408c07a8da8989ad6`; the export is not committed.
* The settings locale corpus
  `crates/workshop-rs/src/settings/data/locales.json` has a machine-written
  `coverage` header from an earlier full corpus pipeline run. The heroes and
  maps sections have since grown through reviewed additions beyond that
  recorded coverage; the header understates the committed sections until the
  next `workshop-catalog-gen corpus` run with a current export, and must not be
  hand-refreshed (repository `AGENTS.md`).
  Each mapping records its export source paths; settings without a mapping
  continue to fail explicitly.

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

The `settings.workshop` namespace aliases (`workshop` / `地图工坊`) and the
Wrecking Ball cooldown labels are pinned from the reacquired AI-PVE and Bastion
artifacts described by `docs/evidence/raw-workshop-real-projects-v1.json`; they are source-preserving
custom data or producer aliases, not new builtin gameplay identities.

## Test fixtures (`crates/workshop-rs/tests/fixtures/`)

The corpus Workshop texts and settings sections were extracted from the
Wright repository's compatibility fixtures (pinned OverPy 9.7.10 en-US
reference emissions) on 2026-08-16. The spellings are Blizzard game content
(functional/interoperability data); the texts are observed reference
behavior, not OverPy source. Full provenance, extraction method, and
per-file SHA-256 verification are recorded in
[`crates/workshop-rs/tests/fixtures/README.md`](../crates/workshop-rs/tests/fixtures/README.md).
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

The source-bound settings inventory is regenerated with
`tools/settings/generate_inventory.py`; its committed output is
`crates/workshop-rs/src/settings/data/inventory.json` and records the input
SHA-256 plus per-entry source paths. The generated Rust settings projection
and catalog surface are then checked with `workshop-catalog-gen check`.

Dataset changes are deliberate: edit the data, update this document and the
dataset `provenance` record, run `workshop-catalog-gen check` then `build`,
and commit data and regenerated file together (repo `AGENTS.md`).
