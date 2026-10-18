use crate::frontend::parser::*;

impl ParseContext<'_> {
    pub(crate) fn settings_section(&mut self) -> Result<()> {
        let start = self.expect_keyword("settings")?;
        if self.target.settings.is_some() {
            return Err(self.malformed("duplicate 'settings' block", self.previous()));
        }
        self.expect(TokenKind::LBrace, "expected '{' after 'settings'")?;
        let mut children = Vec::new();
        while !matches!(self.peek().map(|token| token.kind), Some(TokenKind::RBrace)) {
            let (display, child_start, _) = self.phrase()?;
            let canonical_display = self.canonical_keyword(&display);
            let name = ["main", "lobby", "modes", "heroes", "extensions", "workshop"]
                .into_iter()
                .find(|name| self.settings_namespace_matches(name, &display))
                .unwrap_or_else(|| match canonical_display.as_str() {
                    value
                        if value == "extensions"
                            || self.settings_name_matches("labels", "Extensions", &display) =>
                    {
                        "extensions"
                    }
                    value
                        if value == "workshop"
                            || table::localized_name(
                                self.locale.as_str(),
                                "namespaces",
                                "workshop",
                            )
                            .is_some_and(|name| table::names_eq(name, &display)) =>
                    {
                        "workshop"
                    }
                    value => value,
                });
            self.expect(TokenKind::LBrace, "expected '{' after settings group")?;
            let node = match name {
                "main" | "lobby" => SettingsNode::Group {
                    name: name.to_string(),
                    children: self.settings_members(
                        &[PathPart::Part(if name == "main" {
                            "main"
                        } else {
                            "lobby"
                        })],
                        None,
                    )?,
                    span: Some(self.settings_span(child_start)),
                },
                "modes" => self.settings_modes(child_start)?,
                "heroes" => self.settings_heroes(child_start)?,
                "extensions" => SettingsNode::Group {
                    name: "extensions".to_string(),
                    children: self.settings_members(&[PathPart::Part("extensions")], None)?,
                    span: Some(self.settings_span(child_start)),
                },
                "workshop" => SettingsNode::Workshop {
                    children: self.settings_opaque_members()?,
                    span: Some(self.settings_span(child_start)),
                },
                _ => self.settings_opaque_group(name, child_start)?,
            };
            children.push(node);
        }
        let end = match self.next() {
            Some(Token {
                kind: TokenKind::RBrace,
                end,
                ..
            }) => end,
            _ => unreachable!("settings loop checks for closing brace"),
        };
        self.target.settings = Some(Settings {
            span: Some(Span::new(self.file(), start, end)),
            children,
        });
        Ok(())
    }

    pub(crate) fn settings_modes(&mut self, start: Position) -> Result<SettingsNode> {
        let mut children = Vec::new();
        while !matches!(self.peek().map(|token| token.kind), Some(TokenKind::RBrace)) {
            let mut disabled = false;
            if let Some(Token {
                kind: TokenKind::Word(word),
                ..
            }) = self.peek()
            {
                if self.settings_name_matches("tokens", "disabled", &word) {
                    self.pos += 1;
                    disabled = true;
                }
            }
            if !disabled {
                // Multi-token disabled prefixes (`Devre Dışı`,
                // `No permitir`, `Откл.`) sit on the same line as the mode
                // name, so the run must stop at the longest declared
                // spelling rather than consuming the mode name (#448).
                let mut accepted = vec!["disabled", "Disabled"];
                if let Some(localized) =
                    table::localized_name(self.locale.as_str(), "tokens", "disabled")
                {
                    accepted.push(localized);
                }
                disabled = self
                    .catalog_spelling(|spelling| {
                        accepted
                            .iter()
                            .any(|candidate| table::names_eq(candidate, spelling))
                    })
                    .is_some();
            }
            let (display, mode_start, mode_end) = self.phrase_on_line()?;
            let mode = self
                .resolve_settings_name_extended(
                    table::MODE_NAMES,
                    table::GENERATED_MODE_NAMES,
                    "modes",
                    &display,
                    mode_start,
                    mode_end,
                )
                .ok();
            self.expect(TokenKind::LBrace, "expected '{' after game mode")?;
            let mut mode_children = if let Some(mode) = mode {
                self.settings_members(&[PathPart::Part("gamemodes"), PathPart::Part(mode)], None)?
            } else {
                self.settings_opaque_members()?
            };
            if disabled {
                mode_children.insert(
                    0,
                    SettingsNode::Bool {
                        name: "enabled".to_string(),
                        value: false,
                        span: None,
                    },
                );
            }
            children.push(SettingsNode::Group {
                name: mode.map(str::to_string).unwrap_or(display),
                children: mode_children,
                span: Some(self.settings_span(mode_start)),
            });
        }
        self.expect(TokenKind::RBrace, "expected '}' after modes")?;
        Ok(SettingsNode::Group {
            name: "gamemodes".to_string(),
            children,
            span: Some(self.settings_span(start)),
        })
    }

    pub(crate) fn settings_heroes(&mut self, start: Position) -> Result<SettingsNode> {
        let mut teams = Vec::new();
        while !matches!(self.peek().map(|token| token.kind), Some(TokenKind::RBrace)) {
            let (team_display, team_start, team_end) = self.phrase_on_line_with_colon()?;
            let team = self.resolve_settings_name(
                table::TEAM_NAMES,
                "teams",
                &team_display,
                team_start,
                team_end,
            )?;
            self.expect(TokenKind::LBrace, "expected '{' after team settings group")?;
            let mut team_children = Vec::new();
            while !matches!(self.peek().map(|token| token.kind), Some(TokenKind::RBrace)) {
                let (display, child_start, child_end) = self.phrase_on_line_with_colon()?;
                if matches!(self.peek().map(|token| token.kind), Some(TokenKind::LBrace))
                    && self
                        .resolve_settings_name_extended(
                            table::HERO_NAMES,
                            table::GENERATED_HERO_NAMES,
                            "heroes",
                            &display,
                            child_start,
                            child_end,
                        )
                        .is_ok()
                {
                    let hero = self.resolve_settings_name_extended(
                        table::HERO_NAMES,
                        table::GENERATED_HERO_NAMES,
                        "heroes",
                        &display,
                        child_start,
                        child_end,
                    )?;
                    self.expect(TokenKind::LBrace, "expected '{' after hero settings group")?;
                    let children = self.settings_members(
                        &[PathPart::Part("heroes"), PathPart::Team, PathPart::Hero],
                        Some(hero),
                    )?;
                    team_children.push(SettingsNode::Group {
                        name: hero.to_string(),
                        children,
                        span: Some(self.settings_span(child_start)),
                    });
                } else {
                    team_children.push(self.settings_member_named(
                        display,
                        child_start,
                        child_end,
                        &[PathPart::Part("heroes"), PathPart::Team],
                        None,
                    )?);
                }
            }
            self.expect(TokenKind::RBrace, "expected '}' after team settings group")?;
            teams.push(SettingsNode::Group {
                name: team.to_string(),
                children: team_children,
                span: Some(self.settings_span(team_start)),
            });
        }
        self.expect(TokenKind::RBrace, "expected '}' after heroes")?;
        Ok(SettingsNode::Group {
            name: "heroes".to_string(),
            children: teams,
            span: Some(self.settings_span(start)),
        })
    }

    pub(crate) fn settings_members(
        &mut self,
        path: &[PathPart<'static>],
        hero: Option<&str>,
    ) -> Result<Vec<SettingsNode>> {
        let mut children = Vec::new();
        while !matches!(self.peek().map(|token| token.kind), Some(TokenKind::RBrace)) {
            let (display, start, end) = self.phrase_on_line()?;
            children.push(self.settings_member_named(display, start, end, path, hero)?);
        }
        self.expect(TokenKind::RBrace, "expected '}' after settings group")?;
        Ok(children)
    }

    fn find_settings_entry(
        &self,
        display: &str,
        path: &[PathPart<'static>],
        hero: Option<&str>,
    ) -> Option<&'static table::TableEntry> {
        table::entries()
            .find(|candidate| {
                candidate.path.len() == path.len() + 1
                    && candidate.path[..path.len()] == *path
                    && self.settings_name_matches_for_path(candidate, display, hero)
            })
            // A declared mode group inherits the `gamemodes.general` leaf set
            // (the same fallback `table::lookup` applies); resolve the authored
            // spelling against the shared general entry so an inherited member
            // stays typed instead of demoting to a carried raw member.
            .or_else(|| {
                let [PathPart::Part("gamemodes"), PathPart::Part(mode)] = path else {
                    return None;
                };
                table::entries()
                    .filter(|candidate| {
                        matches!(
                            candidate.path,
                            [
                                PathPart::Part("gamemodes"),
                                PathPart::Part("general"),
                                PathPart::Part(key)
                            ] if table::mode_inherited_entry(&[
                                PathPart::Part("gamemodes"),
                                PathPart::Part(mode),
                                PathPart::Part(key),
                            ])
                            .is_some()
                        )
                    })
                    .find(|candidate| self.settings_name_matches_for_path(candidate, display, hero))
            })
    }

    pub(crate) fn settings_member_named(
        &mut self,
        mut display: String,
        start: Position,
        _end: Position,
        path: &[PathPart<'static>],
        hero: Option<&str>,
    ) -> Result<SettingsNode> {
        // Localized labels can embed a colon (`Время восстановления: Удар
        // щитом`, #448): candidates are tried shortest-prefix first, and a
        // prefix that resolves but whose value fails — or that resolves to
        // nothing — is extended across the next colon and retried.
        let fallback_pos = self.pos;
        let fallback_display = display.clone();
        let mut first_error = None;
        loop {
            if let Some(entry) = self.find_settings_entry(&display, path, hero) {
                let checkpoint = self.pos;
                match self.settings_typed_member(entry, start) {
                    Ok(node) => return Ok(node),
                    Err(err) => {
                        first_error.get_or_insert(err);
                        self.pos = checkpoint;
                    }
                }
            }
            let checkpoint = self.pos;
            let extended = match self.peek().map(|token| token.kind) {
                Some(TokenKind::Colon) => {
                    self.pos += 1;
                    self.phrase_on_line()
                        .ok()
                        .filter(|(_, _, end)| end.line == start.line)
                        .map(|(extra, _, _)| format!("{display}:{extra}"))
                }
                _ => None,
            };
            match extended {
                Some(candidate) => display = candidate,
                None => {
                    self.pos = checkpoint;
                    break;
                }
            }
        }
        if let Some(err) = first_error {
            return Err(err);
        }
        self.pos = fallback_pos;
        if matches!(self.peek().map(|token| token.kind), Some(TokenKind::LBrace)) {
            self.pos += 1;
            return self.settings_opaque_group(&fallback_display, start);
        }
        self.settings_raw_member(fallback_display, start)
    }

    fn settings_typed_member(
        &mut self,
        entry: &'static table::TableEntry,
        start: Position,
    ) -> Result<SettingsNode> {
        let name = match entry.path.last() {
            Some(PathPart::Part(name)) => *name,
            _ => return Err(self.malformed("settings entry has no leaf key", self.previous())),
        };
        if matches!(self.peek().map(|token| token.kind), Some(TokenKind::LBrace)) {
            // A block under a catalogued non-list key carries an undeclared
            // value: the pinned source compilers write it as an opaque
            // block, so it parses like an uncatalogued-key block (#412).
            if !matches!(entry.kind, KeyKind::ListMap | KeyKind::ListHero) {
                self.pos += 1;
                return self.settings_opaque_group(name, start);
            }
            self.expect(TokenKind::LBrace, "expected '{' after settings list")?;
            let mut elements = Vec::new();
            while !matches!(self.peek().map(|token| token.kind), Some(TokenKind::RBrace)) {
                let (value, value_start, value_end) = self.settings_list_name()?;
                let canonical = match entry.kind {
                    KeyKind::ListMap => self
                        .resolve_settings_name_extended(
                            table::MAP_NAMES,
                            table::GENERATED_MAP_NAMES,
                            "maps",
                            &value,
                            value_start,
                            value_end,
                        )
                        .or_else(|_| {
                            value
                                .split_whitespace()
                                .next()
                                .and_then(|name| {
                                    self.resolve_settings_name_extended(
                                        table::MAP_NAMES,
                                        table::GENERATED_MAP_NAMES,
                                        "maps",
                                        name,
                                        value_start,
                                        value_end,
                                    )
                                    .ok()
                                })
                                .ok_or_else(|| {
                                    self.unknown_at("setting", &value, value_start, value_end)
                                })
                        })
                        .unwrap_or(value.as_str()),
                    KeyKind::ListHero => self
                        .resolve_settings_name_extended(
                            table::HERO_NAMES,
                            table::GENERATED_HERO_NAMES,
                            "heroes",
                            &value,
                            value_start,
                            value_end,
                        )
                        .unwrap_or(value.as_str()),
                    _ => unreachable!("non-list kinds return an opaque group above"),
                };
                elements.push(SettingsListElement {
                    value: canonical.to_string(),
                    span: Some(Span::new(self.file(), value_start, value_end)),
                });
            }
            self.expect(TokenKind::RBrace, "expected '}' after settings list")?;
            return Ok(SettingsNode::List {
                name: name.to_string(),
                elements,
                span: Some(Span::new(self.file(), start, self.previous_span().1)),
            });
        }
        if matches!(entry.kind, KeyKind::Flag) {
            return Ok(SettingsNode::Flag {
                name: name.to_string(),
                span: Some(Span::new(self.file(), start, self.previous_span().1)),
            });
        }
        self.expect(TokenKind::Colon, "expected ':' after settings key")?;
        let colon_line = self.previous_span().0.line;
        match entry.kind {
            KeyKind::Flag => unreachable!("presence-only settings returned before ':'"),
            KeyKind::String => {
                let value = self.expect_string("expected a settings string")?;
                Ok(SettingsNode::String {
                    name: name.to_string(),
                    value,
                    span: Some(Span::new(self.file(), start, self.previous_span().1)),
                })
            }
            KeyKind::Number => {
                let value = self.settings_number()?;
                Ok(SettingsNode::Number {
                    name: name.to_string(),
                    value,
                    span: Some(Span::new(self.file(), start, self.previous_span().1)),
                })
            }
            KeyKind::Percent => {
                let value = self.settings_number_percent()?;
                Ok(SettingsNode::Number {
                    name: name.to_string(),
                    value,
                    span: Some(Span::new(self.file(), start, self.previous_span().1)),
                })
            }
            KeyKind::Bool | KeyKind::YesNo => {
                let value = self.settings_bool_for(entry.path.last())?;
                Ok(SettingsNode::Bool {
                    name: name.to_string(),
                    value,
                    span: Some(Span::new(self.file(), start, self.previous_span().1)),
                })
            }
            KeyKind::BoolEnum(domain) => {
                let value = self.settings_bool_enum(domain)?;
                Ok(SettingsNode::Bool {
                    name: name.to_string(),
                    value,
                    span: Some(Span::new(self.file(), start, self.previous_span().1)),
                })
            }
            KeyKind::Enum(domain) => {
                let value_start = self.pos;
                match self.resolve_enum_settings_name(domain) {
                    Ok(value) => Ok(SettingsNode::String {
                        name: name.to_string(),
                        value,
                        span: Some(Span::new(self.file(), start, self.previous_span().1)),
                    }),
                    Err(_) => {
                        self.pos = value_start;
                        let value = self.raw_settings_line(colon_line)?;
                        Ok(SettingsNode::RawValue {
                            name: name.to_string(),
                            value,
                            span: Some(Span::new(self.file(), start, self.previous_span().1)),
                        })
                    }
                }
            }
            KeyKind::ListMap | KeyKind::ListHero => {
                Err(self.malformed("settings list requires a brace block", self.previous()))
            }
        }
    }

    pub(crate) fn settings_opaque_group(
        &mut self,
        name: &str,
        start: Position,
    ) -> Result<SettingsNode> {
        Ok(SettingsNode::Group {
            name: name.to_string(),
            children: self.settings_opaque_members()?,
            span: Some(self.settings_span(start)),
        })
    }

    pub(crate) fn settings_opaque_members(&mut self) -> Result<Vec<SettingsNode>> {
        let mut children = Vec::new();
        while !matches!(self.peek().map(|token| token.kind), Some(TokenKind::RBrace)) {
            let (display, start, _) = self.opaque_name_on_line()?;
            if matches!(self.peek().map(|token| token.kind), Some(TokenKind::LBrace)) {
                self.pos += 1;
                children.push(self.settings_opaque_group(&display, start)?);
            } else {
                children.push(self.settings_raw_member(display, start)?);
            }
        }
        self.expect(TokenKind::RBrace, "expected '}' after settings group")?;
        Ok(children)
    }

    pub(crate) fn settings_raw_member(
        &mut self,
        name: String,
        start: Position,
    ) -> Result<SettingsNode> {
        let mut value = String::new();
        let colon = match self.peek() {
            Some(Token {
                kind: TokenKind::Colon,
                start: colon_start,
                ..
            }) => {
                self.pos += 1;
                Some(colon_start.line)
            }
            _ => None,
        };
        if let Some(line) = colon {
            value = self.raw_settings_line(line)?;
        }
        let end = self.previous_span().1;
        let span = Some(Span::new(self.file(), start, end));
        if value.is_empty() && colon.is_some() {
            return Ok(SettingsNode::RawValue { name, value, span });
        }
        Ok(SettingsNode::Raw { name, value, span })
    }

    pub(crate) fn opaque_name_on_line(&mut self) -> Result<(String, Position, Position)> {
        let first = self
            .peek()
            .ok_or_else(|| self.malformed("expected an identifier", self.eof()))?;
        let start = first.start;
        let line = first.start.line;
        let mut end = first.end;
        let mut parts = Vec::new();
        while let Some(token) = self.peek() {
            if token.start.line != line
                || matches!(
                    token.kind,
                    TokenKind::Colon | TokenKind::LBrace | TokenKind::RBrace
                )
            {
                break;
            }
            self.pos += 1;
            end = token.end;
            parts.push((raw_token_text(&token.kind), token.start, token.end));
        }
        if parts.is_empty() {
            return Err(self.malformed("expected an identifier", &first));
        }
        Ok((join_as_written(&parts), start, end))
    }

    pub(crate) fn raw_settings_line(&mut self, colon_line: u32) -> Result<String> {
        let mut text = String::new();
        let mut previous_end = None;
        while let Some(token) = self.peek() {
            if token.start.line != colon_line || matches!(token.kind, TokenKind::RBrace) {
                break;
            }
            self.pos += 1;
            if previous_end.is_some_and(|end| end != token.start) {
                text.push(' ');
            }
            text.push_str(&raw_token_text(&token.kind));
            previous_end = Some(token.end);
        }
        Ok(text)
    }

    pub(crate) fn settings_number(&mut self) -> Result<f64> {
        match self.next() {
            Some(Token {
                kind: TokenKind::Number { value, .. },
                ..
            }) => Ok(value),
            Some(token) => Err(self.malformed("expected a settings number", &token)),
            None => Err(self.malformed("expected a settings number", self.eof())),
        }
    }

    pub(crate) fn settings_number_percent(&mut self) -> Result<f64> {
        let value = self.settings_number()?;
        if matches!(
            self.peek(),
            Some(Token {
                kind: TokenKind::Op(op),
                ..
            }) if op == "%"
        ) {
            self.pos += 1;
        }
        Ok(value)
    }

    /// `settings_bool`, first accepting the reviewed per-key value aliases.
    fn settings_bool_for(&mut self, key: Option<&PathPart>) -> Result<bool> {
        if let Some(PathPart::Part(key)) = key {
            let locale = self.locale.as_str();
            let alias =
                self.spelling_run(self.pos, None)
                    .into_iter()
                    .find_map(|(count, display, _)| {
                        table::bool_value_alias(key, locale, &display).map(|value| (count, value))
                    });
            if let Some((count, value)) = alias {
                self.pos += count;
                return Ok(value);
            }
        }
        self.settings_bool()
    }

    /// A settings boolean spelled in the page locale: the whole value run
    /// so dotted abbreviations (`Wł.`, `Вкл.`, #448) resolve like full
    /// words, with the diagnostic pointing at the spelled value.
    pub(crate) fn settings_bool(&mut self) -> Result<bool> {
        let token = self
            .peek()
            .ok_or_else(|| self.malformed("expected a settings boolean", self.eof()))?;
        if !matches!(token.kind, TokenKind::Word(_) | TokenKind::Number { .. }) {
            return Err(self.malformed("expected a settings boolean", &token));
        }
        // Settings members can share a line (`Cryo-Freeze: Off Ice Wall:
        // On`), so the value cannot read to end-of-line: consume the
        // longest run prefix that resolves to a boolean token and leave
        // the rest for the next member (#448).
        let start = token.start;
        let run = self.spelling_run(self.pos, None);
        for (count, value, _) in &run {
            if self.settings_name_matches("tokens", "On", value)
                || self.settings_name_matches("tokens", "Yes", value)
            {
                self.pos += count;
                return Ok(true);
            }
            if self.settings_name_matches("tokens", "Off", value)
                || self.settings_name_matches("tokens", "No", value)
            {
                self.pos += count;
                return Ok(false);
            }
        }
        let (value, end) = run
            .first()
            .map(|(_, text, end)| (text.clone(), *end))
            .unwrap_or_else(|| (raw_token_text(&token.kind), token.end));
        Err(self
            .unknown_at("setting boolean", &value, start, end)
            .with_candidates(self.settings_name_candidates(
                ["On", "Off", "Yes", "No"],
                "tokens",
                &value,
            )))
    }

    pub(crate) fn settings_bool_enum(&mut self, domain: &str) -> Result<bool> {
        let member = self.resolve_enum_settings_name(domain)?;
        if member == "enabled" {
            Ok(true)
        } else {
            Err(self.unknown("setting boolean", &member).with_candidates(
                self.settings_name_candidates(
                    table::ENUM_MEMBERS
                        .iter()
                        .chain(table::GENERATED_ENUM_MEMBERS.iter())
                        .filter(|entry| entry.domain == domain)
                        .map(|entry| entry.name),
                    "enums",
                    &member,
                ),
            ))
        }
    }

    pub(crate) fn resolve_enum_settings_name(&mut self, domain: &str) -> Result<String> {
        let token = self
            .peek()
            .ok_or_else(|| self.malformed("expected a settings enum member", self.eof()))?;
        let start = token.start;
        // Same bounded match as `settings_bool`: the next member can follow
        // on the same line, so only the longest run prefix resolving to a
        // member of this domain is consumed (#448).
        let run = self.spelling_run(self.pos, None);
        for (count, display, _) in &run {
            let member = table::ENUM_MEMBERS
                .iter()
                .chain(table::GENERATED_ENUM_MEMBERS.iter())
                .find(|member| {
                    member.domain == domain
                        && self.settings_name_matches("enums", member.name, display)
                });
            if let Some(member) = member {
                self.pos += count;
                return Ok(member.member.to_string());
            }
        }
        let (display, end) = run
            .first()
            .map(|(_, text, end)| (text.clone(), *end))
            .unwrap_or_else(|| (raw_token_text(&token.kind), token.end));
        Err(self
            .unknown_at("settings enum", &display, start, end)
            .with_candidates(
                self.settings_name_candidates(
                    table::ENUM_MEMBERS
                        .iter()
                        .chain(table::GENERATED_ENUM_MEMBERS.iter())
                        .filter(|entry| entry.domain == domain)
                        .map(|entry| entry.name),
                    "enums",
                    &display,
                ),
            ))
    }

    pub(crate) fn resolve_settings_name(
        &self,
        names: &[table::NameMap],
        section: &str,
        display: &str,
        start: Position,
        end: Position,
    ) -> Result<&'static str> {
        names
            .iter()
            .find(|candidate| self.settings_name_matches(section, candidate.name, display))
            .map(|candidate| candidate.key)
            .ok_or_else(|| {
                self.unknown_at("setting", display, start, end)
                    .with_candidates(self.settings_name_candidates(
                        names.iter().map(|entry| entry.name),
                        section,
                        display,
                    ))
            })
    }

    pub(crate) fn resolve_settings_name_extended(
        &self,
        names: &[table::NameMap],
        generated: &[table::NameMap],
        section: &str,
        display: &str,
        start: Position,
        end: Position,
    ) -> Result<&'static str> {
        names
            .iter()
            .chain(generated.iter())
            .find(|candidate| self.settings_name_matches(section, candidate.name, display))
            .map(|candidate| candidate.key)
            .ok_or_else(|| {
                self.unknown_at("setting", display, start, end)
                    .with_candidates(self.settings_name_candidates(
                        names.iter().chain(generated.iter()).map(|entry| entry.name),
                        section,
                        display,
                    ))
            })
    }

    fn settings_name_candidates<'b>(
        &'b self,
        names: impl IntoIterator<Item = &'b str>,
        section: &'b str,
        display: &str,
    ) -> Vec<String> {
        crate::core::suggest::nearest(
            display,
            names.into_iter().map(|name| {
                table::localized_name(self.locale.as_str(), section, name).unwrap_or(name)
            }),
            crate::core::suggest::CANDIDATE_LIMIT,
        )
    }

    pub(crate) fn settings_name_matches_for_path(
        &self,
        candidate: &table::TableEntry,
        display: &str,
        hero: Option<&str>,
    ) -> bool {
        if let Some(PathPart::Part(key)) = candidate.path.last() {
            if table::names_eq(display, key) {
                return true;
            }
        }
        if let (Some(hero), Some(PathPart::Part(key))) = (hero, candidate.path.last()) {
            if table::hero_setting_name(hero, key, self.locale.as_str())
                .is_some_and(|name| table::names_eq(name, display))
            {
                return true;
            }
            // Duplicate upstream rows carry additional reviewed spellings the
            // first-row `hero_setting_name` lookup never returns.
            if table::hero_setting_display_matches(hero, key, self.locale.as_str(), display) {
                return true;
            }
            if table::hero_setting_alias(hero, key, self.locale.as_str(), display) {
                return true;
            }
        }
        if let (Some(hero), Some(slot)) = (hero, table::ability_slot_for_path(candidate.path)) {
            // A producer alias is the spelling emission prefers; the export's
            // generated label and `%1$s` template are shadowed and must not
            // bind a display the emitter cannot reproduce.
            let aliased = candidate
                .path
                .last()
                .and_then(|part| match part {
                    PathPart::Part(key) => Some(*key),
                    _ => None,
                })
                .is_some_and(|key| table::hero_setting_has_alias(hero, key, self.locale.as_str()));
            let unshadowed_key = candidate.path.last().and_then(|part| match part {
                PathPart::Part(key) if !aliased => Some(*key),
                _ => None,
            });
            if let Some(key) = unshadowed_key {
                if let Some(template) = table::hero_key_template(key).or_else(|| {
                    candidate
                        .workshop_name
                        .contains("%1$s")
                        .then_some(candidate.workshop_name)
                }) {
                    // A `%1$s`-only template substitutes to the bare
                    // ability name; longer templates (`%1$s Cooldown
                    // Time`) must not bind it (#448). The producer fills
                    // the locale's template spelling when one exists
                    // (`Tempo de recarga de %1$s`), and mixed-locale
                    // sources may carry the en-US form.
                    let ability_name = |locale: &str| {
                        crate::gameplay::data::builtin()
                            .ok()?
                            .query()
                            .ability_name(hero, slot, None, locale)
                            .ok()
                            .map(str::to_string)
                    };
                    let substitute = |template: &str, name: &str| {
                        table::names_eq(template.replace("%1$s", name).as_str(), display)
                    };
                    let localized_template =
                        table::localized_name(self.locale.as_str(), "labels", template)
                            .unwrap_or(template);
                    let matched = ability_name(self.locale.as_str()).is_some_and(|name| {
                        substitute(localized_template, &name) || substitute(template, &name)
                    }) || (self.locale.as_str() != "en-US"
                        && ability_name("en-US").is_some_and(|name| substitute(template, &name)));
                    if matched {
                        return true;
                    }
                }
            }
        }
        self.settings_name_matches("labels", candidate.workshop_name, display)
    }

    pub(crate) fn settings_name_matches(
        &self,
        section: &str,
        english: &str,
        display: &str,
    ) -> bool {
        let localized = table::localized_name(self.locale.as_str(), section, english);
        localized
            .is_some_and(|localized| table::names_eq(localized, display))
            // Real Workshop exports can mix the selected locale with
            // primary-locale labels when a reviewed mapping is absent.
            // Accept that source spelling for parsing, while emission
            // still fails explicitly if the target mapping is missing.
            || table::names_eq(display, english)
    }

    fn settings_namespace_matches(&self, name: &str, display: &str) -> bool {
        table::localized_name(self.locale.as_str(), "namespaces", name)
            .is_some_and(|localized| table::names_eq(localized, display))
            || table::names_eq(display, name)
    }

    pub(crate) fn settings_span(&self, start: Position) -> Span {
        Span::new(self.file(), start, self.previous_span().1)
    }
}
