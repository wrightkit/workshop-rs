# ADR-0006: Canonical typed Workshop settings semantics

## Status

Accepted for the #109 foundation and #110 canonical catalog projection.
Query and edit API ergonomics are outside this decision.

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
hero is `Unknown`. For hero-ability targets, gameplay kit topology is checked
first; missing slots or variants are `NotApplicable`. When topology is valid
but explicit applicability evidence is absent, the target is `Unknown`. Locale
label quality never changes applicability. `gamemodes.general` is a literal
Workshop settings group and therefore has a global/no semantic target, not a
mode target.

`SettingValueDomain` records the value kind and an optional effective numeric
range. Numeric or percent bounds without independent evidence are explicitly
unknown (`None`). Unknown bounds do not produce an effective value. A partially
known range only produces an effective value when the known bound necessarily
clamps the authored value; otherwise the result remains unknown. Validated
`NumericBounds` supports evidenced Workshop clamping while preserving the
authored value in `EffectiveNumber`; the source-preserving
`SettingsNode::Number` is not changed by this schema.

Locale names are presentation metadata resolved through the generated
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
semantic identity when no canonical typed identity has been established.

## Consequences

The `TableEntry` inventory is the parser/emitter source and the schema is its
single typed semantic projection, avoiding a parallel settings
framework. The effective catalog preserves the established hand-authored
parser precedence while collapsing duplicate paths from the generated
projection. It validates unresolved scopes, missing identities, missing
presentation, and conflicting domains before catalog-check success. Canonical
concept identities normalize reusable hero/ability settings
without embedding localized ability display names; mode-specific enum concepts
remain distinct when their reviewed domains differ. Query/edit operations can
use definitions and source-preserving occurrences without inventing another
identity, scope, target, domain, or provenance model.

No UI step metadata, source-language carrier parsing, per-hero Rust structs,
consumer-side applicability hacks, or query/edit API ergonomics are part of this
contract.
