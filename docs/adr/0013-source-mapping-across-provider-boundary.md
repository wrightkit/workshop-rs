# ADR-0013: Source mapping across the provider boundary

- Status: Accepted
- Date: 2026-09-24
- Related: workshop-rs #271, #252, #187, #178; wrightkit/wright #246; ADR-0008

## Context

Source-language implementations lower into the public `Program` and attach
authored spans through `Program::set_rule_span`, `set_condition_span`,
`set_action_span`, and `set_action_argument_span`. Wright's canonical analyzer
already reads the same public span accessors. Across a provider process
boundary, however, only Workshop text crosses: the provider artifact format
`workshop-rs/text-v1` is used by consumers but not defined by this repository,
and Wright therefore reports all Wright-owned evidence for provider-backed
sources as unmapped.

The current span model is a private side table indexed by public rule,
condition, action, and direct-argument position. `Rule::actions` is a linear
stream that includes explicit control-flow lines, so these positions are
stable across deterministic emission and re-parsing. The model can silently
desynchronize if a consumer inserts or removes public elements after spans
are attached.

## Decision

1. **Identity model.** The 1.x contract keeps position-indexed source mapping.
   Stable node identities and inline spans on public nodes are rejected: the
   former conflicts with the public construction API (#178) and the latter
   cannot be carried by public enum variants.
2. **Shape guard.** Attached mappings record the program shape they were
   attached to (rule count, per-rule condition and action counts, and the
   variable and subroutine declaration counts). Span
   accessors return `None` when the current shape differs, instead of
   returning a displaced span.
3. **Granularity.** The durable mapping granularity is rule, condition, action,
   direct action argument, and declarations. Nested value mappings may be added
   later as additive API.
4. **Mapping target.** A mapping is a `workshop-rs` `Span` into a file table
   attached to the program. Nodes without an authored origin carry no span;
   consumers report evidence on them as unmapped. Source-language
   implementations own attribution policy for their constructs (includes,
   macro expansion, generated helpers).
5. **Canonical artifact formats.** This repository defines the canonical
   Workshop artifact formats:
   - `workshop-rs/text-v1`: Workshop text.
   - `workshop-rs/mapped-text-v1`: Workshop text plus a file table, the
     program shape, and position-keyed spans.
   A public `SourceMap` type extracts a mapping from a span-bearing `Program`
   and applies it to a `Program` parsed from the same text, rejecting the whole
   mapping on shape mismatch.
6. **Transport.** Provider protocols carry these formats as opaque artifacts.
   Format negotiation is a protocol concern and does not give the protocol
   Workshop semantics.
7. **1.0 scope.** Only decision 1 constrains the 1.0 API freeze. Decisions 2–5
   are additive and may land in 1.x. Before 1.0, `Rule` and `Condition` become
   `#[non_exhaustive]` so future mapping or metadata fields remain non-breaking.

## Rejected alternatives

- Serializing the complete `Program` as a wire format: freezes the whole public
  model as a transport contract to deliver a small mapping.
- Source-language-owned opaque location references: the existing `Span` plus a
  file table covers includes and multi-file projects; opaque references would
  prevent Workshop-level consumers from interpreting locations.

## Consequences

- Wright can attribute canonical findings to authored source without owning
  source-language semantics.
- Consumers that mutate program shape lose mappings instead of reporting wrong
  locations.
- Column units follow `Position` (1-based Unicode scalar values); producers
  must convert, and editor presentation converts to UTF-16.
