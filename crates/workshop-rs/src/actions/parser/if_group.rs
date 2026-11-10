use super::*;

impl ParseContext<'_> {
    pub(crate) fn if_group(&mut self, start: Position) -> Result<wir::ActionId> {
        self.expect(TokenKind::LParen, "expected '(' after 'If'")?;
        let condition = self.condition_value()?;
        self.expect(TokenKind::RParen, "expected ')' after If condition")?;
        self.expect_semi("expected ';' after If condition")?;

        let mut branches = Vec::new();
        let mut stop = {
            let (body, stop) = self.actions_until_end()?;
            branches.push(wir::IfBranch {
                condition: Some(condition),
                body,
            });
            stop
        };

        let end_pos;
        loop {
            match stop {
                Stop::End => {
                    self.consume_keyword("end")?;
                    end_pos = self.expect_semi("expected ';' after 'End'")?;
                    break;
                }
                Stop::ElseIf => {
                    self.consume_keyword("elseIf")?;
                    self.expect(TokenKind::LParen, "expected '(' after 'Else If'")?;
                    let condition = self.condition_value()?;
                    self.expect(TokenKind::RParen, "expected ')' after Else If condition")?;
                    self.expect_semi("expected ';' after Else If condition")?;
                    let (body, next) = self.actions_until_end()?;
                    branches.push(wir::IfBranch {
                        condition: Some(condition),
                        body,
                    });
                    stop = next;
                }
                Stop::Else => {
                    self.consume_keyword("else")?;
                    self.expect_semi("expected ';' after 'Else'")?;
                    let (body, next) = self.actions_until_end()?;
                    branches.push(wir::IfBranch {
                        condition: None,
                        body,
                    });
                    stop = next;
                }
                Stop::SectionClosed => {
                    // The oracle closes a rule-final if/if-else with the
                    // enclosing actions-section `}` (no trailing `End;`,
                    // #87). Rewind so the enclosing actions section consumes
                    // that `}` and the rule's own `}` stays intact.
                    self.pos -= 1;
                    end_pos = self.previous_span().1;
                    break;
                }
            }
        }
        let action = Action::If {
            branches,
            span: Some(Span::new(self.file(), start, end_pos)),
        };
        Ok(self.target.actions.push(action))
    }
}
