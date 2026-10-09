//! Fixture-backed settings emission table (#86).
//!
//! SOURCE: observed from the pinned oracle 9.7.10 en-US output of the
//! oracle-success settings programs (`compile.workshop` settings section of
//! the committed snapshots pixelart/santa/broken-weapons/client-to-server,
//! plus the parabola/crosshair/inputhud oracle runs) at OverPy commit
//! `eea67ad`. This is observed-behavior data, not copied OverPy source
//! (LICENSE-BOUNDARY policy). Additions to the table (e.g. the acquired
//! candidate snapshots) are data-only.

use serde::Deserialize;
use serde_json::Value;
use std::sync::OnceLock;

use super::PathPart::{self, Hero, Part, Team};

/// Locale-specific settings names generated from the reviewed Workshop data
/// export. The projection contains every reviewed locale as data; adding a
/// locale changes this file, not the parser or emitter architecture.
const LOCALE_DATA: &str = include_str!("data/locales.json");

fn locale_data() -> &'static Value {
    static DATA: OnceLock<Value> = OnceLock::new();
    DATA.get_or_init(|| {
        serde_json::from_str(LOCALE_DATA).expect("generated settings locale data is valid JSON")
    })
}

/// Resolve a settings display name from the generated locale corpus.
///
/// The English table names are intentionally not duplicated in the locale
/// data. A missing entry means the target locale is not covered and callers
/// must preserve the explicit missing-mapping contract.
pub(crate) fn localized_name(locale: &str, section: &str, english: &str) -> Option<&'static str> {
    let data = locale_data();
    let aliases = data.get(section)?.get(english)?.as_object()?;
    aliases.get(locale).and_then(Value::as_str).or_else(|| {
        aliases.iter().find_map(|(known, value)| {
            known
                .eq_ignore_ascii_case(locale)
                .then(|| value.as_str())
                .flatten()
        })
    })
}

/// A leaf key kind: how a settings leaf renders and validates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum KeyKind {
    /// A presence-only extension setting.
    Flag,
    /// A quoted string (`Description: "..."`).
    String,
    /// A boolean rendered `On`/`Off`.
    Bool,
    /// A boolean rendered `Yes`/`No`.
    YesNo,
    /// A boolean carrier rendered through a source-supported true-value enum
    /// token. The false value remains unsupported until independently sourced.
    BoolEnum(&'static str),
    /// A plain number.
    Number,
    /// A number rendered with a `%` suffix (`Respawn Time Scalar: 30%`).
    Percent,
    /// A string-valued enumeration with a per-domain member map
    /// (`Enum(domain)`).
    Enum(&'static str),
    /// A list of map names (`enabled maps`).
    ListMap,
    /// A list of hero names (`enabled heroes`).
    ListHero,
}

/// One table entry: an exact key path, its workshop name, and its kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TableEntry {
    pub(crate) path: &'static [PathPart<'static>],
    pub(crate) workshop_name: &'static str,
    pub(crate) kind: KeyKind,
}

macro_rules! entry {
    ($path:expr, $name:expr, $kind:expr) => {
        TableEntry {
            path: &$path,
            workshop_name: $name,
            kind: $kind,
        }
    };
}

