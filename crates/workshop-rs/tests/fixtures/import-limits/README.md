# Client-import probe fixtures

These fixtures are the offline probe corpus for characterizing Overwatch
Workshop client import limits beyond the global 32768-element budget. Each
file is a complete raw en-US Workshop program that isolates one structural
dimension. The measurement table, current findings, and the client-test
protocol live in [`docs/import-limits.md`](../../../../../docs/import-limits.md).

Passing these probes through `workshop-rs` is verified by
`tests/import_limits.rs`. Passing them through the Overwatch client is a
manual evidence step and is **not** asserted anywhere in this repository.

## Probe inventory

| Fixture | Isolated dimension | Total | Largest rule | Largest action | Deepest subtree |
| --- | --- | --- | --- | --- | --- |
| `control-minimal.ws` | paste mechanics sanity | 4 | 4 | 2 | 3 nodes |
| `arg-array-wide.ws` | one wide flat argument (500-item `Array`) | 1004 | 1004 | 1002 | 4 nodes |
| `arg-depth-deep.ws` | one deep argument (48-deep `If-Then-Else` chain) | 436 | 436 | 434 | 52 nodes |
| `arg-line-bytes.ws` | one action whose serialized argument line is ~4.2 kB at a 66-node subtree | 164 | 164 | 162 | 5 nodes |
| `arg-nodes-large.ws` | one action whose argument is a 511-node balanced `And` tree | 513 | 513 | 511 | 11 nodes |
| `prophet-action-single.ws` | one verbatim large `createInWorldText` statement | 406 | 406 | 398 | 14 nodes |
| `prophet-rule-flat.ws` | reproducer rule skeleton with flat arguments (control) | 48 | 48 | 5 | 5 nodes |
| `reproducer-bastion-prophet.ws` | the verbatim rule reported rejected below the budget | 3360 | 3360 | 422 | 14 nodes |
| `rule-elements-large.ws` | per-rule element total (~3.5k in one rule, small actions) | 3522 | 3522 | 22 | 4 nodes |
| `rule-elements-split.ws` | same total spread over four rules (control) | 3528 | 882 | 22 | 4 nodes |
| `rule-value-trees.ws` | one rule aggregating eight 311-node statement value trees (generic twin of the reproducer shape) | 2490 | 2490 | 311 | 11 nodes |

"Total"/"largest rule"/"largest action" are canonical element counts;
"deepest subtree" is `ElementCountNode::height`.

## Provenance

- `reproducer-bastion-prophet.ws` — the rule
  `"[Event/先知] Create per-slot prophet next-event texts"` extracted
  verbatim (lines 4768–4787) from `build/8a4b594-en.ow`, the Workshop
  artifact built from `OWBastion/Bastion` commit
  `8a4b5947d5762690ff88e148582f39be5d795ccf` (en-US). That build totals
  31228 elements, below the global budget, and is the artifact reported
  rejected by the client; its successor `61efca001adf33cff6619bb4006398d615b17a36`
  rewrote this rule. The minimal `variables { }` block declares only the
  variables the rule references, renumbered from the original indices
  (references are by name). Fixture SHA-256:
  `3a1cb89148242ca2ebe40d611ff033ea315b01d8246bde32b5f57c94b584c922`.
- `prophet-action-single.ws` — the same wrapper and first
  `createInWorldText` statement (slot 0) from the same build artifact.
  Fixture SHA-256:
  `b14749af54e35a84a828dd22d887d968194b0f7af82eed47cf1575f2b6cdb04f`.
- All other files are generated deterministically by
  [`tools/import-limits/generate_probes.py`](../../../../../tools/import-limits/generate_probes.py).
  The tool accepts `--actions`, `--rules`, `--array-len`, `--depth`,
  `--leaves`, `--value-actions`, `--value-leaves`, `--string-count`, and
  `--string-len` so a client result can drive a bisection ladder without new
  code.

Regenerate the parametric fixtures from the repository root:

```sh
python3 tools/import-limits/generate_probes.py
```

## Known client outcomes

None recorded yet. Results must be attached to the issue with client
version, locale, artifact SHA-256, and the per-fixture accept/reject
outcome before any boundary is treated as established.
