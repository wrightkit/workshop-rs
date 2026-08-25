# ADR-0002: Hero gameplay domain API and provenance boundary

## Status

Accepted for the `workshop-rs` hero/gameplay data track (#22–#25), pending
maintainer review.

## Context

`workshop-rs` already owns the canonical Workshop language catalog. Tooling
also needs reusable knowledge about hero kits and gameplay facts, but that
knowledge must not be redefined in Wright, OPY, DEL, LSP, MCP, or agent
consumers. Hero kits are not uniform: a logical slot can be absent, have one
entry, or contain several variant entries.

The Workshop catalog identity and the gameplay dataset identity have different
release and evidence lifecycles. Conflating them would make a balance-data
change look like a parser/catalog change and would make provenance ambiguous.

## Decisions

1. Hero, logical-slot, variant, role, stat-key, and unit identities are open
   string-backed newtypes. An ability's canonical reference is the tuple
   `AbilityRef { hero, slot, variant }`; there is no global ability identity
   derived from an English or localized display name. The crate provides
   typed references and constants for the current hero roster and common
   slots, but adding a future identity is a data change rather than a breaking
   enum change. `AbilityRef` rejects unknown wire fields so name-derived
   identity cannot be silently accepted.
2. `LogicalSlot` is a classification (`primaryFire`, `secondaryFire`,
   `ability1`, `ability2`, `ability3`, `ultimate`, `passive`), not a control,
   activation condition, or runtime state-machine model. Multiple abilities in
   one slot require distinguishable variants and ambiguous lookup returns an
   explicit error.
3. Raw gameplay records use `Hero`, `Ability`, `AbilityRef`, `Fact<T>`,
   `StatValue`, and `Quantity`. Ability localized/display names are metadata
   on the hero/slot/variant record and can change without changing its
   `AbilityRef`. Facts carry one or more `EvidenceRef` values; the enclosing
   `GameplayDatasetIdentity` carries dataset id, version, digest, source,
   license, target, and review status. Missing facts stay absent.
4. `GameplayDatasetIdentity` is separate from
   `crate::catalog::CatalogIdentity`. A gameplay-data update must not silently
   change Workshop parser/WIR/catalog identity.
5. Semantic calculations are a separate layer from raw records. The initial
   query/calculation layer may consume this model, but raw records do not
   embed Workshop-specific presentation or provider semantics.

## Topology evidence

The complete row-by-row survey is pinned in
[`docs/gameplay-roster-survey.md`](../gameplay-roster-survey.md). The
user-provided `workshop-data` export at commit
`d854bf01fc7bbf3b2169f67408c07a8da8989ad6` (commit date 2026-08-12) contains
hero identities and logical-slot records with localized/display names. Its topology categories
include:

| Shape | Evidence examples | API consequence |
| --- | --- | --- |
| Normal two-ability kit | Ana, Cassidy, Winston | `ability1`/`ability2` are ordinary data entries. |
| Extra logical ability | Brigitte, Hanzo, Wrecking Ball | `ability3` is a normal open slot, not a hero-specific field. |
| Form/configuration variant | Ramattra, Bastion, D.Va | multiple entries can share a slot with explicit variants. |
| Weapon/configuration entries | Mauga, Venture, Juno | `primaryFire` and `secondaryFire` are optional entries. |
| Passive-heavy kit | Echo, Mercy, Pharah, Juno | `passive` is optional and data-driven. |
| Data with absent fields | several export entries omit localized names or some slots | absence is represented explicitly; no synthetic values are inferred. |

The export is evidence for canonical Workshop-facing identity and naming, not
proof of every live-client balance value. Balance facts require their own
dataset evidence and remain absent when unsupported.

## Consequences

Consumers use typed accessors and deterministic catalog lookup rather than
parsing JSON or maintaining hero-specific schema branches. A normal new hero,
slot, keyword, or stat can be added in data. A new semantic concept that is
not representable by the open identity/value types requires an intentional API
review.

This ADR does not define a combat simulator, player controls, form-transition
rules, OPY/DEL syntax, or Wright integration.
