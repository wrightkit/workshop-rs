# Public API and compatibility contract

The supported Workshop API centers on the public `Program` model and
domain-oriented data and operations. Consumers should be able to parse,
validate, inspect, transform, and emit Workshop programs without depending on
internal storage or parser implementation details.

## Supported public API

The table lists every crate-root public module and re-export from
[`crates/workshop-rs/src/lib.rs`](../crates/workshop-rs/src/lib.rs). Rustdoc
remains the item-level reference for symbols nested under those modules.

| Contract area | Crate-root public paths |
| --- | --- |
| Canonical program model | `program` |
| Root model re-exports | `Action`, `Condition`, `Event`, `EventTarget`, `EventTeam`, `ModifyOp`, `PlayerEventKind`, `MappedText`, `Program`, `SourceMap`, `SourceMapError`, `SourceMappingError`, `Rule`, `Subroutine`, `Value`, `Variable` |
| Workshop domains | `actions`, `catalog`, `events`, `gameplay`, `rules`, `settings`, `values` |
| Source and provenance | `source` |
| Canonical lookup | `lookup` |
| Workshop operations | `convert`, `detect`, `emitter`, `format`, `parser`, `roundtrip`, `validate` |
| Parse-context contract | `signatures` |
| Public errors | `CatalogError`, `WorkshopError` |

## Supported public aliases

`detect` keeps locale detection discoverable at the crate root while
`catalog::detect` owns its implementation. `signatures` exposes the
parse-context contract shared by Workshop parsing and frontends that supply
expected enum domains. Both are intentional public APIs, not compatibility-only
facades; the catalog remains the sole source of signature data.

The canonical `Program` model types are also reachable from the domain module
that owns their concept, so contributors and consumers can start from either
the model or the domain: `actions::{Action, ModifyOp}`, `events::{Event,
EventTarget, EventTeam, PlayerEventKind}`, `rules::{Condition, Program, Rule,
Subroutine, Variable}`, and `values::Value` are the same items as their
crate-root and `program::` re-exports, not copies. `program`, the crate root,
and these domain modules are the discoverable Workshop domains described in
the crate's top-level docs; every one of these paths is an intentional public
API and a separate 1.x compatibility commitment.

Every other public item is reachable through exactly one path. `actions`
re-exports the `Program` model types above and its own element-count analysis
types (`ElementCountError`, `ElementCountNode`, `ElementCountReport`,
`ElementNodeKind`); action layout and element-count operations (`ActionLayout`,
`ActionLayoutError`, `action_width`) are public only from `emitter`. `rules`
re-exports the `Program` model types above and its own inspection types
(`IncompletenessKind`, `ResidualClassification`, `SemanticIssue`, `inspect`);
canonical validation (`validate_canonical_ids`,
`validate_canonical_ids_tolerating_residuals`) is public only from
`validate`. `settings::schema` is an internal module; its types and functions
(including `validate_catalog`) are public only from `settings` directly.

## Catalog-backed actions and values

Catalog actions and values are built with `Action::call` and `Value::call`,
passing the canonical catalog id and the arguments in catalog parameter order.
Canonical validation against a catalog decides whether the call is valid. No
Rust item is generated from catalog data, so adding, removing, or renaming a
catalog entry or parameter does not change any Rust item's name, signature, or
type and can ship in a minor release
([ADR-0016](adr/0016-catalog-content-outside-rust-api.md)). The embedded
catalog text in `catalog::CATALOG_DATA` changes with the data; its type does
not.

Build `null` and `eventPlayer` with the `Value::Null` and `Value::EventPlayer`
variants, which is what the parser produces. `Value::call("null", [])` and
`Value::call("eventPlayer", [])` emit the same text but are not round-trip
equivalent to their re-parsed form. For every other catalog value, including
`vector`, `array`, and `emptyArray`, the parser produces `Value::Call` with the
canonical id, so `Value::call` builds the parsed shape.

## Compatibility-only facades

No compatibility-only public facades are intentionally retained. WIR storage
identities, internal WIR types, parser internals, and static settings-table
representations are implementation details. Their former compatibility paths
have been retired or made private; consumers should use the public `Program`
model and domain APIs above.

## Compatibility rule

`#[doc(hidden)]` affects generated documentation only; it does not make a
public Rust item private or remove its compatibility consequences. Before 1.0,
implementation-only public exports may be removed as intentional breaking
changes. Consumer migrations are owned by their respective repositories and do
not need to precede an owner-side breaking release.

## Growth without breaking changes

Public types expected to grow in 1.x are `#[non_exhaustive]`: error enums and
error records, options, operation outputs and reports, catalog and gameplay
identity/metadata records, lookup matches and signature records, settings
source metadata, and the settings scope and
value-domain enums. `Rule`, `Condition`, `PlayerEventKind`, and `ModifyOp` are
also `#[non_exhaustive]`. Consumers match them with a wildcard arm and read
their fields, but do not build them with struct literals. Options are built
from `Default` and then assigned field by field; records that consumers need to
build, such as `MappedText`, `gameplay::SourceReference`, and
`gameplay::GameplayDatasetIdentity`, have a `new` constructor.

The canonical program model (`Action`, `Value`, `Event`, `EventTeam`,
`EventTarget`), `catalog::Kind`, source positions and spans, and values that
consumers construct stay exhaustive. New Workshop content reaches them through
catalog ids and `Call`, not new variants.

## Versioning of catalog, behavior, and output changes

`cargo-semver-checks` sees only the Rust API. Catalog content, parse results,
validation results, and emitted text also change between releases, and
consumers with a caret requirement pick up every minor release. From 1.0 on,
releases are versioned as follows.

- **Major:** an incompatible change to the documented public Rust API, as
  reported by `cargo-semver-checks`.
- **Minor:**
  - catalog updates: added actions, values, heroes, maps, and settings;
    changed parameters; new or changed locale spellings; corrections that
    remove ids which were never native Workshop elements (such as [#299](https://github.com/wrightkit/workshop-rs/pull/299));
  - parse, validation, or emit changes that align behavior or output with the
    live client (such as [#300](https://github.com/wrightkit/workshop-rs/pull/300) and [#303](https://github.com/wrightkit/workshop-rs/pull/303));
  - new public API;
  - MSRV increases.
- **Patch:** fixes with no observable change to the parsed `Program`, emitted
  text, validation results, or catalog content.

Catalog corrections do not keep the removed id: the parser rejects it like any
other unknown id, matching the live client.

Overwatch has not removed a native Workshop action or value so far; its updates
change parameters, spellings, and engine behavior instead. `workshop-rs` keeps
no legacy ids for that case. If a native element is ever removed, the catalog
follows the client and the release notes declare the removal as a breaking
behavior change; the maintainer decides at that point whether it ships in a
minor or a major release.

The catalog dataset carries its own version, `catalog_version` in
`catalog::CatalogIdentity`, which changes with any dataset change independently
of the crate version.

GitHub Release notes are generated from merged pull request titles, so the
title marks the kind of change. This applies to every pull request, before
and after 1.0:

- catalog updates use `feat(catalog): ...`;
- other changes to parse, validation, or emit results use `feat(<area>): ...`
  rather than `fix`, and the pull request body has a `## Behavior change`
  section describing the observable difference;
- `fix` is reserved for changes that qualify as a patch under the rules above.

release-plz derives a minor release from `feat`, so these titles also select
the right version.
