# Hero values catalog (design)

Status: implemented for [#541](https://github.com/wrightkit/workshop-rs/issues/541)
as the public module `workshop_rs::hero_values`. The example file uses an
invented hero and invented numbers. It shows the shape only.

Scope follows principle 9 of the goal document: technical facts about the Workshop environment. The catalog does not judge balance or game design.

## Decisions for the owner

1. **Pinned version: game client version `2.25.0.0 - 154461.1`.** It is the client version recorded in the athena probe captures (`client.version`, read from the client options menu). It is a client version, not a named patch or season. The `balanceVersion` field holds this string.
2. **Value kinds in v1.** Health (pools `health`, `shield`, `armor`), cooldown (seconds, per ability slot) and ultimate cost (points). A new kind needs a schema change.
3. **No per-value source or verification state.** A value is its kind and its number. The owner judged per-value evidence unnecessary overhead. Principle 9 is updated to match in wrightkit/.github#117.
   Values come from public patch notes and the community wiki. One recorded exception applies to that source rule (owner decision, 2026-10-27): `dmon` `ultimateCost` = 2000 is taken from the non-English fan wikis because no allowed source states it.
4. **One file per version.** A new version is a new file. Old files stay unchanged.
5. **Location.** `crates/workshop-rs/src/gameplay/values/<version>.json`, next to the hero dataset. `catalog.json` is the language catalog and holds no hero data.
6. **athena reads the data through a new public Rust module, `hero_values`**, not the raw file and not the gameplay catalog. The gameplay catalog requires a source on every stat. See "Reading the data".

## Data shape

The schema is
[`schemas/hero-values-v1.schema.json`](../crates/workshop-rs/schemas/hero-values-v1.schema.json);
a test validates the checked-in files against it. One file holds one client
version.

```json
{
  "format": "workshop-rs/hero-values-v1",
  "balanceVersion": "2.25.0.0 - 154461.1",
  "heroes": [
    {
      "id": "<hero id from the gameplay dataset>",
      "values": [
        { "kind": "health", "pool": "shield", "amount": 0 },
        { "kind": "cooldown", "slot": "ability1", "seconds": 0 },
        { "kind": "ultimateCost", "points": 0 }
      ]
    }
  ]
}
```

(The zeros are placeholders and fail validation. Amounts must be greater than zero.)

| Kind | Fields | Unit |
| --- | --- | --- |
| `health` | `pool`: `health`, `shield`, `armor`; `amount` | health points |
| `cooldown` | `slot` (logical slot id), optional `variant`; `seconds` | seconds |
| `ultimateCost` | `points` | ultimate points |

The schema rejects an unknown value kind, an unknown pool and a non-positive number. Hero ids and slot ids are open strings in the gameplay dataset, so the schema cannot check them. The loader checks them later, in the implementation: it rejects a hero, slot or variant that the gameplay dataset does not know.

## Adding a version

1. Copy the previous file to `values/<client version>.json`.
2. Change `balanceVersion` and the values that changed.
3. Register the file in the loader and run the tests.

## Selecting a version

A consumer names the version it wants. The loader returns an error for an unknown version. The crate holds a constant for the pinned version.

## Reading the data

The values have their own small read path. They are not loaded into the gameplay catalog. Every stat in that catalog carries a source list, and loading rejects a stat without one (`gameplay-data.md`). Hero values have no source, so they do not fit that type. The gameplay catalog, its `Fact` type and `gameplay-data.md` stay unchanged.

The smallest public API is the module `workshop_rs::hero_values`:

```rust
hero_values::CURRENT_VERSION: &str                       // "2.25.0.0 - 154461.1"
hero_values::builtin(version: &str) -> Result<&'static HeroValues, HeroValuesError>

HeroValues::version(&self) -> &str
HeroValues::health(&self, hero: &str, pool: HealthPool) -> Option<f64>
HeroValues::cooldown(&self, hero: &str, slot: &str, variant: Option<&str>) -> Option<f64>  // seconds
HeroValues::ultimate_cost(&self, hero: &str) -> Option<f64>                                 // points
```

- `HealthPool` is `Health`, `Shield` or `Armor`.
- `builtin` returns an error for an unknown version.
- A lookup returns `None` when the file has no such value. The toolchain then reports the behavior as not modeled. It never uses a default.
- The loader validates the file against the schema. It also checks that every hero, slot and variant exists in the gameplay dataset, and rejects duplicate values.
- Custom-game cooldown percentages are not part of this API. The existing gameplay cooldown calculation applies to catalog stats only; athena#11 applies settings modifiers to the numbers it reads here.

athena#9 (state table) and athena#11 (settings modifiers) read hero values only through these calls.

## Out of scope

- Damage, healing, and other ability numbers.
- Judgement of balance.
- `deltin-rs`.
