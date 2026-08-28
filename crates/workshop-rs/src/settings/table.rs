//! Fixture-evidenced settings emission table (#86).
//!
//! PROVENANCE: observed from the pinned oracle 9.7.10 en-US output of the
//! oracle-success settings programs (`compile.workshop` settings section of
//! the committed snapshots pixelart/santa/broken-weapons/client-to-server,
//! plus the parabola/crosshair/inputhud oracle runs) at OverPy commit
//! `eea67ad`. This is observed-behavior data, not copied OverPy source
//! (LICENSE-BOUNDARY policy). Additions to the table (e.g. the acquired
//! candidate snapshots) are data-only.

use serde::Deserialize;
use serde_json::Value;
use std::sync::OnceLock;

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
pub fn localized_name(locale: &str, section: &str, english: &str) -> Option<&'static str> {
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
pub enum KeyKind {
    /// A presence-only extension setting.
    Flag,
    /// A quoted string (`Description: "..."`).
    String,
    /// A boolean rendered `On`/`Off`.
    Bool,
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

/// One segment of an exact settings path.
#[derive(Debug, Clone, Copy)]
pub enum PathPart<'a> {
    /// A literal key (mode names under `gamemodes` are literal keys too:
    /// per-key subsets are exact-path entries, #86).
    Part(&'a str),
    /// Any team slot (allTeams), rendered through [`team_name`].
    Team,
    /// Any hero-config slot, rendered through [`hero_name`].
    Hero,
}

impl<'b> PartialEq<PathPart<'b>> for PathPart<'_> {
    fn eq(&self, other: &PathPart<'b>) -> bool {
        match (self, other) {
            (PathPart::Part(left), PathPart::Part(right)) => left == right,
            (PathPart::Team, PathPart::Team) => true,
            (PathPart::Hero, PathPart::Hero) => true,
            _ => false,
        }
    }
}

impl Eq for PathPart<'_> {}

/// One table entry: an exact key path, its workshop name, and its kind.
#[derive(Debug, Clone, Copy)]
pub struct TableEntry {
    pub path: &'static [PathPart<'static>],
    pub workshop_name: &'static str,
    pub kind: KeyKind,
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

