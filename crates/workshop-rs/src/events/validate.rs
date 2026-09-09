use crate::catalog::{Catalog, Kind};
use crate::core::error::WorkshopError;
use crate::wir;

pub(crate) fn validate_event(
    event: &wir::Event,
    span: Option<crate::core::source::Span>,
    catalog: &Catalog,
    errors: &mut Vec<WorkshopError>,
) {
    let (id, filters) = match event {
        wir::Event::Global => ("global", None),
        wir::Event::EachPlayer => ("eachPlayer", None),
        wir::Event::EachPlayerWithFilters { team, target } => ("eachPlayer", Some((*team, target))),
        wir::Event::Player { kind, team, target } => (kind.catalog_id(), Some((*team, target))),
        wir::Event::Subroutine(_) => ("subroutine", None),
    };
    if catalog.entry(Kind::Event, id).is_none() {
        errors.push(WorkshopError::Unknown {
            kind: "event",
            spelling: id.to_string(),
            locale: crate::catalog::Locale::new("en-US"),
            span,
        });
        return;
    }
    let Some((team, target)) = filters else {
        return;
    };
    let en = crate::catalog::Locale::new("en-US");
    let team_member = match team {
        wir::EventTeam::All => "ALL",
        wir::EventTeam::Team1 => "TEAM_1",
        wir::EventTeam::Team2 => "TEAM_2",
    };
    if catalog
        .enum_spelling("EventTeam", &en, team_member)
        .is_none()
    {
        errors.push(WorkshopError::Unknown {
            kind: "event team",
            spelling: team_member.to_string(),
            locale: en.clone(),
            span,
        });
    }
    let target_member = match target {
        wir::EventTarget::All => Some("ALL".to_string()),
        wir::EventTarget::Slot(slot) => Some(format!("SLOT_{slot}")),
        wir::EventTarget::Hero(hero) => {
            if catalog.enum_spelling("Hero", &en, hero).is_none() {
                errors.push(WorkshopError::Unknown {
                    kind: "event player",
                    spelling: hero.clone(),
                    locale: en.clone(),
                    span,
                });
            }
            None
        }
    };
    if let Some(target_member) = target_member {
        if catalog
            .enum_spelling("EventPlayer", &en, &target_member)
            .is_none()
        {
            errors.push(WorkshopError::Unknown {
                kind: "event player",
                spelling: target_member,
                locale: en,
                span,
            });
        }
    }
}
