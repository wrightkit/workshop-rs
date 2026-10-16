use crate::core::source::Span;

use super::SubroutineId;
pub(crate) use crate::program::shared::{EventTarget, EventTeam, PlayerEventKind};

#[derive(Debug, Clone)]
pub(crate) enum Event {
    Global,
    EachPlayer,
    EachPlayerWithFilters {
        team: EventTeam,
        target: EventTarget,
    },
    Player {
        kind: PlayerEventKind,
        team: EventTeam,
        target: EventTarget,
    },
    Subroutine {
        subroutine: SubroutineId,
        name_span: Option<Span>,
    },
}