/// The fixture-evidenced settings surface.
///
/// Slot sets (evidenced): teams {allTeams}, heroes {mei} config groups +
/// the 10 ListHero names. `enabled: true` is not evidenced; it renders with
/// no prefix. Keys outside this table (e.g. team1Slots, scoreToWin,
/// gamemodeStartTrigger, spawnHealthPacks, healthPackRespawnTime%,
/// abilityCooldown%, healingReceived%, primaryFireKb%, enableSpawningWithUlt,
/// resetPlayersAfterGoalScored, scoreLeadToWin, gameLengthInSec,
/// heroes.<team>.general, roleLimit under general, heroLimit under a named
/// mode) are `settings-unknown-key` at validation (only evidenced in
/// oracle-failing programs; corpus-bounded).
pub static ENTRIES: &[TableEntry] = &[
    // main
    entry!(
        [PathPart::Part("main"), PathPart::Part("description")],
        "Description",
        KeyKind::String
    ),
    entry!(
        [PathPart::Part("main"), PathPart::Part("modeName")],
        "Mode Name",
        KeyKind::String
    ),
    // lobby
    entry!(
        [PathPart::Part("lobby"), PathPart::Part("ffaSlots")],
        "Max FFA Players",
        KeyKind::Number
    ),
    entry!(
        [PathPart::Part("lobby"), PathPart::Part("mapRotation")],
        "Map Rotation",
        KeyKind::Enum("mapRotation")
    ),
    entry!(
        [PathPart::Part("lobby"), PathPart::Part("spectatorSlots")],
        "Max Spectators",
        KeyKind::Number
    ),
    entry!(
        [PathPart::Part("lobby"), PathPart::Part("matchVoiceChat")],
        "Match Voice Chat",
        KeyKind::Enum("matchVoiceChat")
    ),
    entry!(
        [PathPart::Part("lobby"), PathPart::Part("team1Slots")],
        "Max Team 1 Players",
        KeyKind::Number
    ),
    entry!(
        [PathPart::Part("lobby"), PathPart::Part("team2Slots")],
        "Max Team 2 Players",
        KeyKind::Number
    ),
    entry!(
        [PathPart::Part("lobby"), PathPart::Part("returnToLobby")],
        "Return To Lobby",
        KeyKind::Enum("returnToLobby")
    ),
    entry!(
        [
            PathPart::Part("lobby"),
            PathPart::Part("allowPlayersInQueue")
        ],
        "Allow Players Who Are In Queue",
        KeyKind::Bool
    ),
    entry!(
        [
            PathPart::Part("lobby"),
            PathPart::Part("swapTeamsAfterMatch")
        ],
        "Swap Teams After Match",
        KeyKind::Bool
    ),
    // gamemodes.<mode> — per-key subsets (exact-path entries, #86):
    // enabledMaps under modes {assault, control, escort, hybrid, skirmish,
    // ffa}; enabled/roleLimit/enableCompetitiveRules under {assault, control,
    // escort, hybrid}; heroLimit/respawnTime%/enableHeroSwitching/
    // enableRandomHeroes under general only (general is a literal group name,
    // not a mode slot).
    entry!(
        [
            PathPart::Part("gamemodes"),
            PathPart::Part("assault"),
            PathPart::Part("enabled")
        ],
        "enabled",
        KeyKind::Bool
    ),
    entry!(
        [
            PathPart::Part("gamemodes"),
            PathPart::Part("control"),
            PathPart::Part("enabled")
        ],
        "enabled",
        KeyKind::Bool
    ),
    entry!(
        [
            PathPart::Part("gamemodes"),
            PathPart::Part("escort"),
            PathPart::Part("enabled")
        ],
        "enabled",
        KeyKind::Bool
    ),
    entry!(
        [
            PathPart::Part("gamemodes"),
            PathPart::Part("hybrid"),
            PathPart::Part("enabled")
        ],
        "enabled",
        KeyKind::Bool
    ),
    entry!(
        [
            PathPart::Part("gamemodes"),
            PathPart::Part("assault"),
            PathPart::Part("enabledMaps")
        ],
        "enabled maps",
        KeyKind::ListMap
    ),
    entry!(
        [
            PathPart::Part("gamemodes"),
            PathPart::Part("control"),
            PathPart::Part("enabledMaps")
        ],
        "enabled maps",
        KeyKind::ListMap
    ),
    entry!(
        [
            PathPart::Part("gamemodes"),
            PathPart::Part("escort"),
            PathPart::Part("enabledMaps")
        ],
        "enabled maps",
        KeyKind::ListMap
    ),
    entry!(
        [
            PathPart::Part("gamemodes"),
            PathPart::Part("hybrid"),
            PathPart::Part("enabledMaps")
        ],
        "enabled maps",
        KeyKind::ListMap
    ),
    entry!(
        [
            PathPart::Part("gamemodes"),
            PathPart::Part("skirmish"),
            PathPart::Part("enabledMaps")
        ],
        "enabled maps",
        KeyKind::ListMap
    ),
    entry!(
        [
            PathPart::Part("gamemodes"),
            PathPart::Part("assault"),
            PathPart::Part("disabledMaps")
        ],
        "disabled maps",
        KeyKind::ListMap
    ),
    entry!(
        [
            PathPart::Part("gamemodes"),
            PathPart::Part("skirmish"),
            PathPart::Part("disabledMaps")
        ],
        "disabled maps",
        KeyKind::ListMap
    ),
    entry!(
        [
            PathPart::Part("gamemodes"),
            PathPart::Part("ffa"),
            PathPart::Part("enabledMaps")
        ],
        "enabled maps",
        KeyKind::ListMap
    ),
    entry!(
        [
            PathPart::Part("gamemodes"),
            PathPart::Part("tdm"),
            PathPart::Part("enabledMaps")
        ],
        "enabled maps",
        KeyKind::ListMap
    ),
    entry!(
        [
            PathPart::Part("gamemodes"),
            PathPart::Part("assault"),
            PathPart::Part("roleLimit")
        ],
        "Limit Roles",
        KeyKind::Enum("roleLimit")
    ),
    entry!(
        [
            PathPart::Part("gamemodes"),
            PathPart::Part("control"),
            PathPart::Part("roleLimit")
        ],
        "Limit Roles",
        KeyKind::Enum("roleLimit")
    ),
    entry!(
        [
            PathPart::Part("gamemodes"),
            PathPart::Part("escort"),
            PathPart::Part("roleLimit")
        ],
        "Limit Roles",
        KeyKind::Enum("roleLimit")
    ),
    entry!(
        [
            PathPart::Part("gamemodes"),
            PathPart::Part("hybrid"),
            PathPart::Part("roleLimit")
        ],
        "Limit Roles",
        KeyKind::Enum("roleLimit")
    ),
    entry!(
        [
            PathPart::Part("gamemodes"),
            PathPart::Part("general"),
            PathPart::Part("roleLimit")
        ],
        "Limit Roles",
        KeyKind::Enum("roleLimit")
    ),
    entry!(
        [
            PathPart::Part("gamemodes"),
            PathPart::Part("assault"),
            PathPart::Part("enableCompetitiveRules")
        ],
        "Competitive Rules",
        KeyKind::Bool
    ),
    entry!(
        [
            PathPart::Part("gamemodes"),
            PathPart::Part("control"),
            PathPart::Part("enableCompetitiveRules")
        ],
        "Competitive Rules",
        KeyKind::Bool
    ),
    entry!(
        [
            PathPart::Part("gamemodes"),
            PathPart::Part("escort"),
            PathPart::Part("enableCompetitiveRules")
        ],
        "Competitive Rules",
        KeyKind::Bool
    ),
    entry!(
        [
            PathPart::Part("gamemodes"),
            PathPart::Part("hybrid"),
            PathPart::Part("enableCompetitiveRules")
        ],
        "Competitive Rules",
        KeyKind::Bool
    ),
    // gamemodes.general
    entry!(
        [
            PathPart::Part("gamemodes"),
            PathPart::Part("general"),
            PathPart::Part("enableCompetitiveRules")
        ],
        "Competitive Rules",
        KeyKind::Bool
    ),
    entry!(
        [
            PathPart::Part("gamemodes"),
            PathPart::Part("general"),
            PathPart::Part("enablePerks")
        ],
        "Enable Perks",
        KeyKind::Bool
    ),
    entry!(
        [
            PathPart::Part("gamemodes"),
            PathPart::Part("general"),
            PathPart::Part("heroLimit")
        ],
        "Hero Limit",
        KeyKind::Enum("heroLimit")
    ),
    entry!(
        [
            PathPart::Part("gamemodes"),
            PathPart::Part("general"),
            PathPart::Part("respawnTime%")
        ],
        "Respawn Time Scalar",
        KeyKind::Percent
    ),
    entry!(
        [
            PathPart::Part("gamemodes"),
            PathPart::Part("general"),
            PathPart::Part("enableHeroSwitching")
        ],
        "Allow Hero Switching",
        KeyKind::Bool
    ),
    entry!(
        [
            PathPart::Part("gamemodes"),
            PathPart::Part("general"),
            PathPart::Part("enableRandomHeroes")
        ],
        "Respawn As Random Hero",
        KeyKind::Bool
    ),
    entry!(
        [
            PathPart::Part("gamemodes"),
            PathPart::Part("general"),
            PathPart::Part("gameModeStartTrigger")
        ],
        "Game Mode Start",
        KeyKind::Enum("gameModeStartTrigger")
    ),
    entry!(
        [
            PathPart::Part("gamemodes"),
            PathPart::Part("assault"),
            PathPart::Part("gameModeStartTrigger")
        ],
        "Game Mode Start",
        KeyKind::Enum("gameModeStartTrigger")
    ),
    entry!(
        [
            PathPart::Part("gamemodes"),
            PathPart::Part("assault"),
            PathPart::Part("tankPassiveHealthBonus")
        ],
        "Tank Role Passive Health Bonus",
        KeyKind::Enum("tankPassiveHealthBonus")
    ),
    entry!(
        [
            PathPart::Part("gamemodes"),
            PathPart::Part("general"),
            PathPart::Part("spawnHealthPacks")
        ],
        "Spawn Health Packs",
        KeyKind::Enum("spawnHealthPacks")
    ),
    // heroes.<team>
    entry!(
        [
            PathPart::Part("heroes"),
            PathPart::Team,
            PathPart::Part("enabledHeroes")
        ],
        "enabled heroes",
        KeyKind::ListHero
    ),
    entry!(
        [
            PathPart::Part("heroes"),
            PathPart::Team,
            PathPart::Part("disabledHeroes")
        ],
        "disabled heroes",
        KeyKind::ListHero
    ),
    entry!(
        [
            PathPart::Part("heroes"),
            PathPart::Part("general"),
            PathPart::Part("disabledHeroes")
        ],
        "disabled heroes",
        KeyKind::ListHero
    ),
    // heroes.<team>.<hero> config groups
    entry!(
        [
            PathPart::Part("heroes"),
            PathPart::Team,
            PathPart::Hero,
            PathPart::Part("enablePrimaryFire")
        ],
        "Primary Fire",
        KeyKind::Bool
    ),
    entry!(
        [
            PathPart::Part("heroes"),
            PathPart::Team,
            PathPart::Hero,
            PathPart::Part("enableSecondaryFire")
        ],
        "Secondary Fire",
        KeyKind::Bool
    ),
    entry!(
        [
            PathPart::Part("heroes"),
            PathPart::Team,
            PathPart::Hero,
            PathPart::Part("enableAbility1")
        ],
        "Ability 1",
        KeyKind::Bool
    ),
    entry!(
        [
            PathPart::Part("heroes"),
            PathPart::Team,
            PathPart::Hero,
            PathPart::Part("enableAbility2")
        ],
        "Ability 2",
        KeyKind::Bool
    ),
    entry!(
        [
            PathPart::Part("heroes"),
            PathPart::Team,
            PathPart::Hero,
            PathPart::Part("health%")
        ],
        "Health",
        KeyKind::Percent
    ),
    entry!(
        [
            PathPart::Part("heroes"),
            PathPart::Team,
            PathPart::Hero,
            PathPart::Part("passiveUltGen%")
        ],
        "Ultimate Generation - Passive Blizzard",
        KeyKind::Percent
    ),
    entry!(
        [
            PathPart::Part("heroes"),
            PathPart::Team,
            PathPart::Hero,
            PathPart::Part("combatUltGen%")
        ],
        "Ultimate Generation - Combat Blizzard",
        KeyKind::Percent
    ),
];

