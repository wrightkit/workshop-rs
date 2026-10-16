use crate::frontend::parser::*;

const TOP_LEVEL_SECTIONS: &[&str] = &["settings", "variables", "subroutines", "rule", "disabled"];
const RULE_SECTIONS: &[&str] = &["event", "conditions", "actions"];

impl ParseContext<'_> {
    pub(crate) fn program(mut self) -> Result<wir::Program> {
        let file = self
            .target
            .add_file(SourceFile::with_source("workshop.txt", self.source));
        let _ = file;

        loop {
            let word = match self.peek() {
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
            let rule_start = self.span_here().0;
            let saved = self.pos;
            let section = self.keyword_prefix(TOP_LEVEL_SECTIONS);
            self.pos = saved;
            match section.as_deref() {
                Some("settings") => self.settings_section()?,
                Some("variables") => self.variables_section()?,
                Some("subroutines") => self.subroutines_section()?,
                Some("rule") => self.rule(false, rule_start)?,
                Some("disabled") => {
                    self.consume_keyword("disabled")?;
                    self.rule(true, rule_start)?;
                }
                _ => {
                    return Err(self
                        .unknown("top-level section", &word)
                        .with_candidates(self.section_candidates(&word, TOP_LEVEL_SECTIONS)));
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
                    kind: TokenKind::Word(_),
                    ..
                }) if self.keyword_prefix(&["global"]).is_some() => {
                    self.expect(TokenKind::Colon, "expected ':' after 'global'")?;
                    while let Some(Token {
                        kind: TokenKind::Number { .. },
                        ..
                    }) = self.peek()
                    {
                        let variable = self.variable_line()?;
                        self.next_global_index =
                            self.next_global_index.max(variable.index.saturating_add(1));
                        if let Some(id) = self.globals.get(&variable.name).copied() {
                            if is_forward_declaration(self.target.global_variables.get(id).unwrap())
                            {
                                *self.target.global_variables.get_mut(id).unwrap() = variable;
                                continue;
                            }
                            return Err(self.malformed(
                                &format!(
                                    "duplicate variable declaration '{}'",
                                    self.target.global_variables.get(id).unwrap().name
                                ),
                                self.previous(),
                            ));
                        }
                        let id = self.target.global_variables.push(variable);
                        self.globals.insert(
                            self.target.global_variables.get(id).unwrap().name.clone(),
                            id,
                        );
                    }
                    saw_section = true;
                }
                Some(Token {
                    kind: TokenKind::Word(_),
                    ..
                }) if self.keyword_prefix(&["player"]).is_some() => {
                    self.expect(TokenKind::Colon, "expected ':' after 'player'")?;
                    while let Some(Token {
                        kind: TokenKind::Number { .. },
                        ..
                    }) = self.peek()
                    {
                        let variable = self.variable_line()?;
                        self.next_player_index =
                            self.next_player_index.max(variable.index.saturating_add(1));
                        if let Some(id) = self.players.get(&variable.name).copied() {
                            if is_forward_declaration(self.target.player_variables.get(id).unwrap())
                            {
                                *self.target.player_variables.get_mut(id).unwrap() = variable;
                                continue;
                            }
                            return Err(self.malformed(
                                &format!(
                                    "duplicate variable declaration '{}'",
                                    self.target.player_variables.get(id).unwrap().name
                                ),
                                self.previous(),
                            ));
                        }
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

    /// A declaration index must be a whole u32: `as u32` alone truncates
    /// fractions and saturates overflow, silently merging declarations.
    fn declaration_index(&mut self) -> Result<u32> {
        match self.next() {
            Some(Token {
                kind: TokenKind::Number { value, .. },
                ..
            }) if value.fract() == 0.0 && (0.0..=u32::MAX as f64).contains(&value) => {
                Ok(value as u32)
            }
            Some(token) => Err(self.malformed("expected an integer index", &token)),
            None => Err(self.malformed("expected an index", self.eof())),
        }
    }

    fn check_referenceable_name(&mut self, name: &str) -> Result<()> {
        if name.contains('.') || name.contains('%') || name.contains(':') || name.contains(" - ") {
            return Err(self.malformed(
                &format!("declaration name '{name}' cannot be referenced"),
                self.previous(),
            ));
        }
        Ok(())
    }

    pub(crate) fn variable_line(&mut self) -> Result<wir::WorkshopVariable> {
        let index = self.declaration_index()?;
        self.expect(TokenKind::Colon, "expected ':' after variable index")?;
        let (name, name_start, name_end) = self.phrase_on_line()?;
        self.check_referenceable_name(&name)?;
        let name_span = Span::new(self.file(), name_start, name_end);
        Ok(wir::WorkshopVariable {
            name,
            index,
            span: Some(name_span),
            name_span: Some(name_span),
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
            let index = self.declaration_index()?;
            self.expect(TokenKind::Colon, "expected ':' after subroutine index")?;
            let (name, start, end) = self.phrase_on_line()?;
            self.check_referenceable_name(&name)?;
            let id = self.target.subroutines.push(wir::WorkshopSubroutine {
                name,
                index,
                span: Some(Span::new(self.file(), start, end)),
                name_span: Some(Span::new(self.file(), start, end)),
            });
            self.subroutines
                .insert(self.target.subroutines.get(id).unwrap().name.clone(), id);
        }
        self.expect(TokenKind::RBrace, "expected '}' after subroutines")?;
        Ok(())
    }

    fn string_contents_span(&self, start: Position, end: Position) -> Span {
        if start.line == end.line && end.col > start.col + 1 {
            Span::new(
                self.file(),
                Position::new(start.line, start.col + 1),
                Position::new(end.line, end.col - 1),
            )
        } else {
            Span::new(self.file(), start, end)
        }
    }

    pub(crate) fn rule(&mut self, disabled: bool, rule_start: Position) -> Result<()> {
        self.expect_keyword("rule")?;
        self.expect(TokenKind::LParen, "expected '(' after 'rule'")?;
        let name = self.expect_string("expected a rule name string")?;
        let (name_start, name_end) = self.previous_span();
        let name_span = self.string_contents_span(name_start, name_end);
        self.expect(TokenKind::RParen, "expected ')' after rule name")?;
        self.expect(TokenKind::LBrace, "expected '{' after rule header")?;

        let mut rule = wir::Rule {
            name,
            span: None,
            name_span: Some(name_span),
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
                    _ => {
                        return Err(self
                            .unknown("rule section", &word)
                            .with_candidates(self.section_candidates(&word, RULE_SECTIONS)));
                    }
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

    pub(crate) fn next_variable_index(&mut self, player: bool) -> u32 {
        if player {
            let index = self.next_player_index;
            self.next_player_index = self.next_player_index.saturating_add(1);
            index
        } else {
            let index = self.next_global_index;
            self.next_global_index = self.next_global_index.saturating_add(1);
            index
        }
    }

    fn section_candidates(&self, spelling: &str, ids: &[&str]) -> Vec<String> {
        let catalog = self.catalog;
        let primary = catalog.primary_locale();
        let include_primary = self.locale != *primary;
        crate::core::suggest::nearest(
            spelling,
            catalog
                .entries_of(Kind::Structural)
                .filter(move |entry| ids.contains(&entry.id.as_str()))
                .flat_map(move |entry| {
                    let local: &[String] = entry.spellings(&self.locale);
                    let fallback: &[String] = if include_primary {
                        entry.spellings(primary)
                    } else {
                        &[]
                    };
                    local
                        .iter()
                        .chain(fallback)
                        .map(String::as_str)
                        .chain(std::iter::once(entry.id.as_str()))
                }),
            crate::core::suggest::CANDIDATE_LIMIT,
        )
    }

    pub(crate) fn subroutine_by_name(
        &self,
        name: &str,
        span: Option<Span>,
    ) -> Result<wir::SubroutineId> {
        self.subroutines.get(name).copied().ok_or_else(|| {
            WorkshopError::unknown("subroutine", name.to_string(), self.locale.clone(), span)
                .with_candidates(crate::core::suggest::nearest(
                    name,
                    self.subroutines.keys().map(String::as_str),
                    crate::core::suggest::CANDIDATE_LIMIT,
                ))
        })
    }
}

fn is_forward_declaration(variable: &wir::WorkshopVariable) -> bool {
    variable.span.is_none()
}
