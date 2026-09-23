# Hero gameplay source attribution

[← Source attribution index](README.md)

## Hero gameplay data (`crates/workshop-rs/src/data/gameplay.json`)

The gameplay dataset is a workshop-rs-owned, MIT-compatible projection of the
user-provided `workshop-data/workshop-data.json` export. Its source is pinned
to commit `d854bf01fc7bbf3b2169f67408c07a8da8989ad6` (commit date
2026-08-12). The export is used only for hero identity, localized naming, and
declared named ability-slot topology; no OverPy or OSTW data is copied.

The committed projection contains hero identities, role facts, and
export-declared named ability slots, plus official-detail variant records for
Bastion, D.Va, and Ramattra. Each hero/ability name fact and export record
carries the export path as a `SourceReference`. Each role fact carries its
official Blizzard hero-detail URL and access date 2026-08-18 as a separate
source reference.

The current identity digest is
`sha256:0902a247fb709bf5e326bbdb5475b41d4062b991ba3e0aea9350e5ecd404bc3c`.

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
