# Hero gameplay source attribution

[← Source attribution index](README.md)

## Hero gameplay data (`crates/workshop-rs/src/data/gameplay.json`)

The gameplay dataset is a workshop-rs-owned, MIT-compatible projection of the
user-provided `workshop-data/workshop-data.json` export. Its source is pinned
to commit `5a7d0e294b8cad73b9701987bb584d0551d7fa4d` (commit date
2026-10-06). The export is used only for hero identity, localized naming, and
declared named ability-slot topology; no OverPy or OSTW data is copied.

The committed projection contains hero identities, role facts, and
export-declared named ability slots, plus official-detail variant records for
Bastion, D.Va, and Ramattra. Each hero/ability name fact and export record
carries the export path as a `SourceReference`. Each role fact carries its
official Blizzard hero-detail URL and access date 2026-08-18 as a separate
source reference.

The current identity digest is
`sha256:06e33045154d9f32553b09bf70bb5156556bb4fe8d28812422b51dbbb5ceabb9`.

Representative ability keywords are semantic labels, not Blizzard or Workshop
enum values. Their labels and the six variant names/shapes are source-linked to
the official Blizzard hero-detail URLs for Ana, Brigitte, Ramattra, D.Va,
Bastion, and Venture, accessed 2026-08-18;
the variants intentionally carry no fabricated Workshop-export source
attribution.
Venture base health and Drill Dash cooldown/damage are intentionally absent
because the cited June 30, 2026 Community Crafted patch source is scoped to a
limited mode and that scope is not modeled. Other base stats, armor/shields, cooldowns, damage,
healing, ammo, durations, ranges, projectile speeds, resources, and balance
values remain absent unless supported by a current explicit source. The
loader verifies the separate gameplay dataset identity and deterministic
SHA-256 digest; it does not alter the Workshop catalog identity.
