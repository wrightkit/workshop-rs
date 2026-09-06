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

Some operations necessarily cross every domain and therefore have one explicit
shared home:

- `frontend/` owns tokenization and parsing of raw Workshop text;
- `analysis/` owns complete-program inspection, canonical validation, and
  resource counting;
- `output/` owns deterministic emission, locale conversion, and round-trip
  comparison;
- `core/` owns storage, typed IDs, source spans, formatting, and shared errors;
- `evidence/` owns census, conformance, client captures, and real-project
  support, outside the normal semantic path.

The historical root modules (`parser`, `emitter`, `validate`, and the other
phase or support names) remain thin re-export compatibility paths for existing
consumers. New implementation belongs in the domain or shared-boundary trees
above; these paths are not additional implementation homes.