/// The fixture-backed settings surface.
///
/// Slot sets (source-backed): teams {allTeams}, heroes {mei} config groups +
/// the 10 ListHero names. `enabled: true` is not source-backed; it renders with
/// no prefix. Keys outside this table (e.g. team1Slots, scoreToWin,
/// gamemodeStartTrigger, spawnHealthPacks, healthPackRespawnTime%,
/// abilityCooldown%, healingReceived%, primaryFireKb%, enableSpawningWithUlt,
/// resetPlayersAfterGoalScored, scoreLeadToWin, gameLengthInSec,
/// heroes.<team>.general, roleLimit under general, heroLimit under a named
/// mode) are `settings-unknown-key` at validation (only source-backed in
/// oracle-failing programs; corpus-bounded).
pub(crate) static ENTRIES: &[TableEntry] = &[
    // main
    entry!(
        [Part("main"), Part("description")],
        "Description",
        KeyKind::String
    ),
    entry!(
        [Part("main"), Part("modeName")],
        "Mode Name",
        KeyKind::String
    ),
    // lobby
    entry!(
        [Part("lobby"), Part("ffaSlots")],
        "Max FFA Players",
        KeyKind::Number
    ),
    entry!(
        [Part("lobby"), Part("mapRotation")],
        "Map Rotation",
        KeyKind::Enum("mapRotation")
    ),
    entry!(
        [Part("lobby"), Part("spectatorSlots")],
        "Max Spectators",
        KeyKind::Number
    ),
    entry!(
        [Part("lobby"), Part("enableMatchVoiceChat")],
        "Match Voice Chat",
        KeyKind::BoolEnum("matchVoiceChat")
    ),
    entry!(
        [Part("lobby"), Part("team1Slots")],
        "Max Team 1 Players",
        KeyKind::Number
    ),
    entry!(
        [Part("lobby"), Part("team2Slots")],
        "Max Team 2 Players",
        KeyKind::Number
    ),
    entry!(
        [Part("lobby"), Part("returnToLobby")],
        "Return To Lobby",
        KeyKind::Enum("returnToLobby")
    ),
    entry!(
        [Part("lobby"), Part("allowPlayersInQueue")],
        "Allow Players Who Are In Queue",
        KeyKind::YesNo
    ),
    entry!(
        [Part("lobby"), Part("swapTeamsAfterMatch")],
        "Swap Teams After Match",
        KeyKind::YesNo
    ),
    // gamemodes.<mode> — per-key subsets (exact-path entries, #86):
    // enabledMaps under modes {assault, control, escort, hybrid, skirmish,
    // ffa}; enabled/roleLimit/enableCompetitiveRules under {assault, control,
    // escort, hybrid}; heroLimit/respawnTime%/enableHeroSwitching/
    // enableRandomHeroes under general only (general is a literal group name,
    // not a mode slot).
    entry!(
        [Part("gamemodes"), Part("assault"), Part("enabled")],
        "enabled",
        KeyKind::Bool
    ),
    entry!(
        [Part("gamemodes"), Part("control"), Part("enabled")],
        "enabled",
        KeyKind::Bool
    ),
    entry!(
        [Part("gamemodes"), Part("escort"), Part("enabled")],
        "enabled",
        KeyKind::Bool
    ),
    entry!(
        [Part("gamemodes"), Part("hybrid"), Part("enabled")],
        "enabled",
        KeyKind::Bool
    ),
    entry!(
        [Part("gamemodes"), Part("assault"), Part("enabledMaps")],
        "enabled maps",
        KeyKind::ListMap
    ),
    entry!(
        [Part("gamemodes"), Part("control"), Part("enabledMaps")],
        "enabled maps",
        KeyKind::ListMap
    ),
    entry!(
        [Part("gamemodes"), Part("escort"), Part("enabledMaps")],
        "enabled maps",
        KeyKind::ListMap
    ),
    entry!(
        [Part("gamemodes"), Part("hybrid"), Part("enabledMaps")],
        "enabled maps",
        KeyKind::ListMap
    ),
    entry!(
        [Part("gamemodes"), Part("skirmish"), Part("enabledMaps")],
        "enabled maps",
        KeyKind::ListMap
    ),
    entry!(
        [Part("gamemodes"), Part("assault"), Part("disabledMaps")],
        "disabled maps",
        KeyKind::ListMap
    ),
    entry!(
        [Part("gamemodes"), Part("skirmish"), Part("disabledMaps")],
        "disabled maps",
        KeyKind::ListMap
    ),
    entry!(
        [Part("gamemodes"), Part("ffa"), Part("enabledMaps")],
        "enabled maps",
        KeyKind::ListMap
    ),
    entry!(
        [Part("gamemodes"), Part("tdm"), Part("enabledMaps")],
        "enabled maps",
        KeyKind::ListMap
    ),
    entry!(
        [Part("gamemodes"), Part("assault"), Part("roleLimit")],
        "Limit Roles",
        KeyKind::Enum("roleLimit")
    ),
    entry!(
        [Part("gamemodes"), Part("control"), Part("roleLimit")],
        "Limit Roles",
        KeyKind::Enum("roleLimit")
    ),
    entry!(
        [Part("gamemodes"), Part("escort"), Part("roleLimit")],
        "Limit Roles",
        KeyKind::Enum("roleLimit")
    ),
    entry!(
        [Part("gamemodes"), Part("hybrid"), Part("roleLimit")],
        "Limit Roles",
        KeyKind::Enum("roleLimit")
    ),
    entry!(
        [Part("gamemodes"), Part("general"), Part("roleLimit")],
        "Limit Roles",
        KeyKind::Enum("roleLimit")
    ),
    entry!(
        [
            Part("gamemodes"),
            Part("assault"),
            Part("enableCompetitiveRules")
        ],
        "Competitive Rules",
        KeyKind::Bool
    ),
    entry!(
        [
            Part("gamemodes"),
            Part("control"),
            Part("enableCompetitiveRules")
        ],
        "Competitive Rules",
        KeyKind::Bool
    ),
    entry!(
        [
            Part("gamemodes"),
            Part("escort"),
            Part("enableCompetitiveRules")
        ],
        "Competitive Rules",
        KeyKind::Bool
    ),
    entry!(
        [
            Part("gamemodes"),
            Part("hybrid"),
            Part("enableCompetitiveRules")
        ],
        "Competitive Rules",
        KeyKind::Bool
    ),
    // gamemodes.general
    entry!(
        [
            Part("gamemodes"),
            Part("general"),
            Part("enableCompetitiveRules")
        ],
        "Competitive Rules",
        KeyKind::Bool
    ),
    entry!(
        [Part("gamemodes"), Part("general"), Part("enablePerks")],
        "Enable Perks",
        KeyKind::Bool
    ),
    entry!(
        [Part("gamemodes"), Part("general"), Part("heroLimit")],
        "Hero Limit",
        KeyKind::Enum("heroLimit")
    ),
    entry!(
        [Part("gamemodes"), Part("general"), Part("respawnTime%")],
        "Respawn Time Scalar",
        KeyKind::Percent
    ),
    entry!(
        [
            Part("gamemodes"),
            Part("general"),
            Part("enableHeroSwitching")
        ],
        "Allow Hero Switching",
        KeyKind::Bool
    ),
    entry!(
        [
            Part("gamemodes"),
            Part("general"),
            Part("enableRandomHeroes")
        ],
        "Respawn As Random Hero",
        KeyKind::Bool
    ),
    entry!(
        [
            Part("gamemodes"),
            Part("general"),
            Part("gameModeStartTrigger")
        ],
        "Game Mode Start",
        KeyKind::Enum("gameModeStartTrigger")
    ),
    entry!(
        [
            Part("gamemodes"),
            Part("assault"),
            Part("gameModeStartTrigger")
        ],
        "Game Mode Start",
        KeyKind::Enum("gameModeStartTrigger")
    ),
    entry!(
        [
            Part("gamemodes"),
            Part("assault"),
            Part("tankPassiveHealthBonus")
        ],
        "Tank Role Passive Health Bonus",
        KeyKind::Enum("tankPassiveHealthBonus")
    ),
    entry!(
        [Part("gamemodes"), Part("general"), Part("spawnHealthPacks")],
        "Spawn Health Packs",
        KeyKind::Enum("spawnHealthPacks")
    ),
    // heroes.<team>
    entry!(
        [Part("heroes"), Team, Part("enabledHeroes")],
        "enabled heroes",
        KeyKind::ListHero
    ),
    entry!(
        [Part("heroes"), Team, Part("disabledHeroes")],
        "disabled heroes",
        KeyKind::ListHero
    ),
    entry!(
        [Part("heroes"), Part("general"), Part("disabledHeroes")],
        "disabled heroes",
        KeyKind::ListHero
    ),
    // heroes.<team>.<hero> config groups
    entry!(
        [Part("heroes"), Team, Hero, Part("enablePrimaryFire")],
        "Primary Fire",
        KeyKind::Bool
    ),
    entry!(
        [Part("heroes"), Team, Hero, Part("enableSecondaryFire")],
        "Secondary Fire",
        KeyKind::Bool
    ),
    entry!(
        [Part("heroes"), Team, Hero, Part("enableAbility1")],
        "Ability 1",
        KeyKind::Bool
    ),
    entry!(
        [Part("heroes"), Team, Hero, Part("enableAbility2")],
        "Ability 2",
        KeyKind::Bool
    ),
    entry!(
        [Part("heroes"), Team, Hero, Part("health%")],
        "Health",
        KeyKind::Percent
    ),
    entry!(
        [Part("heroes"), Team, Hero, Part("passiveUltGen%")],
        "Ultimate Generation - Passive Blizzard",
        KeyKind::Percent
    ),
    entry!(
        [Part("heroes"), Team, Hero, Part("combatUltGen%")],
        "Ultimate Generation - Combat Blizzard",
        KeyKind::Percent
    ),
];

