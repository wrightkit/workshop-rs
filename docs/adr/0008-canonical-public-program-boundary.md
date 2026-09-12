# ADR-0008: Canonical public Workshop `Program` boundary

- Status: Accepted
- Date: 2026-09-12
- Related:
  [Issue #32](https://github.com/wrightkit/workshop-rs/issues/32),
  [Issue #112](https://github.com/wrightkit/workshop-rs/issues/112),
  [Issue #129](https://github.com/wrightkit/workshop-rs/issues/129),
  [Issue #177](https://github.com/wrightkit/workshop-rs/issues/177),
  [Issue #178](https://github.com/wrightkit/workshop-rs/issues/178),
  [Issue #179](https://github.com/wrightkit/workshop-rs/issues/179),
  [Issue #187](https://github.com/wrightkit/workshop-rs/issues/187);
  implementation:
  [PR #180](https://github.com/wrightkit/workshop-rs/pull/180),
  [PR #181](https://github.com/wrightkit/workshop-rs/pull/181),
  [PR #183](https://github.com/wrightkit/workshop-rs/pull/183),
  [PR #185](https://github.com/wrightkit/workshop-rs/pull/185),
  [PR #188](https://github.com/wrightkit/workshop-rs/pull/188),
  [PR #130](https://github.com/wrightkit/workshop-rs/pull/130)

## Context

The original public surface exposed arena-backed WIR storage, typed node IDs,
and phase-oriented compatibility paths. That surface made ordinary raw
Workshop use and independent source-language lowering depend on implementation
details rather than Workshop concepts. The contract recorded in #112 and the
implementation work in #177–#179 replaced that boundary. #32 and #187
established how source-aware consumers can attach optional provenance without
making metadata part of ordinary semantic construction.

The same boundary audit found that dedicated `Debug` and `Print` WIR nodes
were provider helpers rather than native Workshop semantics (#129). Keeping
such carriers in canonical WIR would make a consumer's source-language
convenience part of the Workshop contract.

## Decision

1. `Program`, `Rule`, `Condition`, `Action`, `Value`, `Event`, declarations,
   subroutines, and settings are the ordinary public Workshop concepts. Raw
   parsing and independent source-language lowerings target the same
   constructible `Program` model.
2. Workshop execution structure is preserved in that model. Rule actions are
   a linear Workshop action stream, values are composable expressions, and
   conditions remain explicit rule-condition concepts.
3. Arena allocation, storage IDs, normalized WIR, generator tables, and
   compatibility modules are implementation or support details. They may be
   used internally, but they are not a second ordinary public semantic
   contract and are not required knowledge for consumers.
4. Public parsing, validation, analysis, conversion, source editing,
   round-trip comparison, emission, and CLI workflows operate through
   `Program`. Compatibility re-exports may be retained only where an existing
   consumer contract requires them and do not define the preferred boundary.
5. Typed action and value constructors are generated from canonical catalog
   facts. They constrain builtin identity and parameter shape while canonical
   validation remains authoritative. Generic catalog-backed calls remain an
   explicit escape hatch for dynamic consumers.
6. Source documents, spans, comments, trivia, and provenance are optional
   metadata associated with the semantic program. Raw parsing may attach them;
   an external consumer may add a source file and attach authored spans; and
   programmatic construction may omit them. Source-preservation and checked
   edit operations must fail closed for stale, invalid, overlapping, foreign,
   or unsupported source relationships.
7. Provider-specific helpers, aliases, runtime layouts, and reconstruction
   carriers do not become canonical Workshop nodes without independent
   Workshop evidence. Their lowering and source semantics remain owned by the
   corresponding provider.

## Consequences

Consumers can construct and inspect a Workshop program without importing
storage internals, while the implementation can retain normalized WIR where
it provides analysis or performance value. Public operations share one
semantic boundary, so a consumer cannot accidentally obtain different
behavior by choosing a parser, emitter, or storage-shaped API.

Optional provenance supports diagnostics, source-aware edits, comments, and
reconstruction without forcing synthetic spans into generated programs.
Provider conveniences that have no independent Workshop meaning must be
lowered by their owner instead of being preserved by the canonical core.

This ADR does not define OPY/DEL lowering policy, source-language project
models, or exact whole-file formatting.
