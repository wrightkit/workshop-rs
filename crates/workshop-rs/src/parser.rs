//! Native localized Workshop parser.
//!
//! Parses vanilla Workshop text directly into validated, locale-independent
//! Workshop IR. Localized actions, values, events, enums, and structural
//! keywords resolve through the canonical catalog; malformed input,
//! unknown spellings, and recognized-but-unsupported constructs are reported
//! as distinct structured diagnostics with source spans.

use std::collections::HashMap;

use crate::settings::table::{self, KeyKind, PathPart};
use crate::settings::{Settings, SettingsListElement, SettingsNode};
use crate::signatures::{ExpectedDomain, NoExpectedDomain};
use crate::source::{Position, SourceFile, Span};
use crate::wir::{
    self, Action, Event, EventTarget, EventTeam, ModifyOp, PlayerEventKind, Value, ValueNode,
};

use crate::catalog::{Catalog, Kind, Locale};
use crate::error::{Result, WorkshopError};
use crate::lexer::{Token, TokenKind, tokenize};

/// Where action parsing stopped.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Stop {
    /// The enclosing `}` was consumed.
    SectionClosed,
    /// The `End` keyword is next (not consumed).
    End,
    /// The `Else If` keyword is next (not consumed).
    ElseIf,
    /// The `Else` keyword is next (not consumed).
    Else,
}

enum AssignmentOperator {
    Set,
    Modify(ModifyOp),
}

/// Parse localized Workshop text into Workshop IR with no signature context:
/// ambiguous bare enum members (e.g. the `None` shared by several domains)
/// stay rejected. See [`parse_with_context`] for the context-sensitive form.
pub fn parse(input: &str, catalog: &Catalog, locale: &Locale) -> Result<wir::Program> {
    parse_with_context(input, catalog, locale, &NoExpectedDomain)
}

/// Parse localized Workshop text into Workshop IR, resolving ambiguous bare
/// enum members from the enclosing call's canonical signature context (#111).
///
/// When a bare member spelling matches several enum domains, the parser asks
/// [`ExpectedDomain::expected_domain`] for the domain the enclosing call's
/// signature expects at that argument position; the member resolves only when
/// that expected domain is one of the matching domains (i.e. the signature
/// pins exactly one). Without a pin the ambiguity diagnostic is unchanged.
pub fn parse_with_context(
    input: &str,
    catalog: &Catalog,
    locale: &Locale,
    context: &dyn ExpectedDomain,
) -> Result<wir::Program> {
    let tokens = tokenize(input).map_err(|error| WorkshopError::Malformed {
        message: error.message,
        span: Some(synthetic_span(error.position)),
    })?;
    Parser {
        tokens,
        pos: 0,
        catalog,
        locale: locale.clone(),
        context,
        expected_domain: None,
        call_stack: Vec::new(),
        target: wir::Program::default(),
        globals: HashMap::new(),
        players: HashMap::new(),
        subroutines: HashMap::new(),
    }
    .program()
}

/// A synthetic single-position span (used before a file registry exists).
fn synthetic_span(position: Position) -> Span {
    Span::new(crate::ids::Id::from_index(0), position, position)
}

struct Parser<'a> {
    tokens: Vec<Token>,
    pos: usize,
    catalog: &'a Catalog,
    locale: Locale,
    /// Canonical signature context (#111): supplies the expected enum domain
    /// for the call argument currently being parsed.
    context: &'a dyn ExpectedDomain,
    /// The expected enum domain for the value currently being parsed, set by
    /// [`Parser::value_args`] from the enclosing call's signature.
    expected_domain: Option<&'a str>,
    call_stack: Vec<String>,
    target: wir::Program,
    globals: HashMap<String, wir::GlobalVarId>,
    players: HashMap<String, wir::PlayerVarId>,
    subroutines: HashMap<String, wir::SubroutineId>,
}

