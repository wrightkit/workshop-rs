use crate::frontend::parser::*;

impl ParseContext<'_> {
    pub(crate) fn event_section(&mut self) -> Result<Event> {
        self.expect_keyword("event")?;
        self.expect(TokenKind::LBrace, "expected '{' after 'event'")?;
        let mut lines: Vec<(String, Option<Span>)> = Vec::new();
        loop {
            match self.peek() {
                Some(Token {
                    kind: TokenKind::RBrace,
                    ..
                }) => {
                    self.pos += 1;
                    break;
                }
                Some(Token {
                    kind: TokenKind::Semi,
                    ..
                }) => {
                    self.pos += 1;
                    lines.push((String::new(), None));
                }
                Some(_) => {
                    let (text, span) = self.line_text()?;
                    lines.push((text, span));
                }
                None => return Err(self.malformed("unexpected end of input in event", self.eof())),
            }
        }
        let Some((name_line, _)) = lines.first() else {
            return Err(self.malformed("event section is empty", self.previous()));
        };
        let name_line = name_line.trim();
        let entry = self
            .catalog
            .resolve(Kind::Event, &self.locale, name_line)
            .ok_or_else(|| {
                WorkshopError::unknown("event", name_line.to_string(), self.locale.clone(), None)
            })?;
        match entry.id.as_str() {
            "global" => {
                if lines[1..].iter().any(|(line, _)| !line.trim().is_empty()) {
                    return Err(self.unsupported_event_parameters("global"));
                }
                Ok(Event::Global)
            }
            "eachPlayer" => {
                if lines[1..].iter().all(|(line, _)| line.trim().is_empty()) {
                    return Ok(Event::EachPlayer);
                }
                let (team, target) = self.event_filters(&lines, "eachPlayer", true)?;
                Ok(Event::EachPlayerWithFilters { team, target })
            }
            "subroutine" => {
                if lines
                    .get(2..)
                    .unwrap_or(&[])
                    .iter()
                    .any(|(line, _)| !line.trim().is_empty())
                {
                    return Err(self.unsupported_event_parameters("subroutine"));
                }
                let Some((sub_name, sub_span)) = lines.get(1) else {
                    return Err(self.malformed(
                        "subroutine event requires a subroutine name",
                        self.previous(),
                    ));
                };
                let id = self.subroutine_by_name(sub_name.trim(), *sub_span)?;
                Ok(Event::Subroutine {
                    subroutine: id,
                    name_span: *sub_span,
                })
            }
            other => match PlayerEventKind::from_catalog_id(other) {
                Some(kind) => self.player_event(&lines, kind),
                None => Err(WorkshopError::unsupported(
                    format!("unsupported event '{other}'"),
                    None,
                )),
            },
        }
    }

    pub(crate) fn player_event(
        &self,
        lines: &[(String, Option<Span>)],
        kind: PlayerEventKind,
    ) -> Result<Event> {
        let (team, target) = self.event_filters(lines, kind.catalog_id(), false)?;
        Ok(Event::Player { kind, team, target })
    }

    pub(crate) fn event_filters(
        &self,
        lines: &[(String, Option<Span>)],
        event_id: &str,
        allow_empty: bool,
    ) -> Result<(EventTeam, EventTarget)> {
        let parameters: Vec<&str> = lines[1..]
            .iter()
            .map(|(line, _)| line.trim())
            .filter(|line| !line.is_empty())
            .collect();
        if parameters.is_empty() {
            if allow_empty {
                return Ok((EventTeam::All, EventTarget::All));
            }
            return Err(WorkshopError::malformed(
                format!("event '{event_id}' requires team and player parameters"),
                None,
            ));
        }
        if parameters.len() != 2 {
            if event_id == "eachPlayer" {
                return Err(WorkshopError::unsupported(
                    format!("event '{event_id}' requires both team and player parameters"),
                    None,
                ));
            }
            return Err(WorkshopError::malformed(
                format!("event '{event_id}' requires team and player parameters"),
                None,
            ));
        }
        let team_member = self
            .resolve_enum_member_mixed("EventTeam", parameters[0])
            .map(|(_, member)| member);
        let team = match team_member.as_deref() {
            Some("ALL") => EventTeam::All,
            Some("TEAM_1") => EventTeam::Team1,
            Some("TEAM_2") => EventTeam::Team2,
            _ => return Err(self.unknown("event team", parameters[0])),
        };
        let target = if let Some((_, member)) =
            self.resolve_enum_member_mixed("EventPlayer", parameters[1])
        {
            if member == "ALL" {
                EventTarget::All
            } else if let Some(slot) = member.strip_prefix("SLOT_") {
                let slot = slot
                    .parse::<u8>()
                    .map_err(|_| self.unknown("event player", parameters[1]))?;
                EventTarget::Slot(slot)
            } else {
                return Err(self.unknown("event player", parameters[1]));
            }
        } else if let Some((_, hero)) = self
            .catalog
            .bare_member_matches(&self.locale, parameters[1])
            .into_iter()
            .chain(
                self.catalog
                    .bare_member_matches(&self.locale, &parameters[1].replace(':', ": ")),
            )
            .find(|(domain, _)| domain == "Hero")
        {
            EventTarget::Hero(hero)
        } else {
            return Err(self.unknown("event player", parameters[1]));
        };
        Ok((team, target))
    }

    pub(crate) fn unsupported_event_parameters(&self, event_id: &str) -> WorkshopError {
        WorkshopError::unsupported(
            format!("event '{event_id}' does not accept parameters"),
            None,
        )
    }
}
