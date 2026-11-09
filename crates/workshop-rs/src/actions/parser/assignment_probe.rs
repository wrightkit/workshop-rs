use super::*;

impl ParseContext<'_> {
    pub(crate) fn try_assignment_action(&mut self) -> Result<Option<wir::ActionId>> {
        self.with_assignment_probe(|p| p.try_assignment_action_inner())
    }

    pub(crate) fn member_assignment_action(
        &mut self,
        saved: usize,
        start: Position,
    ) -> Result<Option<wir::ActionId>> {
        self.with_assignment_probe(|p| p.member_assignment_action_inner(saved, start))
    }

    fn with_assignment_probe(
        &mut self,
        parse: impl FnOnce(&mut Self) -> Result<Option<wir::ActionId>>,
    ) -> Result<Option<wir::ActionId>> {
        let globals = self.target.global_variables.len();
        let players = self.target.player_variables.len();
        let values = self.target.values.len();
        let next_global = self.next_global_index;
        let next_player = self.next_player_index;
        let result = parse(self);
        if matches!(result, Ok(None)) {
            if self.target.global_variables.len() != globals {
                self.globals
                    .retain(|_, binding| binding.id.index() < globals);
            }
            if self.target.player_variables.len() != players {
                self.players
                    .retain(|_, binding| binding.id.index() < players);
            }
            self.target.global_variables.truncate(globals);
            self.target.player_variables.truncate(players);
            self.target.values.truncate(values);
            self.next_global_index = next_global;
            self.next_player_index = next_player;
        }
        result
    }
}
