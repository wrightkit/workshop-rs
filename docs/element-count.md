# Canonical Workshop element count

`workshop-rs` exposes `Program::element_count(&Catalog)`, which analyzes the
canonical WIR program and returns an `ElementCountReport`. The report contains
the total, one recursive node tree per rule, and per-node WIR arena indexes,
source spans, base costs, adjustments, and final subtree costs.

The model follows the documented Workshop element-count rules:

| WIR component | Base cost |
| --- | ---: |
| Rule | 1 |
| Action | 1 |
| Condition | 1 |
| Ordinary value or literal | 1 |
| Array | 2 |
| Workshop setting value (`Workshop Setting ...`) | 2 |
| Evaluate Once | 2 |
| Localized/preset string | 2 |

Rule event parameters, action syntax parameters such as a variable name or
modify operator, comments, and custom game settings do not contribute. A
direct action or condition argument is reduced by one. For each pair of hero
literals anywhere below the direct arguments of one action or condition, one
element is added. Disabling a rule, action, or condition has no effect.

The calculator is locale-independent: it reads canonical WIR identities and
never emitted spellings. It validates WIR and catalog identities before
producing a report. Unknown or unsupported constructs return
`ElementCountError` instead of yielding a misleading exact total. In
particular, the current `Debug` and `Print` nodes are presentation helpers that
the emitter expands into `Create HUD Text`; callers should count that canonical
action until a reviewed expansion contract is added.

The independent behavioral source for the supported rules is the
[Workshop.codes element-count calculation reference](https://workshop.codes/wiki/articles/element-count-calculation).

Known evidence gap: this initial API does not claim live-client/editor
validation, source-language debug-count compatibility, or an exact count for
presentation-only WIR helpers. Those belong to later
client-backed/consumer integration work after the canonical WIR surface is
stable. The current real-project `rework.ow` fixture still stops in the parser
on an ambiguous bare `None` enum spelling, so it is not counted as a passing
real-program result until that independent parser gap is resolved.
