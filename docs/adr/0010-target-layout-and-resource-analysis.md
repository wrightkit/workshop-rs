# ADR-0010: Canonical Workshop target-layout and resource analysis

- Status: Accepted (backfilled)
- Date: 2026-09-12
- Related:
  [Issue #89](https://github.com/wrightkit/workshop-rs/issues/89),
  [Issue #102](https://github.com/wrightkit/workshop-rs/issues/102);
  implementation:
  [PR #93](https://github.com/wrightkit/workshop-rs/pull/93),
  [PR #103](https://github.com/wrightkit/workshop-rs/pull/103)

## Context

Element count and native action width are properties of the canonical
Workshop target, not of an OPY or DEL source AST. Before these APIs existed,
consumers had to estimate target cost or reproduce emitter expansion rules.
That duplication could diverge when canonical Workshop representation or
emission changed.

## Decision

1. `workshop-rs` owns reusable, source-language-neutral analysis of target
   layout and resource cost over the canonical public `Program` and catalog.
2. Element count is exposed as a structured report with total, per-rule, and
   attributable node contributions, provenance where available, and explicit
   incompleteness/errors for unknown or unsupported constructs. It is
   locale-independent and does not depend on formatted output.
3. Native action width is computed from the same authoritative recursive
   action expansion used by canonical emission. The public query reports
   structural headers and terminators and returns explicit errors for invalid
   programs or emission failures; it does not maintain a second synchronized
   width table.
4. Consumers may use these APIs for their own lowering, diagnostics, or
   budgets, but source-language control-flow policy, placement, thresholds,
   and user-facing compatibility behavior remain consumer-owned.
5. A documented or independently evidenced Workshop rule is required for an
   exact result. Unsupported or evidence-insufficient behavior must not be
   silently converted into a zero, partial, or guessed result.

## Consequences

Cost and layout calculations have one canonical Workshop owner and are stable
across locale conversion and textual formatting choices. Consumers can use
these results without maintaining duplicated emitter-width or cost models,
while retaining their own source-language semantics. The API does not claim
live-client/editor validation or implement a source-language debug directive.

The supported element-count rules and action-layout behavior are documented in
[`docs/element-count.md`](../element-count.md) and
[`docs/action-layout.md`](../action-layout.md); those documents describe
detailed contract behavior, while this ADR records the ownership decision.
