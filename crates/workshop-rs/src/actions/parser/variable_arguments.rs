use super::*;

impl ParseContext<'_> {
    pub(super) fn player_variable_arg(&mut self) -> Result<(wir::ValueId, wir::PlayerVarId, Span)> {
        let player = self.value()?;
        self.expect(TokenKind::Comma, "expected ',' after player")?;
        let (name, name_start, name_end) = self.phrase()?;
        let variable = self.player_by_name(&name)?;
        Ok((
            player,
            variable,
            Span::new(self.file(), name_start, name_end),
        ))
    }

    pub(super) fn global_variable_arg(&mut self) -> Result<(wir::GlobalVarId, Span)> {
        let (name, name_start, name_end) = self.phrase()?;
        let variable = self.global_by_name(&name)?;
        Ok((variable, Span::new(self.file(), name_start, name_end)))
    }
}
