# workshop-rs

`workshop-rs` is WrightKit's standalone Rust implementation of raw Overwatch
Workshop and the canonical Workshop semantic core shared by the ecosystem. It
provides an independently usable library and CLI for parsing, validating,
analyzing, converting, querying, and emitting Workshop text and reviewed
Workshop gameplay/catalog data.

`workshop-rs` is an independent library and CLI used across WrightKit as the
shared semantic core for raw Workshop code. Frontends such as `opy-rs` and
`del-rs` consume its public contracts to emit validated Workshop output without
tying their implementations to Wright internals.

```text
Raw Workshop text
    ↓
workshop-rs parser
    ↓
canonical public Workshop `Program`
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

`opy-rs` and `del-rs` own their respective frontend semantics, lowering
strategies, and source reconstruction. `workshop-rs` provides the canonical
Workshop target data and public `Program` model they compile into.

## Key features

- Canonical program API: raw parsing and independent language lowerings share
  the same constructible `Program`, `Rule`, `Action`, `Condition`, and `Value`
  types; storage IDs remain an implementation detail.
- Code generation and conversion: deterministic emission and translation
  (`en-US` ↔ `zh-CN`) with strict validation for missing terms.
- Catalog validation: canonical identities and allowlists for Workshop actions,
  values, events, enums, operators, and settings.
- Game domain data: typed queries for hero abilities, weapon slots, and custom
  game modifiers.
- Verification tools: offline feature census, regression test runners, and
  client version drift analysis.

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
let program = workshop_rs::parser::parse(text, &catalog, &locale)?;
program.validate()?;
workshop_rs::validate::validate_canonical_ids(&program, &catalog)?;
let emitted = emit(&program, &catalog, &locale)?;

// An independent language frontend can lower directly into the same model.
let mut generated = workshop_rs::Program::new();
generated
    .global_variable(workshop_rs::Variable::new("Score"))
    .rule(workshop_rs::Rule::new("generated", workshop_rs::Event::Global)
        .action(workshop_rs::Action::SetGlobalVariable {
            variable: "Score".to_string(),
            value: workshop_rs::Value::number(1.0),
        }));
let generated_text = emit(&generated, &catalog, &locale)?;

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
- **Events & event filters** (canonical rule events and all filter parameters)
- **Conditions & control flow** (branching, loops, jumps, aborts, waits)
- **Operators & variable modifications** (comparison operators, arithmetic operations, array modifications)
- **Actions inventory** (canonical Workshop actions)
- **Values inventory** (canonical Workshop values)
- **Enumerated domains** (enum domains)
- **Custom-game settings** (lobby, modes, heroes, extensions, and custom workshop settings)
- **Strings & localization** (`Custom String`, `en-US`, `zh-CN`, bidirectional conversion)
- **Tooling & semantic capabilities** (parsing, validation, emission, conversion, hero gameplay query APIs)

All features in `workshop-rs` are modeled directly on canonical Overwatch
Workshop behavior, keeping definitions language-neutral rather than shaped by
specific compiler frontends.

## Relationship with WrightKit implementations

- `workshop-rs`: raw Workshop implementation and canonical Workshop owner.
- `opy-rs`: standalone OverPy implementation; depends on `workshop-rs` for
  canonical Workshop target/source semantics.
- `del-rs`: standalone DEL/OSTW implementation; depends on `workshop-rs` for
  canonical Workshop target/source semantics.
- `wright`: unified tooling/integration product that consumes all three and adds
  cross-language lint, analysis, edits, agents, CI, embedding, and language
  services.

See [`docs/architecture/README.md`](docs/architecture/README.md) for the current
architecture boundary and routing.

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

Current architecture contracts, ADR history, catalog/gameplay contracts,
contract tests and fixture provenance, and release procedures are indexed in
[`docs/README.md`](docs/README.md).

## Releases

`workshop-rs` is published on crates.io with precompiled CLI archives attached
to GitHub Releases. Release automation is documented in [`docs/release.md`](docs/release.md).

## License

`workshop-rs` is distributed under the [MIT License](LICENSE). Committed dataset
and fixture mappings carry recorded provenance in [`docs/provenance.md`](docs/provenance.md).
