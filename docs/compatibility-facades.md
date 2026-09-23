# Public API and compatibility contract

The supported Workshop API centers on the public `Program` model and
domain-oriented data and operations. Consumers should be able to parse,
validate, inspect, transform, and emit Workshop programs without depending on
internal storage or parser implementation details.

## Supported public API

| Contract area | Public paths |
| --- | --- |
| Canonical program model | `program`, the crate-root model re-exports |
| Workshop domains and data | `actions`, `catalog`, `events`, `gameplay`, `rules`, `settings`, `values`, `source` |
| Workshop operations | `parser`, `validate`, `convert`, `roundtrip`, `emitter`, `format` |
| Locale detection | `detect` is the crate-root entry point; `catalog::detect` owns the implementation |
| Parse-context contract | `signatures::{ExpectedDomain, NoExpectedDomain, ChainedExpectedDomain}` |
| Public errors | crate-root `WorkshopError` and `CatalogError` |

These are API areas, not a complete item-level inventory. Their public
contracts express Workshop concepts and operations independently of the
current internal representation.

## Supported public aliases

`detect` and `signatures` are intentionally supported public entry points,
even though their modules forward to canonical implementations elsewhere.
`detect` keeps locale detection discoverable at the crate root, while
`signatures` defines the parse-context contract shared by Workshop parsing and
frontends that supply expected enum domains. The catalog remains the sole
source of signature data.

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
