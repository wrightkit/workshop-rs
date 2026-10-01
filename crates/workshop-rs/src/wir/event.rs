//! Canonical Workshop event forms and filters.

use crate::core::source::Span;

use super::SubroutineId;
pub(crate) use crate::program::shared::{EventTarget, EventTeam, PlayerEventKind};

/// A workshop event.
#[derive(Debug, Clone)]
pub(crate) enum Event {
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
    Subroutine {
        subroutine: SubroutineId,
        /// The span of the subroutine name in the event section.
        name_span: Option<Span>,
    },
}
