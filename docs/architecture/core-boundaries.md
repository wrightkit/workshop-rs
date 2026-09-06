# Workshop Core Boundaries

This document is the current architecture contract for `workshop-rs` ownership and canonical Workshop semantics. It describes durable boundaries, not feature-completion status.

## Ownership

`workshop-rs` is both an independently usable raw Workshop implementation and WrightKit's canonical Workshop semantic core.

It owns:

- raw Workshop syntax, parsing, validation, and deterministic emission;
- canonical WIR and locale-independent Workshop identities;
- Workshop catalog, settings, localization, and Workshop-owned gameplay facts;
- Workshop semantic/query contracts used by downstream tooling;
- Workshop conformance, corpus, provenance, and seasonal-client evidence.

It does not own OverPy or DEL/OSTW syntax, preprocessing, project models, source-language semantics, runtime/compiler lowering policy, reconstruction policy, or Wright tooling behavior.

The durable dependency direction is:

```text
opy-rs ─────► workshop-rs ◄───── deltin-rs
                  ▲
                  │
                Wright
```

No source-language representation becomes canonical Workshop behavior merely because a consumer needs it. A consumer gap belongs here only when it demonstrates a missing Workshop concept or observable Workshop semantic contract.

## Canonical WIR boundary

WIR represents canonical Workshop meaning. It may expose structured forms where Workshop itself has a stable semantic distinction, but it must not carry provider-specific syntax, aliases, helper identities, source-language runtime layouts, or reconstruction-only carriers.

Source-language implementations lower their constructs into canonical WIR. If a source construct has no correct Workshop representation, the owning source implementation reports that boundary rather than widening WIR with a source-language surrogate.

## Behavior and data

Use data for large declarative fact sets whose meaning is already defined by the Workshop domain, for example:

- canonical content identities and membership;
- signatures and declared parameter domains when they are factual catalog properties;
- locale spellings and aliases;
- roster/catalog/settings inventories;
- provenance and evidence references.

Use typed Rust code for behavior and invariants, including:

- context-sensitive semantic transformations;
- normalization/coercion behavior;
- evaluation or control-flow rules;
- semantic validation decisions;
- relationships whose interpretation changes program meaning.

A data field may describe a fact consumed by typed behavior; it should not become a new interpreted language for defining Workshop semantics. If adding metadata would make generic Rust code decide new program behavior from that metadata, treat that as an architecture question rather than assuming the existing data-driven placement is precedent.

Existing metadata-driven behavior is implementation reality to audit against this boundary, not proof that further semantics belong in metadata.

## Localization and source-language separation

Canonical identities are locale-independent and independent from OPY/DEL naming. Semantic code contains no per-locale spelling branches; localization tables map canonical identities to presentation spellings.

Likewise, OverPy or DEL/OSTW names, defaults, helpers, or compiler quirks do not become Workshop identities unless independent Workshop evidence establishes the same concept.

## Support and evidence

Architecture does not prove feature support. Current support claims come from the language-support surface and executable evidence: code, tests, corpus/fixtures, catalog checks, real consumer workflows, and live/runtime evidence where applicable.

ADRs explain why earlier decisions were made. When an ADR, Issue, this current contract, and implementation reality disagree, implementation work must surface the mismatch rather than selecting whichever source is most convenient.