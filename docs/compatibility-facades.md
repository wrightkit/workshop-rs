# Public API and compatibility inventory

This is the current `workshop-rs#253` audit of legacy, hidden, and
implementation-shaped public paths. A `#[doc(hidden)]` item is still public
Rust API, so its disposition must be explicit before 1.0.

## Audit baseline

The tracked-code audit used these fetched `origin/main` revisions before the
issue changes:

| Repository | Revision |
| --- | --- |
| `workshop-rs` | `09f2ef5bc15d96d61044d73fe26ca66a90cbf68a` |
| `opy-rs` | `4b91f59b5e4e1caa444335823a8970175ed44925` |
| `deltin-rs` | `4c0c04ec118277ad823ce5e4001c5e10dc60f186` |
| `wright` | `50689d22aaf557868b7086abbb6025e9f50ca181` |

The audit used tracked source, tests, and documentation in those revisions.
Generated build output and untracked files were excluded. First-party use was
checked by repository, not inferred from the earlier inventory.

## Current consumer inventory

| Public path | Observed first-party use at the audit baseline | Status in this owner PR |
| --- | --- | --- |
| `arena` | DEL source bridge; Wright analyzer CFG/symbol storage and driver diagnostics | Still public and used by Wright. `deltin-rs#113` proposes replacing DEL's bridge storage; that PR is open and has not landed. |
| `ids` | Wright analyzer, language document, and transform code; Workshop WIR tests | Still public and used by Wright and WIR-backed tests. No removal is included here. |
| `wir` and WIR adapters | OPY and DEL reconstruction; Wright analyzer and transform; Workshop integration tests | Still public and used by first-party consumers. This PR does not remove `parser::{parse_wir, parse_wir_with_context}`, `emitter::{emit_wir, emit_wir_with_options}`, `roundtrip::equivalent_wir`, or `rules::validate_canonical_ids_wir`. |
| `semantic` | Wright provider maps semantic issues; Workshop tests | Still public and used by Wright and tests. The `rules` exports and `Program` inspection do not yet replace all observed uses. |
| `element_count` | OPY compiler | Still public and used by OPY. `opy-rs#356` proposes moving the imports to `actions`; that PR is open and has not landed. |
| `actions::{WIRActionLayoutError, action_width_wir}`, re-exported from `emitter` | Workshop action-layout tests only | Removed in this PR. The tests now use the public `Program` action sequence and `actions::action_width`. |
| `gameplay_data`, `gameplay_query` | Workshop tests, internal schema code, and docs only | Removed in this PR. The canonical paths are `gameplay::data` and `gameplay::query`. |
| `lexer` | Workshop parser-error tests and benchmark only | Removed in this PR. Lexer implementation types are crate-private; parser behavior and source spans are covered through `parser`. |
| `format` | OPY numeric lowering/reconstruction; Wright constant folding | Documented as public in this PR; `format::format_number` formats computed Workshop numbers. |
| `settings::table` | CLI census and catalog generator; parser/emitter internals and static-data benchmark; OPY imported `PathPart` through this module | Still public and used by the CLI, catalog generator, parser/emitter, and benchmark. The CLI import change is in this PR. OPY's `settings::PathPart` migration is proposed in open PR #356 and has not landed. |
| `source`, `signatures`, `parser`, `emitter`, `roundtrip`, `validate`, `convert`, `catalog`, `detect`, `actions`, `events`, `rules`, `settings`, `gameplay`, `values`, `program` | Public domain operations and types used by first-party consumers | No compatibility-facade removal in this PR. |
| `WorkshopError`, `CatalogError` | `WorkshopError` is used throughout first-party callers; no tracked external `CatalogError` path was found | Remain public; this PR makes no change to either error type. |

The hidden root re-exports `settings::{KeyKind, TableEntry, entries,
enum_name, mode_name, path_string}` were used by `workshop-rs-cli`. This PR
moves that consumer to `settings::table`; the table itself remains public.

## Scope status

The owner and consumer changes in this PR set do not complete `workshop-rs#253`:
the OPY and DEL migration PRs are still open, and Wright remains a current
consumer of `arena`, `ids`, WIR, and `semantic`. This PR makes no final 1.x
retention or removal decision for those still-used paths. Issue #253 remains
open until the remaining consumer evidence is landed and every retained or
removed facade has a final disposition. The public `Program` and domain APIs
are available to new consumers.