/// A slot name mapping (key -> localized workshop name).
#[derive(Debug, Clone, Copy)]
pub(crate) struct NameMap {
    pub(crate) key: &'static str,
    pub(crate) name: &'static str,
}

const fn named(key: &'static str, name: &'static str) -> NameMap {
    NameMap { key, name }
}

/// Game-mode names (source-backed: assault, control, escort, hybrid, skirmish,
/// ffa, tdm, general).
pub(crate) static MODE_NAMES: &[NameMap] = &[
    named("assault", "Assault"),
    named("control", "Control"),
    named("escort", "Escort"),
    named("hybrid", "Hybrid"),
    named("skirmish", "Skirmish"),
    named("ffa", "Deathmatch"),
    named("tdm", "Team Deathmatch"),
    named("general", "General"),
];

/// Map names inside `enabledMaps` lists.
pub(crate) static MAP_NAMES: &[NameMap] = &[
    named("workshopIsland", "Workshop Island"),
    named("kingsRowWinter", "King's Row Winter"),
];

/// Hero names inside hero lists and hero-config groups.
pub(crate) static HERO_NAMES: &[NameMap] = &[
    named("anran", "Anran"),
    named("ana", "Ana"),
    named("ashe", "Ashe"),
    named("bastion", "Bastion"),
    named("baptiste", "Baptiste"),
    named("brigitte", "Brigitte"),
    named("cassidy", "Cassidy"),
    named("dmon", "D.Mon"),
    named("domina", "Domina"),
    named("dva", "D.Va"),
    named("doomfist", "Doomfist"),
    named("echo", "Echo"),
    named("emre", "Emre"),
    named("freja", "Freja"),
    named("genji", "Genji"),
    named("hanzo", "Hanzo"),
    named("moira", "Moira"),
    named("reinhardt", "Reinhardt"),
    named("hammond", "Wrecking Ball"),
    named("hazard", "Hazard"),
    named("illari", "Illari"),
    named("juno", "Juno"),
    named("jetpackCat", "Jetpack Cat"),
    named("junkerQueen", "Junker Queen"),
    named("junkrat", "Junkrat"),
    named("kiriko", "Kiriko"),
    named("lucio", "Lúcio"),
    named("mauga", "Mauga"),
    named("mercy", "Mercy"),
    named("mizuki", "Mizuki"),
    named("orisa", "Orisa"),
    named("pharah", "Pharah"),
    named("reaper", "Reaper"),
    named("roadhog", "Roadhog"),
    named("shion", "Shion"),
    named("sierra", "Sierra"),
    named("sigma", "Sigma"),
    named("ramattra", "Ramattra"),
    named("lifeweaver", "Lifeweaver"),
    named("sojourn", "Sojourn"),
    named("soldier", "Soldier: 76"),
    named("sombra", "Sombra"),
    named("symmetra", "Symmetra"),
    named("torbjorn", "Torbjörn"),
    named("tracer", "Tracer"),
    named("venture", "Venture"),
    named("widowmaker", "Widowmaker"),
    named("winston", "Winston"),
    named("wuyang", "Wuyang"),
    named("wreckingBall", "Wrecking Ball"),
    named("zarya", "Zarya"),
    named("zenyatta", "Zenyatta"),
    named("mei", "Mei"),
];

