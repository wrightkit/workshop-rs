# Hero values catalog (design)

Status: design proposal for [#541](https://github.com/wrightkit/workshop-rs/issues/541). Nothing here is implemented
except the schema, one example file and the test that checks them. The example
file uses an invented hero and invented numbers. It shows the shape only.

Scope follows principle 9 of the goal document: technical facts with a source
and a verification state. The catalog does not judge balance or game design.

## Decisions for the owner

Each item has a proposal. Change any of them before the issue becomes ready for implementation.

1. **Balance version to pin: open.** The repository does not name the current game balance version.
   - The gameplay dataset version `2026-10-06` is the date of the data export. It is not a balance version.
   - `gameplay-data.md` mentions a "June 30, 2026 Community Crafted limited-mode patch". That patch is a limited mode and the dataset deliberately does not model it.
   - Proposal: the owner names the balance version. Until then the schema takes any non-empty string, and the implementation issue stays blocked on this choice.
2. **Value kinds in v1.** Three kinds, as decided: `health` (pool is `health`, `shield` or `armor`), `cooldown` (seconds, per ability slot), `ultimateCost` (ultimate points). Other kinds are added later with a schema change.
3. **Verification states.** Four states: `measuredInClient`, `modeled`, `patchNotes`, `statedSource`.
   - Principle 9 names three: measured in the client, modeled, and patch notes or other stated source.
   - Proposal: split the third into `patchNotes` and `statedSource` (for example an official hero page). A reader sees at once which one applies.
   - "Not modeled" is not a state. A value that is not modeled has no entry.
4. **Where the data lives.** A new file per balance version, next to the existing hero dataset `crates/workshop-rs/src/gameplay/gameplay.json`, in a folder `crates/workshop-rs/src/gameplay/values/`. The issue says "next to `catalog.json`". That file is the Workshop language catalog and has no hero data. The hero dataset is the closer neighbour. The owner may prefer `src/catalog/data/`.
5. **One file per version, not one file with all versions.** Old versions stay untouched. A diff for a new version is small.
6. **No values from older versions.** A hero or value without a source is absent, not guessed. This matches the #11 acceptance criterion "no hero values older than the current version are claimed".
7. **athena reads the data through the public Rust API of `workshop-rs`.** See "Reading the data". The alternative is to read the JSON file directly. That alternative bypasses validation and the source check.
8. **Verification state is a new field on facts.** The existing `Fact` type has sources but no verification state. The implementation adds that field to hero values. Existing dataset facts are not changed.

## Data shape

One file holds one balance version. The schema is
[`schemas/hero-values-v1.schema.json`](../crates/workshop-rs/schemas/hero-values-v1.schema.json).

```json
{
  "format": "workshop-rs/hero-values-v1",
  "balanceVersion": "<balance version>",
  "heroes": [
    {
      "id": "<hero id from the gameplay dataset>",
      "values": [
        { "kind": "health", "pool": "shield", "amount": 0,
          "source": { "reference": "...", "locator": "..." },
          "verification": "modeled" }
      ]
    }
  ]
}
```

| Kind | Fields | Unit |
| --- | --- | --- |
| `health` | `pool`: `health`, `shield`, `armor`; `amount` | health points |
| `cooldown` | `slot` (logical slot id), optional `variant`; `seconds` | seconds |
| `ultimateCost` | `points` | ultimate points |

Hero ids, slot ids and variant ids use the identities of the gameplay dataset. The loader rejects an id that the gameplay dataset does not know. Amounts must be greater than zero.

Every value has `source` and `verification`. Both are required:

- `source.reference` names the origin (patch notes, official page, probe capture).
- `source.locator` points inside it (section, URL fragment, capture id).
- `source.note` is optional.
- For `measuredInClient`, the source is the probe capture that backs the value.

A file with a missing source, an unknown verification state, or an unknown value kind does not validate. The test fixtures `invalid-*.json` cover these three cases.

## Adding a balance version

1. Copy the previous file to `values/<balance-version>.json`.
2. Change `balanceVersion` and the values that changed. Update each source and verification state.
3. Remove values that have no source for the new version. Do not carry a value forward without a source.
4. Register the file in the loader and run the tests.

## Selecting a version

A consumer names the version it wants. The loader returns an error for an unknown version. No default version exists in the API. The newest version is a constant in the crate, so a consumer can ask for "the current version" by name.

## Reading the data

Proposed smallest API: the values load into the existing gameplay catalog as stats.

- `health` pools become hero stats (`health`, `shield`, `armor`).
- `cooldown` becomes the existing ability stat `cooldown`. The existing `query().cooldown(...)` works unchanged.
- `ultimateCost` becomes the ability stat `ultimateCost` in the `ultimate` slot.
- One new function loads a catalog for a named balance version, for example `gameplay::values::builtin(version)`.
- The verification state is read from the fact (decision 8).

athena#9 (state table) and athena#11 (settings modifiers on cooldowns and health) then use the same query calls they use for other hero data. No second reader exists.

## Out of scope

- Damage, healing, and other ability numbers.
- Judgement of balance (principle 9).
- `deltin-rs`.
- The loader and query code. They follow in the implementation issue after the owner accepts this design.