/// A slot name mapping (key -> localized workshop name).
#[derive(Debug, Clone, Copy)]
pub struct NameMap {
    pub key: &'static str,
    pub name: &'static str,
}

/// Game-mode names (evidenced: assault, control, escort, hybrid, skirmish,
/// ffa, tdm, general).
pub static MODE_NAMES: &[NameMap] = &[
    NameMap {
        key: "assault",
        name: "Assault",
    },
    NameMap {
        key: "control",
        name: "Control",
    },
    NameMap {
        key: "escort",
        name: "Escort",
    },
    NameMap {
        key: "hybrid",
        name: "Hybrid",
    },
    NameMap {
        key: "skirmish",
        name: "Skirmish",
    },
    NameMap {
        key: "ffa",
        name: "Deathmatch",
    },
    NameMap {
        key: "tdm",
        name: "Team Deathmatch",
    },
    NameMap {
        key: "general",
        name: "General",
    },
];

/// Map names inside `enabledMaps` lists.
pub static MAP_NAMES: &[NameMap] = &[
    NameMap {
        key: "workshopIsland",
        name: "Workshop Island",
    },
    NameMap {
        key: "kingsRowWinter",
        name: "King's Row Winter",
    },
];

/// Hero names inside hero lists and hero-config groups.
pub static HERO_NAMES: &[NameMap] = &[
    NameMap {
        key: "anran",
        name: "Anran",
    },
    NameMap {
        key: "ana",
        name: "Ana",
    },
    NameMap {
        key: "ashe",
        name: "Ashe",
    },
    NameMap {
        key: "bastion",
        name: "Bastion",
    },
    NameMap {
        key: "baptiste",
        name: "Baptiste",
    },
    NameMap {
        key: "brigitte",
        name: "Brigitte",
    },
    NameMap {
        key: "cassidy",
        name: "Cassidy",
    },
    NameMap {
        key: "dmon",
        name: "D.Mon",
    },
    NameMap {
        key: "domina",
        name: "Domina",
    },
    NameMap {
        key: "dva",
        name: "D.Va",
    },
    NameMap {
        key: "doomfist",
        name: "Doomfist",
    },
    NameMap {
        key: "echo",
        name: "Echo",
    },
    NameMap {
        key: "emre",
        name: "Emre",
    },
    NameMap {
        key: "freja",
        name: "Freja",
    },
    NameMap {
        key: "genji",
        name: "Genji",
    },
    NameMap {
        key: "hanzo",
        name: "Hanzo",
    },
    NameMap {
        key: "moira",
        name: "Moira",
    },
    NameMap {
        key: "reinhardt",
        name: "Reinhardt",
    },
    NameMap {
        key: "hammond",
        name: "Wrecking Ball",
    },
    NameMap {
        key: "hazard",
        name: "Hazard",
    },
    NameMap {
        key: "illari",
        name: "Illari",
    },
    NameMap {
        key: "juno",
        name: "Juno",
    },
    NameMap {
        key: "jetpackCat",
        name: "Jetpack Cat",
    },
    NameMap {
        key: "junkerQueen",
        name: "Junker Queen",
    },
    NameMap {
        key: "junkrat",
        name: "Junkrat",
    },
    NameMap {
        key: "kiriko",
        name: "Kiriko",
    },
    NameMap {
        key: "lucio",
        name: "Lúcio",
    },
    NameMap {
        key: "mauga",
        name: "Mauga",
    },
    NameMap {
        key: "mercy",
        name: "Mercy",
    },
    NameMap {
        key: "mizuki",
        name: "Mizuki",
    },
    NameMap {
        key: "orisa",
        name: "Orisa",
    },
    NameMap {
        key: "pharah",
        name: "Pharah",
    },
    NameMap {
        key: "reaper",
        name: "Reaper",
    },
    NameMap {
        key: "roadhog",
        name: "Roadhog",
    },
    NameMap {
        key: "shion",
        name: "Shion",
    },
    NameMap {
        key: "sierra",
        name: "Sierra",
    },
    NameMap {
        key: "sigma",
        name: "Sigma",
    },
    NameMap {
        key: "ramattra",
        name: "Ramattra",
    },
    NameMap {
        key: "lifeweaver",
        name: "Lifeweaver",
    },
    NameMap {
        key: "sojourn",
        name: "Sojourn",
    },
    NameMap {
        key: "soldier",
        name: "Soldier: 76",
    },
    NameMap {
        key: "sombra",
        name: "Sombra",
    },
    NameMap {
        key: "symmetra",
        name: "Symmetra",
    },
    NameMap {
        key: "torbjorn",
        name: "Torbjörn",
    },
    NameMap {
        key: "tracer",
        name: "Tracer",
    },
    NameMap {
        key: "venture",
        name: "Venture",
    },
    NameMap {
        key: "widowmaker",
        name: "Widowmaker",
    },
    NameMap {
        key: "winston",
        name: "Winston",
    },
    NameMap {
        key: "wuyang",
        name: "Wuyang",
    },
    NameMap {
        key: "wreckingBall",
        name: "Wrecking Ball",
    },
    NameMap {
        key: "zarya",
        name: "Zarya",
    },
    NameMap {
        key: "zenyatta",
        name: "Zenyatta",
    },
    NameMap {
        key: "mei",
        name: "Mei",
    },
];

