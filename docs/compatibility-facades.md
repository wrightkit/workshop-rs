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
| Workshop operations | `convert`, `detect`, `emitter`, `format`, `parser`, `roundtrip`, `validate` |
| Parse-context contract | `signatures` |
| Public errors | `CatalogError`, `WorkshopError` |

## Supported public aliases

`detect` keeps locale detection discoverable at the crate root while
`catalog::detect` owns its implementation. `signatures` exposes the
parse-context contract shared by Workshop parsing and frontends that supply
expected enum domains. Both are intentional public APIs, not compatibility-only
facades; the catalog remains the sole source of signature data.

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
identity/metadata records, settings source metadata, and the settings scope and
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
