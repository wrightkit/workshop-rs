# workshop-rs implementation role

`workshop-rs` is both:

1. an independently usable Rust implementation of raw Overwatch Workshop; and
2. the canonical Workshop semantic core shared by WrightKit consumers.

Those roles reinforce each other. The repository must be useful without Wright,
while its public contracts provide one reviewed Workshop implementation for
`opy-rs`, `del-rs`, and Wright to reuse.

## Durable dependency model

```text
opy-rs ─────► workshop-rs ◄───── del-rs
                  ▲
                  │
                Wright
```

`workshop-rs` does not depend back on OPY, DEL/OSTW, or Wright tooling internals.

## What workshop-rs owns

- raw Workshop syntax/parsing;
- canonical Workshop identities, catalog, settings, and localization;
- Workshop WIR and validation;
- deterministic Workshop emission and supported locale conversion;
- Workshop-owned gameplay/catalog data and semantic queries;
- conformance, corpus, and seasonal-client evidence.

## What source-language implementations own

`opy-rs` and `del-rs` own their source-language semantics, project models,
high-level IR/HIR, compiler/runtime lowering choices, diagnostics/provenance,
and Workshop-to-source reconstruction.

They may depend on `workshop-rs` to compile to or reconstruct from Workshop.
That dependency does not make them incomplete implementations and does not make
`workshop-rs` their language owner.

## What Wright owns

Wright consumes all three implementations and adds unified tooling and
orchestration: lint, analysis, source-edit transactions, agent interfaces,
CI/embedding, language services, and cross-source workflow UX.

## Consumer-driven evolution

A consumer request should change `workshop-rs` only when it reveals a missing
canonical Workshop capability. Source-language-specific runtime layouts,
aliases, syntax, or compatibility quirks remain in the owning source-language
implementation.

This keeps the shared Workshop contract useful without turning it into a union
of every frontend/compiler's internal model.
