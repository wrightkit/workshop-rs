# Public API and compatibility inventory

This is the current `workshop-rs#253` audit of legacy, hidden, and
implementation-shaped public paths. A `#[doc(hidden)]` item is still public
Rust API, so its disposition must be explicit before 1.0.

## Audit baseline

The current consumer audit uses these fetched `origin/main` revisions. The
Workshop snapshot includes merged PR #262, and the OPY snapshot includes
merged PR #356.

| Repository | Revision |
| --- | --- |
| `workshop-rs` | `2ce425f897b583606828208f4201351a8448661a` |
| `opy-rs` | `da226c5973a07fe1bbd562370ab371dba1039117` |
| `deltin-rs` | `4c0c04ec118277ad823ce5e4001c5e10dc60f186` |
| `wright` | `50689d22aaf557868b7086abbb6025e9f50ca181` |

The audit used tracked source, tests, and documentation in those revisions.
Generated build output and untracked files were excluded. First-party use was
checked by repository, not inferred from the earlier inventory.

## Current consumer inventory

| Public path | Observed first-party use at the audit baseline | Status at the current snapshot |
| --- | --- | --- |
| `arena` | DEL source bridge; Wright analyzer CFG/symbol storage and driver diagnostics | Still public and used by DEL and Wright. `deltin-rs#113` proposes replacing DEL's bridge storage but is not part of this snapshot. |
| `ids` | Wright analyzer, language document, and transform code; Workshop WIR tests | Still public and used by Wright and WIR-backed tests. |
| `wir` and WIR adapters | OPY and DEL reconstruction; Wright analyzer and transform; Workshop integration tests | Still public and used by first-party consumers, including `parser::{parse_wir, parse_wir_with_context}`, `emitter::{emit_wir, emit_wir_with_options}`, `roundtrip::equivalent_wir`, and `rules::validate_canonical_ids_wir`. |
| `semantic` | Wright provider maps semantic issues; Workshop tests | Still public and used by Wright and tests. The `rules` exports and `Program` inspection do not yet replace all observed uses. |
| `element_count` | No first-party use remains in the audited snapshots; OPY moved its import in merged PR #356 | Still present in merged PR #262. This follow-up removes the facade; canonical element-count exports remain under `actions`. |
| `actions::{WIRActionLayoutError, action_width_wir}`, re-exported from `emitter` | Workshop action-layout tests only | Removed in merged PR #262. The tests use the public `Program` action sequence and `actions::action_width`. |
| `gameplay_data`, `gameplay_query` | Workshop tests, internal schema code, and docs only | Removed in merged PR #262. The canonical paths are `gameplay::data` and `gameplay::query`. |
| `lexer` | Workshop parser-error tests and benchmark only | Removed in merged PR #262. Lexer implementation types are crate-private; parser behavior and source spans are covered through `parser`. |
| `format` | OPY numeric lowering/reconstruction; Wright constant folding | Supported public formatting operation: `format::format_number` formats computed Workshop numbers; merged PR #262 documents this contract. |
| `settings::table` | CLI census and catalog generator; parser/emitter internals and static-data benchmark | Still public and used by the CLI, catalog generator, parser/emitter, and benchmark. OPY now imports `settings::PathPart` after merged PR #356. No final 1.x disposition is made here. |
| `source`, `signatures`, `parser`, `emitter`, `roundtrip`, `validate`, `convert`, `catalog`, `detect`, `actions`, `events`, `rules`, `settings`, `gameplay`, `values`, `program` | Public domain operations and types used by first-party consumers | No compatibility-facade change in these PRs. |
| `WorkshopError`, `CatalogError` | `WorkshopError` is used throughout first-party callers; no tracked external `CatalogError` path was found | Remain public; these PRs do not change either error type. |

Before merged PR #356, OPY imported `PathPart` through `settings::table`; at
the audited revision it imports `settings::PathPart`. PR #262 moved the CLI
from the hidden root re-exports `settings::{KeyKind, TableEntry, entries,
enum_name, mode_name, path_string}` to `settings::table`, which remains public
for current tool and benchmark use.

## Scope status

OPY PR #356 is merged at `da226c5973a07fe1bbd562370ab371dba1039117`. The
tracked consumer search found no remaining first-party use of `element_count`,
so this follow-up removes that facade. DEL PR #113 is not part of the current
DEL main snapshot; OPY, DEL, and Wright still use WIR, while DEL and Wright use
`arena` and Wright uses `ids` and `semantic`. This follow-up does not complete
`workshop-rs#253` or decide the 1.x status of those still-consumed paths. The
issue remains open. The public `Program` and domain APIs are available to new
consumers.