include!("data/generated_map_entries.rs");
include!("data/generated_hero_entries.rs");
include!("data/generated_mode_entries.rs");

/// Team names inside `heroes` (source-backed: allTeams).
pub(crate) static TEAM_NAMES: &[NameMap] = &[
    named("allTeams", "General"),
    named("team1", "Team 1"),
    named("team2", "Team 2"),
];

/// An enum domain member (domain -> localized workshop name).
#[derive(Debug, Clone, Copy)]
pub(crate) struct EnumMember {
    pub(crate) domain: &'static str,
    pub(crate) member: &'static str,
    pub(crate) name: &'static str,
}

const fn enum_member(domain: &'static str, member: &'static str, name: &'static str) -> EnumMember {
    EnumMember {
        domain,
        member,
        name,
    }
}

include!("data/generated_entries.rs");
include!("data/generated_hero_settings.rs");

/// Fixture-owned canonical enum member names. Additional reviewed
/// Workshop-data export members are retained through
/// `projection_reconciliation.json`, which maps their source identities into
/// these canonical domains without replacing fixture-backed display names.
pub(crate) static ENUM_MEMBERS: &[EnumMember] = &[
    enum_member("mapRotation", "afterAGame", "After A Game"),
    enum_member("mapRotation", "afterMirrorMatch", "After A Mirror Match"),
    enum_member("mapRotation", "paused", "Paused"),
    enum_member("matchVoiceChat", "enabled", "Enabled"),
    enum_member("returnToLobby", "never", "Never"),
    enum_member("returnToLobby", "afterAGame", "After A Game"),
    enum_member("returnToLobby", "afterMirrorMatch", "After A Mirror Match"),
    enum_member("gameModeStartTrigger", "immediately", "Immediately"),
    enum_member("gameModeStartTrigger", "manual", "Manual"),
    enum_member("spawnHealthPacks", "disabled", "Disabled"),
    enum_member("spawnHealthPacks", "modeDependent", "Determined By Mode"),
    enum_member("spawnHealthPacks", "enabled", "Enabled"),
    enum_member("roleLimit", "2OfEachRolePerTeam", "2 Of Each Role Per Team"),
    enum_member(
        "roleLimit",
        "1Tank2Offense2Support",
        "1 Tank 2 Offense 2 Support",
    ),
    enum_member("roleLimit", "off", "Off"),
    enum_member("tankPassiveHealthBonus", "alwaysEnabled", "Always Enabled"),
    enum_member("tankPassiveHealthBonus", "disabled", "Disabled"),
    enum_member("heroLimit", "off", "Off"),
    enum_member("heroLimit", "1PerTeam", "1 Per Team"),
    enum_member("heroLimit", "2PerTeam", "2 Per Team"),
    enum_member("heroLimit", "1PerGame", "1 Per Game"),
    enum_member("heroLimit", "2PerGame", "2 Per Game"),
];

