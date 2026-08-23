# workshop-rs

`workshop-rs` is WrightKit's standalone Rust implementation of raw Overwatch
Workshop and the canonical Workshop semantic core shared by the ecosystem. It
provides an independently usable library and CLI for parsing, validating,
analyzing, converting, querying, and emitting Workshop text and reviewed
Workshop gameplay/catalog data.

It is not merely a backend hidden behind Wright. Wright, `opy-rs`, and `del-rs`
are consumers of its public contracts. `workshop-rs` has no dependency on
Wright tooling internals or source-language implementation internals.

```text
Raw Workshop text
    ↓
workshop-rs parser
    ↓
canonical Workshop WIR / catalog identities
    ↓
validation / semantic query / transformation
    ↓
localized emission
    ↓
Raw Workshop text
```

For source languages that compile to Workshop, the durable dependency direction
is:

```text
opy-rs ─────► workshop-rs
del-rs ─────► workshop-rs
wright  ─────► workshop-rs
```

`opy-rs` and `del-rs` remain responsible for their own source-language
semantics, compiler lowering choices, and Workshop-to-source reconstruction.
`workshop-rs` does not become an OverPy or DEL/OSTW implementation simply
because those projects depend on its Workshop capabilities.

## Features

- **Raw Workshop parsing & WIR:** parse raw Workshop text into a validated,
  locale-independent Workshop Intermediate Representation (WIR).
- **Deterministic emission & conversion:** deterministic localized emission and
  raw Workshop conversion (`en-US` ↔ `zh-CN`) with fail-explicit missing-mapping
  safety and opt-in fallback.
- **Catalog & allowlist validation:** canonical, locale-independent identities
  for Workshop actions, values, events, enums, operators, settings, and content.
- **Hero gameplay & query domain:** embedded reviewed hero/gameplay data and
  typed semantic queries for abilities, slots, variants, custom-game modifiers,
  and localized ability-name resolution.
- **Conformance & census harness:** deterministic offline feature census,
  real-project regression runner, and seasonal client-drift analysis.
- **Standalone architecture:** zero dependency on upstream compiler runtimes or
  Wright tooling internals.

## CLI usage

```sh
workshop-rs-cli parse file.ws
workshop-rs-cli emit file.ws
workshop-rs-cli convert file.ws --from en-US --to zh-CN
workshop-rs-cli convert file.ws --from en-US --to zh-CN --fallback-locale en-US
workshop-rs-cli locales
workshop-rs-cli version --json
workshop-rs-cli census [--json]
workshop-rs-cli corpus manifest.json [--json]
workshop-rs-cli seasonal-diff previous.json current.json [--json]
```

Exit codes: `0` success, `1` parse/emit/conversion/catalog failure, `2` usage
error.

## Library usage

```rust
use workshop_rs::catalog::{Catalog, Locale};
use workshop_rs::convert::{convert, ConvertOptions};
use workshop_rs::emitter::emit;

let catalog = Catalog::builtin()?;
let locale = Locale::new("en-US");
let program = workshop_rs::parser::parse_with_context(text, &catalog, &locale, &catalog)?;
let emitted = emit(&program, &catalog, &locale)?;

let converted = convert(
    text,
    &catalog,
    &Locale::new("en-US"),
    &Locale::new("zh-CN"),
    &ConvertOptions::default(),
)?;
```

## Language support

`workshop-rs` maintains a complete, human-readable language support matrix that serves as the single authoritative source of truth for declared Workshop language capabilities.

See [`docs/language-support.md`](docs/language-support.md) for the complete capability matrix across:
- **Program structure & rules** (settings, variables, subroutines, rules, conditions, actions, disabled modifiers)
- **Variables & subroutines** (global and player variable declaration, read, write, indexed modify, subroutine calls and events)
- **Events & event filters** (14 canonical rule events and all filter parameters)
- **Conditions & control flow** (branching, loops, jumps, aborts, waits)
- **Operators & variable modifications** (comparison operators, arithmetic operations, array modifications)
- **Actions inventory** (all 219 canonical Workshop actions)
- **Values inventory** (all 255 canonical Workshop values)
- **Enumerated domains** (all 52 enum domains)
- **Custom-game settings** (lobby, modes, heroes, extensions, and custom workshop settings)
- **Strings & localization** (`Custom String`, `en-US`, `zh-CN`, bidirectional conversion)
- **Tooling & semantic capabilities** (parsing, validation, emission, conversion, hero gameplay query APIs)

The canonical Workshop baseline is intentionally independent of any single
source-language compiler. Consumer-driven additions must remain generic
Workshop contracts rather than OPY- or DEL-shaped special cases.

## Relationship with WrightKit implementations

- `workshop-rs`: raw Workshop implementation and canonical Workshop owner.
- `opy-rs`: standalone OverPy implementation; depends on `workshop-rs` for
  canonical Workshop target/source semantics.
- `del-rs`: standalone DEL/OSTW implementation; depends on `workshop-rs` for
  canonical Workshop target/source semantics.
- `wright`: unified tooling/integration product that consumes all three and adds
  cross-language lint, analysis, edits, agents, CI, embedding, and language
  services.

See [`docs/implementation-role.md`](docs/implementation-role.md) for the durable
boundary.

## Catalog data pipeline

Catalog and gameplay dataset updates are bounded reviewed data changes verified
by deterministic generators:

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

CI runs the same checks on stable and the pinned toolchain.

## Documentation

Detailed architecture specifications, ADRs, catalog/gameplay contracts,
conformance evidence, and release procedures are indexed in
[`docs/README.md`](docs/README.md).

## Releases

`workshop-rs` is published on crates.io with precompiled CLI archives attached
to GitHub Releases. Release automation is documented in [`docs/release.md`](docs/release.md).

## License

`workshop-rs` is distributed under the [MIT License](LICENSE). Committed dataset
and fixture mappings carry recorded provenance in [`docs/provenance.md`](docs/provenance.md).
