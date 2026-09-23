# Public API and compatibility contract

The supported Workshop API centers on the public `Program` model and
domain-oriented data and operations. Consumers should be able to parse,
validate, inspect, transform, and emit Workshop programs without depending on
internal storage or parser implementation details.

## Supported public API

| Contract area | Public paths |
| --- | --- |
| Canonical program model | `program`, the crate-root model re-exports |
| Workshop domains and data | `actions`, `catalog`, `events`, `gameplay`, `rules`, `settings`, `values`, `signatures`, `source` |
| Workshop operations | `parser`, `validate`, `convert`, `roundtrip`, `emitter`, `format`, `detect` |
| Public errors | crate-root `WorkshopError` and `CatalogError` |

These surfaces express Workshop concepts or operations independently of their
current internal representation.

## Compatibility-only public paths

The following paths expose implementation-shaped types or operations for
compatibility. They are not part of the intended supported Workshop API.

| Path | Compatibility surface |
| --- | --- |
| `arena` | Shared WIR storage and allocation types |
| `ids` | WIR storage identities |
| `semantic` | Implementation-facing semantic issue types |
| `wir` | WIR representation types |
| `parser::{parse_wir, parse_wir_with_context}` | Parsing directly into WIR |
| `emitter::{emit_wir, emit_wir_with_options}` | Emitting directly from WIR |
| `roundtrip::equivalent_wir` | WIR structural comparison |
| `rules::validate_canonical_ids_wir` | Validation over WIR identities |
| `settings::table` | Static settings-table representation |

## Compatibility rule

`#[doc(hidden)]` affects generated documentation only; it does not make a
public Rust item private or remove compatibility consequences. Before 1.0,
each compatibility-only path must either be removed after supported consumers
move to the public `Program` and domain API, or be explicitly promoted to the
supported contract. Internal storage and implementation details remain behind
that boundary for ordinary consumers.
