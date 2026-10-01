# Workshop client import limits

The Overwatch client enforces a global element budget of **32768** elements;
see [`element-count.md`](element-count.md) for the canonical counting model.
Beyond that budget, the client rejects some Workshop texts during import or
paste for structural reasons that are **not yet established**. This document
records what is known, which candidate limits have been falsified, the probe
corpus used to isolate the remaining candidates, and the protocol for turning
a client observation into an established boundary.

`workshop-rs` does not claim client importability. Validation, parsing, and
`Program::element_count` describe canonical Workshop structure only.

## Structural measures

`ElementCountReport` nodes expose four structural measures:

| Measure | Accessor | Meaning |
| --- | --- | --- |
| Element count | `node.count` | weighted client element cost of the subtree |
| Subtree cardinality | `node.node_count()` | plain number of report nodes in the subtree |
| Subtree depth | `node.height()` | longest downward path, in nodes |
| Statement value-tree size | `node.statement_value_nodes()` | value nodes in the statement's own argument/condition trees, excluding nested action subtrees |

`emitter::action_width` remains the measure for how many action slots a
native display action occupies. None of these is a limit; they are the
vocabulary in which limits are characterized.

## Motivating evidence

`OWBastion/Bastion` added eight large `Create In-World Text` statements in
one rule. The build produced at `8a4b5947d5762690ff88e148582f39be5d795ccf`
totals **31228** elements — below the global budget — and is the artifact
reported rejected by the client; `OWBastion/Bastion#198` records that the
project treats client import as the deciding test. The successor
`61efca001adf33cff6619bb4006398d615b17a36` rewrote the rule to per-player
texts. Offline comparison of the two builds (all sizes are the canonical
report measures — `count` / `node_count` / `height`):

| Build | Status | Total | Largest rule | Largest action subtree | Report height |
| --- | --- | ---: | ---: | ---: | ---: |
| `8a4b594` (rejected per project report) | reported rejected | 31228 | 3360 el / 2507 nodes | 422 el / 312 nodes, ~3.7 kB line | 16 |
| `61efca0`+ (shipped) | accepted | 27953 | 2008 el / 2209 nodes | ≤ 421 el / ≤ 315 nodes | 16 |
| `c010e1a` (vendored `bastion.ow`) | accepted | 29440 | 1444 el / 1453 nodes | 606 el / 500 nodes (`If` block) | 16 |

## Falsified and surviving candidates

Below-budget rejection cannot be explained by any property an accepted build
exceeds:

| Candidate property | Rejected build | Accepted builds | Verdict |
| --- | ---: | ---: | --- |
| Total element count | 31228 | ≤ 29440 | falsified for < 32768 totals |
| Subtree depth (report `height`) | 16 | 16 | falsified (identical) |
| Per-rule serialized size | 29 kB | 44 kB | falsified (accepted is larger) |
| Actions per rule | ≤ 505 | 505 | falsified (identical) |
| Per-action-subtree element count | 422 | 606 | falsified (accepted is larger) |
| Per-argument size | ≤ 155 el | 420 el (flat array arg) | falsified (accepted is larger) |
| Per-condition size | small | comparable | not implicated |
| Per-rule element/node total | 3360 / 2507 | ≤ 2042 / 2209 | **open** |
| Per-statement value-tree nodes | 311 | ≤ ~200 | **open** |
| Serialized line/statement bytes | ~3.7 kB | ≤ ~3.2 kB | **open** |

"Per-statement value-tree nodes" counts the value nodes inside one
statement's argument subtrees — `ElementCountNode::statement_value_nodes()`.
An `If`/`While` block's descendant *actions* are separate statements, so
block nesting does not inflate it. Accepted builds do carry larger `If`
blocks (606 elements / 500 nodes) than any statement in the rejected build,
which is why only the value-tree measure of a single statement remains
open.

## Real-world mitigations observed

Bastion's history contains two independent "flatten the giant nested
expression" corrections, neither of which isolated the client trigger:

- `OWBastion/Bastion#199` (`5051379`, fixing `#198`) split a monolithic
  nested `buildCandidatePool` eligibility expression — `Filtered Array`
  plus chained `And`/`Or` and indexing — into sequential small filter
  statements.
- `OWBastion/Bastion#289` (`61efca0`) reworked the prophet slot matrix into
  per-player texts: the largest rule shrank 3360 → 74 elements and the
  largest statement ~422 → ~60 elements (~3.7 kB → 0.6 kB line), with all
  inline `getPlayers().filter()` expansions replaced by a cached
  `prophetViewers` variable refreshed at 1 Hz.

