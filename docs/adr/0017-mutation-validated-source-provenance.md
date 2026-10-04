# ADR-0017: Mutation-validated source provenance records

- Status: Accepted
- Date: 2026-10-30
- Related:
  [Issue #364](https://github.com/wrightkit/workshop-rs/issues/364);
  amends [ADR-0013](0013-source-mapping-across-provider-boundary.md)
  decision 2

## Context

ADR-0013 kept position-indexed source mapping behind a shape guard: private
provenance tables record the public list counts they were attached to, and
span accessors return `None` when a count differs. Issue #364 showed the
guard was incomplete twice over. Structurally, inserting into a public list
hid displaced rows only until a span setter resized the table back to the
public length, at which point the stale row reappeared on the wrong node.
Semantically, equal-length mutations — swapping two positions, replacing a
node, or removing and re-inserting a different node — kept every recorded
mapping visible because the counts still matched.

The issue required an explicit owner decision for equal-length mutation:
drop mappings on any recorded drift, or validate mapping content.

## Decision

1. **Position plus content.** A mapping record is valid while its position
   still exists in the public model and the node there still has the
   content the record was attached to. Every provenance record carries a
   process-local content digest; accessors return `None` when either check
   fails. Removing and re-inserting the same content restores its mapping;
   swaps, replacements, and different re-inserted content stay unmapped.
2. **Subtree identity for expression nodes.** A condition, action, or value
   record's identity covers the node's whole expression subtree, because the
   node's authored span covers that whole text. Mutating a nested value
   hides the enclosing argument, action, and condition spans. Rules and
   declarations keep the digest at their own scalar fields; their condition
   and action tables remain positional tables screened by count.
3. **Granularity of descent.** Hiding is per level: a record's own fields
   are gated by its identity, while descending into its positional child
   table requires only that level's count to match. A node that went stale
   loses its own span without hiding unaffected sibling or child mappings.
4. **Setter reseating.** Every `set_*` span setter bounds-checks the public
   position, resizes the table to the current shape, clears the record's own
   fields when the node content changed, records the current identity, and
   writes the span. Resize alone can never make a stale row visible again.
5. **Identity scope.** Digests are process-local bookkeeping for record
   validity; they are never serialized into `SourceMap` artifacts. The
   wire contract stays position-keyed spans plus shape, as ADR-0013 decided.

## Rejected alternatives

- **Shape-only guard plus setter table clearing.** Fixes the reported
  resurrection but leaves equal-length mutations returning stale spans, and
  cannot express "same content re-inserted keeps its mapping".
- **Stable node identities or inline spans.** Already rejected by ADR-0013
  decision 1: both conflict with the public construction API.

## Consequences

- Public `Vec` mutations can no longer expose displaced or stale spans
  through any accessor, `SourceMap::extract`, or `to_wir` emission.
- Consumers that mutate a program keep mappings for every node whose
  position, table shape, and content are unchanged; they lose — rather than
  corrupt — the rest.
- Equal-length mutation semantics are defined: only content that returns to
  a recorded identity is mapped again.
- The `mapped-text-v1` artifact format is unchanged.
