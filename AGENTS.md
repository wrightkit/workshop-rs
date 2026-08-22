# AGENTS.md

This repository is part of the **WrightKit** multi-repository workspace. Apply
the workspace-level `AGENTS.md` when available, then follow this file.

`workshop-rs` is WrightKit's standalone Rust implementation of raw Overwatch
Workshop and the canonical Workshop semantic core shared by the ecosystem. It
is not an internal Wright backend repository: Wright, `opy-rs`, and `del-rs`
consume its reviewed public contracts while `workshop-rs` remains independently
usable as a library and CLI.

## Ownership

This repository owns:

- canonical Workshop semantics and locale-independent identities for actions,
  values, events, enums/members, operators, settings, heroes, maps, modes, and
  other declared Workshop content;
- raw Workshop parsing, canonical WIR, validation, deterministic emission, and
  locale conversion;
- Workshop-owned source/provenance contracts needed by consumers;
- reviewed Workshop gameplay/catalog data and semantic query APIs;
- Workshop conformance, corpus, and seasonal-client evidence.

This repository does **not** own:

- OverPy syntax, preprocessing, macros, source semantics, or reconstruction
  (`opy-rs`);
- DEL/OSTW syntax, project/runtime semantics, or reconstruction (`del-rs`);
- Wright lint/analyze/agent/CI/LSP/orchestration behavior (`wright`);
- LPP protocol semantics (`language-provider-protocol`).

The durable dependency direction is consumer → Workshop core:

```text
opy-rs ─────► workshop-rs
del-rs ─────► workshop-rs
wright  ─────► workshop-rs
```

Do not introduce dependencies from `workshop-rs` back to source-language
implementations or Wright tooling internals merely to make one integration
simpler.

A request from `opy-rs` or `del-rs` becomes `workshop-rs` work only when the
missing capability is genuinely canonical Workshop behavior. Do not add
provider-shaped nodes, aliases, runtime layouts, or source-language semantics to
WIR/catalog contracts unless they are independently justified as Workshop
semantics.

See [`docs/implementation-role.md`](docs/implementation-role.md) for the durable
relationship with the other WrightKit implementations.

## Architecture boundaries

- Semantic code contains no locale-specific branches or per-locale spelling
  knowledge; locale coverage lives in reviewed catalog data.
- Canonical identities are locale-independent and never derived from OPY/DEL
  naming.
- Missing mappings or unsupported catalog entries fail explicitly rather than
  being guessed.
- Public WIR/catalog contracts should remain generic enough for raw Workshop and
  multiple source-language implementations.
- Internal representation can evolve when tests and public contracts remain
  valid; do not spend product-critical time preserving incidental internal
  structure.

## Development priority

Prioritize observable Workshop correctness and real consumer blockers over
architecture polish. New canonical contracts should normally be justified by at
least one of:

1. raw Workshop evidence;
2. seasonal/client/catalog evidence;
3. a real `opy-rs` or `del-rs` integration blocker demonstrating a missing
   canonical Workshop capability;
4. a Wright tooling blocker that cannot be solved correctly above the Workshop
   layer.

When the problem is source-language-specific, route it back to the owning
implementation instead of widening `workshop-rs`.

## Catalog data pipeline

Catalog updates are bounded data changes with recorded provenance:

1. edit the canonical data source;
2. run `cargo run -p workshop-rs --bin workshop-catalog-gen -- check`;
3. rebuild generated data and review the diff;
4. commit source data, provenance, and generated artifacts together.

Do not use unreviewed or license-incompatible upstream compiler data as a
canonical source.

## Validation

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets
cargo run -p workshop-rs --bin workshop-catalog-gen -- check
git diff --check
```

A local pass is not proof of live-client behavior. Claims about current
Workshop acceptance or seasonal behavior require the corresponding client or
provenance-backed evidence.

## Delivery

- Use focused branches and PRs; never push directly to `main`.
- Keep commits scoped and avoid unrelated repository changes.
- Review-time verification results, including hashes, residual counts, and
  pass/fail status, must come from the test/CI run under review. Never hand-write
  or manually refresh a committed evidence/result file; put results in the PR
  description and CI logs/artifacts. Committed fixtures and provenance/input
  manifests are allowed only as reproducible, machine-validated inputs.
- Never commit credentials, private runtime data, or unreviewed third-party
  material.