Both fixes shrink every surviving candidate dimension at once (per-rule
totals, per-statement tree size, depth, serialized size), so they are
consistent with — but cannot discriminate between — the open candidates.
The shared mitigation pattern is hoisting repeated nested subexpressions
into a periodically refreshed variable; it removes the hotspot regardless
of which limit, if any, the client enforces.

## Corroborating community evidence

Independent of the Bastion case, the client has long had a per-rule
complexity limit distinct from the global element budget. The public record
is anecdotal and predates the current element system, so it establishes that
structural limits exist — not where their boundaries are:

- The client rejects oversized rules with a dedicated error,
  `Error: Action list is too complex`, reported as most often triggered by
  many `Skip If` actions (`Abort If` being cheaper). A community
  measurement table built by packing one action kind per rule found a
  `Skip If` carrying a small compare/index value tree capped at **14 per
  rule** while the same action reached 949 across a whole script — i.e. the
  per-rule budget scales with argument complexity, not statement count
  alone. Source: `us.forums.blizzard.com/en/overwatch/t/excessive-workshop-scriptload/368113`,
  post by Delwion-2667, 2019-08-03.
- The same table records a whole-script serialized size limit (e.g. 1817
  repetitions of a `Small Message` action before exceeding it) that was
  later relaxed; the current global budget is the element system described
  in `docs/element-count.md`. The per-rule capacities above are from that
  older regime and must not be read as current thresholds.
- A 2023 follow-up in the same thread independently attributes server-side
  failures to "expression/statement complexity in a single rule" —
  consistent direction, still anecdotal.

These observations keep the per-rule/per-statement candidates plausible
while the protocol below is required to establish any boundary.

## Probe corpus

`crates/workshop-rs/tests/fixtures/import-limits/` holds the offline probe
programs: a verbatim extraction of the reported-rejected rule, a single
verbatim statement, a shape-matched flat control, and parametric probes that
isolate per-rule totals, per-action argument-tree size, argument width,
argument depth, serialized statement bytes, and per-rule aggregate
value-tree load. Each probe stays
below the global budget; the fixture README carries the measured values and
the provenance of the extracted artifacts.

The probes pin the discriminating experiment:

- If `reproducer-bastion-prophet.ws` is rejected while
  `prophet-action-single.ws` and `prophet-rule-flat.ws` import, the trigger
  is per-rule scale, not statement size or depth.
- If `prophet-action-single.ws` alone is rejected, the trigger is inside one
  statement; `arg-nodes-large.ws` (511 nodes, depth 11),
  `arg-array-wide.ws` (1002 elements, depth 4), `arg-depth-deep.ws`
  (depth 52), and `arg-line-bytes.ws` (a 4.2 kB serialized line at only ~65
  value nodes) then separate value-tree size, element count, depth, and
  serialized statement bytes.
- If `rule-elements-large.ws` is rejected while `rule-elements-split.ws`
  imports, a per-rule budget exists somewhere in (882, 3522].
- `rule-value-trees.ws` reproduces the rejected rule's composite shape with
  generic content: one rule carrying eight statements of 311 value nodes
  each — the same per-statement size as the verbatim statements — for 2499
  rule nodes against the reproducer's 2507. If it is rejected alongside the
  reproducer, per-rule structural scale is implicated regardless of the
  specific actions or values involved; if it imports while the reproducer
  is rejected, the trigger is content-specific — the `createInWorldText`
  action kind, its player-set filter values, or its localized string
  arguments — rather than aggregate shape.

## Client test protocol

Client results are a manual evidence step (see
[`adr/0005-seasonal-client-validation.md`](adr/0005-seasonal-client-validation.md)):

1. Paste/import each probe in the Overwatch Workshop editor and record
   accept or reject, verbatim. Do not infer acceptance from a successful
   paste of a smaller program.
2. Record client version/season, locale, the fixture SHA-256, and the
   verbatim result for every probe, attached to the tracking issue.
3. On a boundary result, bisect with the generator flags
   (`--actions`, `--rules`, `--array-len`, `--depth`, `--leaves`,
   `--value-actions`, `--value-leaves`, `--string-count`, `--string-len`)
   rather than editing probes by hand, and attach the ladder results.
4. A candidate becomes an established limit only when an accept and a reject
   bracket it on the same client version. Only then may `workshop-rs` expose
   it as a validation contract.

Until such evidence exists, no threshold in this repository should be treated
as an import boundary.