impl Parser<'_> {
    fn resolve_entry(&self, kind: Kind, spelling: &str) -> Option<crate::catalog::CatalogEntry> {
        self.catalog
            .resolve(kind, &self.locale, spelling)
            .cloned()
            .or_else(|| {
                if self.locale != *self.catalog.primary_locale() {
                    self.catalog
                        .resolve(kind, self.catalog.primary_locale(), spelling)
                        .cloned()
                } else {
                    None
                }
            })
    }

    fn resolve_enum_domain_mixed(&self, spelling: &str) -> Option<&str> {
        self.catalog
            .resolve_enum_domain(&self.locale, spelling)
            .or_else(|| {
                if self.locale != *self.catalog.primary_locale() {
                    self.catalog
                        .resolve_enum_domain(self.catalog.primary_locale(), spelling)
                } else {
                    None
                }
            })
    }

    fn resolve_enum_member_mixed(&self, domain: &str, spelling: &str) -> Option<(String, String)> {
        let alternate = (!spelling.contains(": ") && spelling.contains(':'))
            .then(|| spelling.replacen(':', ": ", 1));
        self.catalog
            .resolve_enum_member(domain, &self.locale, spelling)
            .or_else(|| {
                alternate.as_deref().and_then(|spelling| {
                    self.catalog
                        .resolve_enum_member(domain, &self.locale, spelling)
                })
            })
            .or_else(|| {
                if self.locale != *self.catalog.primary_locale() {
                    self.catalog.resolve_enum_member(
                        domain,
                        self.catalog.primary_locale(),
                        alternate.as_deref().unwrap_or(spelling),
                    )
                } else {
                    None
                }
            })
    }

    fn program(mut self) -> Result<wir::Program> {
        let file = self.target.files.push(SourceFile::new("workshop.txt"));
        // Re-point synthetic spans at the real file id by keeping a helper.
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
            match canonical_keyword(&phrase) {
                "settings" => self.settings_section()?,
                "variables" => self.variables_section()?,
                "subroutines" => self.subroutines_section()?,
                "rule" => self.rule(false)?,
                "disabled" => {
                    self.pos += 1;
                    self.rule(true)?;
                }
                other => {
                    return Err(self.unknown("top-level section", other));
                }
            }
        }
        Ok(self.target)
    }

    fn settings_section(&mut self) -> Result<()> {
        let start = self.expect_keyword("settings")?;
        self.expect(TokenKind::LBrace, "expected '{' after 'settings'")?;
        let mut children = Vec::new();
        while !matches!(self.peek().map(|token| token.kind), Some(TokenKind::RBrace)) {
            let (display, child_start, _) = self.phrase()?;
            let name = match canonical_keyword(&display) {
                value
                    if value == "extensions"
                        || display == "扩展"
                        || self.settings_name_matches("labels", "Extensions", &display) =>
                {
                    "extensions"
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

    fn settings_modes(&mut self, start: Position) -> Result<SettingsNode> {
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

    fn settings_heroes(&mut self, start: Position) -> Result<SettingsNode> {
        let mut teams = Vec::new();
        while !matches!(self.peek().map(|token| token.kind), Some(TokenKind::RBrace)) {
            let (team_display, team_start, _) = self.phrase_on_line()?;
            let team = self.resolve_settings_name(table::TEAM_NAMES, "teams", &team_display)?;
            self.expect(TokenKind::LBrace, "expected '{' after team settings group")?;
            let mut team_children = Vec::new();
            while !matches!(self.peek().map(|token| token.kind), Some(TokenKind::RBrace)) {
                let (display, child_start, child_end) = self.phrase_on_line()?;
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

    fn settings_members(
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

    fn settings_member_named(
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

    fn settings_opaque_group(&mut self, name: &str, start: Position) -> Result<SettingsNode> {
        Ok(SettingsNode::Group {
            name: name.to_string(),
            children: self.settings_opaque_members()?,
            span: Some(self.settings_span(start)),
        })
    }

    fn settings_opaque_members(&mut self) -> Result<Vec<SettingsNode>> {
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

    fn settings_raw_member(&mut self, name: String, start: Position) -> Result<SettingsNode> {
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

    fn opaque_name_on_line(&mut self) -> Result<(String, Position, Position)> {
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

    fn raw_settings_line(&mut self) -> Result<String> {
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

    fn settings_number(&mut self, percent: bool) -> Result<f64> {
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

    fn settings_number_percent(&mut self) -> Result<f64> {
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

    fn settings_bool(&mut self) -> Result<bool> {
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

    fn resolve_enum_settings_name(&mut self, domain: &str) -> Result<String> {
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

    fn resolve_settings_name(
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

    fn resolve_settings_name_extended(
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

    fn settings_name_matches_for_path(
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
                return crate::gameplay_data::builtin()
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

    fn settings_name_matches(&self, section: &str, english: &str, display: &str) -> bool {
        if self.locale == Locale::new("en-US") {
            display == english
        } else {
            table::localized_name(self.locale.as_str(), section, english)
                .is_some_and(|localized| localized == display)
                // Real Workshop exports can mix the selected locale with
                // primary-locale labels when a reviewed mapping is absent.
                // Accept that source spelling for parsing, while emission
                // still fails explicitly if the target mapping is missing.
                || display == english
        }
    }

    fn settings_span(&self, start: Position) -> Span {
        Span::new(self.file(), start, self.previous_span().1)
    }

    fn variables_section(&mut self) -> Result<()> {
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

    fn variable_line(&mut self) -> Result<wir::WorkshopVariable> {
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

    fn subroutines_section(&mut self) -> Result<()> {
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

    fn rule(&mut self, disabled: bool) -> Result<()> {
        self.expect_keyword("rule")?;
        self.expect(TokenKind::LParen, "expected '(' after 'rule'")?;
        let name = self.expect_string("expected a rule name string")?;
        self.expect(TokenKind::RParen, "expected ')' after rule name")?;
        let (rule_start, rule_end) = self.previous_span();
        self.expect(TokenKind::LBrace, "expected '{' after rule header")?;

        let mut rule = wir::Rule {
            name,
            span: Some(Span::new(self.file(), rule_start, rule_end)),
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
                }) => match canonical_keyword(&word) {
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
        self.target.rules.push(rule);
        Ok(())
    }

    fn event_section(&mut self) -> Result<Event> {
        self.expect_keyword("event")?;
        self.expect(TokenKind::LBrace, "expected '{' after 'event'")?;
        let mut lines: Vec<String> = Vec::new();
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
                    lines.push(String::new());
                }
                Some(_) => {
                    let text = self.line_text()?;
                    lines.push(text);
                }
                None => return Err(self.malformed("unexpected end of input in event", self.eof())),
            }
        }
        let Some(name_line) = lines.first().cloned() else {
            return Err(self.malformed("event section is empty", self.previous()));
        };
        let name_line = name_line.trim();
        let entry = self
            .catalog
            .resolve(Kind::Event, &self.locale, name_line)
            .ok_or_else(|| WorkshopError::Unknown {
                kind: "event",
                spelling: name_line.to_string(),
                locale: self.locale.clone(),
                span: None,
            })?;
        match entry.id.as_str() {
            "global" => {
                if lines[1..].iter().any(|line| !line.trim().is_empty()) {
                    return Err(self.unsupported_event_parameters("global"));
                }
                Ok(Event::Global)
            }
            "eachPlayer" => {
                if lines[1..].iter().all(|line| line.trim().is_empty()) {
                    return Ok(Event::EachPlayer);
                }
                let (team, target) = self.event_filters(&lines, "eachPlayer", true)?;
                Ok(Event::EachPlayerWithFilters { team, target })
            }
            "playerDealtDamage" => self.player_event(&lines, PlayerEventKind::DealtDamage),
            "playerDealtFinalBlow" => self.player_event(&lines, PlayerEventKind::DealtFinalBlow),
            "playerDealtHealing" => self.player_event(&lines, PlayerEventKind::DealtHealing),
            "playerDealtKnockback" => self.player_event(&lines, PlayerEventKind::DealtKnockback),
            "playerDied" => self.player_event(&lines, PlayerEventKind::Died),
            "playerEarnedElimination" => {
                self.player_event(&lines, PlayerEventKind::EarnedElimination)
            }
            "playerJoined" => self.player_event(&lines, PlayerEventKind::Joined),
            "playerLeft" => self.player_event(&lines, PlayerEventKind::Left),
            "playerReceivedHealing" => self.player_event(&lines, PlayerEventKind::ReceivedHealing),
            "playerReceivedKnockback" => {
                self.player_event(&lines, PlayerEventKind::ReceivedKnockback)
            }
            "playerTookDamage" => self.player_event(&lines, PlayerEventKind::TookDamage),
            "subroutine" => {
                if lines
                    .get(2..)
                    .unwrap_or(&[])
                    .iter()
                    .any(|line| !line.trim().is_empty())
                {
                    return Err(self.unsupported_event_parameters("subroutine"));
                }
                let Some(sub_name) = lines.get(1).map(|s| s.trim()) else {
                    return Err(self.malformed(
                        "subroutine event requires a subroutine name",
                        self.previous(),
                    ));
                };
                let id = self.subroutine_by_name(sub_name)?;
                Ok(Event::Subroutine(id))
            }
            other => Err(WorkshopError::Unsupported {
                message: format!("unsupported event '{other}'"),
                span: None,
            }),
        }
    }

    fn player_event(&self, lines: &[String], kind: PlayerEventKind) -> Result<Event> {
        let (team, target) = self.event_filters(lines, kind.catalog_id(), false)?;
        Ok(Event::Player { kind, team, target })
    }

    fn event_filters(
        &self,
        lines: &[String],
        event_id: &str,
        allow_empty: bool,
    ) -> Result<(EventTeam, EventTarget)> {
        let parameters: Vec<&str> = lines[1..]
            .iter()
            .map(String::as_str)
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .collect();
        if parameters.is_empty() {
            if allow_empty {
                return Ok((EventTeam::All, EventTarget::All));
            }
            return Err(WorkshopError::Malformed {
                message: format!("event '{event_id}' requires team and player parameters"),
                span: None,
            });
        }
        if parameters.len() != 2 {
            if event_id == "eachPlayer" {
                return Err(WorkshopError::Unsupported {
                    message: format!("event '{event_id}' requires both team and player parameters"),
                    span: None,
                });
            }
            return Err(WorkshopError::Malformed {
                message: format!("event '{event_id}' requires team and player parameters"),
                span: None,
            });
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

    fn unsupported_event_parameters(&self, event_id: &str) -> WorkshopError {
        WorkshopError::Unsupported {
            message: format!("event '{event_id}' does not accept parameters"),
            span: None,
        }
    }

    fn conditions_section(&mut self) -> Result<Vec<wir::ValueId>> {
        self.expect_keyword("conditions")?;
        self.expect(TokenKind::LBrace, "expected '{' after 'conditions'")?;
        let mut conditions = Vec::new();
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
                }
                Some(_) => {
                    if matches!(
                        self.peek(),
                        Some(Token {
                            kind: TokenKind::String(_),
                            ..
                        })
                    ) {
                        self.pos += 1;
                        continue;
                    }
                    if let Some(Token {
                        kind: TokenKind::Word(word),
                        ..
                    }) = self.peek()
                    {
                        if self.settings_name_matches("tokens", "disabled", &word) {
                            self.pos += 1;
                            let _disabled_condition = self.value()?;
                            self.expect(TokenKind::Semi, "expected ';' after condition")?;
                            continue;
                        }
                    }
                    let condition = self.value()?;
                    self.expect(TokenKind::Semi, "expected ';' after condition")?;
                    conditions.push(condition);
                }
                None => {
                    return Err(self.malformed("unexpected end of input in conditions", self.eof()));
                }
            }
        }
        Ok(conditions)
    }

    fn actions_section(&mut self) -> Result<Vec<wir::ActionId>> {
        self.expect_keyword("actions")?;
        self.expect(TokenKind::LBrace, "expected '{' after 'actions'")?;
        let mut all_actions = Vec::new();
        loop {
            let (actions, stop) = self.actions_until_end()?;
            all_actions.extend(actions);
            if stop == Stop::SectionClosed {
                return Ok(all_actions);
            }
            // Some exported Workshop artifacts retain an unmatched structural
            // marker after source-side pruning. Preserve it as an opaque raw
            // action so the source remains visible instead of silently
            // discarding it or rejecting the whole project.
            all_actions.push(self.opaque_action()?);
        }
    }

    /// Parse actions until a structural `else`/`elseIf`/`end` terminator
    /// (not consumed; the token position is preserved) or the enclosing `}`
    /// (consumed). Returns where the parse stopped.
    fn actions_until_end(&mut self) -> Result<(Vec<wir::ActionId>, Stop)> {
        let mut actions = Vec::new();
        loop {
            match self.peek() {
                Some(Token {
                    kind: TokenKind::RBrace,
                    ..
                }) => {
                    self.pos += 1;
                    return Ok((actions, Stop::SectionClosed));
                }
                Some(Token {
                    kind: TokenKind::Word(_),
                    ..
                }) => {
                    if let Some(action) = self.assignment_action()? {
                        actions.push(action);
                        continue;
                    }
                    let saved = self.pos;
                    let (phrase, start, end) = self.phrase()?;
                    if let Some(rest) = self.disabled_action_rest(&phrase) {
                        if let Some(structural) = self.resolve_entry(Kind::Structural, rest) {
                            match structural.id.as_str() {
                                "while" => {
                                    actions.push(self.while_group()?);
                                    continue;
                                }
                                "if" => {
                                    actions.push(self.if_group()?);
                                    continue;
                                }
                                _ => {}
                            }
                        }
                        if let Some(action) = self.resolve_entry(Kind::Action, rest) {
                            let canonical = if action.id == "abort" {
                                self.catalog
                                    .spelling(Kind::Action, &self.locale, "abortIf")
                                    .unwrap_or(rest)
                            } else {
                                rest
                            };
                            actions.push(self.action_call_from_phrase(
                                canonical.to_string(),
                                start,
                                end,
                            )?);
                            continue;
                        }
                    }
                    if canonical_keyword(&phrase) == "disabled"
                        && matches!(
                            self.peek(),
                            Some(Token {
                                kind: TokenKind::Word(word),
                                ..
                            }) if canonical_keyword(&word) == "While"
                        )
                    {
                        self.pos += 1;
                        actions.push(self.while_group()?);
                        continue;
                    }
                    match canonical_keyword(&phrase) {
                        "End" => {
                            self.pos = saved;
                            return Ok((actions, Stop::End));
                        }
                        "Else If" => {
                            self.pos = saved;
                            return Ok((actions, Stop::ElseIf));
                        }
                        "Else" => {
                            self.pos = saved;
                            return Ok((actions, Stop::Else));
                        }
                        "If" => actions.push(self.if_group()?),
                        "For Global Variable" => actions.push(self.for_group()?),
                        "For Player Variable" => actions.push(self.for_player_group()?),
                        "While" => actions.push(self.while_group()?),
                        "Loop" => actions.push(self.action_call_from_phrase(phrase, start, end)?),
                        "Loop If Condition Is True" => {
                            actions.push(self.action_call_from_phrase(phrase, start, end)?)
                        }
                        "Global" | "Event Player" => {
                            self.pos = saved;
                            actions.push(self.opaque_action()?);
                        }
                        _ => {
                            if self.line_has_assignment() {
                                self.pos = saved;
                                actions.push(self.opaque_action()?);
                            } else {
                                actions.push(self.action_call_from_phrase(phrase, start, end)?);
                            }
                        }
                    }
                }
                Some(Token {
                    kind: TokenKind::String(_),
                    ..
                }) => {
                    // Raw Workshop permits standalone quoted annotations in
                    // generated action blocks. They are inert source text,
                    // not executable actions.
                    self.pos += 1;
                }
                Some(token) => {
                    let saved = self.pos;
                    if let Some(action) = self.member_assignment_action(saved, token.start)? {
                        actions.push(action);
                    } else {
                        return Err(self.malformed("expected an action", &token));
                    }
                }
                None => {
                    return Err(self.malformed("unexpected end of input in actions", self.eof()));
                }
            }
        }
    }

    /// Return the action spelling after the locale-declared disabled
    /// modifier. The modifier itself is settings/catalog data, not a parser
    /// branch for a fixed pair of client locales.
    fn disabled_action_rest<'a>(&self, phrase: &'a str) -> Option<&'a str> {
        if let Some(rest) = phrase.strip_prefix("disabled ") {
            return Some(rest);
        }
        let localized = table::localized_name(self.locale.as_str(), "tokens", "disabled")?;
        phrase.strip_prefix(localized)?.strip_prefix(' ')
    }

    fn assignment_action(&mut self) -> Result<Option<wir::ActionId>> {
        let saved = self.pos;
        let Some(Token {
            kind: TokenKind::Word(first),
            start,
            ..
        }) = self.peek()
        else {
            return Ok(None);
        };

        if matches!(canonical_keyword(&first), "Global" | "global") {
            self.pos += 1;
            self.expect(TokenKind::Dot, "expected '.' after 'Global'")?;
            let (name, _, target_end) = self.phrase()?;
            if matches!(
                self.peek().map(|token| token.kind),
                Some(TokenKind::LBracket)
            ) {
                self.pos += 1;
                let index = self.value()?;
                self.expect(
                    TokenKind::RBracket,
                    "expected ']' after global variable index",
                )?;
                let _operator = self.assignment_operator().ok_or_else(|| {
                    self.malformed(
                        "expected assignment after global variable index",
                        self.peek().as_ref().unwrap_or(self.eof()),
                    )
                })?;
                let variable = self.global_by_name(&name)?;
                let target = self.target.values.push(ValueNode::new(
                    Value::GlobalVariable(variable),
                    Some(Span::new(self.file(), start, target_end)),
                ));
                let value = self.value()?;
                self.expect(TokenKind::Semi, "expected ';' after indexed assignment")?;
                return Ok(Some(self.target.actions.push(Action::Call {
                    name: "setGlobalVariableAtIndex".to_string(),
                    args: vec![target, index, value],
                    span: Some(Span::new(self.file(), start, self.previous_span().1)),
                })));
            }
            let Some(operator) = self.assignment_operator() else {
                return self.member_assignment_action(saved, start);
            };
            let variable = self.global_by_name(&name)?;
            let value = self.value()?;
            self.expect(TokenKind::Semi, "expected ';' after assignment")?;
            let span = Some(Span::new(self.file(), start, self.previous_span().1));
            let target_span = Some(Span::new(self.file(), start, target_end));
            return Ok(Some(self.target.actions.push(match operator {
                AssignmentOperator::Set => Action::SetGlobalVariable {
                    variable,
                    value,
                    span,
                    target_span,
                },
                AssignmentOperator::Modify(op) => Action::ModifyGlobalVariable {
                    variable,
                    op,
                    value,
                    span,
                    target_span,
                },
            })));
        }

        if !matches!(canonical_keyword(&first), "Event" | "event")
            || !matches!(
                self.peek_at(1).map(|token| token.kind),
                Some(TokenKind::Word(word))
                    if matches!(canonical_keyword(&word), "Player" | "player")
            )
        {
            // Object/member assignments use the same value grammar as member
            // reads (`receiver.member` and `receiver.member[index]`). Keep
            // this source-level form distinct from catalog actions: the
            // receiver and member are dynamic Workshop values, not a builtin
            // identity. Global and Event Player assignments are handled by
            // their dedicated variable paths above and below.
            if !matches!(canonical_keyword(&first), "Event" | "event") {
                return self.member_assignment_action(saved, start);
            }
            return Ok(None);
        }

        self.pos += 2;
        let event_player = self.target.values.push(ValueNode::new(
            Value::EventPlayer,
            Some(Span::new(self.file(), start, self.previous_span().1)),
        ));
        self.expect(TokenKind::Dot, "expected '.' after 'Event Player'")?;
        let (name, target_start, target_end) = self.phrase()?;
        let variable = self.player_by_name(&name)?;
        if matches!(
            self.peek().map(|token| token.kind),
            Some(TokenKind::LBracket)
        ) {
            self.pos += 1;
            let index = self.value()?;
            self.expect(
                TokenKind::RBracket,
                "expected ']' after player variable index",
            )?;
            let _operator = self.assignment_operator().ok_or_else(|| {
                self.malformed(
                    "expected assignment after player variable index",
                    self.peek().as_ref().unwrap_or(self.eof()),
                )
            })?;
            let value = self.value()?;
            self.expect(TokenKind::Semi, "expected ';' after indexed assignment")?;
            let variable_value = self.target.values.push(ValueNode::new(
                Value::PlayerVariable {
                    player: event_player,
                    variable,
                },
                Some(Span::new(self.file(), target_start, target_end)),
            ));
            return Ok(Some(self.target.actions.push(Action::Call {
                name: "setPlayerVariableAtIndex".to_string(),
                args: vec![variable_value, index, value],
                span: Some(Span::new(self.file(), start, self.previous_span().1)),
            })));
        }
        let Some(operator) = self.assignment_operator() else {
            return self.member_assignment_action(saved, start);
        };
        let value = self.value()?;
        self.expect(TokenKind::Semi, "expected ';' after assignment")?;
        let span = Some(Span::new(self.file(), start, self.previous_span().1));
        let target_span = Some(Span::new(self.file(), target_start, target_end));
        Ok(Some(self.target.actions.push(match operator {
            AssignmentOperator::Set => Action::SetPlayerVariable {
                player: event_player,
                variable,
                value,
                span,
                target_span,
            },
            AssignmentOperator::Modify(op) => Action::ModifyPlayerVariable {
                player: event_player,
                variable,
                op,
                value,
                span,
                target_span,
            },
        })))
    }

    fn member_assignment_action(
        &mut self,
        saved: usize,
        start: Position,
    ) -> Result<Option<wir::ActionId>> {
        self.pos = saved;
        if !self.line_has_assignment() {
            return Ok(None);
        }
        let target = self.value()?;
        let Some(operator) = self.assignment_operator() else {
            self.pos = saved;
            return Ok(None);
        };
        let value = self.value()?;
        self.expect(TokenKind::Semi, "expected ';' after member assignment")?;
        let op = match operator {
            AssignmentOperator::Set => None,
            AssignmentOperator::Modify(op) => Some(op),
        };
        Ok(Some(self.target.actions.push(Action::AssignMember {
            target,
            op,
            value,
            span: Some(Span::new(self.file(), start, self.previous_span().1)),
        })))
    }

    fn assignment_operator(&mut self) -> Option<AssignmentOperator> {
        let operator = match self.peek()?.kind {
            TokenKind::Op(operator) => operator,
            _ => return None,
        };
        if operator == "=" {
            self.pos += 1;
            return Some(AssignmentOperator::Set);
        }
        let op = match operator.as_str() {
            "+" => ModifyOp::Add,
            "-" => ModifyOp::Subtract,
            "*" => ModifyOp::Multiply,
            "/" => ModifyOp::Divide,
            "%" => ModifyOp::Modulo,
            _ => return None,
        };
        if !matches!(self.peek_at(1).map(|token| token.kind), Some(TokenKind::Op(equal)) if equal == "=")
        {
            return None;
        }
        self.pos += 2;
        Some(AssignmentOperator::Modify(op))
    }

    fn opaque_action(&mut self) -> Result<wir::ActionId> {
        let start = self
            .peek()
            .map(|token| token.start)
            .unwrap_or(self.eof().start);
        while let Some(token) = self.peek() {
            self.pos += 1;
            if matches!(token.kind, TokenKind::Semi) {
                break;
            }
        }
        Ok(self.target.actions.push(Action::Call {
            name: "rawWorkshopAction".to_string(),
            args: Vec::new(),
            span: Some(Span::new(self.file(), start, self.previous_span().1)),
        }))
    }

    fn if_group(&mut self) -> Result<wir::ActionId> {
        let start = self.previous_span().0;
        self.expect(TokenKind::LParen, "expected '(' after 'If'")?;
        let condition = self.value()?;
        self.expect(TokenKind::RParen, "expected ')' after If condition")?;
        self.expect(TokenKind::Semi, "expected ';' after If condition")?;

        let mut branches = Vec::new();
        let mut stop = {
            let (body, stop) = self.actions_until_end()?;
            branches.push(wir::IfBranch { condition, body });
            stop
        };

        let mut else_body = None;
        loop {
            match stop {
                Stop::End => {
                    self.consume_phrase("End")?;
                    self.expect(TokenKind::Semi, "expected ';' after 'End'")?;
                    break;
                }
                Stop::ElseIf => {
                    self.consume_phrase("Else If")?;
                    self.expect(TokenKind::LParen, "expected '(' after 'Else If'")?;
                    let condition = self.value()?;
                    self.expect(TokenKind::RParen, "expected ')' after Else If condition")?;
                    self.expect(TokenKind::Semi, "expected ';' after Else If condition")?;
                    let (body, next) = self.actions_until_end()?;
                    branches.push(wir::IfBranch { condition, body });
                    stop = next;
                }
                Stop::Else => {
                    self.consume_phrase("Else")?;
                    self.expect(TokenKind::Semi, "expected ';' after 'Else'")?;
                    let (body, next) = self.actions_until_end()?;
                    else_body = Some(body);
                    stop = next;
                }
                Stop::SectionClosed => {
                    // The oracle closes a rule-final if/if-else with the
                    // enclosing actions-section `}` (no trailing `End;`,
                    // #87). Rewind so the enclosing actions section consumes
                    // that `}` and the rule's own `}` stays intact.
                    self.pos -= 1;
                    break;
                }
            }
        }
        let end_span = self.previous_span();
        let action = Action::If {
            branches,
            else_body,
            span: Some(Span::new(self.file(), start, end_span.1)),
        };
        Ok(self.target.actions.push(action))
    }

    fn for_group(&mut self) -> Result<wir::ActionId> {
        let start = self.previous_span().0;
        self.expect(
            TokenKind::LParen,
            "expected '(' after 'For Global Variable'",
        )?;
        let (name, _, _) = self.phrase()?;
        let variable = self.global_by_name(&name)?;
        self.expect(TokenKind::Comma, "expected ',' after loop variable")?;
        let start_value = self.value()?;
        self.expect(TokenKind::Comma, "expected ',' after start")?;
        let stop = self.value()?;
        self.expect(TokenKind::Comma, "expected ',' after stop")?;
        let step = self.value()?;
        self.expect(TokenKind::RParen, "expected ')' after For bounds")?;
        self.expect(TokenKind::Semi, "expected ';' after For Global Variable")?;
        let (body, loop_stop) = self.actions_until_end()?;
        if loop_stop != Stop::End {
            return Err(self.malformed(
                "'For Global Variable' requires a matching 'End'",
                self.previous(),
            ));
        }
        self.consume_phrase("End")?;
        self.expect(TokenKind::Semi, "expected ';' after 'End'")?;
        let end_span = self.previous_span();
        let action = Action::ForGlobalVariable {
            variable,
            start: start_value,
            stop,
            step,
            body,
            span: Some(Span::new(self.file(), start, end_span.1)),
            target_span: None,
        };
        Ok(self.target.actions.push(action))
    }

    fn while_group(&mut self) -> Result<wir::ActionId> {
        let start = self.previous_span().0;
        self.expect(TokenKind::LParen, "expected '(' after 'While'")?;
        let condition = self.value()?;
        self.expect(TokenKind::RParen, "expected ')' after While condition")?;
        self.expect(TokenKind::Semi, "expected ';' after While condition")?;
        let (body, stop) = self.actions_until_end()?;
        if stop != Stop::End {
            return Err(self.malformed("'While' requires a matching 'End'", self.previous()));
        }
        self.consume_phrase("End")?;
        self.expect(TokenKind::Semi, "expected ';' after 'End'")?;
        let end_span = self.previous_span();
        let action = Action::While {
            condition,
            body,
            span: Some(Span::new(self.file(), start, end_span.1)),
        };
        Ok(self.target.actions.push(action))
    }

    /// `For Player Variable(player, name, start, stop, step)` — the
    /// reference's per-player loop form (parsed from pinned reference
    /// evidence; the differential gate normalizes it to the declared global
    /// form, #119).
    fn for_player_group(&mut self) -> Result<wir::ActionId> {
        let start = self.previous_span().0;
        self.expect(
            TokenKind::LParen,
            "expected '(' after 'For Player Variable'",
        )?;
        let player = self.value()?;
        self.expect(TokenKind::Comma, "expected ',' after loop player")?;
        let (name, _, _) = self.phrase()?;
        let variable = self.player_by_name(&name)?;
        self.expect(TokenKind::Comma, "expected ',' after loop variable")?;
        let start_value = self.value()?;
        self.expect(TokenKind::Comma, "expected ',' after start")?;
        let stop = self.value()?;
        self.expect(TokenKind::Comma, "expected ',' after stop")?;
        let step = self.value()?;
        self.expect(TokenKind::RParen, "expected ')' after For bounds")?;
        self.expect(TokenKind::Semi, "expected ';' after For Player Variable")?;
        let (body, loop_stop) = self.actions_until_end()?;
        if loop_stop != Stop::End {
            return Err(self.malformed(
                "'For Player Variable' requires a matching 'End'",
                self.previous(),
            ));
        }
        self.consume_phrase("End")?;
        self.expect(TokenKind::Semi, "expected ';' after 'End'")?;
        let end_span = self.previous_span();
        let action = Action::ForPlayerVariable {
            player,
            variable,
            start: start_value,
            stop,
            step,
            body,
            span: Some(Span::new(self.file(), start, end_span.1)),
        };
        Ok(self.target.actions.push(action))
    }

    fn action_call_from_phrase(
        &mut self,
        phrase: String,
        start: Position,
        end: Position,
    ) -> Result<wir::ActionId> {
        match self
            .catalog
            .resolve(Kind::Structural, &self.locale, &phrase)
        {
            Some(entry) => match entry.id.as_str() {
                "setGlobalVariable" => {
                    self.expect(
                        TokenKind::LParen,
                        "expected '(' after 'Set Global Variable'",
                    )?;
                    let (name, _, _) = self.phrase()?;
                    let variable = self.global_by_name(&name)?;
                    self.expect(TokenKind::Comma, "expected ',' after variable")?;
                    let value = self.value()?;
                    self.expect(TokenKind::RParen, "expected ')'")?;
                    self.expect(TokenKind::Semi, "expected ';'")?;
                    Ok(self.target.actions.push(Action::SetGlobalVariable {
                        variable,
                        value,
                        span: Some(Span::new(self.file(), start, end)),
                        target_span: None,
                    }))
                }
                "modifyGlobalVariable" => {
                    self.expect(TokenKind::LParen, "expected '('")?;
                    let (name, _, _) = self.phrase()?;
                    let variable = self.global_by_name(&name)?;
                    self.expect(TokenKind::Comma, "expected ',' after variable")?;
                    let op = self.modify_op()?;
                    self.expect(TokenKind::Comma, "expected ',' after modify operator")?;
                    let value = self.value()?;
                    self.expect(TokenKind::RParen, "expected ')'")?;
                    self.expect(TokenKind::Semi, "expected ';'")?;
                    Ok(self.target.actions.push(Action::ModifyGlobalVariable {
                        variable,
                        op,
                        value,
                        span: Some(Span::new(self.file(), start, end)),
                        target_span: None,
                    }))
                }
                "setPlayerVariable" => {
                    self.expect(TokenKind::LParen, "expected '('")?;
                    let player = self.value()?;
                    self.expect(TokenKind::Comma, "expected ',' after player")?;
                    let (name, _, _) = self.phrase()?;
                    let variable = self.player_by_name(&name)?;
                    self.expect(TokenKind::Comma, "expected ',' after variable")?;
                    let value = self.value()?;
                    self.expect(TokenKind::RParen, "expected ')'")?;
                    self.expect(TokenKind::Semi, "expected ';'")?;
                    Ok(self.target.actions.push(Action::SetPlayerVariable {
                        player,
                        variable,
                        value,
                        span: Some(Span::new(self.file(), start, end)),
                        target_span: None,
                    }))
                }
                "modifyPlayerVariable" => {
                    self.expect(TokenKind::LParen, "expected '('")?;
                    let player = self.value()?;
                    self.expect(TokenKind::Comma, "expected ',' after player")?;
                    let (name, _, _) = self.phrase()?;
                    let variable = self.player_by_name(&name)?;
                    self.expect(TokenKind::Comma, "expected ',' after variable")?;
                    let op = self.modify_op()?;
                    self.expect(TokenKind::Comma, "expected ',' after modify operator")?;
                    let value = self.value()?;
                    self.expect(TokenKind::RParen, "expected ')'")?;
                    self.expect(TokenKind::Semi, "expected ';'")?;
                    Ok(self.target.actions.push(Action::ModifyPlayerVariable {
                        player,
                        variable,
                        op,
                        value,
                        span: Some(Span::new(self.file(), start, end)),
                        target_span: None,
                    }))
                }
                "forGlobalVariable" => self.for_group(),
                "forPlayerVariable" => self.for_player_group(),
                "callSubroutine" => {
                    self.expect(TokenKind::LParen, "expected '('")?;
                    let (name, _, _) = self.phrase()?;
                    let subroutine = self.subroutine_by_name(&name)?;
                    self.expect(TokenKind::RParen, "expected ')'")?;
                    self.expect(TokenKind::Semi, "expected ';'")?;
                    Ok(self.target.actions.push(Action::CallSubroutine {
                        subroutine,
                        span: Some(Span::new(self.file(), start, end)),
                        callee_span: None,
                    }))
                }
                other => Err(WorkshopError::Unsupported {
                    message: format!(
                        "structural action '{other}' is not supported in action position"
                    ),
                    span: Some(Span::new(self.file(), start, end)),
                }),
            },
            None => {
                // Generic action call; the argument list is optional.
                let Some(action) = self
                    .resolve_entry(Kind::Action, &phrase)
                    .or_else(|| self.resolve_entry(Kind::Action, &format!("{phrase} ")))
                    .or_else(|| {
                        let alias = match phrase.as_str() {
                            "Set Player Allowed Heroes" => "Set Allowed Heroes",
                            "设置技能充能" => "设置终极技能充能",
                            _ => return None,
                        };
                        self.resolve_entry(Kind::Action, alias)
                    })
                else {
                    return Err(WorkshopError::Unknown {
                        kind: "action",
                        spelling: phrase,
                        locale: self.locale.clone(),
                        span: Some(Span::new(self.file(), start, end)),
                    });
                };
                // The player-variable chase forms lay the variable out as
                // `player, name` leading arguments (the pinned oracle's
                // spelling, #110); the name is not a value, so the action is
                // parsed like `Set Player Variable` and reconstructed as the
                // canonical `chaseAtRate`/`chaseOverTime` call with a
                // player-variable first argument (the shape the emitter
                // dispatches on).
                match action.id.as_str() {
                    "chasePlayerVariableAtRate" | "chasePlayerVariableOverTime" => {
                        self.expect(TokenKind::LParen, "expected '('")?;
                        let player = self.value()?;
                        self.expect(TokenKind::Comma, "expected ',' after player")?;
                        let (name, _, _) = self.phrase()?;
                        let variable = self.player_by_name(&name)?;
                        let mut args = Vec::with_capacity(4);
                        args.push(self.target.values.push(wir::ValueNode::new(
                            wir::Value::PlayerVariable { player, variable },
                            None,
                        )));
                        // The remaining arguments sit at overall argument
                        // indexes 2.. (player and name consumed indexes 0-1),
                        // so the signature context resolves their expected
                        // domains at the shifted positions.
                        let mut arg_index = 2usize;
                        loop {
                            match self.peek() {
                                Some(Token {
                                    kind: TokenKind::RParen,
                                    ..
                                }) => break,
                                Some(Token {
                                    kind: TokenKind::Comma,
                                    ..
                                }) => {
                                    self.pos += 1;
                                }
                                _ => {}
                            }
                            let saved = self.expected_domain;
                            self.expected_domain =
                                self.context.expected_domain(action.id.as_str(), arg_index);
                            let arg = self.value()?;
                            self.expected_domain = saved;
                            args.push(arg);
                            arg_index += 1;
                        }
                        self.expect(TokenKind::RParen, "expected ')'")?;
                        self.expect(TokenKind::Semi, "expected ';' after action")?;
                        let canonical = if action.id == "chasePlayerVariableAtRate" {
                            "chaseAtRate"
                        } else {
                            "chaseOverTime"
                        };
                        return Ok(self.target.actions.push(Action::Call {
                            name: canonical.to_string(),
                            args,
                            span: Some(Span::new(self.file(), start, end)),
                        }));
                    }
                    "startRule" => {
                        self.expect(TokenKind::LParen, "expected '('")?;
                        let (name, _, _) = self.phrase()?;
                        let subroutine = self.subroutine_by_name(&name)?;
                        self.expect(TokenKind::Comma, "expected ',' after subroutine")?;
                        let saved = self.expected_domain;
                        self.expected_domain = self.context.expected_domain(action.id.as_str(), 1);
                        let behavior = self.value()?;
                        self.expected_domain = saved;
                        self.expect(TokenKind::RParen, "expected ')'")?;
                        self.expect(TokenKind::Semi, "expected ';' after action")?;
                        let subroutine_value = self
                            .target
                            .values
                            .push(ValueNode::new(Value::Subroutine(subroutine), None));
                        return Ok(self.target.actions.push(Action::Call {
                            name: action.id.clone(),
                            args: vec![subroutine_value, behavior],
                            span: Some(Span::new(self.file(), start, end)),
                        }));
                    }
                    "stopChasingPlayerVariable" => {
                        self.expect(TokenKind::LParen, "expected '('")?;
                        let player = self.value()?;
                        self.expect(TokenKind::Comma, "expected ',' after player")?;
                        let (name, _, _) = self.phrase()?;
                        let variable = self.player_by_name(&name)?;
                        self.expect(TokenKind::RParen, "expected ')'")?;
                        self.expect(TokenKind::Semi, "expected ';' after action")?;
                        let player_variable = self.target.values.push(ValueNode::new(
                            Value::PlayerVariable { player, variable },
                            None,
                        ));
                        return Ok(self.target.actions.push(Action::Call {
                            name: action.id.clone(),
                            args: vec![player_variable],
                            span: Some(Span::new(self.file(), start, end)),
                        }));
                    }
                    _ => {}
                }
                let args = if let Some(Token {
                    kind: TokenKind::LParen,
                    ..
                }) = self.peek()
                {
                    self.pos += 1;
                    let args = self.value_args(action.id.as_str())?;
                    self.expect(TokenKind::RParen, "expected ')'")?;
                    args
                } else {
                    Vec::new()
                };
                self.expect(TokenKind::Semi, "expected ';' after action")?;
                Ok(self.target.actions.push(Action::Call {
                    name: action.id.clone(),
                    args,
                    span: Some(Span::new(self.file(), start, end)),
                }))
            }
        }
    }

    fn modify_op(&mut self) -> Result<ModifyOp> {
        let (phrase, start, end) = self.phrase()?;
        if phrase == "根据值从数组中移除" {
            return Ok(ModifyOp::RemoveFromArray);
        }
        if phrase == "根据索引从数组中移除" {
            return Ok(ModifyOp::RemoveFromArrayByIndex);
        }
        let entry = self
            .catalog
            .resolve(Kind::Operator, &self.locale, &phrase)
            .ok_or_else(|| WorkshopError::Unknown {
                kind: "modify operator",
                spelling: phrase.clone(),
                locale: self.locale.clone(),
                span: Some(Span::new(self.file(), start, end)),
            })?;
        let op = match entry.id.as_str() {
            "add" => ModifyOp::Add,
            "subtract" => ModifyOp::Subtract,
            "multiply" => ModifyOp::Multiply,
            "divide" => ModifyOp::Divide,
            "modulo" => ModifyOp::Modulo,
            "raiseToPower" => ModifyOp::RaiseToPower,
            "appendToArray" => ModifyOp::AppendToArray,
            "removeFromArray" | "removeFromArrayByValue" => ModifyOp::RemoveFromArray,
            "removeFromArrayByIndex" => ModifyOp::RemoveFromArrayByIndex,
            other => {
                return Err(WorkshopError::Unsupported {
                    message: format!("unsupported modify operator '{other}'"),
                    span: Some(Span::new(self.file(), start, end)),
                });
            }
        };
        Ok(op)
    }

    fn value(&mut self) -> Result<wir::ValueId> {
        let mut value = self.primary()?;
        loop {
            if let Some(Token {
                kind: TokenKind::LBracket,
                start,
                ..
            }) = self.peek()
            {
                self.pos += 1;
                let index = self.value()?;
                let end = self.peek().map(|token| token.end).unwrap_or(start);
                self.expect(TokenKind::RBracket, "expected ']' after array index")?;
                value = self.target.values.push(ValueNode::new(
                    Value::Call {
                        name: "valueInArray".to_string(),
                        args: vec![value, index],
                    },
                    Some(Span::new(self.file(), start, end)),
                ));
                continue;
            }
            if matches!(
                self.peek(),
                Some(Token {
                    kind: TokenKind::Dot,
                    ..
                })
            ) {
                self.pos += 1;
                let (name, _, _) = self.phrase()?;
                let member = self
                    .target
                    .values
                    .push(ValueNode::new(Value::String(name), None));
                let mut args = vec![value, member];
                if matches!(
                    self.peek(),
                    Some(Token {
                        kind: TokenKind::LBracket,
                        ..
                    })
                ) {
                    self.pos += 1;
                    args.push(self.value()?);
                    self.expect(TokenKind::RBracket, "expected ']' after member index")?;
                }
                value = self.target.values.push(ValueNode::new(
                    Value::Call {
                        name: "memberAccess".to_string(),
                        args,
                    },
                    None,
                ));
                continue;
            }
            if let Some(Token {
                kind: TokenKind::Op(op),
                start,
                end,
            }) = self.peek()
            {
                if op == "?" {
                    self.pos += 1;
                    let when_true = self.value()?;
                    self.expect(TokenKind::Colon, "expected ':' in conditional value")?;
                    let when_false = self.value()?;
                    value = self.target.values.push(ValueNode::new(
                        Value::Call {
                            name: "ifThenElse".to_string(),
                            args: vec![value, when_true, when_false],
                        },
                        Some(Span::new(self.file(), start, end)),
                    ));
                    continue;
                }
                let compound_assignment = matches!(
                    self.peek_at(1).map(|token| token.kind),
                    Some(TokenKind::Op(equal)) if equal == "="
                );
                if !compound_assignment
                    && (is_comparison(&op)
                        || matches!(op.as_str(), "and" | "or" | "+" | "-" | "*" | "/" | "%"))
                {
                    self.pos += 1;
                    let right = self.primary()?;
                    value = self.target.values.push(ValueNode::new(
                        Value::Call {
                            name: op,
                            args: vec![value, right],
                        },
                        Some(Span::new(self.file(), start, end)),
                    ));
                    continue;
                }
            }
            break;
        }
        Ok(value)
    }

    fn primary(&mut self) -> Result<wir::ValueId> {
        match self.peek() {
            Some(Token {
                kind: TokenKind::Op(op),
                start,
                ..
            }) if op == "not" => {
                self.pos += 1;
                let value = self.primary()?;
                Ok(self.target.values.push(ValueNode::new(
                    Value::Call {
                        name: "not".to_string(),
                        args: vec![value],
                    },
                    Some(Span::new(self.file(), start, self.previous_span().1)),
                )))
            }
            Some(Token {
                kind: TokenKind::Number { value, text },
                start,
                end,
            }) => {
                let span = Some(Span::new(self.file(), start, end));
                self.pos += 1;
                Ok(self
                    .target
                    .values
                    .push(ValueNode::new(Value::Number { value, text }, span)))
            }
            Some(Token {
                kind: TokenKind::Op(op),
                start,
                ..
            }) if op == "-" => {
                if let Some(Token {
                    kind: TokenKind::Number { value, text },
                    end: number_end,
                    ..
                }) = self.peek_at(1)
                {
                    let span = Some(Span::new(self.file(), start, number_end));
                    self.pos += 2;
                    Ok(self.target.values.push(ValueNode::new(
                        Value::Number {
                            value: -value,
                            text: format!("-{text}"),
                        },
                        span,
                    )))
                } else {
                    Err(self.malformed("expected a number after '-'", &self.peek().unwrap()))
                }
            }
            Some(Token {
                kind: TokenKind::String(content),
                start,
                end,
            }) => {
                let span = Some(Span::new(self.file(), start, end));
                self.pos += 1;
                Ok(self
                    .target
                    .values
                    .push(ValueNode::new(Value::String(content), span)))
            }
            Some(Token {
                kind: TokenKind::Word(word),
                ..
            }) if matches!(canonical_keyword(&word), "Global" | "global") => {
                let (start, end) = self.span_here();
                self.pos += 1;
                // The reference's value spelling `Global Variable(name)`
                // (#119 differential evidence); the OPY form `Global.name`
                // stays supported.
                if let Some(Token {
                    kind: TokenKind::Word(next),
                    ..
                }) = self.peek()
                {
                    if canonical_keyword(&next) == "Variable"
                        || canonical_keyword(&next) == "variable"
                    {
                        self.pos += 1;
                        self.expect(TokenKind::LParen, "expected '(' after 'Global Variable'")?;
                        let (name, _, _) = self.phrase()?;
                        let variable = self.global_by_name(&name)?;
                        self.expect(TokenKind::RParen, "expected ')' after Global Variable")?;
                        let span = Some(Span::new(self.file(), start, end));
                        return Ok(self
                            .target
                            .values
                            .push(ValueNode::new(Value::GlobalVariable(variable), span)));
                    }
                }
                self.expect(TokenKind::Dot, "expected '.' after 'Global'")?;
                let (name, _, _) = self.phrase()?;
                let variable = self.global_by_name(&name)?;
                let span = Some(Span::new(self.file(), start, end));
                Ok(self
                    .target
                    .values
                    .push(ValueNode::new(Value::GlobalVariable(variable), span)))
            }
            Some(Token {
                kind: TokenKind::Word(word),
                ..
            }) if matches!(canonical_keyword(&word), "Player" | "player")
                && matches!(
                    self.peek_at(1),
                    Some(Token {
                        kind: TokenKind::Word(next),
                        ..
                    }) if matches!(canonical_keyword(&next), "Variable" | "variable")
                ) =>
            {
                // The reference's playervar-read spelling
                // `Player Variable(player, name)` (#119 differential
                // evidence).
                let (start, end) = self.span_here();
                self.pos += 1;
                let saved = self.expected_domain;
                self.expected_domain = None;
                let result = (|| {
                    self.consume_phrase("Variable")?;
                    self.expect(TokenKind::LParen, "expected '(' after 'Player Variable'")?;
                    let player = self.value()?;
                    self.expect(TokenKind::Comma, "expected ',' after player")?;
                    let (name, _, _) = self.phrase()?;
                    let variable = self.player_by_name(&name)?;
                    self.expect(TokenKind::RParen, "expected ')' after Player Variable")?;
                    Ok(self.target.values.push(ValueNode::new(
                        Value::PlayerVariable { player, variable },
                        Some(Span::new(self.file(), start, end)),
                    )))
                })();
                self.expected_domain = saved;
                result
            }
            Some(Token {
                kind: TokenKind::Word(word),
                ..
            }) if word == "Event" => {
                let (start, _) = self.span_here();
                self.pos += 1;
                if matches!(self.peek(), Some(Token { kind: TokenKind::Word(next), .. }) if matches!(canonical_keyword(&next), "Player" | "player"))
                {
                    self.pos += 1;
                    let player = self.target.values.push(ValueNode::new(
                        Value::EventPlayer,
                        Some(Span::new(self.file(), start, self.previous_span().1)),
                    ));
                    if matches!(
                        self.peek(),
                        Some(Token {
                            kind: TokenKind::Dot,
                            ..
                        })
                    ) {
                        self.pos += 1;
                        let (name, name_start, name_end) = self.phrase()?;
                        let variable = self.player_by_name(&name)?;
                        return Ok(self.target.values.push(ValueNode::new(
                            Value::PlayerVariable { player, variable },
                            Some(Span::new(self.file(), name_start, name_end)),
                        )));
                    }
                    return Ok(player);
                }
                self.pos -= 1;
                let (phrase, start, end) = self.phrase()?;
                if matches!(
                    self.peek(),
                    Some(Token {
                        kind: TokenKind::LParen,
                        ..
                    })
                ) {
                    self.call_or_enum(&phrase, start, end)
                } else {
                    self.bare_member(&phrase, start, end)
                }
            }
            Some(Token {
                kind: TokenKind::LParen,
                ..
            }) => {
                // The oracle's playervar-read spelling parenthesizes the
                // receiver: `(Event Player).p` (#87).
                self.pos += 1;
                let inner = self.value()?;
                if matches!(
                    self.peek(),
                    Some(Token {
                        kind: TokenKind::Dot,
                        ..
                    })
                ) {
                    self.pos += 1;
                    let (name, _, _) = self.phrase()?;
                    let member = self
                        .target
                        .values
                        .push(ValueNode::new(Value::String(name), None));
                    let mut args = vec![inner, member];
                    if matches!(
                        self.peek(),
                        Some(Token {
                            kind: TokenKind::LBracket,
                            ..
                        })
                    ) {
                        self.pos += 1;
                        args.push(self.value()?);
                        self.expect(TokenKind::RBracket, "expected ']' after member index")?;
                    }
                    let mut accessed = self.target.values.push(ValueNode::new(
                        Value::Call {
                            name: "memberAccess".to_string(),
                            args,
                        },
                        None,
                    ));
                    if matches!(self.peek(), Some(Token { kind: TokenKind::Op(op), .. }) if op == "?")
                    {
                        self.pos += 1;
                        let when_true = self.value()?;
                        self.expect(TokenKind::Colon, "expected ':' in conditional value")?;
                        let when_false = self.value()?;
                        accessed = self.target.values.push(ValueNode::new(
                            Value::Call {
                                name: "ifThenElse".to_string(),
                                args: vec![accessed, when_true, when_false],
                            },
                            None,
                        ));
                    }
                    self.expect(TokenKind::RParen, "expected ')' after member access")?;
                    return Ok(accessed);
                }
                self.expect(TokenKind::RParen, "expected ')' after parenthesized value")?;
                if let Some(Token {
                    kind: TokenKind::Dot,
                    ..
                }) = self.peek()
                {
                    self.pos += 1;
                    let (name, _, _) = self.phrase()?;
                    if matches!(
                        self.target.values.get(inner),
                        Some(ValueNode {
                            value: Value::EventPlayer,
                            ..
                        })
                    ) {
                        let variable = self.player_by_name(&name)?;
                        Ok(self.target.values.push(ValueNode::new(
                            Value::PlayerVariable {
                                player: inner,
                                variable,
                            },
                            None,
                        )))
                    } else {
                        let member = self
                            .target
                            .values
                            .push(ValueNode::new(Value::String(name), None));
                        let mut args = vec![inner, member];
                        if matches!(
                            self.peek(),
                            Some(Token {
                                kind: TokenKind::LBracket,
                                ..
                            })
                        ) {
                            self.pos += 1;
                            args.push(self.value()?);
                            self.expect(TokenKind::RBracket, "expected ']' after member index")?;
                        }
                        Ok(self.target.values.push(ValueNode::new(
                            Value::Call {
                                name: "memberAccess".to_string(),
                                args,
                            },
                            None,
                        )))
                    }
                } else {
                    Ok(inner)
                }
            }
            _ => {
                let (phrase, start, end) = self.phrase()?;
                match canonical_keyword(&phrase) {
                    "True" | "真" => Ok(self.push_bool(true, start, end)),
                    "False" | "假" => Ok(self.push_bool(false, start, end)),
                    "Event Player" => Ok(self.target.values.push(ValueNode::new(
                        Value::EventPlayer,
                        Some(Span::new(self.file(), start, end)),
                    ))),
                    "Null" => Ok(self.target.values.push(ValueNode::new(
                        Value::Null,
                        Some(Span::new(self.file(), start, end)),
                    ))),
                    _ => {
                        if let Some(Token {
                            kind: TokenKind::LParen,
                            ..
                        }) = self.peek()
                        {
                            self.call_or_enum(&phrase, start, end)
                        } else {
                            self.bare_member(&phrase, start, end)
                        }
                    }
                }
            }
        }
    }

    fn call_or_enum(
        &mut self,
        phrase: &str,
        start: Position,
        end: Position,
    ) -> Result<wir::ValueId> {
        // A value function wins over an enum domain of the same spelling
        // (e.g. `Vector(x, y, z)` is the value function; `Vector` as an enum
        // domain only appears through bare members like `Up`).
        let prefer_enum = self.catalog.enum_domain("Hero").is_some()
            && (canonical_keyword(phrase) == "Hero"
                || self.resolve_enum_domain_mixed(phrase) == Some("Hero"));
        if !prefer_enum {
            if let Some(entry) = self.resolve_entry(Kind::Value, phrase) {
                self.expect(TokenKind::LParen, "expected '(' after value name")?;
                if entry.id == "compare" {
                    // Compare(a, op, b) -> Call(op, [a, b]). The operands are
                    // value positions, not signature-pinned arguments, so the
                    // enclosing expected domain must not leak in (#111).
                    let saved = self.expected_domain;
                    self.expected_domain = None;
                    let left = self.value();
                    self.expected_domain = saved;
                    let left = left?;
                    self.expect(TokenKind::Comma, "expected ',' after Compare operand")?;
                    let (op, op_start, op_end) = match self.next() {
                        Some(Token {
                            kind: TokenKind::Op(op),
                            start,
                            end,
                        }) => (op, start, end),
                        Some(token) => {
                            return Err(
                                self.malformed("expected a comparison operator in Compare", &token)
                            );
                        }
                        None => {
                            return Err(self.malformed(
                                "expected a comparison operator in Compare",
                                self.eof(),
                            ));
                        }
                    };
                    self.expect(TokenKind::Comma, "expected ',' after Compare operator")?;
                    let saved = self.expected_domain;
                    self.expected_domain = None;
                    let right = self.value();
                    self.expected_domain = saved;
                    let right = right?;
                    self.expect(TokenKind::RParen, "expected ')'")?;
                    let span = Some(Span::new(self.file(), op_start, op_end));
                    return Ok(self.target.values.push(ValueNode::new(
                        Value::Call {
                            name: op,
                            args: vec![left, right],
                        },
                        span,
                    )));
                }
                let args = self.value_args(entry.id.as_str())?;
                self.expect(TokenKind::RParen, "expected ')'")?;
                return Ok(self.target.values.push(ValueNode::new(
                    Value::Call {
                        name: entry.id.clone(),
                        args,
                    },
                    Some(Span::new(self.file(), start, end)),
                )));
            }
        }
        if let Some(domain_name) = self
            .resolve_enum_domain_mixed(phrase)
            .or_else(|| self.resolve_enum_domain_mixed(canonical_keyword(phrase)))
        {
            let domain = self
                .catalog
                .enum_domain(domain_name)
                .expect("resolved enum domain must exist");
            // Enum call: `Color(Yellow)`.
            self.expect(TokenKind::LParen, "expected '('")?;
            let (member_phrase, _, _) = self.enum_member_phrase()?;
            let member = self
                .resolve_enum_member_mixed(&domain.domain, &member_phrase)
                .unwrap_or_else(|| (domain.domain.clone(), member_phrase.clone()));
            self.expect(TokenKind::RParen, "expected ')' after enum member")?;
            return Ok(self.target.values.push(ValueNode::new(
                Value::Enum {
                    value_type: member.0,
                    value: member.1,
                },
                Some(Span::new(self.file(), start, end)),
            )));
        }
        if matches!(self.peek().map(|token| token.kind), Some(TokenKind::LParen)) {
            self.pos += 1;
            self.call_stack.push(phrase.to_string());
            let args = self.opaque_value_args()?;
            self.call_stack.pop();
            self.expect(TokenKind::RParen, "expected ')' after value call")?;
            return Ok(self.target.values.push(ValueNode::new(
                Value::Call {
                    name: phrase.to_string(),
                    args,
                },
                Some(Span::new(self.file(), start, end)),
            )));
        }
        Ok(self.target.values.push(ValueNode::new(
            Value::Call {
                name: phrase.to_string(),
                args: Vec::new(),
            },
            Some(Span::new(self.file(), start, end)),
        )))
    }

    fn bare_member(
        &mut self,
        phrase: &str,
        start: Position,
        end: Position,
    ) -> Result<wir::ValueId> {
        // Some enum members use a colon in their Workshop spelling (for
        // example `Arrow: Up`). `phrase()` intentionally stops at the colon
        // for ordinary identifiers, so complete the member only when the
        // enclosing signature has already declared an enum domain.
        if self.expected_domain.is_some()
            && matches!(self.peek().map(|token| token.kind), Some(TokenKind::Colon))
        {
            self.pos += 1;
            let (suffix, _, suffix_end) = self.phrase()?;
            let owned_phrase = format!("{phrase}: {suffix}");
            return self.bare_member_resolved(&owned_phrase, start, suffix_end);
        }
        self.bare_member_resolved(phrase, start, end)
    }

    fn bare_member_resolved(
        &mut self,
        phrase: &str,
        start: Position,
        end: Position,
    ) -> Result<wir::ValueId> {
        match (matches!(phrase, "None" | "无"), self.expected_domain) {
            (true, Some(expected))
                if matches!(
                    expected,
                    "ChaseTimeReeval"
                        | "ChaseRateReeval"
                        | "Invis"
                        | "ThrottleReeval"
                        | "EffectReeval"
                ) =>
            {
                return Ok(self.target.values.push(ValueNode::new(
                    Value::Enum {
                        value_type: expected.to_string(),
                        value: "NONE".to_string(),
                    },
                    Some(Span::new(self.file(), start, end)),
                )));
            }
            _ => {}
        }
        if let Some(expected) = self.expected_domain {
            if let Some((value_type, value)) = self.resolve_enum_member_mixed(expected, phrase) {
                return Ok(self.target.values.push(ValueNode::new(
                    Value::Enum { value_type, value },
                    Some(Span::new(self.file(), start, end)),
                )));
            }
        }
        if let Some((value_type, value)) = self.resolve_enum_member_mixed("Team", phrase) {
            return Ok(self.target.values.push(ValueNode::new(
                Value::Enum { value_type, value },
                Some(Span::new(self.file(), start, end)),
            )));
        }
        if phrase == "Visible To and String" {
            return Ok(self.target.values.push(ValueNode::new(
                Value::Enum {
                    value_type: "HudReeval".to_string(),
                    value: "VISIBILITY_AND_STRING".to_string(),
                },
                Some(Span::new(self.file(), start, end)),
            )));
        }
        if (self.expected_domain.is_none()
            && matches!(phrase, "Up" | "上")
            && (self
                .call_stack
                .last()
                .is_some_and(|call| call == "multiply")
                || self.call_stack.is_empty()
                || self
                    .call_stack
                    .iter()
                    .any(|call| call == "startAcceleration")
                || self.call_stack.iter().any(|call| {
                    call == "raycastHitPosition"
                        || call == "Direction Towards"
                        || call == "directionTowards"
                })))
            || (matches!(self.expected_domain, Some("Position")) && matches!(phrase, "Up" | "上"))
        {
            return Ok(self.target.values.push(ValueNode::new(
                Value::Enum {
                    value_type: "Vector".to_string(),
                    value: "UP".to_string(),
                },
                Some(Span::new(self.file(), start, end)),
            )));
        }
        // Event filter domains are resolved by the event parser and are not
        // value-argument domains. Excluding them here keeps their spellings
        // from making unrelated bare value arguments ambiguous (for example,
        // `All` in `Set Invisible(..., All)`).
        let matches: Vec<(String, String)> = self
            .catalog
            .bare_member_matches(&self.locale, phrase)
            .into_iter()
            .filter(|(domain, _)| domain != "EventTeam" && domain != "EventPlayer")
            .collect();
        if matches.len() == 1 {
            return Ok(self.target.values.push(ValueNode::new(
                Value::Enum {
                    value_type: matches[0].0.clone(),
                    value: matches[0].1.clone(),
                },
                Some(Span::new(self.file(), start, end)),
            )));
        }
        if matches.len() > 1 {
            // #111: a bare spelling shared by several enum domains resolves
            // only when the enclosing call's canonical signature pins exactly
            // one of the matching domains. No pin keeps the deterministic
            // ambiguity diagnostic — no guessing, no global precedence.
            if let Some(expected) = self.expected_domain {
                let pinned: Vec<&(String, String)> = matches
                    .iter()
                    .filter(|(domain, _)| domain == expected)
                    .collect();
                if pinned.len() == 1 {
                    return Ok(self.target.values.push(ValueNode::new(
                        Value::Enum {
                            value_type: pinned[0].0.clone(),
                            value: pinned[0].1.clone(),
                        },
                        Some(Span::new(self.file(), start, end)),
                    )));
                }
            }
            return Err(WorkshopError::Unsupported {
                message: format!("ambiguous enum member '{phrase}' (multiple domains match)"),
                span: Some(Span::new(self.file(), start, end)),
            });
        }
        // A bare value constant (e.g. Empty Array).
        if let Some(entry) = self.resolve_entry(Kind::Value, phrase) {
            if entry.id == "null" {
                return Ok(self.target.values.push(ValueNode::new(
                    Value::Null,
                    Some(Span::new(self.file(), start, end)),
                )));
            }
            return Ok(self.target.values.push(ValueNode::new(
                Value::Call {
                    name: entry.id.clone(),
                    args: Vec::new(),
                },
                Some(Span::new(self.file(), start, end)),
            )));
        }
        Ok(self.target.values.push(ValueNode::new(
            Value::Call {
                name: phrase.to_string(),
                args: Vec::new(),
            },
            Some(Span::new(self.file(), start, end)),
        )))
    }

    fn value_args(&mut self, call_id: &str) -> Result<Vec<wir::ValueId>> {
        let mut args = Vec::new();
        if let Some(Token {
            kind: TokenKind::RParen,
            ..
        }) = self.peek()
        {
            return Ok(args);
        }
        self.call_stack.push(call_id.to_string());
        let mut arg_index = 0usize;
        loop {
            // Each argument is parsed with the domain its position expects
            // per the enclosing call's canonical signature (#111); nested
            // calls override the expectation for their own arguments.
            let saved = self.expected_domain;
            self.expected_domain = self.context.expected_domain(call_id, arg_index);
            let arg = self.value();
            self.expected_domain = saved;
            args.push(arg?);
            arg_index += 1;
            match self.peek() {
                Some(Token {
                    kind: TokenKind::Comma,
                    ..
                }) => {
                    self.pos += 1;
                }
                Some(Token {
                    kind: TokenKind::Colon,
                    ..
                }) => {
                    self.pos += 1;
                    if !matches!(
                        self.peek(),
                        Some(Token {
                            kind: TokenKind::RParen,
                            ..
                        })
                    ) {
                        let _ = self.phrase()?;
                    }
                    match self.peek() {
                        Some(Token {
                            kind: TokenKind::Comma,
                            ..
                        }) => self.pos += 1,
                        Some(Token {
                            kind: TokenKind::RParen,
                            ..
                        }) => break,
                        Some(token) => return Err(self.malformed("expected ',' or ')'", &token)),
                        None => {
                            return Err(self.malformed("unexpected end of value call", self.eof()));
                        }
                    }
                }
                _ => break,
            }
        }
        self.call_stack.pop();
        Ok(args)
    }

    fn opaque_value_args(&mut self) -> Result<Vec<wir::ValueId>> {
        let mut args = Vec::new();
        if matches!(self.peek().map(|token| token.kind), Some(TokenKind::RParen)) {
            return Ok(args);
        }
        loop {
            args.push(self.value()?);
            match self.peek() {
                Some(Token {
                    kind: TokenKind::Colon,
                    ..
                }) => {
                    self.pos += 1;
                    if !matches!(
                        self.peek(),
                        Some(Token {
                            kind: TokenKind::RParen,
                            ..
                        })
                    ) {
                        let _ = self.phrase()?;
                    }
                    match self.peek() {
                        Some(Token {
                            kind: TokenKind::Comma,
                            ..
                        }) => self.pos += 1,
                        Some(Token {
                            kind: TokenKind::RParen,
                            ..
                        }) => break,
                        Some(token) => return Err(self.malformed("expected ',' or ')'", &token)),
                        None => {
                            return Err(self.malformed("unexpected end of value call", self.eof()));
                        }
                    }
                }
                Some(Token {
                    kind: TokenKind::Comma,
                    ..
                }) => self.pos += 1,
                Some(Token {
                    kind: TokenKind::RParen,
                    ..
                }) => break,
                Some(token) => return Err(self.malformed("expected ',' or ')'", &token)),
                None => return Err(self.malformed("unexpected end of value call", self.eof())),
            }
        }
        Ok(args)
    }

    fn line_has_assignment(&self) -> bool {
        self.tokens[self.pos..]
            .iter()
            .take_while(|token| !matches!(token.kind, TokenKind::Semi | TokenKind::RBrace))
            .any(|token| matches!(&token.kind, TokenKind::Op(op) if matches!(op.as_str(), "=" | "+=" | "-=" | "*=" | "/=" | "%=")))
    }

    fn push_bool(&mut self, value: bool, start: Position, end: Position) -> wir::ValueId {
        self.target.values.push(ValueNode::new(
            Value::Bool(value),
            Some(Span::new(self.file(), start, end)),
        ))
    }

    fn global_by_name(&mut self, name: &str) -> Result<wir::GlobalVarId> {
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

    fn player_by_name(&mut self, name: &str) -> Result<wir::PlayerVarId> {
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

    fn next_variable_index(&self, player: bool) -> u32 {
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

    fn subroutine_by_name(&self, name: &str) -> Result<wir::SubroutineId> {
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

    /// Read the maximal phrase of consecutive words (space-joined). Phrases
    /// may span lines because long Workshop action arguments wrap mid-phrase.
    fn phrase(&mut self) -> Result<(String, Position, Position)> {
        let mut words = Vec::new();
        let (start, mut end) = match self.peek() {
            Some(Token {
                kind: TokenKind::Word(word),
                start,
                end,
            }) => {
                words.push(word.clone());
                (start, end)
            }
            Some(Token {
                kind: TokenKind::Number { text, .. },
                start,
                end,
            }) => {
                words.push(text.clone());
                (start, end)
            }
            Some(token) => return Err(self.malformed("expected an identifier", &token)),
            None => return Err(self.malformed("expected an identifier", self.eof())),
        };
        self.pos += 1;
        while let Some(token) = self.peek() {
            match token {
                Token {
                    kind: TokenKind::Word(word),
                    end: word_end,
                    ..
                } => {
                    words.push(word.clone());
                    end = word_end;
                    self.pos += 1;
                }
                // Enum members embed numbers (`Team 2`, `Ability 2`); the
                // lexer splits them from the word, so phrases join Number
                // tokens too (#119 reference evidence).
                Token {
                    kind: TokenKind::Number { text, .. },
                    end: number_end,
                    ..
                } => {
                    words.push(text.clone());
                    end = number_end;
                    self.pos += 1;
                }
                _ => break,
            }
        }
        Ok((words.join(" "), start, end))
    }

    /// Read a single-line phrase (stops at a line boundary). Used for names
    /// that are structurally one per line, such as variable declarations.
    fn phrase_on_line(&mut self) -> Result<(String, Position, Position)> {
        let mut words = Vec::new();
        let (start, mut end, line) = match self.peek() {
            Some(Token {
                kind: TokenKind::Word(word),
                start,
                end,
            }) => {
                words.push(word.clone());
                (start, end, start.line)
            }
            Some(Token {
                kind: TokenKind::Number { text, .. },
                start,
                end,
            }) => {
                words.push(text.clone());
                (start, end, start.line)
            }
            Some(token) => return Err(self.malformed("expected an identifier", &token)),
            None => return Err(self.malformed("expected an identifier", self.eof())),
        };
        self.pos += 1;
        while let Some(token) = self.peek() {
            let (word, word_start, word_end) = match token {
                Token {
                    kind: TokenKind::Word(word),
                    start,
                    end,
                } => (word, start, end),
                Token {
                    kind: TokenKind::Number { text, .. },
                    start,
                    end,
                } => (text, start, end),
                Token {
                    kind: TokenKind::Dot,
                    start,
                    end,
                } => (".".to_string(), start, end),
                Token {
                    kind: TokenKind::Op(op),
                    start,
                    end,
                } if matches!(op.as_str(), "-" | "%") => (op.clone(), start, end),
                _ => break,
            };
            if word_start.line != line {
                break;
            }
            words.push(word);
            end = word_end;
            self.pos += 1;
        }
        Ok((
            words
                .join(" ")
                .replace(" .", ".")
                .replace(". ", ".")
                .replace(" %", "%"),
            start,
            end,
        ))
    }

    fn enum_member_phrase(&mut self) -> Result<(String, Position, Position)> {
        let first = self
            .peek()
            .ok_or_else(|| self.malformed("expected an enum member", self.eof()))?;
        let start = first.start;
        let line = first.start.line;
        let mut end = first.end;
        let mut parts = Vec::new();
        while let Some(token) = self.peek() {
            if token.start.line != line
                || matches!(token.kind, TokenKind::RParen | TokenKind::Comma)
            {
                break;
            }
            self.pos += 1;
            end = token.end;
            parts.push(raw_token_text(&token.kind));
        }
        if parts.is_empty() {
            return Err(self.malformed("expected an enum member", &first));
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

    /// Read a text line (tokens until `;`), joining words and dashes into
    /// the literal text, and consume the terminating `;`.
    fn line_text(&mut self) -> Result<String> {
        let mut parts = Vec::new();
        loop {
            match self.peek() {
                Some(Token {
                    kind: TokenKind::Semi,
                    ..
                }) => {
                    self.pos += 1;
                    break;
                }
                Some(Token {
                    kind: TokenKind::Word(word),
                    ..
                }) => {
                    parts.push(word.clone());
                    self.pos += 1;
                }
                Some(Token {
                    kind: TokenKind::Op(op),
                    ..
                }) if op == "-" => {
                    parts.push("-".to_string());
                    self.pos += 1;
                }
                Some(Token {
                    kind: TokenKind::Number { value, .. },
                    ..
                }) => {
                    parts.push(value.to_string());
                    self.pos += 1;
                }
                Some(Token {
                    kind: TokenKind::Dot,
                    ..
                }) => {
                    parts.push(".".to_string());
                    self.pos += 1;
                }
                Some(Token {
                    kind: TokenKind::Colon,
                    ..
                }) => {
                    parts.push(":".to_string());
                    self.pos += 1;
                }
                Some(token) => return Err(self.malformed("expected a text line", &token)),
                None => return Err(self.malformed("unexpected end of input in line", self.eof())),
            }
        }
        Ok(parts
            .join(" ")
            .replace(" .", ".")
            .replace(". ", ".")
            .replace(" : ", ":"))
    }

    /// Consume a known keyword phrase, verifying its spelling.
    fn consume_phrase(&mut self, expected: &str) -> Result<()> {
        let (phrase, _, _) = self.phrase()?;
        if phrase != expected {
            return Err(self.malformed(&format!("expected '{expected}'"), self.previous()));
        }
        Ok(())
    }

    fn expect_keyword(&mut self, expected: &str) -> Result<Position> {
        match self.next() {
            Some(Token {
                kind: TokenKind::Word(word),
                start,
                ..
            }) if canonical_keyword(&word) == expected => Ok(start),
            Some(token) => Err(self.malformed(&format!("expected '{expected}'"), &token)),
            None => Err(self.malformed(&format!("expected '{expected}'"), self.eof())),
        }
    }

    fn expect(&mut self, kind: TokenKind, message: &str) -> Result<()> {
        match self.next() {
            Some(token) if token.kind == kind => Ok(()),
            Some(token) => Err(self.malformed(message, &token)),
            None => Err(self.malformed(message, self.eof())),
        }
    }

    fn expect_string(&mut self, message: &str) -> Result<String> {
        match self.next() {
            Some(Token {
                kind: TokenKind::String(content),
                ..
            }) => Ok(content),
            Some(token) => Err(self.malformed(message, &token)),
            None => Err(self.malformed(message, self.eof())),
        }
    }

    fn malformed(&self, message: &str, token: &Token) -> WorkshopError {
        WorkshopError::Malformed {
            message: message.to_string(),
            span: Some(Span::new(self.file(), token.start, token.end)),
        }
    }

    fn unknown(&self, kind: &'static str, spelling: &str) -> WorkshopError {
        WorkshopError::Unknown {
            kind,
            spelling: spelling.to_string(),
            locale: self.locale.clone(),
            span: None,
        }
    }

    fn peek(&self) -> Option<Token> {
        self.tokens.get(self.pos).cloned()
    }

    fn peek_at(&self, offset: usize) -> Option<Token> {
        self.tokens.get(self.pos + offset).cloned()
    }

    fn next(&mut self) -> Option<Token> {
        let token = self.tokens.get(self.pos).cloned();
        if token.is_some() {
            self.pos += 1;
        }
        token
    }

    fn previous(&self) -> &Token {
        self.tokens
            .get(self.pos.saturating_sub(1))
            .unwrap_or_else(|| self.tokens.last().unwrap())
    }

    fn previous_span(&self) -> (Position, Position) {
        let token = self.previous();
        (token.start, token.end)
    }

    fn span_here(&self) -> (Position, Position) {
        let token = self
            .peek()
            .unwrap_or_else(|| self.tokens.last().unwrap().clone());
        (token.start, token.end)
    }

    fn eof(&self) -> &Token {
        self.tokens.last().unwrap()
    }

    fn file(&self) -> crate::ids::Id<SourceFile> {
        crate::ids::Id::from_index(0)
    }
}

fn is_comparison(op: &str) -> bool {
    matches!(op, "==" | "!=" | "<" | "<=" | ">" | ">=")
}

fn canonical_keyword(keyword: &str) -> &str {
    match keyword {
        "设置" => "settings",
        "变量" => "variables",
        "子程序" => "subroutines",
        "规则" => "rule",
        "事件" => "event",
        "条件" => "conditions",
        "动作" => "actions",
        "主程序" => "main",
        "大厅" => "lobby",
        "模式" => "modes",
        "英雄" => "heroes",
        "地图" => "Map",
        "循环" => "Loop",
        "For 玩家变量" => "For Player Variable",
        "如条件为“真”则循环" => "Loop If Condition Is True",
        "全局" => "global",
        "玩家" => "player",
        "事件玩家" => "Event Player",
        "禁用" => "disabled",
        "结束" => "End",
        "否则如果" => "Else If",
        "否则" => "Else",
        other => other,
    }
}

fn raw_token_text(kind: &TokenKind) -> String {
    match kind {
        TokenKind::Word(value) => value.clone(),
        TokenKind::Number { text, .. } => text.clone(),
        TokenKind::String(value) => format!("\"{}\"", value.replace('"', "\\\"")),
        TokenKind::Op(value) => value.clone(),
        TokenKind::LParen => "(".to_string(),
        TokenKind::RParen => ")".to_string(),
        TokenKind::Comma => ",".to_string(),
        TokenKind::Semi => ";".to_string(),
        TokenKind::LBrace => "{".to_string(),
        TokenKind::RBrace => "}".to_string(),
        TokenKind::Colon => ":".to_string(),
        TokenKind::Dot => ".".to_string(),
        TokenKind::LBracket => "[".to_string(),
        TokenKind::RBracket => "]".to_string(),
        TokenKind::Eof => String::new(),
    }
}
