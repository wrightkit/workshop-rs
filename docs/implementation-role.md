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

## Indexed members and control-flow lowering contract

The canonical WIR contract for the interoperability surface in
`wrightkit/workshop-rs#123` is deliberately composed from native Workshop
identities:

- indexing a declared global or player variable uses the catalog-backed
  `set*VariableAtIndex` and `modify*VariableAtIndex` action calls;
- a member read is the `memberAccess(receiver, "member"[, index])` value call,
  and `AssignMember` accepts only that canonical member-access value as its
  assignment target without assigning it dictionary, object, or container type
  semantics;
- native `Break`, `Continue`, `Skip`, and `Skip If` actions remain generic
  catalog action calls and can occur inside the existing structured `If`,
  `While`, and `For` actions.

There are no `Dictionary`, `Switch`, or `Break` WIR nodes. OPY dictionary and
switch lowering remains an `opy-rs` concern: a source form is consumable only
after it is lowered to the native indexed/member or control-flow calls above.
If no lossless lowering exists, `opy-rs` must report its explicit integration
boundary rather than extending this Workshop contract with source-language
carriers.

This classifies the remaining interoperability probes without using OPY syntax
as Workshop evidence: the dictionary-literal probe is an OPY-owned lowering
gap unless it folds to one of the native indexed/member forms, while the
nested and multiple switch-break probes have the required native `Break`,
`Skip`, and `Skip If` primitives but still require `opy-rs` to prove a lossless
source-to-Workshop lowering for their control-flow shape.
