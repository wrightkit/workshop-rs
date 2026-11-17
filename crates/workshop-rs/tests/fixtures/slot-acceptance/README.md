# Slot-acceptance probe fixtures

These fixtures record which values the Overwatch client accepts in a slot
whose declared type the value does not match.
[ADR-0014](../../../../../docs/adr/0014-validation-evidence-for-slot-acceptance.md)
rejects such a value until a client capture proves acceptance; these captures
are that evidence for `String Slice` and `Square Root` argument 1.

`tests/slot_acceptance.rs` checks that validation agrees with every recorded
client verdict, that each client export validates, and that each zh-CN probe
is the same program as its en-US source.

## Files

| File | Content |
| --- | --- |
| `round-1.ws`, `round-1.zh-CN.ow` | Round 1 probe; the zh-CN file is the text pasted into the client |
| `round-2.ws`, `round-2.zh-CN.ow` | Round 2 probe; same |
| `client-export-round-1.zh-CN.ow`, `client-export-round-2.zh-CN.ow` | The rule copied back out of the client after the accepted paste, unedited |
| `client-log-round-1.txt`, `client-log-round-2.txt` | The Inspector log of that run, unedited |

## Procedure

1. Paste the zh-CN probe into an empty Workshop.
2. If the client reports an incompatible value on a line, delete that line and
   paste again. Repeat until the client keeps the rule. Each deleted line is a
   rejected form.
3. Copy the rule back out of the client and save it unedited. A kept line is
   accepted only if its value is unchanged in the export.
4. Start the game and save the Inspector log through the `done` line.

## Results

zh-CN client, version 2.25.1.0-155284.1, 2026-11-16.

| Round | Form | Client |
| --- | --- | --- |
| 1 | `String Slice(Custom String("abcdef"), 1, 2)` (control) | accepted |
| 1 | `String Slice(100, 1, 2)` | rejected (line 15 reported) |
| 1 | `String Slice(Add(Modulo(Global.t, 60), 100), 1, 2)` | accepted |
| 1 | `String Slice(Vector(1, 2, 3), 1, 2)` | rejected (line 16 reported after line 15 was deleted) |
| 1 | `String Slice(Host Player, 1, 2)` | accepted |
| 1 | `Square Root(Host Player)` | accepted |
| 2 | `String Slice(Custom String("abcdef"), 1, 2)` (control) | accepted |
| 2 | `String Slice(Absolute Value(Global.t), 1, 2)` | rejected (deleted) |
| 2 | `String Slice(Vector(Global.t, 0, 0), 1, 2)` | rejected (deleted) |
| 2 | `String Slice(Add(100, 0), 1, 2)` | accepted |
| 2 | `String Slice(Global.t, 1, 2)` | accepted |
| 2 | `Square Root(Vector(1, 2, 3))` | rejected (deleted) |
| 2 | `Square Root(Custom String("a"))` | rejected (deleted) |

Every accepted value is unchanged in the export; the client only reformats
`Add`/`Modulo` as infix operators and drops trailing `Null` arguments. The
logged values match the unchanged values: `107` sliced gives `07`, `100` gives
`00`, and `"7"` gives an empty string.

The catalog records the accepted forms at the probed positions only:
`Player` joins the declared type of both positions, following the existing
Player positional unions, and `stringSlice` argument 1 lists `add` in
`paramAcceptedValues`. A type union cannot express the `add` result: the
client rejects a Number literal and an `Absolute Value` result in the same
slot. Why the client accepts `add` results is not established.

In round 2 the client exported the final `Custom String("SB done")` as
`Custom String("")` and logged an empty line; the round 1 `SA done` line was
unchanged. The cause is not established.
