# AGENTS.md

This repository is part of the **WrightKit** multi-repository workspace. Apply
the workspace-level `AGENTS.md` when available, then follow this file.

`workshop-rs` is the canonical, separately versioned Overwatch Workshop
semantic core and standalone Workshop tooling foundation for WrightKit. It is
MIT-licensed.

## Ownership

This repository owns:

* canonical Workshop semantics: the catalog of Workshop-defined content
  (actions, values, events, enums and members, operators, structural and
  settings entries) bound to locale-independent canonical identities;
* the raw Workshop parser/CST-to-WIR frontend, catalog-backed validation,
  deterministic emitter, and WIR (Workshop IR);
* locale tables mapping canonical identities to client spellings, and
  locale detection/conversion.

This repository does **not** own, and must not depend on:

* Wright tooling crates (`wright-*`), Wright services, or their internals;
* any source-language provider (OPY, OSTW, DEL) semantics or internals;
* OverPy or OSTW implementations or data (GPL-3.0 reference data, including
  OverPy translation tables, is not a permissible data source — see
  [`docs/provenance.md`](docs/provenance.md) and Wright
  [`docs/adr/0004-overpy-licensing-boundary.md`](https://github.com/wrightkit/wright/blob/main/docs/adr/0004-overpy-licensing-boundary.md)).

All code and data in this repository must be MIT-compatible with recorded
provenance. Observed reference behavior is an interoperability input, never
permission to copy an implementation.

## Architecture boundaries

* Semantic code (parser, emitter, validation, WIR) contains no locale-specific
  branches and no per-locale spelling knowledge. Locale coverage lives
  exclusively in the catalog dataset.
* Canonical identities are locale-independent and never derived from provider
  naming. The catalog is an allowlist: anything not in the catalog is
  diagnosed, never guessed, never silently accepted.
* Missing target-locale mappings fail explicitly (error, not fallback);
  fallback is opt-in and visible in tooling output.
* See [`docs/adr/0001-catalog-boundaries.md`](docs/adr/0001-catalog-boundaries.md)
  for the full contract this implementation conforms to.

## Catalog data pipeline

Catalog updates are bounded data changes, never parser/emitter rewrites:

1. Edit `crates/workshop-rs/src/catalog/data/catalog.json` with updated
   provenance in `docs/provenance.md`.
2. Run `cargo run -p workshop-rs --bin workshop-catalog-gen -- check` — it
   must pass (schema, duplicate ids, alias collisions, undeclared locales,
   param arity, digest).
3. Run `cargo run -p workshop-rs --bin workshop-catalog-gen -- build` and
   review the canonical diff (recomputed digest).
4. Commit data and regenerated file together.

Adding a locale or aliases requires reviewed, MIT-permissible evidence
(workspace evidence hierarchy: reproducible behavior > contracts > tests and
fixtures > consumer projects > upstream references > documented community
evidence > assumptions) with provenance recorded per entry. Do not add
spellings you cannot evidence.

## Validation

Before declaring implementation work complete:

* `cargo fmt --all --check`
* `cargo clippy --workspace --all-targets -- -D warnings`
* `cargo test --workspace --all-targets`
* `cargo run -p workshop-rs --bin workshop-catalog-gen -- check`
* `git diff --check`

CI runs the same checks on stable and the pinned toolchain (1.85.0). A local
pass is not final acceptance.

## Delivery

* Conventional Commits; focused commits; no pushes to `main`; deliver through
  PRs.
* Never commit credentials, private runtime data, or unreviewed third-party
  material. Fixtures carry provenance (see `crates/workshop-rs/tests/fixtures/README.md`).
