# ADR-0006: Canonical typed Workshop settings semantics

## Status

Accepted for the #109 foundation; full catalog population remains #110 and
ergonomic query/edit APIs remain #111.

## Decision

`workshop-rs` exposes typed setting facts through `settings::schema`.
`SettingId` is an open, locale-independent identity for a Workshop setting
concept. A concrete hero or ability display label is never required in that
identity; hero and logical ability-slot information is represented by
`SettingTarget` and effective applicability.

`SettingScope` follows the existing Workshop settings sections: `Main`,
`Lobby`, `GameModes`, `Heroes`, `Extensions`, and `Workshop`. `SettingTarget`
is separate and can represent global, mode, team, hero, and hero-plus-logical
ability-slot targets, including team/common hero ability slots. The table's
wildcard hero entries are projected as definitions whose applicability is
resolved against gameplay topology and explicit applicability evidence. A
known hero without explicit applicability evidence is `Unknown`; an unknown
hero is `Unknown`. Gameplay kit topology is checked first for hero-ability targets;
missing slots or variants are `NotApplicable`. The current settings projection
has no complete independent applicability matrix, so a topology-valid target
without explicit applicability evidence remains `Unknown`. Locale label quality
never changes applicability. `gamemodes.general` is a literal
Workshop settings group and therefore has a global/no semantic target, not a
mode target.

`SettingValueDomain` records the value kind and an optional effective numeric
range. The current reviewed table does not contain enough independent evidence
to assign numeric or percent bounds to its entries, so those bounds are
explicitly unknown (`None`) until #110 or a separately reviewed evidence update
establishes them. Unknown bounds do not produce an effective value. A partially
known range only produces an effective value when the known bound necessarily
clamps the authored value; otherwise the result remains unknown. Validated
`NumericBounds` supports evidenced Workshop clamping while preserving the
authored value in `EffectiveNumber`; the source-preserving
`SettingsNode::Number` remains unchanged.

Locale names are presentation metadata resolved through the existing generated
locale projection, with the primary `en-US` spelling retained directly. Each
definition reports whether its evidence comes from pinned raw Workshop
fixtures or the reviewed `workshop-data` export. Unknown/raw settings continue
to be carried by `SettingsNode::Raw`; the schema does not turn missing evidence
into a guessed definition.

`SettingIdentity::Known(SettingId)` means that a reviewed canonical concept
identity has been resolved; unresolved projected hero-ability concepts use
`SettingIdentity::Unknown` and `id() == None`. This is independent from
`SettingProvenance`: the latter reports whether the underlying table or export
evidence was reviewed, so reviewed source evidence may still carry an unknown
semantic identity until #110 supplies the canonical typed catalog.

## Consequences

The existing `TableEntry` inventory remains the parser/emitter source and the
schema is a single semantic projection of it, avoiding a parallel settings
framework. #110 can replace or extend the projection with generated catalog
data without changing the public semantic boundary. #111 can build query/edit
operations on definitions and source-preserving occurrences without inventing
another identity, scope, target, domain, or provenance model.

No UI step metadata, source-language carrier parsing, per-hero Rust structs, or
consumer-side applicability hacks are part of this contract.
