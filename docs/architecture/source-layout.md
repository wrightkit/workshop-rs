# Workshop source layout

The crate source is organized by the Workshop domain a contributor is trying
to understand. The public domain entry points are:

| Domain | Entry point | Canonical model and related operations |
| --- | --- | --- |
| Actions | `actions` | WIR action forms, action layout, and element counting |
| Events | `events` | WIR event identities and filters |
| Rules | `rules` | WIR rules, declarations, whole-program inspection, and canonical validation |
| Values | `values` | WIR value and expression forms |
| Settings | `settings` | Typed custom-game settings and locale-aware settings data |
| Gameplay | `gameplay` | Hero facts, gameplay data loading, and semantic queries |
| Catalog | `catalog` | Canonical identities, localization, and locale detection |

`wir/` contains the canonical locale-independent Workshop representation used
by those domains. It is the shared semantic model, not a compiler phase.

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

The historical root modules (`parser`, `emitter`, `validate`, and the other
phase or support names) remain thin re-export compatibility paths for existing
consumers. New implementation belongs in the domain or shared-boundary trees
above; these paths are not additional implementation homes.
