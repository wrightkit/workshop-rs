//! Canonical Workshop event forms and filters.

use super::SubroutineId;

/// The team filter attached to a player-scoped Workshop event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventTeam {
    All,
    Team1,
    Team2,
}

/// The player filter attached to a player-scoped Workshop event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventTarget {
    All,
    Slot(u8),
    Hero(String),
}

/// A non-ongoing player event identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerEventKind {
    DealtDamage,
    DealtFinalBlow,
    DealtHealing,
    DealtKnockback,
    Died,
    EarnedElimination,
    Joined,
    Left,
    ReceivedHealing,
    ReceivedKnockback,
    TookDamage,
}

impl PlayerEventKind {
    /// The locale-independent catalog identity for this event.
    pub fn catalog_id(self) -> &'static str {
        match self {
            PlayerEventKind::DealtDamage => "playerDealtDamage",
            PlayerEventKind::DealtFinalBlow => "playerDealtFinalBlow",
            PlayerEventKind::DealtHealing => "playerDealtHealing",
            PlayerEventKind::DealtKnockback => "playerDealtKnockback",
            PlayerEventKind::Died => "playerDied",
            PlayerEventKind::EarnedElimination => "playerEarnedElimination",
            PlayerEventKind::Joined => "playerJoined",
            PlayerEventKind::Left => "playerLeft",
            PlayerEventKind::ReceivedHealing => "playerReceivedHealing",
            PlayerEventKind::ReceivedKnockback => "playerReceivedKnockback",
            PlayerEventKind::TookDamage => "playerTookDamage",
        }
    }
}

/// A workshop event.
#[derive(Debug, Clone)]
pub enum Event {
    /// `Ongoing - Global` (from `@Event global`).
    Global,
    /// `Ongoing - Each Player` (from `@Event eachPlayer`).
    EachPlayer,
    /// `Ongoing - Each Player` with its canonical team/player filters.
    EachPlayerWithFilters {
        team: EventTeam,
        target: EventTarget,
    },
    /// A player-scoped Workshop event with canonical filters.
    Player {
        kind: PlayerEventKind,
        team: EventTeam,
        target: EventTarget,
    },
    /// A subroutine body (`def name():`), referencing the subroutine.
    Subroutine(SubroutineId),
}