include!("data/generated_map_entries.rs");
include!("data/generated_hero_entries.rs");
include!("data/generated_mode_entries.rs");

/// Team names inside `heroes` (evidenced: allTeams).
pub static TEAM_NAMES: &[NameMap] = &[
    NameMap {
        key: "allTeams",
        name: "General",
    },
    NameMap {
        key: "team1",
        name: "Team 1",
    },
    NameMap {
        key: "team2",
        name: "Team 2",
    },
];

/// An enum domain member (domain -> localized workshop name).
#[derive(Debug, Clone, Copy)]
pub struct EnumMember {
    pub domain: &'static str,
    pub member: &'static str,
    pub name: &'static str,
}

include!("data/generated_entries.rs");
include!("data/generated_hero_settings.rs");

/// Enum member names per domain. `roleLimit` has exactly one evidenced
/// member ("2OfEachRolePerTeam", pixelart + broken-weapons); "off" appears
/// only in the not-acquired skirmish_elim source and is rejected
/// (settings-unknown-value) until a snapshot evidences it. `heroLimit` "off"
/// is evidenced (santa, clientToServer, parabola, crosshair, inputhud).
pub static ENUM_MEMBERS: &[EnumMember] = &[
    EnumMember {
        domain: "mapRotation",
        member: "afterAGame",
        name: "After A Game",
    },
    EnumMember {
        domain: "matchVoiceChat",
        member: "enabled",
        name: "Enabled",
    },
    EnumMember {
        domain: "returnToLobby",
        member: "never",
        name: "Never",
    },
    EnumMember {
        domain: "returnToLobby",
        member: "afterAGame",
        name: "After A Game",
    },
    EnumMember {
        domain: "gameModeStartTrigger",
        member: "immediately",
        name: "Immediately",
    },
    EnumMember {
        domain: "gameModeStartTrigger",
        member: "manual",
        name: "Manual",
    },
    EnumMember {
        domain: "spawnHealthPacks",
        member: "disabled",
        name: "Disabled",
    },
    EnumMember {
        domain: "roleLimit",
        member: "2OfEachRolePerTeam",
        name: "2 Of Each Role Per Team",
    },
    EnumMember {
        domain: "roleLimit",
        member: "1Tank2Offense2Support",
        name: "1 Tank 2 Offense 2 Support",
    },
    EnumMember {
        domain: "tankPassiveHealthBonus",
        member: "alwaysEnabled",
        name: "Always Enabled",
    },
    EnumMember {
        domain: "tankPassiveHealthBonus",
        member: "disabled",
        name: "Disabled",
    },
    EnumMember {
        domain: "heroLimit",
        member: "off",
        name: "Off",
    },
];

