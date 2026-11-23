# Settings inventory

`generate_inventory.py` is the reproducible, source-bound inventory step for
the settings projection. The `workshop-data` export is not redistributed;
provide it explicitly and commit only the generated projection and its
provenance. Example:

```sh
python3 -I tools/settings/generate_inventory.py \
  --export ../workshop-data/workshop-data.json \
  --output crates/workshop-rs/src/settings/data/inventory.json
cargo run -p workshop-rs --bin workshop-catalog-gen --locked -- check \
  --file crates/workshop-rs/src/catalog/data/catalog.json
```

The inventory records the export commit, input SHA-256, counts, and source
paths. The generator rejects an unexpected source commit and duplicate
localized aliases in each exported category or among settings groups. Nested
settings labels are scoped to their parent and are not group aliases. Missing
locale mappings remain explicit in `locales.json`; they are not filled with
guessed translations. Adding a reviewed locale is a data regeneration change:
the projection declares its locale set and each entry carries the available
locale aliases.

CI runs generator tests with a synthetic export to check regeneration and
alias conflicts. It cannot compare the committed inventory with the pinned
export because the export is not redistributed. To check that comparison
locally, generate an inventory to a temporary path and compare it byte for byte
with the committed `inventory.json`.

Reviewed numeric import bounds are maintained separately in
`crates/workshop-rs/src/settings/data/input_bounds.json`, keyed by the settings
projection's canonical paths. Add a record with `min` and/or `max` only when
client evidence establishes the inclusive bound, document its provenance in
`docs/source-attribution/catalog.md`, and run the catalog check above. The
export inventory generator does not write this file.
