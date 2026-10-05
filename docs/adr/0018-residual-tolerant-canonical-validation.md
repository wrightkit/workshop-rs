# ADR-0018: Residual-tolerant canonical validation entry point

- Status: Accepted
- Date: 2026-10-05
- Related:
  [Issue #372](https://github.com/wrightkit/workshop-rs/issues/372),
  [PR #373](https://github.com/wrightkit/workshop-rs/pull/373),
  [wright#501](https://github.com/wrightkit/wright/issues/501);
  amends [ADR-0014](0014-validation-evidence-for-slot-acceptance.md)
  decision 1

## Context

`Program::semantic_issues` reports completeness residuals: constructs the
parser or a producer preserved without a canonical catalog identity
(`UnknownAction`, `OpaqueAction`, `UnknownValue`, `RawSetting`). They are
structurally valid program nodes that downstream analysis must not treat as
fully understood.

`validate_canonical_ids` is fail-fast and rejects the first catalog-unknown
action or value call as `Unknown`. Wright composes structural validation
with canonical validation over raw Workshop source that legitimately
contains residuals (wright#501): the first residual aborted validation and
masked every later canonical violation — `Wait(sqrt(4)); Wait();` even
compiled and emitted an arity-invalid `Wait;`. A known canonically invalid
call nested inside a residual's own arguments was masked the same way.

[ADR-0014](0014-validation-evidence-for-slot-acceptance.md) decision 1
states "there is one validation contract" and rejected "a separate lenient
validation entry point" whose acceptance would be defined by what a source
implementation writes. Issue #372 poses a different question: whether a
catalog-unknown call must abort canonical validation for a consumer that
already reports the residual through the completeness channel.

## Decision

1. **One check set, two entry points.** Canonical validation keeps a single
   set of checks — identity, arity, enum domain, reference category, and
   slot acceptance under the ADR-0014 evidence rules — exposed through two
   public entry points that differ only in how they treat a catalog-unknown
   action or value call:
   - `validate_canonical_ids` (strict) rejects it as `Unknown`, unchanged.
   - `validate_canonical_ids_tolerating_residuals` treats it as opaque: the
     call's signature is unknowable and is not checked, but its argument
     values are still validated recursively and validation continues to
     later siblings. The first real violation still aborts.
2. **Residuals are never silently dropped.** The tolerant entry point is
   the contract for consumers that report residuals through
   `Program::semantic_issues`; strict remains the contract for producer
   artifacts and any consumer without residual reporting. Tolerating a
   residual without reporting it is non-conforming.
3. **Amendment of ADR-0014 decision 1.** "One validation contract" means
   one canonical check set under one ownership, not one entry point. The
   rejected lenient entry point stays rejected for provider-defined or
   evidence-free relaxations: residual tolerance relaxes no canonical check
   and hides no violation — strict rejects the residual in place, tolerant
   defers it to the completeness channel that reports it.
4. **Ownership.** The residual policy lives in `workshop-rs`: a consumer
   that strips or rewrites unknown constructs before validating
   reinterprets Workshop semantics and cannot validate known calls nested
   inside residual arguments.

## Rejected alternatives

- **Consumer-side stripping before strict validation.** Drops the
  residual's argument subtree, so a known violation nested inside it stays
  masked, and makes the consumer reinterpret Workshop semantics.
- **Strict validation only.** Keeps the masking defect: the first residual
  hides every later canonical violation for consumers that use both
  channels.
- **Tolerant behavior as the only contract.** Weakens the producer gate:
  emitted artifacts must not contain residuals, so strict rejection remains
  correct there.

## Consequences

- `docs/architecture/core-boundaries.md` records the two-entry-point
  contract as the current validation contract.
- `opy-rs` keeps the strict entry point for producer artifacts.
- Wright ([wright#518](https://github.com/wrightkit/wright/pull/518))
  consumes the tolerant entry point and maps diagnostics only; residuals
  stay visible through `Program::semantic_issues`.
