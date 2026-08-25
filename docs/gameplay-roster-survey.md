# Hero gameplay topology survey

This survey pins the roster evidence used by ADR-0002. It is a data-scope
survey, not a claim that every row has a balance fact. The source is the
user-provided `workshop-data` export at commit
`d854bf01fc7bbf3b2169f67408c07a8da8989ad6` (commit date 2026-08-12). The export
records Workshop-facing
canonical hero and logical-slot topology plus localized/display names; unsupported gameplay
facts remain absent until separately evidenced.

| Hero identity | Named logical slots in evidence |
| --- | --- |
| ana | ability1, ability2, ultimate |
| anran | ability1, ability2, ultimate |
| ashe | ability1, ability2, ultimate |
| baptiste | ability1, ability2, ultimate |
| bastion | secondaryFire, ability1, ultimate |
| brigitte | secondaryFire, ability1, ability2, ability3, ultimate |
| cassidy | ability1, ability2, ultimate |
| dmon | secondaryFire, ability1, ability2, ultimate |
| domina | primaryFire, secondaryFire, ability1, ability2, ultimate, passive |
| doomfist | secondaryFire, ability1, ability2, ultimate |
| dva | secondaryFire, ability1, ability2, ultimate |
| echo | secondaryFire, ability1, ability2, ultimate, passive |
| emre | ability1, ability2, ultimate |
| freja | secondaryFire, ability1, ability2, ultimate |
| genji | ability1, ability2, ultimate |
| illari | ability1, ability2, ultimate |
| wreckingBall | secondaryFire, ability1, ability2, ability3, ultimate |
| hanzo | ability1, ability2, ability3, ultimate |
| jetpackCat | secondaryFire, ability1, ability2, ultimate |
| junkerQueen | secondaryFire, ability1, ability2, ultimate |
| junkrat | ability1, ability2, ultimate |
| kiriko | ability1, ability2, ultimate |
| lucio | secondaryFire, ability1, ability2, ultimate |
| mauga | primaryFire, secondaryFire, ability1, ability2, ultimate |
| mei | ability1, ability2, ultimate |
| mercy | ability1, ability2, ultimate, passive |
| mizuki | secondaryFire, ability1, ability2, ultimate |
| moira | ability1, ability2, ultimate |
| orisa | secondaryFire, ability1, ability2, ultimate |
| pharah | secondaryFire, ability1, ability2, ultimate, passive |
| reaper | ability1, ability2, ultimate |
| reinhardt | secondaryFire, ability1, ability2, ultimate |
| roadhog | ability1, ability2, ultimate |
| shion | secondaryFire, ability1, ability2, ultimate |
| sierra | secondaryFire, ability1, ability2, ultimate |
| sigma | secondaryFire, ability1, ability2, ultimate |
| sojourn | secondaryFire, ability1, ability2, ultimate |
| soldier | secondaryFire, ability1, ability2, ultimate |
| sombra | secondaryFire, ability1, ability2, ultimate |
| symmetra | ability1, ability2, ultimate |
| torbjorn | ability1, ability2, ultimate |
| tracer | ability1, ability2, ultimate |
| widowmaker | ability1, ability2, ultimate |
| winston | ability1, ability2, ultimate |
| zarya | ability1, ability2, ultimate |
| zenyatta | ability1, ability2, ultimate |
| ramattra | secondaryFire, ability1, ability2, ultimate |
| lifeweaver | secondaryFire, ability1, ability2, ultimate |
| venture | primaryFire, secondaryFire, ability1, ultimate |
| juno | primaryFire, secondaryFire, ability1, ability2, ultimate, passive |
| hazard | primaryFire, secondaryFire, ability1, ability2, ultimate |
| wuyang | secondaryFire, ability1, ability2, ultimate |
| vendetta | primaryFire, secondaryFire, ability1, ability2, ultimate |

## Topology categories

- Normal one-entry logical slots: Ana, Cassidy, Winston.
- Extra logical slots: Brigitte, Hanzo, and Wrecking Ball demonstrate
  `ability3` without a hero-specific field.
- Form/configuration variants: Ramattra, Bastion, and D.Va require multiple
  entries, represented by official-detail variant records without changing the
  hero schema.
- Weapon/configuration entries: Mauga, Venture, and Juno demonstrate optional
  `primaryFire` and `secondaryFire` data.
- Passive and resource/recharge-oriented shapes: Echo, Mercy, Pharah, Juno,
  Illari, and D.Va demonstrate optional passive or non-uniform ability facts.
- Missing localization or unsupported stat data: the export has entries with
  incomplete locale fields; the API preserves absence and does not infer a
  display name, role, or balance quantity.

The export is intentionally not used as evidence for undocumented cooldown,
damage, healing, health, or resource values. Those facts belong to the
versioned gameplay dataset in #24 and must carry their own evidence.
