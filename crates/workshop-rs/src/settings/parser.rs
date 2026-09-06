// ParseContext behavior owned by the Workshop settings domain.

use crate::frontend::parser::*;

impl ParseContext<'_> {
    pub(crate) fn settings_section(&mut self) -> Result<()> {
        let start = self.expect_keyword("settings")?;
        self.expect(TokenKind::LBrace, "expected '{' after 'settings'")?;
        let mut children = Vec::new();
        while !matches!(self.peek().map(|token| token.kind), Some(TokenKind::RBrace)) {
            let (display, child_start, _) = self.phrase()?;
            let canonical_display = self.canonical_keyword(&display);
            let name = match canonical_display.as_str() {
                value
                    if value == "extensions"
                        || display == "扩展"
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
                        .is_some_and(|name| name == display) =>
                {
                    "workshop"
                }
                value => value,
            };
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
            let (display, mode_start, _) = self.phrase_on_line()?;
            let mode = self
                .resolve_settings_name_extended(
                    table::MODE_NAMES,
                    table::GENERATED_MODE_NAMES,
                    "modes",
                    &display,
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
            let (team_display, team_start, _) = self.phrase_on_line_with_colon()?;
            let team = self.resolve_settings_name(table::TEAM_NAMES, "teams", &team_display)?;
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
                        )
                        .is_ok()
                {
                    let hero = self.resolve_settings_name_extended(
                        table::HERO_NAMES,
                        table::GENERATED_HERO_NAMES,
                        "heroes",
                        &display,
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

    pub(crate) fn settings_member_named(
        &mut self,
        display: String,
        start: Position,
        _end: Position,
        path: &[PathPart<'static>],
        hero: Option<&str>,
    ) -> Result<SettingsNode> {
        let entry = table::entries().find(|candidate| {
            candidate.path.len() == path.len() + 1
                && candidate.path[..path.len()]
                    .iter()
                    .zip(path.iter())
                    .all(|(left, right)| left == right)
                && self.settings_name_matches_for_path(candidate, &display, hero)
        });
        let Some(entry) = entry else {
            if matches!(self.peek().map(|token| token.kind), Some(TokenKind::LBrace)) {
                self.pos += 1;
                return self.settings_opaque_group(&display, start);
            }
            return self.settings_raw_member(display, start);
        };
        let name = match entry.path.last() {
            Some(PathPart::Part(name)) => *name,
            _ => return Err(self.malformed("settings entry has no leaf key", self.previous())),
        };
        if matches!(self.peek().map(|token| token.kind), Some(TokenKind::LBrace)) {
            self.expect(TokenKind::LBrace, "expected '{' after settings list")?;
            let mut elements = Vec::new();
            while !matches!(self.peek().map(|token| token.kind), Some(TokenKind::RBrace)) {
                let (value, value_start, value_end) = self.phrase_on_line()?;
                let canonical = match entry.kind {
                    KeyKind::ListMap => self
                        .resolve_settings_name_extended(
                            table::MAP_NAMES,
                            table::GENERATED_MAP_NAMES,
                            "maps",
                            &value,
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
                                    )
                                    .ok()
                                })
                                .ok_or_else(|| self.unknown("setting", &value))
                        })
                        .unwrap_or(value.as_str()),
                    KeyKind::ListHero => self
                        .resolve_settings_name_extended(
                            table::HERO_NAMES,
                            table::GENERATED_HERO_NAMES,
                            "heroes",
                            &value,
                        )
                        .unwrap_or(value.as_str()),
                    _ => {
                        return Err(
                            self.malformed("only settings lists may use braces", self.previous())
                        );
                    }
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
        let end = self.previous_span().1;
        let span = Some(Span::new(self.file(), start, end));
        match entry.kind {
            KeyKind::Flag => unreachable!("presence-only settings returned before ':'"),
            KeyKind::String => Ok(SettingsNode::String {
                name: name.to_string(),
                value: self.expect_string("expected a settings string")?,
                span,
            }),
            KeyKind::Number => Ok(SettingsNode::Number {
                name: name.to_string(),
                value: self.settings_number(false)?,
                span,
            }),
            KeyKind::Percent => Ok(SettingsNode::Number {
                name: name.to_string(),
                value: self.settings_number_percent()?,
                span,
            }),
            KeyKind::Bool => Ok(SettingsNode::Bool {
                name: name.to_string(),
                value: self.settings_bool()?,
                span,
            }),
            KeyKind::BoolEnum(domain) => Ok(SettingsNode::Bool {
                name: name.to_string(),
                value: self.settings_bool_enum(domain)?,
                span,
            }),
            KeyKind::Enum(domain) => Ok(SettingsNode::String {
                name: name.to_string(),
                value: self.resolve_enum_settings_name(domain)?,
                span,
            }),
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
        if matches!(self.peek().map(|token| token.kind), Some(TokenKind::Colon)) {
            self.pos += 1;
            value = self.raw_settings_line()?;
        }
        let end = self.previous_span().1;
        Ok(SettingsNode::Raw {
            name,
            value,
            span: Some(Span::new(self.file(), start, end)),
        })
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
            parts.push(raw_token_text(&token.kind));
        }
        if parts.is_empty() {
            return Err(self.malformed("expected an identifier", &first));
        }
        Ok((
            parts
                .join(" ")
                .replace(" : ", ":")
                .replace(" .", ".")
                .replace(". ", "."),
            start,
            end,
        ))
    }

    pub(crate) fn raw_settings_line(&mut self) -> Result<String> {
        let line = self.peek().map(|token| token.start.line);
        let mut parts = Vec::new();
        while let Some(token) = self.peek() {
            if line.is_some_and(|line| token.start.line != line)
                || matches!(token.kind, TokenKind::RBrace)
            {
                break;
            }
            self.pos += 1;
            parts.push(raw_token_text(&token.kind));
        }
        Ok(parts.join(" "))
    }

    pub(crate) fn settings_number(&mut self, percent: bool) -> Result<f64> {
        let value = match self.next() {
            Some(Token {
                kind: TokenKind::Number { value, .. },
                ..
            }) => value,
            Some(token) => return Err(self.malformed("expected a settings number", &token)),
            None => return Err(self.malformed("expected a settings number", self.eof())),
        };
        if percent {
            self.expect(
                TokenKind::Op("%".to_string()),
                "expected '%' after settings percentage",
            )?;
        }
        Ok(value)
    }

    pub(crate) fn settings_number_percent(&mut self) -> Result<f64> {
        let value = self.settings_number(false)?;
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

    pub(crate) fn settings_bool(&mut self) -> Result<bool> {
        let token = self
            .next()
            .ok_or_else(|| self.malformed("expected a settings boolean", self.eof()))?;
        let TokenKind::Word(value) = token.kind else {
            return Err(self.malformed("expected a settings boolean", &token));
        };
        if self.settings_name_matches("tokens", "On", &value)
            || self.settings_name_matches("tokens", "Yes", &value)
        {
            Ok(true)
        } else if self.settings_name_matches("tokens", "Off", &value)
            || self.settings_name_matches("tokens", "No", &value)
        {
            Ok(false)
        } else {
            Err(self.unknown("setting boolean", &value))
        }
    }

    pub(crate) fn settings_bool_enum(&mut self, domain: &str) -> Result<bool> {
        let member = self.resolve_enum_settings_name(domain)?;
        if member == "enabled" {
            Ok(true)
        } else {
            Err(self.unknown("setting boolean", &member))
        }
    }

    pub(crate) fn resolve_enum_settings_name(&mut self, domain: &str) -> Result<String> {
        let (display, _, _) = self.phrase_on_line()?;
        table::ENUM_MEMBERS
            .iter()
            .find(|member| {
                member.domain == domain
                    && self.settings_name_matches("enums", member.name, &display)
            })
            .map(|member| member.member.to_string())
            .or_else(|| {
                table::GENERATED_ENUM_MEMBERS
                    .iter()
                    .find(|member| {
                        member.domain == domain
                            && self.settings_name_matches("enums", member.name, &display)
                    })
                    .map(|member| member.member.to_string())
            })
            .ok_or_else(|| self.unknown("settings enum", &display))
    }

    pub(crate) fn resolve_settings_name(
        &self,
        names: &[table::NameMap],
        section: &str,
        display: &str,
    ) -> Result<&'static str> {
        names
            .iter()
            .find(|candidate| self.settings_name_matches(section, candidate.name, display))
            .map(|candidate| candidate.key)
            .ok_or_else(|| self.unknown("setting", display))
    }

    pub(crate) fn resolve_settings_name_extended(
        &self,
        names: &[table::NameMap],
        generated: &[table::NameMap],
        section: &str,
        display: &str,
    ) -> Result<&'static str> {
        names
            .iter()
            .chain(generated.iter())
            .find(|candidate| self.settings_name_matches(section, candidate.name, display))
            .map(|candidate| candidate.key)
            .ok_or_else(|| self.unknown("setting", display))
    }

    pub(crate) fn settings_name_matches_for_path(
        &self,
        candidate: &table::TableEntry,
        display: &str,
        hero: Option<&str>,
    ) -> bool {
        if let Some(PathPart::Part(key)) = candidate.path.last() {
            if display == *key {
                return true;
            }
        }
        if let (Some(hero), Some(PathPart::Part(key))) = (hero, candidate.path.last()) {
            if table::hero_setting_name(hero, key, self.locale.as_str()) == Some(display) {
                return true;
            }
            if table::hero_setting_alias(hero, key, self.locale.as_str(), display) {
                return true;
            }
        }
        if let (Some(hero), Some(slot)) = (hero, table::ability_slot_for_path(candidate.path)) {
            if candidate.workshop_name.contains("%1$s")
                || matches!(
                    candidate.path.last(),
                    Some(PathPart::Part("enableAbility1" | "enableAbility2"))
                )
            {
                return crate::gameplay::data::builtin()
                    .ok()
                    .and_then(|catalog| {
                        catalog
                            .query()
                            .ability_name(hero, slot, None, self.locale.as_str())
                            .ok()
                            .map(|name| name == display)
                    })
                    .unwrap_or(false);
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
            .is_some_and(|localized| localized == display)
            // Real Workshop exports can mix the selected locale with
            // primary-locale labels when a reviewed mapping is absent.
            // Accept that source spelling for parsing, while emission
            // still fails explicitly if the target mapping is missing.
            || display == english
    }

    pub(crate) fn settings_span(&self, start: Position) -> Span {
        Span::new(self.file(), start, self.previous_span().1)
    }
}
