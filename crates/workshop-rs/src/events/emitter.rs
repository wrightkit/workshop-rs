// Emitter behavior owned by the Workshop events domain.

use super::*;

impl Emitter<'_> {
    pub(super) fn event_filters(
        &mut self,
        team: wir::EventTeam,
        target: &wir::EventTarget,
    ) -> Result<()> {
        let team = match team {
            wir::EventTeam::All => "ALL",
            wir::EventTeam::Team1 => "TEAM_1",
            wir::EventTeam::Team2 => "TEAM_2",
        };
        let team = self.enum_spelling("EventTeam", team)?;
        self.line(2, &format!("{team};"))?;
        let target = match target {
            wir::EventTarget::All => self.enum_spelling("EventPlayer", "ALL")?,
            wir::EventTarget::Slot(slot) => {
                self.enum_spelling("EventPlayer", &format!("SLOT_{slot}"))?
            }
            wir::EventTarget::Hero(hero) => self.enum_spelling("Hero", hero)?,
        };
        self.line(2, &format!("{target};"))?;
        Ok(())
    }
}
