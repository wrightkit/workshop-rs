# Hero gameplay dataset

`crates/workshop-rs/src/data/gameplay.json` is the embedded gameplay dataset
for the `workshop-rs` hero domain. `workshop_rs::gameplay_data::builtin()`
loads it through the same validation path as external JSON and verifies its
content digest before constructing a `GameplayCatalog`.

## Schema and identity

The file has `schemaVersion: 1`, a `GameplayDatasetIdentity`, and a sorted
hero list. Heroes and abilities carry evidence references. Hero and ability
IDs are open strings; the ability IDs in this dataset are lowerCamel names
derived from the export's `en-US` ability labels, with the source path retained
in evidence. The current dataset uses the seven canonical slots from ADR-0002;
the open string-backed API also accepts future non-empty slot identities.
Multiple abilities in one slot must have distinct, non-empty variants.

The dataset digest is `sha256:` followed by the SHA-256 of the canonical JSON
content with `identity.digest` removed. Object keys are sorted before hashing,
so formatting and input key order do not change the identity. The loader also
sorts heroes and abilities before building lookup indexes.

The committed dataset identity is version `2026-08-12` with digest
`sha256:d15bf17d413e7057bc7ef25e90a6e33df1a79e279a9dbff41e643a30fb9f7635`.

## Evidence and known gaps

The current dataset is an identity/naming and kit-topology dataset: it contains
all 53 hero identities, their role facts, and all 201 named ability slots
declared by the pinned export. It also contains six official-detail variant
records for Bastion, D.Va, and Ramattra, for 207 ability records total. The
embedded projection currently has nine keyword-bearing abilities, six variants,
and three evidence-backed quantity facts. Every hero and ability name is linked to
`workshop-data/workshop-data.json` at commit
`d854bf01fc7bbf3b2169f67408c07a8da8989ad6` (commit date 2026-08-12).

Role facts are evidenced independently by the official Blizzard hero-detail
URL recorded on each role fact, with access date 2026-08-18. The export and
official ability descriptions provide evidence for the representative ability
keywords. These keywords are semantic labels owned by this dataset (for
example `crowdControl`, `healing`, `damage`, `buff`, `knockback`, `barrier`,
`resource`, `mobility`, and `form`); they are not claimed to be Blizzard or
Workshop enum values. The six variants likewise preserve official hero-detail
ability shapes and names, but do not claim Workshop-export provenance. The
ability evidence uses the official [Ana](https://overwatch.blizzard.com/en-us/heroes/ana/),
[Brigitte](https://overwatch.blizzard.com/en-us/heroes/brigitte/),
[Ramattra](https://overwatch.blizzard.com/en-us/heroes/ramattra/),
[D.Va](https://overwatch.blizzard.com/en-us/heroes/dva/),
[Bastion](https://overwatch.blizzard.com/en-us/heroes/bastion/), and
[Venture](https://overwatch.blizzard.com/en-us/heroes/venture/) pages, accessed
2026-08-18.

The Venture facts currently supported by an official live patch entry are base
health `225 health`, and Drill Dash cooldown `6 seconds` and damage `35
damage`, each evidenced by the [June 30, 2026 live patch notes](https://overwatch.blizzard.com/en-us/news/patch-notes/live/2026/6/)
and accessed 2026-08-18. Armor/shields, other hero and ability stats,
cooldowns, healing, ammo, durations, ranges, projectile speeds, resources,
and other balance values remain explicitly absent where no current evidence is
recorded; no older or inferred balance values are included.

## Validation boundary

Loading rejects malformed JSON, unsupported schema versions, stale digests,
duplicate hero or per-hero ability IDs, duplicate slot/variant pairs,
unqualified multiple entries in one slot, missing
evidence, empty identities (including empty slots), and non-finite quantities.
Unknown non-empty logical slot identities remain valid because `LogicalSlot` is
open string-backed. No query
or calculation behavior is implemented by this dataset track; those concerns
belong to #25.
