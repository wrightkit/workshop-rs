# Hero values catalog (design)

Status: design proposal for [#541](https://github.com/wrightkit/workshop-rs/issues/541). Nothing here is implemented
except the schema, one example file and the test that checks them. The example
file uses an invented hero and invented numbers. It shows the shape only.

Scope follows principle 9 of the goal document: technical facts about the Workshop environment. The catalog does not judge balance or game design.

## Decisions for the owner

1. **Pinned version: game client version `2.25.0.0 - 154461.1`.** It is the client version recorded in the athena probe captures (`client.version`, read from the client options menu). It is a client version, not a named patch or season. The `balanceVersion` field holds this string.
2. **Value kinds in v1.** Health (pools `health`, `shield`, `armor`), cooldown (seconds, per ability slot) and ultimate cost (points). A new kind needs a schema change.
3. **No per-value source or verification state.** A value is its kind and its number. The owner judged per-value evidence unnecessary overhead. Principle 9 is updated to match in wrightkit/.github#117.
4. **One file per version.** A new version is a new file. Old files stay unchanged.
5. **Location.** `crates/workshop-rs/src/gameplay/values/<version>.json`, next to the hero dataset. `catalog.json` is the language catalog and holds no hero data.
6. **athena reads the data through the public Rust API** of `workshop-rs`, not the raw file. See "Reading the data".

## Data shape

The schema is
[`schemas/hero-values-v1.schema.json`](../crates/workshop-rs/schemas/hero-values-v1.schema.json).
One file holds one client version.

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

Proposed smallest API: the values load into the existing gameplay catalog as stats.

- `health` pools become hero stats (`health`, `shield`, `armor`).
- `cooldown` becomes the existing ability stat `cooldown`. The existing `query().cooldown(...)` works unchanged.
- `ultimateCost` becomes the ability stat `ultimateCost` in the `ultimate` slot.
- One new function loads a catalog for a named version, for example `gameplay::values::builtin(version)`.

athena#9 (state table) and athena#11 (settings modifiers) then use the same query calls as for other hero data. The existing `Fact` type is not changed.

## Out of scope

- Damage, healing, and other ability numbers.
- Judgement of balance.
- `deltin-rs`.
- The loader and query code. They follow in the implementation issue after the owner accepts this design.
