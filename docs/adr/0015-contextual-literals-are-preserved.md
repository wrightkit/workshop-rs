# ADR-0015: Contextual literal substitutions are accepted, not normalized

- Status: Accepted
- Date: 2026-09-26
- Related: [Issue #298](https://github.com/wrightkit/workshop-rs/issues/298),
  [Issue #297](https://github.com/wrightkit/workshop-rs/issues/297),
  [Issue #286](https://github.com/wrightkit/workshop-rs/issues/286);
  [ADR-0005](0005-seasonal-client-validation.md),
  [ADR-0008](0008-canonical-public-program-boundary.md),
  [ADR-0011](0011-contextual-semantic-placement.md),
  [ADR-0014](0014-validation-evidence-for-slot-acceptance.md)

## Context

[ADR-0011](0011-contextual-semantic-placement.md) decision 2 had the parser
rewrite an accepted contextual literal into one canonical value: `False` and
`True` into `0` and `1`, `0` and `Vector(0, 0, 0)` into `Null`, and
`Empty Array` into an empty string, wherever a catalog `paramCoercions` fact
allowed the substitution. Emission never wrote the authored form back, so
parsing Workshop text and emitting it again changed it:
`Set Move Speed(Event Player, False)` became `Set Move Speed(Event Player, 0)`.

The normalization was not needed for acceptance. Canonical validation already
accepts the authored forms at the same positions from the same catalog facts.
Its only effect was a single WIR form per value.

These substitutions exist to lower the Workshop element count. OverPy writes
them under `#!optimizeForSize` and charges a numeric literal argument one
element and `False`, `True`, or `Null` none. On a production project
(`OWBastion/Bastion`, 30096 elements in its `main` entry by OverPy's count), the
normalization removed about 500 `False`/`True` literals, which is about 500
elements if OverPy's accounting holds. The element limit is enforced by the
client, so a canonical form that erases the difference can make a program that
fit stop fitting.

The normalization also made results depend on locale: a spelling that resolved
to an identity without coercions (#297) kept its literal while the same action
in another locale lost it.

## Decision

1. Catalog coercion facts decide which literal substitutions are accepted at a
   parameter position. They do not rewrite the value. The parser keeps the
   literal as written, and `Program` and emission carry it unchanged. This
   replaces the normalization in ADR-0011 decision 2; the rest of ADR-0011 is
   unchanged.
2. Parsing and then emitting Workshop text preserves these literals in every
   locale. Two programs that differ only in such a literal are different
   programs for round-trip comparison, because their element counts may
   differ.
3. `workshop-rs` does not add an emission-side size optimization. A
   source-language implementation that wants the substituted form writes it
   into `Program`, which can now represent it; OverPy's `optimizeForSize`
   remains `opy-rs` policy.
4. The element cost of a numeric literal compared with `False`, `True`, or
   `Null` in the same position is not established by a client capture. The
   element-count model and its documentation state this gap rather than
   claiming the costs are equal. Resolving it is a capture under
   [ADR-0005](0005-seasonal-client-validation.md) and does not change this
   decision.

   Update: OverPy's per-action element counts agree with the client's total on
   a production project to 0.01%, and the model now charges the difference
   (see [element-count.md](../element-count.md)); a numeric literal costs one
   element more than `False`, `True`, or `Null` in the same position.

## Alternatives considered

- **Keep normalization and document that round trips are lossy.** Rejected:
  every consumer that passes Workshop text through `workshop-rs` would pay the
  element cost of the substituted literals, and raw Workshop users would have
  their text rewritten without a semantic reason.
- **Keep normalized WIR and add a selectable emission-side optimization.**
  Rejected: it places a source-language optimization policy in the Workshop
  emitter, and a provider that wrote the substituted form would need an emitter
  option to get back what it had already written. A Workshop-level size
  transformation can still be added later as a separate `Program` transform; it
  does not need normalized WIR.

## Consequences

- Parse then emit of a coerced literal is lossless, and the result no longer
  depends on which identity a locale spelling resolves to.
- Round-trip and WIR equivalence distinguish `0` from `False`, `1` from `True`,
  and `0` or `Vector(0, 0, 0)` from `Null` at coerced positions. A consumer that
  needs equality modulo substitution must ask for it explicitly; no such
  comparison exists yet.
- The element-count analysis sees the authored literal, so a cost difference
  can be modelled once client evidence establishes it.
