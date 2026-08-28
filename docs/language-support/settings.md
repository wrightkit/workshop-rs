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
use workshop_rs::settings::{definitions, SettingTarget, SettingValueDomain};

let health = definitions()
    .find(|definition| definition.id().is_some_and(|id| id.as_str() == "setting.hero.health"))
    .expect("canonical hero health setting");
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
