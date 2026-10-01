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

`ElementCountReport` nodes expose three orthogonal structural measures:

| Measure | Accessor | Meaning |
| --- | --- | --- |
| Element count | `node.count` | weighted client element cost of the subtree |
| Subtree cardinality | `node.node_count()` | plain number of report nodes in the subtree |
| Subtree depth | `node.height()` | longest downward path, in nodes |

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
texts. Offline comparison of the two builds:

| Build | Status | Total | Largest rule | Largest action | Depth |
| --- | --- | ---: | ---: | ---: | ---: |
| `8a4b594` (rejected per project report) | reported rejected | 31228 | 3360 el / 2507 nodes | 422 el / 312 nodes, ~3.7 kB line | 14 |
| `61efca0`+ (shipped) | accepted | 27944 | 2042 el / 1661 nodes | 421 el / ~200 nodes | 14 |
| `c010e1a` (vendored `bastion.ow`) | accepted | 29440 | 1444 el | 606 el (`If` block) | 14 |

## Falsified and surviving candidates

Below-budget rejection cannot be explained by any property an accepted build
exceeds:

| Candidate property | Rejected build | Accepted builds | Verdict |
| --- | ---: | ---: | --- |
| Total element count | 31228 | ≤ 29440 | falsified for < 32768 totals |
| Value-expression depth | 14 | 14 | falsified (identical) |
| Per-rule serialized size | 29 kB | 44 kB | falsified (accepted is larger) |
| Actions per rule | ≤ 505 | 505 | falsified (identical) |
| Per-action element count | 422 | 606 | falsified (accepted is larger) |
| Per-condition size | small | comparable | not implicated |
| Per-rule element/node total | 3360 / 2507 | ≤ 2042 / 1661 | **open** |
| Per-action value-tree nodes | 312 | ≤ ~200 | **open** |
| Serialized line/statement bytes | ~3.7 kB | ≤ ~3.2 kB | **open** |

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

## Probe corpus

`crates/workshop-rs/tests/fixtures/import-limits/` holds the offline probe
programs: a verbatim extraction of the reported-rejected rule, a single
verbatim statement, a shape-matched flat control, and parametric probes that
isolate per-rule totals, per-action argument-tree size, argument width, and
argument depth. Each probe stays below the global budget; the fixture README
carries the measured values and the provenance of the extracted artifacts.

The probes pin the discriminating experiment:

- If `reproducer-bastion-prophet.ws` is rejected while
  `prophet-action-single.ws` and `prophet-rule-flat.ws` import, the trigger
  is per-rule scale, not statement size or depth.
- If `prophet-action-single.ws` alone is rejected, the trigger is inside one
  statement; `arg-nodes-large.ws` (511 nodes, depth 11),
  `arg-array-wide.ws` (1002 elements, depth 4), and `arg-depth-deep.ws`
  (depth 52) then separate value-tree size, element count, and depth.
- If `rule-elements-large.ws` is rejected while `rule-elements-split.ws`
  imports, a per-rule budget exists somewhere in (882, 3522].

## Client test protocol

Client results are a manual evidence step (see
[`adr/0005-seasonal-client-validation.md`](adr/0005-seasonal-client-validation.md)):

1. Paste/import each probe in the Overwatch Workshop editor and record
   accept or reject, verbatim. Do not infer acceptance from a successful
   paste of a smaller program.
2. Record client version/season, locale, the fixture SHA-256, and the
   verbatim result for every probe, attached to the tracking issue.
3. On a boundary result, bisect with the generator flags
   (`--actions`, `--rules`, `--array-len`, `--depth`, `--leaves`) rather than
   editing probes by hand, and attach the ladder results.
4. A candidate becomes an established limit only when an accept and a reject
   bracket it on the same client version. Only then may `workshop-rs` expose
   it as a validation contract.

Until such evidence exists, no threshold in this repository should be treated
as an import boundary.