/// Look up a settings leaf entry by its exact path.
///
/// `gamemodes.<mode>` groups inherit the `gamemodes.general` leaf set: the
/// pinned oracle's settings schema merges `general.values` into every declared
/// mode, with `elimination` inheriting only [`ELIMINATION_GENERAL_KEYS`]
/// (source: `computeCustomGameSettingsSchema` in the pinned 9.7.10 oracle).
/// The fallback resolves to the same `general` table entry, so an inherited
/// per-mode path shares that entry's canonical identity.
pub(crate) fn lookup(path: &[PathPart<'_>]) -> Option<&'static TableEntry> {
    exact_entry(path).or_else(|| mode_inherited_entry(path))
}

fn exact_entry(path: &[PathPart<'_>]) -> Option<&'static TableEntry> {
    entries().find(|entry| {
        entry.path.len() == path.len() && entry.path.iter().zip(path.iter()).all(|(a, b)| a == b)
    })
}

/// The `gamemodes.general` leaf keys `elimination` inherits (the pinned
/// oracle's restricted copy list; every other declared mode inherits the full
/// general set).
static ELIMINATION_GENERAL_KEYS: &[&str] = &[
    "disabledMaps",
    "enableEnemyHealthBars",
    "enableKillCam",
    "enableKillFeed",
    "enableSkins",
    "enabledMaps",
    "gamemodeStartTrigger",
    "healthPackRespawnTime%",
    "perkEliminationCatchupLevelAmount%",
    "perkGeneration%",
    "spawnHealthPacks",
    "teamOverlay",
];

fn mode_inherited_entry(path: &[PathPart<'_>]) -> Option<&'static TableEntry> {
    let [
        PathPart::Part("gamemodes"),
        PathPart::Part(mode),
        PathPart::Part(key),
    ] = path
    else {
        return None;
    };
    if *mode == "general" {
        return None;
    }
    let inherits = if *mode == "elimination" {
        ELIMINATION_GENERAL_KEYS.contains(key)
    } else {
        // A mode with declared entries is a reviewed mode slot and inherits
        // the general leaf set.
        entries().any(|entry| {
            matches!(
                entry.path,
                [PathPart::Part("gamemodes"), PathPart::Part(declared), ..] if *declared == *mode
            )
        })
    };
    if !inherits {
        return None;
    }
    exact_entry(&[
        PathPart::Part("gamemodes"),
        PathPart::Part("general"),
        PathPart::Part(key),
    ])
}

/// Iterate the reviewed settings inventory with the hand-written projection
/// taking precedence over the generated export projection. Duplicate paths
/// are represented once in the semantic catalog while the parser and emitter
/// continue to use the same lookup table.
pub(crate) fn entries() -> impl Iterator<Item = &'static TableEntry> {
    deduplicated_entries(ENTRIES.iter().chain(GENERATED_ENTRIES.iter()))
}

/// Iterate both catalog projections without applying effective lookup
/// precedence. The semantic validator uses this to compare duplicate paths
/// instead of allowing `entries()` to hide stale or conflicting data.
pub(crate) fn raw_entries() -> impl Iterator<Item = ProjectedEntry> {
    ENTRIES
        .iter()
        .map(|entry| ProjectedEntry {
            source: ProjectionSource::FixtureTable,
            entry,
        })
        .chain(GENERATED_ENTRIES.iter().map(|entry| ProjectedEntry {
            source: ProjectionSource::WorkshopDataExport,
            entry,
        }))
}

/// One setting leaf before effective lookup resolves duplicated projections.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ProjectedEntry {
    pub(crate) source: ProjectionSource,
    pub(crate) entry: &'static TableEntry,
}

/// The source that supplied a raw settings projection entry.
#[derive(Debug, Clone, Copy)]
pub(crate) enum ProjectionSource {
    FixtureTable,
    WorkshopDataExport,
}

impl ProjectionSource {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::FixtureTable => "fixture table",
            Self::WorkshopDataExport => "Workshop-data export",
        }
    }
}

fn deduplicated_entries(
    entries: impl Iterator<Item = &'static TableEntry>,
) -> impl Iterator<Item = &'static TableEntry> {
    let mut paths = std::collections::HashSet::new();
    entries.filter(move |entry| paths.insert(path_string(entry.path)))
}

pub(crate) fn is_generated_entry(entry: &TableEntry) -> bool {
    GENERATED_ENTRIES
        .iter()
        .any(|candidate| std::ptr::eq(candidate, entry))
}

/// Map the existing hero-settings leaf keys to canonical gameplay slots.
/// The setting tree remains the owner of the keys; display names are resolved
/// from the gameplay catalog by the parser/emitter when a hero context exists.
pub(crate) fn ability_slot_for_path(path: &[PathPart<'_>]) -> Option<&'static str> {
    match path.last() {
        Some(PathPart::Part("ability1Cooldown%" | "enableAbility1")) => Some("ability1"),
        Some(PathPart::Part("ability2Cooldown%" | "enableAbility2")) => Some("ability2"),
        Some(PathPart::Part("ability3Cooldown%" | "enableAbility3")) => Some("ability3"),
        Some(PathPart::Part(
            "secondaryFireCooldown%"
            | "secondaryFireEnergyChargeRate%"
            | "secondaryFireMaximumTime%"
            | "secondaryFireRechargeRate%"
            | "enableSecondaryFire"
            | "enableGenericSecondaryFire",
        )) => Some("secondaryFire"),
        Some(PathPart::Part("combatUltGen%" | "passiveUltGen%" | "ultGen%" | "enableUlt")) => {
            Some("ultimate")
        }
        Some(PathPart::Part("enablePassive")) => Some("passive"),
        _ => None,
    }
}

/// Resolve a source-backed hero-specific setting label.
pub(crate) fn hero_setting_name(hero: &str, key: &str, locale: &str) -> Option<&'static str> {
    // A producer alias is the spelling pinned OverPy writes, so it is emitted
    // in preference to the export label.
    hero_setting_aliases()
        .iter()
        .find(|alias| {
            alias.hero == hero && alias.key == key && alias.locale.eq_ignore_ascii_case(locale)
        })
        .map(|alias| alias.display.as_str())
        .or_else(|| {
            GENERATED_HERO_SETTING_NAMES
                .iter()
                .find(|entry| entry.hero == hero && entry.key == key)
                .and_then(|entry| entry.localized(locale))
        })
        .or_else(|| {
            if locale.eq_ignore_ascii_case("en-US") {
                GENERATED_HERO_SETTING_NAMES
                    .iter()
                    .find(|entry| entry.hero == hero && entry.key == key)
                    .filter(|entry| {
                        entry.locales.iter().any(|(known, value)| {
                            known.eq_ignore_ascii_case(locale)
                                && (value.trim().is_empty() || value.starts_with(' '))
                        })
                    })
                    .map(|entry| entry.key)
            } else {
                None
            }
        })
}

#[derive(Deserialize)]
struct HeroSettingAlias {
    hero: String,
    key: String,
    locale: String,
    display: String,
}

fn hero_setting_aliases() -> &'static [HeroSettingAlias] {
    static ALIASES: OnceLock<Vec<HeroSettingAlias>> = OnceLock::new();
    ALIASES.get_or_init(|| {
        serde_json::from_str(include_str!("data/hero_setting_aliases.json"))
            .expect("hero setting alias data is valid JSON")
    })
}

/// Reviewed producer aliases observed in the pinned AI-PVE artifact. These
/// labels omit the export's `倍率` suffix or use the producer's shorter
/// ability label, but identify the same canonical setting path.
pub(crate) fn hero_setting_alias(hero: &str, key: &str, locale: &str, display: &str) -> bool {
    hero_setting_aliases().iter().any(|alias| {
        alias.hero == hero
            && alias.key == key
            && alias.locale.eq_ignore_ascii_case(locale)
            && names_eq(&alias.display, display)
    })
}

fn name_in(maps: &[NameMap], key: &str) -> Option<&'static str> {
    maps.iter().find(|m| m.key == key).map(|m| m.name)
}

