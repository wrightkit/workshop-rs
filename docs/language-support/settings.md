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
`SettingValueDomain`, locale presentation metadata, and evidence provenance.
The source-preserving `Settings` / `SettingsNode` tree remains the authored
value carrier; the catalog does not regenerate or discard unknown settings.

```rust
use workshop_rs::gameplay::{hero_ids, slots, HeroId, LogicalSlot};
use workshop_rs::settings::{
    definitions_by_id, Applicability, NumericBounds, SettingId, SettingTarget,
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

let ashe_only = definitions_by_id(&SettingId::from("setting.hero.ability.knockback.enemy"))
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

// A definition whose reviewed evidence exposes bounds reports both values;
// the authored source value remains unchanged until an explicit write.
let evidenced = SettingValueDomain::Percent(
    NumericBounds::new(Some(0.0), Some(500.0)).expect("valid bounds"),
);
let effective = evidenced.effective_number(650.0).expect("clamped value");
assert_eq!((effective.authored, effective.effective), (650.0, 500.0));

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
resolution details. Numeric bounds are explicit when reviewed evidence proves
them, and otherwise remain unknown rather than being guessed.

`SettingDefinition::read` and `write` operate on existing occurrences. A
write changes only the typed leaf value, preserving its span and all unrelated
settings structure; inserting or resizing a source list is rejected so an
edit cannot silently become whole-tree regeneration.
