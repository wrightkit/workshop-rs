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

The model counts every occurrence of every component, at any nesting depth;
nothing is deduplicated. The base costs are the Workshop.codes reference
calibrated against the client-checked counts of pinned OverPy (see Evidence):

| Program component | Cost |
| --- | ---: |
| Rule, action, condition | 1 each |
| Value | 1 |
| Number literal (the value and its literal) | 2 |
| `False`, `True`, `Null` | 1 |
| Constant written as a value wrapping a literal: `Hero`, `Team`, `Color`, `Button`, `Map` | 2 |
| Any other enum choice (a wait behavior, a reevaluation mode, ...) | 1 |
| Comparison value outside a rule condition (its operator is a literal) | 2 |
| Array, Evaluate Once, localized/preset string | 2 |
| String literal | 1 |
| Read of a global or player variable (the value and the variable name) | 2 |
| Workshop setting value | 1, then -3 for integer and float, -2 for combo |
| `Else If`, `Else`, `End` | 1 each, as actions of their own |

Rule event parameters, action syntax parameters, comments, and custom game
settings do not contribute. A variable named as an argument (`Set Global
Variable At Index`, the chase actions) counts as that argument only; the player
and the name of a named player variable are two direct arguments. A direct
action or condition argument costs one less. For each pair of hero literals
anywhere below one direct argument, one element is added; pairs are counted per
argument, not across arguments. An argument a call leaves out still fills its
slot: the catalog default counts at its cost (the three replacement slots of
`Custom String` each count a `Null`).

`End` closes a block, with two exceptions. An `If` still open where a rule ends
is closed by the rule and its `End` is neither written nor counted; a `While`,
a `For`, and any block in a subroutine keep theirs. Disabling a rule, action,
or condition has no effect on the count.

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

## Evidence

The rules above were derived and checked against `#!debugElementCount` of pinned
OverPy 9.7.10, which reports an element count for every action and condition.
On the `main` entry of a production project (309 rules, 2867 leaf actions)
every action count and every rule count agrees, and the total is 30067, OverPy's
own figure. The client counted OverPy's build of that project at 30070, three
elements more; the cause of those three is not known. Small programs that
isolate one rule each, with OverPy's totals, are in
`tests/element_count.rs`.

The model is calibrated on one project. Constructs it does not exercise follow
the Workshop.codes reference, and the client has not been captured per
construct.

Numeric literals cost more than `False`, `True`, and `Null`, which is what
OverPy's `#!optimizeForSize` substitutions exploit
([ADR-0015](adr/0015-contextual-literals-are-preserved.md)). Parsing keeps the
authored literal, so the analysis sees the difference.

The analysis does not claim live-client/editor validation beyond the evidence
above. The current real-project `rework.ow` fixture still stops in the parser
on an ambiguous bare `None` enum spelling, so it is not counted as a passing
real-program result until that independent parser gap is resolved.
