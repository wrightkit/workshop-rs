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

Settings-bearing programs are parsed into the canonical WIR settings carrier
and emitted by the library. Locale detection is available via
`workshop_rs::detect`.

### Rule event contract

The public WIR event contract is locale- and provider-independent. It includes
`Event::Global`, `Event::EachPlayer`, filtered `Event::EachPlayerWithFilters`,
nine filtered `Event::Player` identities (`PlayerEventKind`), and
`Event::Subroutine`. Filtered events carry a canonical `EventTeam` and an
`EventTarget` (`All`, Workshop slot `0..=11`, or a canonical hero id).
Raw Workshop player events require both their team and player filters;
parameterless `Ongoing - Each Player` remains supported for existing programs.
Event identities and filter members are checked against the catalog before a
program is accepted for canonical emission, so source-language providers can
consume this public model without defining a second event table.

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

* `en-US`: complete declared surface (366/366 canonical entries), corpus
  round-trips and settings emission tested.
* `zh-CN`: the reviewed export-backed corpus covers **366/366** canonical
  entries (structural 11/11, actions 60/60, values 77/77, events 12/12,
  operators 14/14, enum members 192/192). The declared surface is complete;
  settings data covers labels 19/19 and all other declared settings sections.

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

## Releases

Maintainers publish a versioned library and CLI release from the manually
triggered `Release` GitHub Actions workflow. Select the `patch`, `minor`, or
`major` bump while dispatching from `main`; the workflow runs the quality and
catalog gates, publishes the library before the CLI, and attaches checksummed
cross-platform CLI artifacts. Repository setup and retry behavior are
documented in [docs/release.md](docs/release.md).

## License

MIT — see [LICENSE](LICENSE). Committed mapping data carries recorded
provenance ([docs/provenance.md](docs/provenance.md)); the user-provided JSON
is build input and is not redistributed.
