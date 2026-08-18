# Gameplay query API

`workshop-rs` exposes `GameplayCatalog::query()` as a read-only semantic view
over validated `GameplayCatalog` records. Queries do not load a data file,
invent missing facts, or mutate the catalog.

## Deterministic lookup

```rust
use workshop_rs::gameplay::{slots, AbilityVariant};
use workshop_rs::gameplay_query::GameplayQueryError;

let query = catalog.query();
let ana = query.hero("ana")?;
let kit = query.kit("ana")?;
let sleep_dart = query.slot_ability("ana", slots::ABILITY_1)?;
let stat = query.quantity_stat("ana", slots::ABILITY_1, None, "cooldown")?;
# let _: Result<(), GameplayQueryError> = Ok(());
```

The catalog's heroes are in canonical ID order. A kit is sorted by logical
slot, and variant; a slot result is sorted by slot and variant; keyword
matches are sorted by hero ID, slot, and variant. `slot()` returns
all entries and reports an absent slot as `MissingSlot`. `slot_ability()` is the
single-result form and reports `AmbiguousSlot` when a slot has multiple
entries. A form-dependent ability must be selected with `variant()`; no first
entry is selected implicitly.

Missing heroes, abilities, variants, and stats are structured errors. A keyword
query with no matches returns an empty collection because it is a collection
query rather than a single-result lookup.

`stat()` and `quantity_stat()` take hero + logical slot + optional explicit
hero-local variant. They return the extensible `Fact<StatValue>` or a
`Quantity`, and report
`WrongStatType` when the stat is not numeric. Hero-level stats are available
through `hero_stat()`.

## Cooldown calculations

`query.cooldown(&ability_ref)` is the common typed cooldown accessor. It only accepts
a finite, positive `Quantity` whose unit is `seconds`. Missing cooldown stats,
text/boolean/choice values, other units, and non-positive values return an
explicit `CooldownError`; there is no default cooldown.

`CooldownPercentage` accepts the inclusive range `0%..=500%` and rejects
negative, above-maximum, NaN, and infinite values. Effective cooldown uses the
Custom Game convention:

```text
effective cooldown = base cooldown × custom-game percentage ÷ 100
```

Thus `100%` preserves the base value, `50%` halves it, and `0%` produces a
zero-second result. The result remains a seconds `Quantity` and raw data is not
changed.

To calculate a setting for a target, pass a seconds `Quantity` to
`required_cooldown_percentage()`. A target may be zero, but must be finite and
non-negative. The resulting percentage must fit the same `0%..=500%` range;
targets above the supported range return `InvalidPercentage` rather than being
clamped or silently approximated. A target with another unit is rejected.

## Locale-aware ability names

`ability_name(hero, slot, variant, locale)` resolves an evidenced client name,
and `resolve_ability_name(hero, locale, display_name)` performs exact,
hero-context inverse lookup. Unsupported locales, missing mappings, and
ambiguous names are explicit errors; fuzzy aliases are not admitted. Raw
Workshop hero-setting parsing and emission use this resolver for ability
setting labels.

The query layer is intentionally independent of Wright, OPY, DEL, CLI
presentation, source syntax, and runtime simulation.
