# workshop-rs

The canonical, separately versioned **Overwatch Workshop semantic core** for
the WrightKit ecosystem: a standalone, MIT-licensed library and CLI for
parsing, validating, analyzing, converting, and emitting raw Workshop text.

Status: **v0.2 bootstrap** (Issue #2). The repository is under active
development; no release compatibility claims are made yet.

## What it is

`workshop-rs` owns the Workshop-language foundation that WrightKit tooling
(and, in future, the Wright compiler) consume:

* a **catalog** of Workshop-defined content (actions, values, events, enums
  and members, operators, structural and settings entries) bound to
  **locale-independent canonical identities**, with per-locale spelling
  tables;
* a **raw Workshop parser** (lexer/parser) producing validated **Workshop IR
  (WIR)**, a locale-independent semantic representation;
* **catalog-backed validation**, a **deterministic localized emitter**, and
  **locale detection**;
* **raw Workshop locale conversion** (`en-US` <-> other locales), with
  missing target-locale mappings failing explicitly by default and fallback
  opt-in.

It has **no dependency on Wright tooling crates** (`wright-*`) or on any
source-language provider implementation (OPY, OSTW, DEL). Locale coverage
lives exclusively in the catalog dataset; semantic code contains no
locale-specific branches.

Boundary contract: [docs/adr/0001-catalog-boundaries.md](docs/adr/0001-catalog-boundaries.md)
(ADR-0001). Data provenance and licensing: [docs/provenance.md](docs/provenance.md).

## Workspace layout

```
crates/workshop-rs/      library: catalog, lexer/parser, WIR, validation, emitter, detect, convert
  src/bin/               workshop-catalog-gen: the reproducible catalog data pipeline
crates/workshop-rs-cli/  standalone CLI: parse, emit, convert, locales, version
docs/                    ADR-0001 (boundary contract), provenance record
```

## Library usage

```rust
use workshop_rs::catalog::{Catalog, Locale};
use workshop_rs::convert::{convert, ConvertOptions};
use workshop_rs::emitter::{emit, EmitOptions};

let catalog = Catalog::builtin()?;
let locale = Locale::new("en-US");

// Parse raw Workshop text into locale-independent WIR.
let program = workshop_rs::parser::parse_with_context(text, &catalog, &locale, &catalog)?;

// Emit localized Workshop text (fails explicitly on missing mappings).
let emitted = emit(&program, &catalog, &locale)?;

// Convert raw Workshop text between locales. Missing target-locale mappings
// fail explicitly unless a fallback locale is opted into.
let out = convert(text, &catalog, &Locale::new("en-US"), &Locale::new("zh-CN"),
                  &ConvertOptions::default())?; // Err: missing mapping
let out = convert(text, &catalog, &Locale::new("en-US"), &Locale::new("zh-CN"),
                  &ConvertOptions { fallback_locale: Some(Locale::new("en-US")) })?;

// Machine-readable catalog identity (implementation version, catalog
// version + digest, locale coverage, target evidence, provenance).
let identity = catalog.identity();
```

Note: settings-bearing programs cannot be parsed from raw text (a `.ws`
decompiler is a non-goal); settings are carried in WIR and emitted by the
library. Locale detection is available via `workshop_rs::detect`.

## CLI usage

```sh
cargo run -p workshop-rs-cli -- parse file.ws                 # parse -> WIR dump
cargo run -p workshop-rs-cli -- emit file.ws                  # parse -> emit
cargo run -p workshop-rs-cli -- convert file.ws --from en-US --to zh-CN
cargo run -p workshop-rs-cli -- convert file.ws --from en-US --to zh-CN --fallback-locale en-US
cargo run -p workshop-rs-cli -- locales                       # declared locales + coverage
cargo run -p workshop-rs-cli -- version --json                # machine-readable catalog identity
```

Exit codes: `0` success, `1` parse/emit/conversion failure, `2` usage error.
A conversion with missing target-locale mappings fails with exit `1` and a
`missing … mapping for locale …` diagnostic; with `--fallback-locale` the
fallback choice is reported on stderr.

## Catalog data pipeline

Catalog updates are bounded data changes, never code rewrites:

```sh
cargo run -p workshop-rs --bin workshop-catalog-gen -- check
cargo run -p workshop-rs --bin workshop-catalog-gen -- build
```

`check` validates (schema, duplicate ids, alias collisions, undeclared
locales, param arity) and verifies the content digest; `build` regenerates
the canonical form with a fresh digest (byte-idempotent). See
[docs/provenance.md](docs/provenance.md) and the repo `AGENTS.md`.

## Locale status

* `en-US`: complete declared surface (344/344 canonical entries), corpus
  round-trips and settings emission tested.
* `zh-CN`: the reviewed export-backed corpus covers **327/344** canonical
  entries (structural 11/11, actions 55/62, values 77/78, events 3/3,
  operators 8/14, enum members 173/176). The 17 exact-match exclusions remain
  fail-explicit; settings data covers the matched declared surface and records
  its exclusions in `crates/workshop-rs/src/settings/data/zh-cn.json`.

The corpus is reproducible with the user-provided export (not committed):

```sh
cargo run -p workshop-rs --bin workshop-catalog-gen -- corpus \
  --export /path/to/workshop-data.json
cargo run -p workshop-rs --bin workshop-catalog-gen -- build
```

## Validation

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets
cargo run -p workshop-rs --bin workshop-catalog-gen -- check
```

CI runs the same checks on stable and the pinned toolchain (1.85.0).

## License

MIT — see [LICENSE](LICENSE). Committed data carries recorded provenance
([docs/provenance.md](docs/provenance.md)); GPL reference data (e.g. OverPy
translation tables) is not a permissible data source.