/// Settings-name comparison: the Workshop client matches setting keys and
/// value spellings without regard to case (Deltinteger's `TextToElement`
/// settings matcher is `caseSensitive: false`; the pinned oracle's
/// decompiler lowercases both sides). Accents remain significant: `Chateau
/// Guillard` does not match `Château Guillard`.
pub(crate) fn names_eq(left: &str, right: &str) -> bool {
    left == right || left.to_lowercase() == right.to_lowercase()
}

/// Canonical map spellings for suggestion candidates (English names).
pub(crate) fn map_spellings() -> impl Iterator<Item = &'static str> {
    MAP_NAMES
        .iter()
        .chain(GENERATED_MAP_NAMES.iter())
        .map(|m| m.name)
}

/// Canonical hero spellings for suggestion candidates (English names).
pub(crate) fn hero_spellings() -> impl Iterator<Item = &'static str> {
    HERO_NAMES
        .iter()
        .chain(GENERATED_HERO_NAMES.iter())
        .map(|m| m.name)
}

/// Canonical team spellings for suggestion candidates (English names).
pub(crate) fn team_spellings() -> impl Iterator<Item = &'static str> {
    TEAM_NAMES.iter().map(|m| m.name)
}

/// Canonical member spellings of an enum domain for suggestion candidates.
pub(crate) fn enum_spellings(domain: &str) -> impl Iterator<Item = &'static str> {
    enum_members(domain).map(|member| member.name)
}

