# Hero gameplay dataset

`crates/workshop-rs/src/gameplay/gameplay.json` is the embedded gameplay dataset
for the `workshop-rs` hero domain. `workshop_rs::gameplay::data::builtin()`
loads it through the same validation path as external JSON and verifies its
content digest before constructing a `GameplayCatalog`.

## Schema and identity

The file has `schemaVersion: 2`, a `GameplayDatasetIdentity`, and a sorted
hero list. Heroes and abilities carry source references. Hero, slot, and
variant identities are open strings; an ability is canonically referenced by
its hero, logical slot, and optional hero-local variant. Display names are
metadata, not identity. The current dataset uses the seven canonical slots from ADR-0002;
the open string-backed API also accepts future non-empty slot identities.
Multiple abilities in one slot must have distinct, non-empty variants.
Version 1 payloads used the incompatible `evidence` record key and are rejected
before version-specific records are deserialized.

The dataset digest is `sha256:` followed by the SHA-256 of the canonical JSON
content with `identity.digest` removed. Object keys are sorted before hashing,
so formatting and input key order do not change the identity. The loader also
sorts heroes and abilities before building lookup indexes.

The committed dataset identity is version `2026-10-06` with digest
`sha256:0f80dcfbf4014ffe68d05ffba471d26e221ca6789407af6dc4c7eb601e284a87`.

## Source records and known gaps

The current dataset is an identity/naming and kit-topology dataset containing
hero identities, role facts, and named ability slots declared by the pinned
export. It also contains official-detail variant records for Bastion, D.Va,
and Ramattra. The embedded projection includes keyword-bearing abilities and
variants. Every hero and ability name is linked to
`workshop-data/workshop-data.json` at commit
`5a7d0e294b8cad73b9701987bb584d0551d7fa4d` (commit date 2026-10-06).
One hero, Doctrine (`doctrine`, support), is absent from that export and is
recorded with official-source attribution only: its hero identity, role,
ability-slot order, and localized names come from the official localized hero
detail pages and the official Overwatch China hero detail, accessed
2026-10-08. The canonical catalog carries the same boundary: `Hero.DOCTRINE`
is an official-source-only member recorded as an unmatched exclusion in the
corpus manifest. zh-CN names absent from the
export (D.Mon's kit and hero name, the Bastion/D.Va/Ramattra variant
primaries, and Sojourn's Charged Shot) are likewise filled from the official
Overwatch China hero detail pages and patch notes, accessed 2026-10-08.

Role facts are independently sourced from the official Blizzard hero-detail
URL recorded on each role fact, with access date 2026-08-18. The export and
official ability descriptions provide source material for the representative ability
keywords. These keywords are semantic labels owned by this dataset (for
example `crowdControl`, `healing`, `damage`, `buff`, `knockback`, `barrier`,
`resource`, `mobility`, and `form`); they are not claimed to be Blizzard or
Workshop enum values. The six variants likewise preserve official hero-detail
ability shapes and names, but do not claim Workshop-export source lineage. The
ability source references use the official [Ana](https://overwatch.blizzard.com/en-us/heroes/ana/),
[Brigitte](https://overwatch.blizzard.com/en-us/heroes/brigitte/),
[Ramattra](https://overwatch.blizzard.com/en-us/heroes/ramattra/),
[D.Va](https://overwatch.blizzard.com/en-us/heroes/dva/),
[Bastion](https://overwatch.blizzard.com/en-us/heroes/bastion/), and
[Venture](https://overwatch.blizzard.com/en-us/heroes/venture/) pages, accessed
2026-08-18.

The previously proposed Venture base health and Drill Dash cooldown/damage
facts are intentionally absent: their cited June 30, 2026 Community Crafted
limited-mode patch scope is not modeled by this baseline dataset. Armor/shields,
other hero and ability stats,
cooldowns, healing, ammo, durations, ranges, projectile speeds, resources,
and other balance values remain explicitly absent where no current source record is
recorded; no older or inferred balance values are included.

## Validation boundary

Loading rejects malformed JSON, unsupported schema versions, stale digests,
duplicate hero or per-hero slot/variant pairs, duplicate slot/variant pairs,
unqualified multiple entries in one slot, missing
source references, empty identities (including empty slots), and non-finite quantities.
Unknown non-empty logical slot identities remain valid because `LogicalSlot` is
open string-backed. No query
or calculation behavior is implemented by this dataset track; those concerns
belong to #25.
