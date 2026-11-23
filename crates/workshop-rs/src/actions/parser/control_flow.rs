use super::*;

impl ParseContext<'_> {
    pub(super) fn if_group(&mut self, if_marker: Span) -> Result<wir::ActionId> {
        self.expect(TokenKind::LParen, "expected '(' after 'If'")?;
        let mut condition = Some(self.condition_value()?);
        self.expect(TokenKind::RParen, "expected ')' after If condition")?;
        self.expect_semi("expected ';' after If condition")?;

        let mut marker = if_marker;
        let mut branches = Vec::new();
        let (end_marker_span, end) = loop {
            let (body, stop) = self.actions_until_end()?;
            branches.push(wir::IfBranch {
                condition,
                body,
                marker_span: Some(marker),
            });
            match stop {
                Stop::End => {
                    let end = self.consume_end_marker()?;
                    break (Some(end), end.end);
                }
                Stop::ElseIf => {
                    marker = self.consume_keyword("elseIf")?;
                    self.expect(TokenKind::LParen, "expected '(' after 'Else If'")?;
                    condition = Some(self.condition_value()?);
                    self.expect(TokenKind::RParen, "expected ')' after Else If condition")?;
                    self.expect_semi("expected ';' after Else If condition")?;
                }
                Stop::Else => {
                    marker = self.consume_keyword("else")?;
                    condition = None;
                    self.expect_semi("expected ';' after 'Else'")?;
                }
                Stop::SectionClosed => {
                    // The oracle closes a rule-final if/if-else with the
                    // enclosing actions-section `}` (no trailing `End;`,
                    // #87). Rewind so the enclosing actions section consumes
                    // that `}` and the rule's own `}` stays intact.
                    self.pos -= 1;
                    break (None, self.previous_span().1);
                }
            }
        };
        let action = Action::If {
            branches,
            span: Some(Span::new(self.file(), if_marker.start, end)),
            end_marker_span,
        };
        Ok(self.target.actions.push(action))
    }

    pub(super) fn actions_until_matching_end(
        &mut self,
        missing_end_message: &str,
    ) -> Result<(Vec<wir::ActionId>, Span)> {
        let (body, stop) = self.actions_until_end()?;
        if stop != Stop::End {
            return Err(self.malformed(missing_end_message, self.previous()));
        }
        Ok((body, self.consume_end_marker()?))
    }

    fn consume_end_marker(&mut self) -> Result<Span> {
        let marker = self.consume_keyword("end")?;
        self.expect_semi("expected ';' after 'End'")?;
        Ok(marker)
    }
}
