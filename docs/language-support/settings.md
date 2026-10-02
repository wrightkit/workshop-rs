# Custom-Game Settings

[← Back to Language Support Matrix](../language-support.md)

## Settings Blocks

| Feature | Status | Notes |
| --- | --- | --- |
| `main` (Main settings) | ✅ Supported | Custom game mode name and description strings. |
| `lobby` (Lobby settings) | ✅ Supported | Team size, match start rules, spectator settings, map rotation, and lobby options. |
| `modes` (Mode settings) | ✅ Supported | General mode parameters and individual game modes (Assault, Control, Escort, Hybrid, Push, Flashpoint, Clash, Deathmatch, Team Deathmatch, CTF, Elimination, etc.) and map pools (`enabled maps` / `disabled maps`). |
| `heroes` (Hero settings) | ✅ Supported | Global hero rules, roster toggles (`enabled heroes` / `disabled heroes`), role limits, and per-hero ability/weapon/cooldown parameters. |
| `extensions` (Workshop extensions) | ✅ Supported | Extension flags (`Beam Effects`, `Buff Status Effects`, `Debuff Status Effects`, `Buff and Debuff Sounds`, `Energy Explosion Effects`, `Kinetic Explosion Effects`, `Play More Effects`, `Spawn More Dummy Bots`). |
| `workshop` (Custom workshop settings) | ✅ Supported | User-defined custom settings defined via `Workshop Setting ...` values in rules. |

## Canonical typed catalog

The `workshop_rs::settings` module exposes the reviewed settings catalog
through `definitions()`. Each `SettingDefinition` carries a locale-independent
`SettingId`, Workshop `SettingScope`, target shape, typed
`SettingValueDomain`, locale presentation metadata, and setting source metadata.
The source-preserving `Settings` / `SettingsNode` tree remains the authored
value carrier; the catalog does not regenerate or discard unknown settings.

```rust
use workshop_rs::gameplay::{hero_ids, slots, HeroId, LogicalSlot};
use workshop_rs::settings::{
    definitions_by_id, Applicability, SettingId, SettingTarget,
    SettingTargetKind, SettingValueDomain, TeamId,
};

let lobby = definitions_by_id(&SettingId::from("setting.lobby.spectatorSlots"))
    .next()
    .expect("canonical lobby setting");
assert!(matches!(lobby.domain(), SettingValueDomain::Number(_)));

let hero_ability = definitions_by_id(&SettingId::from("setting.hero.ability.enabled"))
    .find(|definition| {
        matches!(definition.target_kind(), SettingTargetKind::HeroAbility { .. })
    })
    .expect("canonical hero ability setting");
let dva_primary = SettingTarget::HeroAbility {
    team: Some(TeamId::new("allTeams")),
    hero: HeroId::from(hero_ids::DVA),
    slot: LogicalSlot::from(slots::PRIMARY_FIRE),
    variant: None,
};
assert_eq!(
    hero_ability.applicability(&dva_primary).expect("applicability"),
    Applicability::Applicable
);

let ashe_only = definitions_by_id(&SettingId::from("setting.hero.ability1EnemyKb"))
    .find(|definition| definition.path().ends_with("ability1EnemyKb%"))
    .expect("exceptional hero setting");
let ana_ability = SettingTarget::HeroAbility {
    team: None,
    hero: HeroId::from(hero_ids::ANA),
    slot: LogicalSlot::from(slots::ABILITY_1),
    variant: None,
};
assert_eq!(
    ashe_only.applicability(&ana_ability).expect("applicability"),
    Applicability::NotApplicable
);

let health = definitions_by_id(&SettingId::from("setting.hero.health"))
    .next()
    .expect("canonical hero setting");
assert!(matches!(health.domain(), SettingValueDomain::Percent(_)));
assert!(health
    .applicability(&SettingTarget::Hero {
        team: None,
        hero: "ana".into(),
    })
    .is_ok());
```

Hero and ability display names are presentation data only. Consumers use the
canonical concept and `SettingTarget`; localized aliases remain parser/emitter
resolution details. Numeric bounds remain unknown until reviewed Workshop
source records establish them; `SettingValueDomain::effective_number` exposes a
clamped effective value only for a definition carrying such source records.

`SettingDefinition::read` and `write` operate on existing occurrences. A
write changes only the typed leaf value, preserving its span and all unrelated
settings structure; inserting or resizing a source list is rejected so an
edit cannot silently become whole-tree regeneration.

`check_emission(&settings)` validates a programmatically built `Settings`
tree against the emission table without emitting, reporting every member the
emitter would reject. `gamemodes.<mode>` members additionally resolve through
the reviewed `gamemodes.general` leaf set (the pinned oracle's schema merge;
`elimination` inherits only its copied subset), and an inherited path shares
the general setting's `SettingId` and definition — typed reads and writes go
through the `gamemodes.general.<key>` identity.

For an edit that must retain the original Workshop text, use
`SettingDefinition::source_edit(source, settings, locale, target, value)`.
It produces a `SettingSourceEdit` for the existing scalar value and verifies
the expected bytes again when `apply(source)` is called. Only that range is
replaced; comments, whitespace, and all bytes outside it are retained. This
does not define comment/trivia attachment semantics beyond the edited setting
occurrence.

## Source name matching

Settings keys and value spellings parse **case-insensitively**: `enabled maps`
and `Enabled Maps` identify the same table entry, and the same holds for mode,
team, hero, map, enum, token (`on`/`off`, `yes`/`no`), and namespace
spellings, in English and in the authored locale. Case is canonicalized at
parse time; emission writes the canonical spelling.

Accents remain significant: `Chateau Guillard` does not resolve to
`Château Guillard`. A rejected name that is close to exactly one canonical
spelling (a case or accent difference, or a small unambiguous edit distance)
names that spelling in the diagnostic message (`did you mean '...'?`); the
same suggestion is also exposed as structured data on
`SettingsDiagnostic::suggestion`, so callers can apply it without parsing
message text. Distant or ambiguous spellings carry none. The input stays
rejected either way — the suggestion never rewrites it.

`Program::validate` runs the same emission acceptance check the emitter
applies (`check_emission`), so checking and compilation agree on a settings
block instead of diverging at emit time; `Program::settings_diagnostics`
exposes every rejection as a `SettingsDiagnostic`.

Sources: Deltinteger's `TextToElement` matches settings, mode, map, hero, and
enum names with `caseSensitive: false`; the pinned oracle's decompiler
lower-cases both sides of settings names. Accent-stripped spellings are
rejected by the same oracle and stay rejected here.
