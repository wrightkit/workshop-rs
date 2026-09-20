# Provenance record

This document records the source provenance of committed datasets and fixtures
in `workshop-rs`, as required for attribution, licensing, and reproducible
regeneration.

Hero/gameplay data follows the separate identity and evidence contract in
[`docs/adr/0007-gameplay-domain-api.md`](adr/0007-gameplay-domain-api.md). The
Workshop catalog identity below does not identify a gameplay dataset.

The repository is MIT-licensed. Committed mapping data is workshop-rs-owned,
with the source and generation method recorded here. The input JSON is a
build-time source artifact and is not redistributed by workshop-rs.

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

### Source provenance per entry group

| Entry group | Source or review basis |
| --- | --- |
| en-US spellings of the supported Workshop surface | Transcribed from the Wright compatibility corpus Workshop snapshots (pinned OverPy 9.7.10 en-US reference emissions) and the Wright support matrix. |
| `squareRoot`, receiver-call action/value spellings (`setMoveSpeed`, `isAlive`, …) | Pinned OverPy 9.7.10 en-US emission surface for the `.opy` forms. |
| Chase family spellings (`Chase Global Variable Over Time`, `Chase Player Variable At Rate`, …) and their expected enum domains (`ChaseTimeReeval`, `ChaseRateReeval`) | Wright-authored OPY semantic manifest probe data, migrated into the canonical catalog so the standalone core resolves ambiguous bare members without any Wright tooling dependency. |
| Rule event identities and filters (`global`, `eachPlayer`, player events, `subroutine`, `EventTeam`, and `EventPlayer`) | User-provided Workshop export at commit `d854bf01fc7bbf3b2169f67408c07a8da8989ad6` (`other.events`, `other.eventTeams`, `other.eventPlayers`, and computed `other.eventSlots`), cross-checked against documented raw Workshop event blocks. |
| OSTW-exercised params/spellings and enum domains (CreateEffect, Workshop Setting, Hero/Map/Button/Icon/Operation/Rounding/InworldTextRev, …) | Pinned OSTW v3.4.0 reference probe emissions (P4/P5/P6/P6b) and the protect-ban entry-point reachable closure. |
| Parameter metadata (`paramDomains`, `paramDefaults`, `paramCoercions`) | Pinned-reference probe data, never copied from upstream game data. Contextual literal substitutions are recorded per parameter position and cross-checked against [OverPy's replacement metadata](https://github.com/Zezombye/overpy/blob/2002431649cbdd7ddc3aa70bd184b598ec2820e5/src/types.d.ts), [Workshop emission path](https://github.com/Zezombye/overpy/blob/2002431649cbdd7ddc3aa70bd184b598ec2820e5/src/compiler/astToWorkshop.ts), and [Wait Until handling](https://github.com/Zezombye/overpy/blob/2002431649cbdd7ddc3aa70bd184b598ec2820e5/src/compiler/functions/waitUntil.ts); they do not establish global type coercions. |
| Localized parameter labels (`paramAliases`) | Separately reviewed owner mappings, stored per declared parameter position and resolved only through the catalog API. The initial `Wait` `zh-CN` labels are `时间` and `等待行为`, as identified by the documented Chinese Workshop action reference at https://overwatch.huijiwiki.com/wiki/%E5%9C%B0%E5%9B%BE%E5%B7%A5%E5%9D%8A/%E5%8A%A8%E4%BD%9C (accessed 2026-09-11); the canonical parameter order and descriptions are cross-checked against the pinned `workshop-data` action definition. They are not generated by the general zh-CN corpus pipeline. |
| Action/Value parameter and return signatures | Workshop.codes structured article properties (Returns, Parameters, Type, and Default), cross-checked against the pinned OverPy metadata and static OSTW data; entries without convergent sources remain explicitly unresolved. Representative article links are embedded in the catalog provenance. |
| `Create Dummy Bot` hero parameter union (`Hero|Array`) | Workshop behavior accepts a single Hero or Hero array and selects randomly when multiple heroes are provided; pinned OverPy 9.7.10 texture-tag setup emits `getAllHeroes()` into `Create Dummy Bot`. |
| Action/Value signature cross-check | Representative Workshop.codes article links remain recorded in the catalog provenance; fetched snapshots and CI results are not generator or runtime inputs. |
| Settings emission table (`src/settings/table.rs` and generated data files) | Hand-written fixture surface plus the reviewed `workshop-data` export at commit `d854bf01fc7bbf3b2169f67408c07a8da8989ad6`; generated entries, names, locale mappings, and source paths are committed together in the declared multi-locale projection, while pinned OverPy 9.7.10 output remains the behavioral check. |

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
* `zh-CN` is an open, source-backed locale. Unmapped spellings remain
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

All committed zh-CN action, value, event, enum, and member spellings come from
the JSON source data through the corpus pipeline. Parameter labels are a separate
owner mapping: the initial `Wait` labels are `时间` and `等待行为`, as identified
by the documented Chinese Workshop action reference cited in the table above;
their parameter order and descriptions are cross-checked against the pinned
`workshop-data` export. The
confirmed legacy mappings use the export identities/GUIDs for
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
artifacts described by `crates/workshop-rs/tests/fixtures/real-projects/provenance.json`; they are source-preserving
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
* `target-source`: recorded in the dataset `target` record.

The source-bound settings inventory is regenerated with
`tools/settings/generate_inventory.py`; its committed output is
`crates/workshop-rs/src/settings/data/inventory.json` and records the input
SHA-256 plus per-entry source paths. The generated Rust settings projection
and catalog surface are then checked with `workshop-catalog-gen check`.

Dataset changes are deliberate: edit the data, update this document and the
dataset `provenance` record, run `workshop-catalog-gen check` then `build`,
and commit data and regenerated file together (repo `AGENTS.md`).

The catalog's `localized_enum_spelling` boundary may expose reviewed partial
locale spellings needed by a consumer while the broader locale remains outside
the complete catalog locale set. Consumer-specific source-language mappings
remain outside `workshop-rs`.
