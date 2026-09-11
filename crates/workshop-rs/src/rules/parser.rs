use crate::frontend::parser::*;

impl ParseContext<'_> {
    pub(crate) fn program(mut self) -> Result<wir::Program> {
        let file = self
            .target
            .add_file(SourceFile::with_source("workshop.txt", self.source));
        let _ = file;

        loop {
            let phrase = match self.peek() {
                Some(Token {
                    kind: TokenKind::Word(word),
                    ..
                }) => word.clone(),
                Some(Token {
                    kind: TokenKind::Eof,
                    ..
                }) => break,
                Some(token) => {
                    return Err(self.malformed("expected a top-level section", &token));
                }
                None => break,
            };
            match self.canonical_keyword(&phrase).as_str() {
                "settings" => self.settings_section()?,
                "variables" => self.variables_section()?,
                "subroutines" => self.subroutines_section()?,
                "rule" => self.rule(false, self.span_here().0)?,
                "disabled" => {
                    let rule_start = self.span_here().0;
                    self.pos += 1;
                    self.rule(true, rule_start)?;
                }
                other => {
                    return Err(self.unknown("top-level section", other));
                }
            }
        }
        Ok(self.target)
    }

    pub(crate) fn variables_section(&mut self) -> Result<()> {
        self.expect_keyword("variables")?;
        self.expect(TokenKind::LBrace, "expected '{' after 'variables'")?;
        let mut saw_section = false;
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
                    kind: TokenKind::Word(word),
                    ..
                }) if matches!(canonical_keyword(&word), "Global" | "global") => {
                    self.pos += 1;
                    self.expect(TokenKind::Colon, "expected ':' after 'global'")?;
                    while let Some(Token {
                        kind: TokenKind::Number { .. },
                        ..
                    }) = self.peek()
                    {
                        let variable = self.variable_line()?;
                        let id = self.target.global_variables.push(variable);
                        self.globals.insert(
                            self.target.global_variables.get(id).unwrap().name.clone(),
                            id,
                        );
                    }
                    saw_section = true;
                }
                Some(Token {
                    kind: TokenKind::Word(word),
                    ..
                }) if canonical_keyword(&word) == "player" => {
                    self.pos += 1;
                    self.expect(TokenKind::Colon, "expected ':' after 'player'")?;
                    while let Some(Token {
                        kind: TokenKind::Number { .. },
                        ..
                    }) = self.peek()
                    {
                        let variable = self.variable_line()?;
                        let id = self.target.player_variables.push(variable);
                        self.players.insert(
                            self.target.player_variables.get(id).unwrap().name.clone(),
                            id,
                        );
                    }
                    saw_section = true;
                }
                Some(token) => {
                    return Err(self.malformed("expected 'global', 'player', or '}'", &token));
                }
                None => {
                    return Err(self.malformed("unexpected end of input in variables", self.eof()));
                }
            }
        }
        if !saw_section {
            return Err(self.malformed("variables section is empty", self.previous()));
        }
        Ok(())
    }

    pub(crate) fn variable_line(&mut self) -> Result<wir::WorkshopVariable> {
        let (index, span) = match self.next() {
            Some(Token {
                kind: TokenKind::Number { value, .. },
                start,
                end,
            }) => (
                value as u32,
                Span::new(synthetic_span(start).file, start, end),
            ),
            Some(token) => return Err(self.malformed("expected a variable index", &token)),
            None => return Err(self.malformed("expected a variable index", self.eof())),
        };
        self.expect(TokenKind::Colon, "expected ':' after variable index")?;
        let (name, name_start, name_end) = self.phrase_on_line()?;
        let name_span = Span::new(self.file(), name_start, name_end);
        Ok(wir::WorkshopVariable {
            name,
            index,
            span: Some(if span.file.index() == 0 {
                name_span
            } else {
                span
            }),
            // Workshop-text sources carry no `.opy` identifier provenance;
            // exact rename occurrences are only produced by the native path.
            name_span: None,
        })
    }

    pub(crate) fn subroutines_section(&mut self) -> Result<()> {
        self.expect_keyword("subroutines")?;
        self.expect(TokenKind::LBrace, "expected '{' after 'subroutines'")?;
        while let Some(Token {
            kind: TokenKind::Number { .. },
            ..
        }) = self.peek()
        {
            let index = match self.next() {
                Some(Token {
                    kind: TokenKind::Number { value, .. },
                    ..
                }) => value as u32,
                _ => unreachable!(),
            };
            self.expect(TokenKind::Colon, "expected ':' after subroutine index")?;
            let (name, start, end) = self.phrase_on_line()?;
            let id = self.target.subroutines.push(wir::WorkshopSubroutine {
                name,
                index,
                span: Some(Span::new(self.file(), start, end)),
                name_span: None,
            });
            self.subroutines
                .insert(self.target.subroutines.get(id).unwrap().name.clone(), id);
        }
        self.expect(TokenKind::RBrace, "expected '}' after subroutines")?;
        Ok(())
    }

    pub(crate) fn rule(&mut self, disabled: bool, rule_start: Position) -> Result<()> {
        self.expect_keyword("rule")?;
        self.expect(TokenKind::LParen, "expected '(' after 'rule'")?;
        let name = self.expect_string("expected a rule name string")?;
        self.expect(TokenKind::RParen, "expected ')' after rule name")?;
        self.expect(TokenKind::LBrace, "expected '{' after rule header")?;

        let mut rule = wir::Rule {
            name,
            span: None,
            name_span: None,
            disabled,
            event: Event::Global,
            conditions: Vec::new(),
            actions: Vec::new(),
        };
        let mut seen_sections = Vec::new();
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
                    kind: TokenKind::Word(word),
                    ..
                }) => match self.canonical_keyword(&word).as_str() {
                    "event" => {
                        if seen_sections.contains(&"event") {
                            return Err(
                                self.malformed("duplicate 'event' section", &self.peek().unwrap())
                            );
                        }
                        seen_sections.push("event");
                        rule.event = self.event_section()?;
                    }
                    "conditions" => {
                        if seen_sections.contains(&"conditions") {
                            return Err(self.malformed(
                                "duplicate 'conditions' section",
                                &self.peek().unwrap(),
                            ));
                        }
                        seen_sections.push("conditions");
                        rule.conditions = self.conditions_section()?;
                    }
                    "actions" => {
                        if seen_sections.contains(&"actions") {
                            return Err(self
                                .malformed("duplicate 'actions' section", &self.peek().unwrap()));
                        }
                        seen_sections.push("actions");
                        rule.actions = self.actions_section()?;
                    }
                    _ => return Err(self.unknown("rule section", &word)),
                },
                Some(token) => {
                    return Err(self.malformed("expected a rule section or '}'", &token));
                }
                None => return Err(self.malformed("unexpected end of input in rule", self.eof())),
            }
        }
        let rule_end = self.previous_span().1;
        rule.span = Some(Span::new(self.file(), rule_start, rule_end));
        self.target.rules.push(rule);
        Ok(())
    }

    pub(crate) fn global_by_name(&mut self, name: &str) -> Result<wir::GlobalVarId> {
        if let Some(id) = self.globals.get(name).copied() {
            return Ok(id);
        }
        let index = self.next_variable_index(false);
        let id = self.target.global_variables.push(wir::WorkshopVariable {
            name: name.to_string(),
            index,
            span: None,
            name_span: None,
        });
        self.globals.insert(name.to_string(), id);
        Ok(id)
    }

    pub(crate) fn player_by_name(&mut self, name: &str) -> Result<wir::PlayerVarId> {
        if let Some(id) = self.players.get(name).copied() {
            return Ok(id);
        }
        let index = self.next_variable_index(true);
        let id = self.target.player_variables.push(wir::WorkshopVariable {
            name: name.to_string(),
            index,
            span: None,
            name_span: None,
        });
        self.players.insert(name.to_string(), id);
        Ok(id)
    }

    pub(crate) fn next_variable_index(&self, player: bool) -> u32 {
        let variables = if player {
            &self.target.player_variables
        } else {
            &self.target.global_variables
        };
        variables
            .iter()
            .map(|variable| variable.index)
            .max()
            .map_or(0, |index| index.saturating_add(1))
    }

    pub(crate) fn subroutine_by_name(&self, name: &str) -> Result<wir::SubroutineId> {
        self.subroutines
            .get(name)
            .copied()
            .ok_or_else(|| WorkshopError::Unknown {
                kind: "subroutine",
                spelling: name.to_string(),
                locale: self.locale.clone(),
                span: None,
            })
    }
}
