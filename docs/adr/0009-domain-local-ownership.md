# ADR-0009: Domain-local Workshop ownership and verification placement

- Status: Accepted
- Date: 2026-09-12
- Related:
  [Issue #149](https://github.com/wrightkit/workshop-rs/issues/149),
  [Issue #150](https://github.com/wrightkit/workshop-rs/issues/150),
  [Issue #152](https://github.com/wrightkit/workshop-rs/issues/152);
  implementation:
  [PR #151](https://github.com/wrightkit/workshop-rs/pull/151),
  [PR #154](https://github.com/wrightkit/workshop-rs/pull/154),
  [PR #155](https://github.com/wrightkit/workshop-rs/pull/155)

## Context

The crate originally grouped much of its implementation by compiler phase.
Parser and emitter modules consequently became homes for unrelated Workshop
features, while census, conformance, capture, and real-project support were
grouped under an `evidence` implementation domain. These groupings obscured
which Workshop concept owned behavior and incorrectly suggested that a
verification label was itself a semantic boundary.

## Decision

1. The default internal organization is Workshop domain/feature first, phase
   second. Actions, events, rules, values, settings, gameplay, catalog, and
   other durable Workshop concepts own the behavior that defines them.
2. Parsing, validation, emission, layout, and semantic queries live beside
   their owning feature when they are feature behavior. Shared parser/emitter
   contexts remain limited to token, catalog, span, presentation, and
   program-level orchestration state.
3. Operations that necessarily cross domains have one explicit shared home.
   Shared homes orchestrate and delegate; they do not become a phase-wide
   implementation bucket for every feature.
4. `evidence` is not a semantic implementation domain. Executable contract
   and regression tests belong with the feature they protect; provenance-
   linked fixtures and corpus data remain beside those tests; census, capture,
   conformance, and similar acquisition/reporting support belongs in tests,
   CLI tooling, or data-generation support according to its responsibility.
5. External expectations and implementation-derived regressions must remain
   distinguishable. Verification placement must not weaken the distinction or
   turn self-derived output into independent Workshop evidence.

The semantic-code versus declarative-data boundary remains the one established
by [ADR-0001](0001-catalog-boundaries.md). This ADR changes discoverability and
ownership placement, not the meaning of catalog facts or the authority of
typed semantic behavior.

## Consequences

Contributors can find a Workshop feature's parser, validator, emitter, and
related operations from the feature's domain. Shared contexts serve as small
orchestration boundaries, and verification code has a separate support role.
Tests and fixtures retain their evidence and provenance without requiring a
replacement verification abstraction.

This decision does not require one file per builtin, a new CST/AST/compiler
framework, or a repository-wide phase-to-domain rewrite.
