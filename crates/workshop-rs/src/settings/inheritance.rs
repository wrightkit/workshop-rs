//! Which `general` mode settings the other game modes take as their own.

use super::PathPart;
use super::table::{TableEntry, declared_modes, exact_entry};

pub(super) static ELIMINATION_GENERAL_KEYS: &[&str] = &[
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

/// General keys the pinned OverPy 9.7.17 table declares for `general` and
/// `elimination` only; every other mode rejects them as raw keys (#618).
pub(super) static GENERAL_ONLY_KEYS: &[&str] = &[
    "drawTime",
    "enableTiebreaker",
    "enableWallhack",
    "tiebreakerTime",
    "wallhackEnabledTime",
];

/// Whether `mode` takes `key` from the `general` settings.
pub(super) fn inherits_general_key(mode: &str, key: &str) -> bool {
    if mode == "general" {
        false
    } else if mode == "elimination" {
        ELIMINATION_GENERAL_KEYS.contains(&key)
    } else {
        declared_modes().contains(mode) && !GENERAL_ONLY_KEYS.contains(&key)
    }
}

pub(crate) fn mode_inherited_entry(path: &[PathPart<'_>]) -> Option<&'static TableEntry> {
    let [
        PathPart::Part("gamemodes"),
        PathPart::Part(mode),
        PathPart::Part(key),
    ] = path
    else {
        return None;
    };
    if !inherits_general_key(mode, key) {
        return None;
    }
    exact_entry(&[
        PathPart::Part("gamemodes"),
        PathPart::Part("general"),
        PathPart::Part(key),
    ])
}
