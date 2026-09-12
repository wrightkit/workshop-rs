# ADR-0011: Contextual Workshop semantics at the catalog/code boundary

- Status: Accepted
- Date: 2026-09-12
- Related:
  [Issue #135](https://github.com/wrightkit/workshop-rs/issues/135);
  implementation:
  [PR #136](https://github.com/wrightkit/workshop-rs/pull/136)

## Context

Workshop parameters can accept context-dependent literal forms. Numeric and
boolean substitutions, `null`/vector replacements, comparison behavior, and
exceptions such as `Wait Until` cannot be represented correctly by a single
global nominal type rule. At the same time, the accepted catalog boundary in
[ADR-0001](0001-catalog-boundaries.md) requires factual Workshop metadata to
remain data-driven and locale-independent.

## Decision

1. The catalog may record independently evidenced, positional facts such as
   parameter domains, defaults, and literal coercion/replacement aliases.
   These facts are part of the canonical Workshop catalog and carry the
   repository's normal provenance.
2. Typed Rust parser, validator, and semantic code interprets those facts in
   the surrounding Workshop context and normalizes accepted literals into
   canonical WIR. Context-sensitive behavior is not delegated to a generic
   global coercion lattice or a provider-specific rule.
3. Only declared aliases and replacements are accepted. Similar-looking
   parameters do not inherit one another's behavior, and explicit exceptions
   remain distinct from ordinary contextual coercion.
4. Each materially distinct accepted or rejected branch is protected by
   executable Workshop contract or regression coverage at a valid structural
   context. Temporary source audits and comparison data are not a new
   long-lived evidence subsystem.

## Consequences

Catalog data remains the source of truth for declarative parameter facts, while
the semantic interpretation stays discoverable in typed Workshop behavior.
This permits a parameter-specific rule to be updated with provenance without
silently changing unrelated parameters or inventing a source-language type
system. The decision does not generalize OverPy or other provider behavior
beyond independently established Workshop semantics.
