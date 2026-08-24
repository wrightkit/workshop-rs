# Canonical action layout

`workshop_rs::emitter::action_width` reports the number of native Workshop
action lines produced by a WIR action sequence. The result includes structural
headers and terminators, so a structured action can have a width greater than
one.

The query validates the complete WIR first and then expands the requested
sequence through the same authoritative action implementation used by
`emitter::emit`. Consumers do not need to reproduce emitter expansion rules or
inspect emitted text. Each requested action is treated as non-rule-final; this
is the action-stream contract for calculating relative native action offsets.

The query is source-language-neutral. Source-language lowering, target
placement, and control-flow policy remain in the consuming compiler.

Malformed WIR and emission failures are returned as
`ActionLayoutError::InvalidWIR` or `ActionLayoutError::Emission`; they are
never converted into a zero or partial width.