/// Canonical `workshop_name` spellings of the leaf entries valid directly
/// under `parent`, including the `gamemodes.general` leaves a mode inherits.
/// Mirrors `lookup`/`mode_inherited_entry`: `elimination` inherits only
/// [`ELIMINATION_GENERAL_KEYS`], other modes inherit the full set only when
/// they have declared entries, and `general` itself has no fallback.
pub(crate) fn key_spellings(parent: &[PathPart<'_>]) -> Vec<&'static str> {
    let mut spellings: Vec<&'static str> = entries()
        .filter(|entry| {
            entry.path.len() == parent.len() + 1
                && entry.path[..parent.len()]
                    .iter()
                    .zip(parent.iter())
                    .all(|(a, b)| a == b)
                // `%1$s` placeholder names are emission templates, not
                // spellings a source could carry.
                && !entry.workshop_name.contains("%1$s")
        })
        .map(|entry| entry.workshop_name)
        .collect();
    let [PathPart::Part("gamemodes"), PathPart::Part(mode)] = parent else {
        return spellings;
    };
    if *mode == "general" {
        return spellings;
    }
    if *mode == "elimination" {
        spellings.extend(entries().filter_map(|entry| {
            let [
                PathPart::Part("gamemodes"),
                PathPart::Part("general"),
                PathPart::Part(key),
            ] = entry.path
            else {
                return None;
            };
            ELIMINATION_GENERAL_KEYS
                .contains(key)
                .then_some(entry.workshop_name)
        }));
    } else if entries().any(|entry| {
        matches!(
            entry.path,
            [PathPart::Part("gamemodes"), PathPart::Part(declared), ..]
                if *declared == *mode
        )
    }) {
        spellings.extend(entries().filter_map(|entry| {
            let [
                PathPart::Part("gamemodes"),
                PathPart::Part("general"),
                PathPart::Part(_),
            ] = entry.path
            else {
                return None;
            };
            (!entry.workshop_name.contains("%1$s")).then_some(entry.workshop_name)
        }));
    }
    spellings
}

/// The localized name of a game mode.
pub(crate) fn mode_name(key: &str) -> Option<&'static str> {
    name_in(MODE_NAMES, key).or_else(|| name_in(GENERATED_MODE_NAMES, key))
}

/// The localized name of a map.
pub(crate) fn map_name(key: &str) -> Option<&'static str> {
    name_in(MAP_NAMES, key).or_else(|| name_in(GENERATED_MAP_NAMES, key))
}

