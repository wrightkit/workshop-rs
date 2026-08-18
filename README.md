# workshop-rs

The canonical, separately versioned **Overwatch Workshop semantic core** for
the WrightKit ecosystem: a standalone, MIT-licensed Rust library and CLI for
parsing, validating, analyzing, converting, and emitting raw Workshop text and
hero gameplay data.

`workshop-rs` has no dependency on Wright compiler crates or source-language
provider internals (OPY, OSTW). Semantic code contains no locale-specific
branches; locale coverage lives exclusively in the catalog dataset.

## Features

- **Raw Workshop parsing & WIR:** parse raw Workshop text into a validated,
  locale-independent Workshop Intermediate Representation (WIR).
- **Deterministic emission & conversion:** deterministic localized emission and
  raw Workshop conversion (`en-US` ↔ `zh-CN`) with fail-explicit missing-mapping
  safety and opt-in fallback.
- **Catalog & allowlist validation:** canonical, locale-independent identities
  for Workshop actions, values, events, enums, operators, and settings blocks.[^catalog-boundary]
- **Hero gameplay & query domain:** embedded hero roster dataset and typed
  semantic queries for abilities, logical slots, variants, Custom Game cooldown
  percentages, and localized ability name resolution.[^gameplay-domain]
- **Conformance & census harness:** deterministic offline feature census,
  real-project regression runner, and seasonal client drift analysis.[^conformance]
- **Standalone architecture:** zero external runtime dependencies.

## CLI usage

The standalone CLI provides tools for parsing, emission, conversion, census, and
conformance:

```sh
workshop-rs-cli parse file.ws                 # parse raw Workshop text -> WIR dump
workshop-rs-cli emit file.ws                  # parse -> emit localized Workshop text
workshop-rs-cli convert file.ws --from en-US --to zh-CN
workshop-rs-cli convert file.ws --from en-US --to zh-CN --fallback-locale en-US
workshop-rs-cli locales                       # list declared locales and mapping coverage
workshop-rs-cli version --json                # machine-readable catalog and provenance identity
workshop-rs-cli census [--json]               # run deterministic offline feature census
workshop-rs-cli corpus manifest.json [--json] # run provenance-linked real-project corpus
```

Exit codes: `0` success, `1` parse/emit/conversion/catalog failure, `2` usage error.

## Library usage

```rust
use workshop_rs::catalog::{Catalog, Locale};
use workshop_rs::convert::{convert, ConvertOptions};
use workshop_rs::emitter::emit;
use workshop_rs::gameplay::{hero_ids, slots};
use workshop_rs::gameplay_data;

// Parse raw Workshop text into locale-independent WIR
let catalog = Catalog::builtin()?;
let locale = Locale::new("en-US");
let program = workshop_rs::parser::parse_with_context(text, &catalog, &locale, &catalog)?;

// Emit localized Workshop text
let emitted = emit(&program, &catalog, &locale)?;

// Convert raw Workshop text between locales (fails explicitly on missing mappings)
let converted = convert(
    text,
    &catalog,
    &Locale::new("en-US"),
    &Locale::new("zh-CN"),
    &ConvertOptions::default(),
)?;

// Query hero gameplay facts and calculate effective cooldowns
let gameplay = gameplay_data::builtin()?;
let query = gameplay.query();
let sleep_dart = query.slot_ability(hero_ids::ANA, slots::ABILITY_1)?;
let base_cooldown = query.cooldown(&sleep_dart)?;
```

## Current support

| Capability | Status | Notes |
| --- | --- | --- |
| Raw Workshop parser & WIR | ✅ Supported | CST-to-WIR frontend, settings blocks, comment/structure validation |
| Deterministic localized emitter | ✅ Supported | Formatted Workshop text emission for declared locales |
| Locale conversion (`en-US` ↔ `zh-CN`) | ✅ Supported | Full declared canonical surface (366/366 entries);[^locale-provenance] fail-explicit on missing mappings |
| Workshop catalog & signatures | ✅ Supported | Locale-independent canonical identities for actions, values, events, enums, and operators |
| Hero gameplay dataset & topology | ✅ Supported | 53 heroes, 207 ability records, role facts, open logical slots, and variant modeling |
| Gameplay query & cooldown math | ✅ Supported | Deterministic kit lookups, Custom Game cooldown percentage math (0%–500%), ability name resolution |
| Offline census & conformance | ✅ Supported | Sharded feature census, real-project regression runner, and seasonal drift detection |
| Additional client locales | ⏳ Not yet | Admitted only through reviewed, MIT-compatible evidence pipelines |

## Catalog data pipeline

Catalog and gameplay dataset updates are bounded data changes verified by
deterministic generators:

```sh
cargo run -p workshop-rs --bin workshop-catalog-gen -- check
cargo run -p workshop-rs --bin workshop-catalog-gen -- build
```

See [`docs/provenance.md`](docs/provenance.md) and [`AGENTS.md`](AGENTS.md) for
provenance requirements and pipeline details.

## Validation

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets
cargo run -p workshop-rs --bin workshop-catalog-gen -- check
```

CI runs the same checks on stable and the pinned toolchain (1.85.0).

## Documentation

Detailed architecture specifications, ADRs, catalog/gameplay contracts,
conformance evidence, and release procedures are indexed in
[`docs/README.md`](docs/README.md).

## Releases

`workshop-rs` is published on [crates.io](https://crates.io/crates/workshop-rs),
with precompiled CLI archives attached to
[GitHub Releases](https://github.com/wrightkit/workshop-rs/releases).
Release automation and repository setup are documented in
[`docs/release.md`](docs/release.md).

## License

`workshop-rs` is distributed under the [MIT License](LICENSE). Committed dataset
and fixture mappings carry recorded provenance ([`docs/provenance.md`](docs/provenance.md)).

[^catalog-boundary]: Contract defined in [ADR-0001: Workshop catalog, locale, provenance, and version boundaries](docs/adr/0001-catalog-boundaries.md).
[^gameplay-domain]: Hero data schema and query model defined in [ADR-0002 (Gameplay)](docs/adr/0002-gameplay-domain-api.md) and [Hero gameplay dataset](docs/gameplay-data.md).
[^conformance]: Conformance contracts and evidence runners defined in [ADR-0002](docs/adr/0002-conformance-contract.md), [ADR-0003](docs/adr/0003-sharded-census.md), [ADR-0004](docs/adr/0004-real-project-evidence.md), and [ADR-0005](docs/adr/0005-seasonal-client-validation.md).
[^locale-provenance]: Declared entries cover all structural tokens, actions, values, events, operators, and enum members. See the [provenance record](docs/provenance.md) for details.
