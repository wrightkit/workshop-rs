# Workshop source layout

The crate source is organized by the Workshop domain a contributor is trying
to understand. The public domain entry points are:

| Domain | Entry point | Canonical model and related operations |
| --- | --- | --- |
| Actions | `actions` | Public action forms, action layout, and element counting |
| Events | `events` | Public event identities and filters |
| Rules | `rules` | Public rules, declarations, whole-program inspection, and canonical validation |
| Values | `values` | Public value and expression forms |
| Settings | `settings` | Typed custom-game settings and locale-aware settings data |
| Gameplay | `gameplay` | Hero facts, gameplay data loading, and semantic queries |
| Catalog | `catalog` | Canonical identities, localization, and locale detection |

`program.rs` contains the canonical public locale-independent Workshop model
used by those domains. `wir/` is private normalized storage used by parser,
validation, analysis, and emission implementations; consumers must not depend
on it for ordinary program construction or inspection.

Feature-owned parser, validation, emitter, and layout implementations live
beside their domain under `actions/`, `events/`, `rules/`, `values/`, and
`settings/`. `frontend/parser.rs` owns the crate-private `ParseContext` and
shared token/catalog/span mechanics. `output/emitter.rs` owns the crate-private
`EmitContext`, complete-program section ordering, and shared presentation
helpers. The domain modules attach their behavior to those state holders
directly; the shared contexts do not mount phase-wide feature implementations
through private path modules.

Some operations necessarily cross every domain and therefore have one explicit
shared home:

- `frontend/` owns tokenization and parsing of raw Workshop text;
- `analysis/` owns complete-program inspection and resource counting;
- `rules/validate.rs` orchestrates canonical validation while delegating event,
  action, and value checks to their owning domains;
- `output/` owns deterministic emission, locale conversion, and round-trip
  comparison;
- `core/` owns storage, typed IDs, source spans, formatting, and shared errors;
- `workshop-rs-cli/src/` owns census execution, conformance report schemas,
  capture comparison, and corpus tooling;
- `tests/` owns executable Workshop contract and regression checks, with
  provenance-linked fixtures beside the tests that exercise them.

The root operation modules (`parser`, `emitter`, `validate`, `convert`, and
`roundtrip`) are the public Workshop operations over `Program`. Storage,
lexer, arena, ID, and census compatibility paths are documentation-hidden
support surfaces and are not ordinary 1.0 contracts.