/// The localized name of a hero.
pub(crate) fn hero_name(key: &str) -> Option<&'static str> {
    name_in(HERO_NAMES, key).or_else(|| name_in(GENERATED_HERO_NAMES, key))
}

/// The localized name of a team.
pub(crate) fn team_name(key: &str) -> Option<&'static str> {
    name_in(TEAM_NAMES, key)
}

/// The localized name of an enum member in a domain.
pub(crate) fn enum_name(domain: &str, member: &str) -> Option<&'static str> {
    ENUM_MEMBERS
        .iter()
        .find(|m| m.domain == domain && m.member == member)
        .map(|m| m.name)
        .or_else(|| {
            GENERATED_ENUM_MEMBERS
                .iter()
                .find(|m| m.domain == domain && m.member == member)
                .map(|m| m.name)
        })
}

pub(crate) fn enum_members(domain: &str) -> impl Iterator<Item = &'static EnumMember> {
    ENUM_MEMBERS
        .iter()
        .chain(GENERATED_ENUM_MEMBERS.iter())
        .filter(move |member| member.domain == domain)
}

/// A human-readable rendering of a path (diagnostics).
pub(crate) fn path_string(path: &[PathPart<'_>]) -> String {
    path.iter()
        .map(|part| match part {
            PathPart::Part(name) => (*name).to_string(),
            PathPart::Team => "<team>".to_string(),
            PathPart::Hero => "<hero>".to_string(),
        })
        .collect::<Vec<_>>()
        .join(".")
}
