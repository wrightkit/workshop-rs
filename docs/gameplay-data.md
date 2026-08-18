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
`sha256:6af26398d2a2967ee5534a0ce194de502cf1c87b90444141b75629c6d05607d3`.

## Evidence and known gaps

The current dataset is an identity/naming and kit-topology dataset: it contains
all 53 hero identities, their role facts, and all 201 named ability slots
declared by the pinned export. Every hero and ability name is linked to
`workshop-data/workshop-data.json` at commit
`d854bf01fc7bbf3b2169f67408c07a8da8989ad6` (commit date 2026-08-12).

Role facts are evidenced independently by the official Blizzard hero-detail
URL recorded on each role fact, with access date 2026-08-18. The export and
official role pages do not provide the committed dataset with independently
reviewed base health/armor/shields, cooldowns, damage, healing, ammo,
durations, ranges, projectile speeds, resources, or semantic ability keywords.
Those balance/gameplay fields remain absent rather than being inferred from
another implementation or undocumented balance knowledge. A later
gameplay-facts dataset may add them with independent evidence without changing
the identity/naming dataset contract.

## Validation boundary

Loading rejects malformed JSON, unsupported schema versions, stale digests,
duplicate hero or per-hero ability IDs, duplicate slot/variant pairs,
unqualified multiple entries in one slot, missing
evidence, empty identities (including empty slots), and non-finite quantities.
Unknown non-empty logical slot identities remain valid because `LogicalSlot` is
open string-backed. No query
or calculation behavior is implemented by this dataset track; those concerns
belong to #25.
