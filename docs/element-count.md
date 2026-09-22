# Canonical Workshop element count

`workshop-rs` exposes `Program::element_count(&Catalog)`, which analyzes the
canonical public `Program` and returns an `ElementCountReport`. The report contains
the total and one recursive node tree per rule. The report types are available
from the public `workshop_rs::actions` module.

`ElementCountReport::rules` and each `ElementCountNode::children` preserve
canonical source/WIR order. A node exposes its `kind`, canonical or analysis
`name`, opaque report-local `id`, optional authored `span`, node-local
`base_count` and signed `adjustment`, and recursive `count`. The `id` is unique
only within one report and is not a WIR or storage arena index. The recursive
count is the node's base count plus its adjustment and child counts. Values and
nested actions remain nodes in the tree instead of being reduced to an
aggregate total. Consumers should map nodes by their ordered tree position and
source span when one is available; report-local IDs have no meaning across
reports.

The model follows the documented Workshop element-count rules:

| Program component | Base cost |
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

The calculator is locale-independent: it reads canonical identities and
never emitted spellings. It validates the public program and catalog identities before
producing a report. Unknown, unsupported, invalid, or cyclic constructs return
`ElementCountError` instead of yielding a misleading exact total; no partial
report is returned. In
Native display actions such as `Create HUD Text` are counted through their
canonical catalog-backed action calls.

Element count is a static structural Workshop complexity measure. It is not an
estimate of runtime CPU cost or execution performance.

The independent behavioral source for the supported rules is the
[Workshop.codes element-count calculation reference](https://workshop.codes/wiki/articles/element-count-calculation).

Known evidence gap: this initial API does not claim live-client/editor
validation or source-language debug-count compatibility. Those belong to later
client-backed/consumer integration work after the canonical Program surface is
stable. The current real-project `rework.ow` fixture still stops in the parser
on an ambiguous bare `None` enum spelling, so it is not counted as a passing
real-program result until that independent parser gap is resolved.
