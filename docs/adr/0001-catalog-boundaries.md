# ADR-0001: Workshop catalog, locale, provenance, and version boundaries

- Status: Proposed
- Date: 2026-08-16
- Related:
  [Issue #1](https://github.com/wrightkit/workshop-rs/issues/1),
  [Issue #2](https://github.com/wrightkit/workshop-rs/issues/2);
  Wright workspace contracts:
  [AGENTS.md](https://github.com/wrightkit/wright/blob/main/AGENTS.md),
  [ADR-0004: OverPy licensing and clean-room boundary](https://github.com/wrightkit/wright/blob/main/docs/adr/0004-overpy-licensing-boundary.md),
  [ADR-0007: Reference pinning policy](https://github.com/wrightkit/wright/blob/main/docs/adr/0007-reference-pinning-policy.md),
  [docs/licensing.md](https://github.com/wrightkit/wright/blob/main/docs/licensing.md),
  [docs/workshop/catalog-pipeline.md](https://github.com/wrightkit/wright/blob/main/docs/workshop/catalog-pipeline.md)

## Context

`workshop-rs` is the separately versioned canonical Workshop semantic core and
standalone tooling foundation for WrightKit
([Issue #2](https://github.com/wrightkit/workshop-rs/issues/2)). It must be able
to absorb data-only Workshop updates without compiler rewrites, and it must
serve Wright and language providers without leaking locale spellings or
provider naming into semantic identity. This ADR fixes the first durable
contract: the boundary between semantic code and catalog/content data, the
shape of locale coverage, and the identities that pin provenance and versions.

It builds on the workspace evidence rules and on Wright's clean-room licensing
policy, adapted to this repository's MIT license.

## Decision

### 1. Semantic code vs. catalog/content data

The boundary is explicit and data-oriented.

- **Semantic code** is the `workshop-rs` implementation: parser, CST/AST/WIR,
  validation, emitter, and the canonical identity model. It contains no
  locale-specific branches and no per-locale spelling knowledge.
- **Catalog/content data** is a machine-readable dataset committed in this
  repository. It declares the supported Workshop-defined content — actions,
  values, events, enums and their members, heroes, maps, game modes, settings,
  and any additional category it declares — each bound to a canonical identity,
  plus locale tables mapping canonical identities to client spellings.

Anything expressible as data without changing semantics must be data. Locale
coverage is represented exclusively in the catalog dataset; there is no
locale-specific code path.

### 2. Locale-independent canonical identities

Every supported content item has a canonical identity that is:

- stable within a catalog dataset version;
- unique within the dataset;
- independent of client locale (no identity is derived from or keyed by any
  locale spelling);
- independent of source-language provider naming (OPY, OSTW, or any future
  provider name is never an identity).

Shapes only, not a frozen syntax: `action.set_global_variable`,
`value.all_players`, `enum.team` / member `all`, `event.ongoing_global`,
`hero.ana`, `map.ilios`, `game_mode.quick_play`, `setting.<key>`. Exact ID
syntax and serialization are implementation detail for Issue #2; the properties
above are the contract.

### 3. Locale tables are mappings, not semantics

Locale coverage is a locale table per declared locale: a mapping from canonical
identities to client spellings (`en-US` "Set Global Variable"; `zh-CN` spelling
supplied only from reviewed provenance data). A locale is a declared set of
such mappings. Locale tables never define or change semantics; they bind
spellings to identities. A spelling without a canonical identity, or two
spellings bound to one identity in one locale without a declared alias, fails
validation.

### 4. Strict catalog/allowlist model

The catalog is an allowlist. The parser, validator, and emitter operate only on
the declared surface. Anything not in the catalog is diagnosed as unknown —
never guessed, never silently accepted, never treated as valid semantics.
Unknown future Workshop tokens are invalid until a reviewed catalog update
declares them. Validation rejects duplicates, colliding aliases, undeclared
locales, and missing mappings instead of resolving them heuristically.

### 5. Independent, machine-identifiable identities

Four identities evolve independently and are machine-readable:

| Identity | Meaning |
| --- | --- |
| `implementation-version` | `workshop-rs` package version (semver); bumped by code changes. |
| `catalog-version` | Version of the catalog dataset plus a deterministic content digest computed by the pipeline; bumped by any dataset change, including locale tables. |
| `locale-coverage` | Declared locales with per-locale mapping counts, carried in the dataset manifest and reported by tooling. |
| `target-evidence` | Identity of the evidence base where available (game patch/runtime version, corpus or fixture identity, oracle/reference version and hashes) from which dataset entries were derived. |

A data-only update changes `catalog-version` (and possibly `locale-coverage`,
`target-evidence`) without changing `implementation-version`; a code change
bumps `implementation-version` without implying a dataset change. Tooling
surfaces all four in machine-readable form (for example `--version` or a
build-info manifest), and tests record them.

### 6. Provenance and the reproducible catalog-update pipeline

Every catalog entry and locale table carries provenance: evidence class,
source, license/review status, and generator identity.

Evidence hierarchy, in order of strength (workspace `AGENTS.md`): reproducible
Workshop behavior; accepted project contracts; repository tests and fixtures;
real consumer projects; upstream/reference implementations; documented
community evidence; assumptions. Entries cite their class; assumptions are
labeled as assumptions.

The dataset is built by a deterministic pipeline: edit the data file -> run
validation (schema, identity uniqueness, alias collisions, undeclared locales,
missing mappings, parameter arity) -> build canonical deterministic form
(byte-idempotent regeneration) -> commit data and regenerated file together
with the evidence reference. A game patch that changes Workshop strings or
content is a bounded data update, never a parser or emitter rewrite.

Licensing: this repository is MIT; committed data must be MIT-compatible with
recorded provenance. OverPy's translation tables are GPL-3.0 reference data and
are not a permissible source for catalog or locale data (Wright ADR-0004,
`docs/licensing.md`). Observed reference behavior is an interoperability input,
not permission to copy an implementation; mechanically translating upstream
data is not permitted. New locales, including the initial `zh-CN` data, require
a permissible reference source with provenance and license review before
inclusion.

### 7. Supported categories and missing-target-locale behavior

Supported content categories are declared in the catalog: actions, values,
events, enums and members, heroes, maps, game modes, settings, and any
additional category the dataset declares explicitly.

Missing target-locale spellings fail explicitly by default: conversion or
emission into a locale that lacks a required mapping is a diagnostic, never a
guess and never silent passthrough of another locale's spelling. Fallback is
opt-in: a caller may enable fallback to a declared default locale, and the
fallback choice is visible in tooling output.

### 8. Pinning for reproducible tests and tooling

Tests and tooling pin the effective catalog identity: every fixture or corpus
records the `implementation-version` and `catalog-version` (and
`locale-coverage`) it was generated against, and a dataset change requires
revalidating and regenerating pinned fixtures through the pipeline of
Decision 6. Tooling reports the effective identity so any run is reproducible
and attributable.

### 9. Initial product gate

The first usable multi-locale release supports a complete declared
`en-US` <-> `zh-CN` raw Workshop conversion surface — parse, validate, emit,
and convert in both directions, including settings — for the declared surface,
verified by corpus tests. Further locales expand only through evidence-backed
catalog updates per Decision 6.

### Non-goals

Deliberately not covered here: Wright/LPP/provider version coordination (those
components own their versions); runtime downloading of unreviewed or latest
catalog data by default; treating unknown future Workshop tokens as valid
semantics; a generic plugin ABI or dynamic grammar system; OPY/DEL source
semantics; copying third-party catalog/source data without provenance and
license review.

## Consequences

- Data-only Workshop updates (new heroes, actions, locale spellings, patch
  string changes) land as reviewed data changes without compiler logic changes.
- Source-language providers consume canonical identities and locale tables
  through `workshop-rs` contracts and do not duplicate locale/catalog
  knowledge; provider naming never enters the core.
- The strict allowlist makes unsupported input loud, which is the intended
  safety property for conversion tooling.
- Costs: every data entry needs provenance; validation and pipeline tooling
  must exist before the first multi-locale release; unknown-token diagnostics
  must be implemented in the core.
- Follow-up: Issue #2 bootstraps the repository implementation against this
  contract.

## Compatibility impact

No runtime compatibility claims exist yet. This ADR defines how future claims
are evidenced, pinned, and attributed. No `zh-CN` compatibility claim may be
made until locale data with reviewed provenance is committed per Decision 6.

## Acceptance criteria

| # | Criterion (Issue #1) | Defining section |
| --- | --- | --- |
| 1 | Semantic implementation and content/localization data have an explicit documented boundary. | Decision 1 |
| 2 | Canonical identities do not depend on client locale or source-language provider naming. | Decision 2 |
| 3 | Catalog/content and implementation versions can evolve independently and are machine-identifiable. | Decision 5 |
| 4 | Locale coverage and missing mappings are explicit and testable. | Decisions 3, 7, 8 |
| 5 | Data-only Workshop updates can be added without compiler logic changes when they are genuinely data-only. | Decisions 1, 6 |
| 6 | The core implementation can consume this contract without duplicating locale/catalog knowledge in source-language providers. | Decisions 1, 2, 3; Consequences |

## Open questions

- Exact canonical identity syntax, dataset schema, and serialization format —
  decided in Issue #2 implementation.
- Whether a candidate `zh-CN` reference source is permissible under the MIT
  policy — resolved by provenance and license review before the first
  multi-locale release.
- How `target-evidence` is recorded when evidence is community documentation
  rather than an executable oracle — resolved per entry by the pipeline.
