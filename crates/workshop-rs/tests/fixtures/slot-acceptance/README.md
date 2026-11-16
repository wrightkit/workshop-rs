# Slot-acceptance probe fixtures

These fixtures ask the Overwatch client one question each: does it accept a
value whose semantic type differs from the declared parameter type?
[ADR-0014](../../../../../docs/adr/0014-validation-evidence-for-slot-acceptance.md)
rejects such a value until acceptance evidence exists. A client capture of
one of these probes is that evidence.

Each file is a complete raw en-US Workshop program. `combined.ws` holds every
form in one rule and logs each value to the Inspector; the other files hold
one form each, so a rejected paste points at a single form.
`tests/slot_acceptance.rs` pins the current `workshop-rs` result: the control
validates and every other probe is rejected. Passing the probes through the client is a manual step and is
**not** asserted anywhere in this repository.

## Probe inventory

| Fixture | Parameter | Value under test | Origin |
| --- | --- | --- | --- |
| `combined.ws` | all rows below | logs `SA1`–`SA6`, then `SA done` | — |
| `control-string-slice-string.ws` | `String Slice` argument 1 | `Custom String` (declared type, control) | — |
| `string-slice-number-literal.ws` | `String Slice` argument 1 | Number literal | — |
| `string-slice-number-expression.ws` | `String Slice` argument 1 | Number expression | OverPy 9.7.17 output for `(t % 60 + 100).substring(1, 2)`, the zero-padding form in `ballhallaa/ballkour` |
| `string-slice-vector.ws` | `String Slice` argument 1 | Vector | — |
| `string-slice-player.ws` | `String Slice` argument 1 | Player | — |
| `square-root-player.ws` | `Square Root` argument 1 | Player | OverPy 9.7.17 output for `sqrt(hostPlayer)` |

## Client test

Follow the client test protocol in
[`docs/import-limits.md`](../../../../../docs/import-limits.md#client-test-protocol),
with these result rules:

1. Paste `combined.ws` into an empty Workshop. The client checks the whole
   paste, so one rejected form rejects all of them.
   - If the client keeps the rule, copy the Workshop text back out unedited,
     start the game, and save the Inspector log through the `SA done` line.
     Each form whose exported value is unchanged is accepted; the log shows
     the value it produces. Stop here.
   - If the client refuses the paste, continue with step 2.
2. Paste `control-string-slice-string.ws`. If the client rejects it, stop: the
   paste mechanics are wrong, and no other result is valid.
3. Paste each single-form probe alone into an empty Workshop. Record the
   client version, locale, the fixture SHA-256, and one result:
   - **reject**: the client refuses the paste.
   - **accept**: the client keeps the rule. Copy the Workshop text back out of
     the client and save it unedited. The value is accepted only if the
     exported action holds the same value.
   - **rewritten**: the client keeps the rule but the exported action holds a
     different value. Record the exported value; this is not acceptance of the
     pasted form.
4. Attach the results and exported texts to the tracking Issue.

An **accept** becomes a positional catalog fact for that parameter and value
type, and the probe moves from the rejected list in `tests/slot_acceptance.rs`
to the accepted side. A **reject** confirms the presumption and changes
nothing.