/// Look up a settings leaf entry by its exact path.
pub fn lookup(path: &[PathPart<'_>]) -> Option<&'static TableEntry> {
    entries().find(|entry| {
        entry.path.len() == path.len() && entry.path.iter().zip(path.iter()).all(|(a, b)| a == b)
    })
}

/// Iterate the reviewed settings inventory with the hand-written projection
/// taking precedence over the generated export projection. Duplicate paths
/// are represented once in the semantic catalog while the parser and emitter
/// continue to use the same lookup table.
pub fn entries() -> impl Iterator<Item = &'static TableEntry> {
    deduplicated_entries(ENTRIES.iter().chain(GENERATED_ENTRIES.iter()))
}

/// Iterate both catalog projections without applying effective lookup
/// precedence. The semantic validator uses this to compare duplicate paths
/// instead of allowing `entries()` to hide stale or conflicting data.
pub(crate) fn raw_entries() -> impl Iterator<Item = &'static TableEntry> {
    ENTRIES.iter().chain(GENERATED_ENTRIES.iter())
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
pub fn ability_slot_for_path(path: &[PathPart<'_>]) -> Option<&'static str> {
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
        Some(PathPart::Part("enableAutomaticFire" | "enableScoping")) => Some("primaryFire"),
        _ => None,
    }
}

/// Resolve an evidence-backed hero-specific setting label.
pub fn hero_setting_name(hero: &str, key: &str, locale: &str) -> Option<&'static str> {
    let generated = GENERATED_HERO_SETTING_NAMES
        .iter()
        .find(|entry| entry.hero == hero && entry.key == key)
        .and_then(|entry| entry.localized(locale));
    generated
        .or_else(|| {
            hero_setting_aliases()
                .iter()
                .find(|alias| {
                    alias.hero == hero
                        && alias.key == key
                        && alias.locale.eq_ignore_ascii_case(locale)
                })
                .map(|alias| alias.display.as_str())
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

/// Return explicit applicability evidence for a hero setting from the
/// reviewed hero-setting export. `None` means the hero is not in the reviewed
/// roster; otherwise the exported presence/absence is the effective setting
/// applicability for this catalog surface.
pub fn hero_setting_applicability(hero: &str, key: &str) -> Option<bool> {
    hero_name(hero)?;
    // These common controls are represented by the Workshop hero settings
    // table for every topology-valid hero, while the export only carries
    // localized labels for a subset. Do not turn that presentation gap into a
    // false negative for typed queries.
    if matches!(
        key,
        "health%"
            | "enablePrimaryFire"
            | "enableSecondaryFire"
            | "enableAbility1"
            | "enableAbility2"
            | "enableAbility3"
            | "combatUltGen%"
            | "passiveUltGen%"
    ) {
        return None;
    }
    let evidenced = GENERATED_HERO_SETTING_NAMES.iter().any(|entry| {
        entry.key == key
            && entry
                .locales
                .iter()
                .any(|(_, value)| !value.trim().is_empty())
    });
    evidenced.then(|| {
        GENERATED_HERO_SETTING_NAMES.iter().any(|entry| {
            entry.hero == hero
                && entry.key == key
                && entry
                    .locales
                    .iter()
                    .any(|(_, value)| !value.trim().is_empty())
        })
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
pub fn hero_setting_alias(hero: &str, key: &str, locale: &str, display: &str) -> bool {
    hero_setting_aliases().iter().any(|alias| {
        alias.hero == hero
            && alias.key == key
            && alias.locale.eq_ignore_ascii_case(locale)
            && alias.display == display
    })
}

fn name_in(maps: &[NameMap], key: &str) -> Option<&'static str> {
    maps.iter().find(|m| m.key == key).map(|m| m.name)
}

/// The localized name of a game mode.
pub fn mode_name(key: &str) -> Option<&'static str> {
    name_in(MODE_NAMES, key).or_else(|| name_in(GENERATED_MODE_NAMES, key))
}

/// The localized name of a map.
pub fn map_name(key: &str) -> Option<&'static str> {
    name_in(MAP_NAMES, key).or_else(|| name_in(GENERATED_MAP_NAMES, key))
}

/// The localized name of a hero.
pub fn hero_name(key: &str) -> Option<&'static str> {
    name_in(HERO_NAMES, key).or_else(|| name_in(GENERATED_HERO_NAMES, key))
}

/// The localized name of a team.
pub fn team_name(key: &str) -> Option<&'static str> {
    name_in(TEAM_NAMES, key)
}

/// The localized name of an enum member in a domain.
pub fn enum_name(domain: &str, member: &str) -> Option<&'static str> {
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

/// A human-readable rendering of a path (diagnostics).
pub fn path_string(path: &[PathPart<'_>]) -> String {
    path.iter()
        .map(|part| match part {
            PathPart::Part(name) => (*name).to_string(),
            PathPart::Team => "<team>".to_string(),
            PathPart::Hero => "<hero>".to_string(),
        })
        .collect::<Vec<_>>()
        .join(".")
}
