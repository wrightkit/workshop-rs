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

The model follows the Workshop element-count rules and the pinned OverPy 9.7.10
element annotations:

| Program component | Base cost |
| --- | ---: |
| Rule | 1 |
| Action | 1 |
| Condition | 1 |
| Ordinary value, boolean, null, or direct enum literal | 1 |
| Global-variable read | 2 |
| Wrapper-backed enum (`Button`, `Color`, `Gamemode`, `Hero`, `Map`, `Team`) | 2 |
| Number literal | 2 |
| Player-variable access | 2 plus its player expression |
| Array | 2 |
| Workshop Setting Integer or Real | 1, then subtract 3 |
| Workshop Setting Combo | 1, then subtract 2 |
| Workshop Setting Toggle | 1 |
| Evaluate Once | 2 |
| Localized/preset string | 2 |

`Custom String` uses the compiler's four argument slots. Its base cost is one,
plus one for each unused trailing slot, in addition to the text and supplied
value arguments. For example, `Custom String("abc")` costs five elements.

Rule event parameters, action syntax parameters such as a variable name or
modify operator, comments, and custom game settings do not contribute. In a
catalog action's variable target parameter, the variable name is syntax; a
`Player Variable` target still counts its player expression as a direct action
argument. A direct action or condition argument is reduced by one. A comparison
value counts its operator syntax as a second base element. `Else If`, `Else`,
and `End` control markers each cost one element.
For each pair of hero literals anywhere below the direct arguments of one
action or condition, one element is added. Disabling a rule, action, or
condition has no effect.

The calculator is locale-independent: it reads canonical identities and
never emitted spellings. It validates the public program and catalog identities before
producing a report. Unknown, unsupported, invalid, or cyclic constructs return
`ElementCountError` instead of yielding a misleading exact total; no partial
report is returned. Native display actions such as `Create HUD Text` are counted through their
canonical catalog-backed action calls.

Element count is a static structural Workshop complexity measure. It is not an
estimate of runtime CPU cost or execution performance.

The general rules follow the
[Workshop.codes element-count calculation reference](https://workshop.codes/wiki/articles/element-count-calculation).
Construct-specific lowering is informed by pinned
[OverPy 9.7.10 element-count code](https://github.com/Zezombye/overpy/blob/v9.7.10/src/compiler/astToWorkshop.ts)
and checked against its per-rule and per-action annotations. The integration
tests preserve three small real-project rules with exact OverPy counts (4, 5,
and 6 elements), as well as representative numeric, variable, control-flow,
and setting expressions. Those checks establish compiler agreement for the
tested constructs, not independent client costs for each construct.

For the Bastion OverPy build, the client capture is 30,070 elements, OverPy
reports 30,067, and this model reports 30,095 across 309 rules. The model is
within the stated 1% aggregate tolerance of the client (25 elements, about
0.08%). The client capture establishes the aggregate target; it does not
isolate individual construct costs.

This API counts the canonical program representation. Source-language debug
counts remain compiler-specific, and the Bastion comparison is evidence for
that real project and pinned compiler version rather than every possible
client/editor context. The current real-project `rework.ow` fixture still
stops in the parser on an ambiguous bare `None` enum spelling, so it is not
counted as a passing real-program result until that independent parser gap is
resolved.
