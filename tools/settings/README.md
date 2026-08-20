# Settings inventory

`generate_inventory.py` is the reproducible, source-bound inventory step for
the settings projection. The `workshop-data` export is not redistributed;
provide it explicitly and commit only the generated projection and its
provenance. Example:

```sh
python3 tools/settings/generate_inventory.py \
  --export ../workshop-data/workshop-data.json \
  --output crates/workshop-rs/src/settings/data/inventory.json
cargo run -p workshop-rs --bin workshop-catalog-gen --locked -- check \
  --file crates/workshop-rs/src/catalog/data/catalog.json
```

The inventory records the export commit, input SHA-256, counts, and source
paths. The generator rejects an unexpected source commit and duplicate
English identities in each exported category. Missing locale mappings remain
explicit in `zh-cn.json`; they are not filled with guessed translations.
