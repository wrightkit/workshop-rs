# Public compatibility facade inventory

This inventory records the compatibility modules retained after the domain
layout introduced by #149. It is a migration preflight, not authorization to
remove a public module. A `#[doc(hidden)]` module remains part of the compile
surface for downstream crates.

## Audit method

The inventory was produced with tracked-file `git grep` for each
`workshop_rs::<facade>` path across the four repositories. Untracked audit
notes and generated build directories were excluded. The snapshot commits
were:

| Repository | Snapshot |
| --- | --- |
| `workshop-rs` | `3d5fe504` |
| `opy-rs` | `5e65d504` |
| `deltin-rs` | `4c0c04ec` |
| `wright` | `637d2baf` |

The counts below are repository files containing the path, not call counts.
They include source, tests, and checked-in documentation where applicable.

| Facade | Hidden | Canonical current surface | `workshop-rs` | `opy-rs` | `deltin-rs` | `wright` | Result |
| --- | :---: | --- | ---: | ---: | ---: | ---: | --- |
| `arena` | yes | no replacement public path; shared arena type | 0 | 0 | 1 | 3 | retain: external consumers |
| `format` | yes | no replacement public path; shared formatter | 0 | 2 | 0 | 1 | retain: external consumers |
| `ids` | yes | no replacement public path; typed WIR identities | 3 | 0 | 0 | 6 | retain: external consumers |
| `signatures` | no | `signatures` remains the public signature-context surface | 1 | 1 | 0 | 0 | retain: external consumer |
| `source` | no | `source` remains the public provenance surface | 8 | 4 | 3 | 12 | retain: external consumers |
| `error` | no | crate-root `WorkshopError`/`CatalogError` re-exports | 0 | 0 | 0 | 0 | retain: downstream use unknown |
| `element_count` | yes | `Program::element_count`, `actions` re-exports | 1 | 0 | 0 | 0 | retain: in-repo compatibility use |
| `semantic` | yes | `Program::semantic_issues`, `rules` re-exports | 3 | 0 | 0 | 3 | retain: external consumers |
| `gameplay_data` | yes | `gameplay::data` | 3 | 0 | 0 | 0 | retain: in-repo/docs use |
| `gameplay_query` | yes | `gameplay::query` | 2 | 0 | 0 | 0 | retain: in-repo/docs use |
| `detect` | no | `catalog::detect` | 3 | 0 | 0 | 4 | retain: external consumers |
| `lexer` | yes | no replacement public path; frontend implementation is private | 2 | 0 | 0 | 0 | retain: in-repo compatibility use |
| `convert` | no | `convert` remains the public conversion operation | 7 | 0 | 0 | 0 | retain: in-repo/README use |
| `emitter` | no | `emitter` remains the public emission operation | 13 | 2 | 1 | 2 | retain: external consumers |
| `roundtrip` | no | `roundtrip` remains the public comparison operation | 8 | 21 | 1 | 1 | retain: external consumers |
| `parser` | no | `parser` remains the public parsing operation | 14 | 20 | 5 | 6 | retain: external consumers |
| `validate` | no | `rules::validate_canonical_ids` plus public facade | 8 | 1 | 2 | 0 | retain: external consumers |

The absence of a match means “unreferenced in this verified snapshot”, not
“safe to remove”: arbitrary downstream crates are outside the repository set,
and public semver compatibility still applies. In particular, no facade has a
complete replacement-and-migration proof in this audit.

## Decisions and migration sequence

No facade is proposed for retirement by this inventory. The active consumer
set prevents removal of `arena`, `format`, `ids`, `signatures`, `source`,
`semantic`, `detect`, `emitter`, `roundtrip`, `parser`, and `validate`. The
remaining paths either have repository-owned uses or have no verified
downstream use but still expose an unresolved public compatibility obligation.

Any future retirement must proceed in this order:

1. `workshop-rs` defines and releases a reviewed public replacement for each
   facade whose implementation path is currently private, preserving the
   accepted type and error contracts.
2. `opy-rs`, `deltin-rs`, and `wright` migrate verified imports in independent
   owner-repository changes. A consumer migration is not implied by this
   inventory.
3. A `workshop-rs` release makes the replacement available and provides any
   approved deprecation window before removal.
4. The supported dependency graph is compiled and its integration tests are
   run against the released replacement and again after facade removal.

Until those steps have independent evidence, all facades remain in place.
