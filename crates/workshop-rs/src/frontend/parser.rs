pub(crate) use std::collections::HashMap;
use std::sync::OnceLock;

pub(crate) use crate::core::signatures::ExpectedDomain;
pub(crate) use crate::core::source::{Position, SourceFile, Span};
use crate::program::Program;
pub(crate) use crate::settings::table::KeyKind;
pub(crate) use crate::settings::{PathPart, table};
pub(crate) use crate::settings::{Settings, SettingsListElement, SettingsNode};
pub(crate) use crate::wir::{
    self, Action, Event, EventTarget, EventTeam, ModifyOp, PlayerEventKind, Value, ValueNode,
};

use super::keywords::{canonical_structural_id, spelling_token_len, structural_spellings};
pub(crate) use super::lexer::{Token, TokenKind, tokenize};
pub(crate) use crate::catalog::{Catalog, Kind, Locale};
pub(crate) use crate::core::error::{Result, WorkshopError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Stop {
    SectionClosed,
    End,
    ElseIf,
    Else,
}

pub(crate) enum AssignmentOperator {
    Set,
    Modify(ModifyOp),
}

/// Parse localized Workshop text into Workshop IR using the catalog's
/// canonical call-signature context. Ambiguous bare enum members resolve when
/// their enclosing call pins one matching domain; unpinned ambiguity remains a
/// structured canonical value. See [`parse_with_context`] when a
/// consumer needs to provide additional signature context.
pub fn parse(input: &str, catalog: &Catalog, locale: &Locale) -> Result<Program> {
    parse_with_context(input, catalog, locale, catalog)
}

#[cfg(test)]
pub(crate) fn parse_wir(input: &str, catalog: &Catalog, locale: &Locale) -> Result<wir::Program> {
    parse_wir_with_context(input, catalog, locale, catalog)
}

/// Compatibility entry point for crate-internal storage tests and migration
/// checks. Ordinary consumers should use [`parse`], which returns [`Program`].
/// Parse localized Workshop text into Workshop IR, resolving ambiguous bare
/// enum members from the enclosing call's canonical signature context (#111).
///
/// When a bare member spelling matches several enum domains, the parser asks
/// [`ExpectedDomain::expected_domain`] for the domain the enclosing call's
/// signature expects at that argument position; the member resolves only when
/// that expected domain is one of the matching domains (i.e. the signature
/// pins exactly one). Without a pin all matching candidates are preserved.
pub fn parse_with_context(
    input: &str,
    catalog: &Catalog,
    locale: &Locale,
    context: &dyn ExpectedDomain,
) -> Result<Program> {
    parse_wir_with_context(input, catalog, locale, context).and_then(Program::from_wir)
}

pub(crate) fn parse_wir_with_context(
    input: &str,
    catalog: &Catalog,
    locale: &Locale,
    context: &dyn ExpectedDomain,
) -> Result<wir::Program> {
    let tokens = tokenize(input).map_err(|error| {
        WorkshopError::malformed(error.message, Some(synthetic_span(error.position)))
    })?;
    ParseContext {
        tokens,
        pos: 0,
        source: input,
        catalog,
        locale: locale.clone(),
        context,
        expected_domain: None,
        condition_pending: false,
        in_condition: false,
        call_stack: Vec::new(),
        target: wir::Program::default(),
        globals: HashMap::new(),
        players: HashMap::new(),
        subroutines: HashMap::new(),
        next_global_index: 0,
        next_player_index: 0,
        structural_spellings: HashMap::new(),
    }
    .program()
}

pub(crate) fn synthetic_span(position: Position) -> Span {
    Span::new(crate::core::ids::Id::from_index(0), position, position)
}

pub(crate) struct ParseContext<'a> {
    pub(crate) tokens: Vec<Token>,
    pub(crate) pos: usize,
    pub(crate) source: &'a str,
    pub(crate) catalog: &'a Catalog,
    pub(crate) locale: Locale,
    pub(crate) context: &'a dyn ExpectedDomain,
    pub(crate) expected_domain: Option<&'a str>,
    pub(crate) condition_pending: bool,
    pub(crate) in_condition: bool,
    pub(crate) call_stack: Vec<String>,
    pub(crate) target: wir::Program,
    pub(crate) globals: HashMap<String, wir::GlobalVarId>,
    pub(crate) players: HashMap<String, wir::PlayerVarId>,
    pub(crate) subroutines: HashMap<String, wir::SubroutineId>,
    pub(crate) next_global_index: u32,
    pub(crate) next_player_index: u32,
    structural_spellings: HashMap<String, Vec<String>>,
}

impl<'a> ParseContext<'a> {
    pub(crate) fn with_domain<T>(
        &mut self,
        domain: Option<&'a str>,
        f: impl FnOnce(&mut Self) -> Result<T>,
    ) -> Result<T> {
        let saved = self.expected_domain;
        self.expected_domain = domain;
        let result = f(self);
        self.expected_domain = saved;
        result
    }

    pub(crate) fn value_in_domain(&mut self, domain: Option<&'a str>) -> Result<wir::ValueId> {
        self.with_domain(domain, |p| p.value())
    }

    pub(crate) fn resolve_entry(
        &self,
        kind: Kind,
        spelling: &str,
    ) -> Option<&'a crate::catalog::CatalogEntry> {
        self.catalog
            .resolve(kind, &self.locale, spelling)
            .or_else(|| {
                if self.locale != *self.catalog.primary_locale() {
                    self.catalog
                        .resolve(kind, self.catalog.primary_locale(), spelling)
                } else {
                    None
                }
            })
    }

    pub(crate) fn canonical_keyword(&self, spelling: &str) -> String {
        canonical_structural_id(self.catalog, &self.locale, spelling)
    }

    pub(crate) fn line_has_assignment(&self) -> bool {
        let tokens: Vec<_> = self.tokens[self.pos..]
            .iter()
            .take_while(|token| !matches!(token.kind, TokenKind::Semi | TokenKind::RBrace))
            .collect();
        tokens.iter().any(|token| {
            matches!(&token.kind, TokenKind::Op(op) if matches!(op.as_str(), "=" | "+=" | "-=" | "*=" | "/=" | "%="))
        }) || tokens.windows(2).any(|window| {
            matches!(&window[0].kind, TokenKind::Word(_))
                && matches!(&window[1].kind, TokenKind::Op(op) if op == "=")
        })
    }

    pub(crate) fn phrase(&mut self) -> Result<(String, Position, Position)> {
        let mut parts: Vec<(String, Position, Position)> = Vec::new();
        let (start, mut end) = match self.peek() {
            Some(Token {
                kind: TokenKind::Word(word),
                start,
                end,
            }) => {
                parts.push((word.clone(), start, end));
                (start, end)
            }
            Some(Token {
                kind: TokenKind::Number { text, .. },
                start,
                end,
            }) => {
                parts.push((text.clone(), start, end));
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
                    start: word_start,
                    end: word_end,
                } => {
                    if matches!(
                        self.peek_kind_at(1),
                        Some(TokenKind::Op(equal)) if equal == "="
                    ) {
                        break;
                    }
                    parts.push((word.clone(), word_start, word_end));
                    end = word_end;
                    self.pos += 1;
                }
                Token {
                    kind: TokenKind::Number { text, .. },
                    start: number_start,
                    end: number_end,
                } => {
                    parts.push((text.clone(), number_start, number_end));
                    end = number_end;
                    self.pos += 1;
                }
                _ => break,
            }
        }
        Ok((join_as_written(&parts), start, end))
    }

    pub(crate) fn phrase_on_line(&mut self) -> Result<(String, Position, Position)> {
        self.phrase_on_line_impl(false)
    }

    pub(crate) fn settings_list_name(&mut self) -> Result<(String, Position, Position)> {
        self.phrase_on_line_impl(true)
    }

    fn phrase_on_line_impl(&mut self, list_name: bool) -> Result<(String, Position, Position)> {
        let mut parts: Vec<(String, Position, Position)> = Vec::new();
        let (start, mut end, line) = match self.peek() {
            Some(Token {
                kind: TokenKind::Word(word),
                start,
                end,
            }) => {
                parts.push((word.clone(), start, end));
                (start, end, start.line)
            }
            Some(Token {
                kind: TokenKind::Number { text, .. },
                start,
                end,
            }) => {
                parts.push((text.clone(), start, end));
                (start, end, start.line)
            }
            Some(token) => return Err(self.malformed("expected an identifier", &token)),
            None => return Err(self.malformed("expected an identifier", self.eof())),
        };
        self.pos += 1;
        while let Some(token) = self.peek() {
            let (text, word_start, word_end) = match token {
                Token {
                    kind: TokenKind::Word(word),
                    start,
                    end,
                } => (word.clone(), start, end),
                Token {
                    kind: TokenKind::Number { text, .. },
                    start,
                    end,
                } => (text.clone(), start, end),
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
                Token {
                    kind: TokenKind::Colon,
                    start,
                    end,
                } if list_name => (":".to_string(), start, end),
                _ => break,
            };
            if word_start.line != line {
                break;
            }
            parts.push((text, word_start, word_end));
            end = word_end;
            self.pos += 1;
        }
        Ok((join_as_written(&parts), start, end))
    }

    pub(crate) fn phrase_on_line_with_colon(&mut self) -> Result<(String, Position, Position)> {
        let (mut phrase, start, mut end) = self.phrase_on_line()?;
        if matches!(
            self.peek(),
            Some(Token {
                kind: TokenKind::Colon,
                ..
            })
        ) {
            let colon_pos = self.pos;
            self.next();
            let (rest, _, rest_end) = self.phrase_on_line()?;
            if matches!(
                self.peek(),
                Some(Token {
                    kind: TokenKind::LBrace,
                    ..
                })
            ) {
                phrase.push(':');
                phrase.push(' ');
                phrase.push_str(&rest);
                end = rest_end;
            } else {
                self.pos = colon_pos;
            }
        }
        Ok((phrase, start, end))
    }

    pub(crate) fn enum_member_phrase(&mut self) -> Result<(String, Position, Position)> {
        let first = self
            .peek()
            .ok_or_else(|| self.malformed("expected an enum member", self.eof()))?;
        let start = first.start;
        let line = first.start.line;
        let mut end = first.end;
        let mut parts: Vec<(String, Position, Position)> = Vec::new();
        while let Some(token) = self.peek() {
            if token.start.line != line
                || matches!(
                    token.kind,
                    TokenKind::RParen | TokenKind::Comma | TokenKind::LParen
                )
            {
                break;
            }
            self.pos += 1;
            end = token.end;
            parts.push((raw_token_text(&token.kind), token.start, token.end));
        }
        if parts.is_empty() {
            return Err(self.malformed("expected an enum member", &first));
        }
        Ok((join_as_written(&parts), start, end))
    }

    pub(crate) fn line_text(&mut self) -> Result<(String, Option<Span>)> {
        let mut parts: Vec<(String, Position, Position)> = Vec::new();
        let mut start = None;
        let mut end = None;
        loop {
            match self.peek() {
                Some(Token {
                    kind: TokenKind::Semi,
                    ..
                }) => {
                    self.pos += 1;
                    break;
                }
                Some(token) => {
                    start.get_or_insert(token.start);
                    end = Some(token.end);
                    match &token.kind {
                        TokenKind::Word(_)
                        | TokenKind::Number { .. }
                        | TokenKind::Dot
                        | TokenKind::Colon
                        | TokenKind::Op(_) => {}
                        _ => return Err(self.malformed("expected a text line", &token)),
                    }
                    parts.push((raw_token_text(&token.kind), token.start, token.end));
                    self.pos += 1;
                }
                None => {
                    return Err(self.malformed("unexpected end of input in line", self.eof()));
                }
            }
        }
        Ok((
            join_as_written(&parts),
            start
                .zip(end)
                .map(|(start, end)| Span::new(self.file(), start, end)),
        ))
    }

    pub(crate) fn spelling_run(
        &self,
        pos: usize,
        attaches_to: Option<Position>,
    ) -> Vec<(usize, String, Position)> {
        let mut ends = Vec::new();
        let mut text = String::new();
        let mut index = pos;
        let mut prev_end = attaches_to;
        while let Some(token) = self.tokens.get(index) {
            let fragment = match &token.kind {
                TokenKind::Word(word) => word.as_str(),
                TokenKind::Number { text, .. } => text.as_str(),
                TokenKind::Dot => ".",
                TokenKind::Colon => ":",
                TokenKind::Op(op) if op == "not" => "!",
                TokenKind::Op(op) if op == "-" => "-",
                _ => break,
            };
            // `-` is allowed spaced (the client emits `X - Y` member
            // spellings); `.`, `:`, `!` only continue a spelling when glued
            // to the previous token.
            let needs_adjacent = matches!(token.kind, TokenKind::Dot | TokenKind::Colon)
                || matches!(&token.kind, TokenKind::Op(op) if op == "not");
            if needs_adjacent && prev_end.is_none_or(|end| token.start != end) {
                break;
            }
            if prev_end.is_some_and(|end| token.start != end) {
                text.push(' ');
            }
            text.push_str(fragment);
            prev_end = Some(token.end);
            index += 1;
            ends.push((text.len(), token.end));
        }
        ends.into_iter()
            .enumerate()
            .rev()
            .map(|(i, (len, end))| (i + 1, text[..len].to_string(), end))
            .collect()
    }

    pub(crate) fn catalog_spelling(
        &mut self,
        accept: impl Fn(&str) -> bool,
    ) -> Option<(String, Position, Position)> {
        let start = self.tokens.get(self.pos)?.start;
        for (count, text, end) in self.spelling_run(self.pos, None) {
            if count < 2 {
                break;
            }
            if accept(&text) {
                self.pos += count;
                return Some((text, start, end));
            }
        }
        None
    }

    pub(crate) fn extend_catalog_spelling(
        &mut self,
        phrase: &str,
        phrase_end: Position,
        accept: impl Fn(&str) -> bool,
    ) -> Option<(String, Position)> {
        for (count, tail, end) in self.spelling_run(self.pos, Some(phrase_end)) {
            let joined = format!("{phrase}{tail}");
            if accept(&joined) {
                self.pos += count;
                return Some((joined, end));
            }
        }
        None
    }

    pub(crate) fn extend_entry_spelling(
        &mut self,
        phrase: &str,
        phrase_end: Position,
        kinds: &[Kind],
    ) -> Option<(String, Position)> {
        let catalog = self.catalog;
        let locale = self.locale.clone();
        let primary = catalog.primary_locale();
        self.extend_catalog_spelling(phrase, phrase_end, |spelling| {
            kinds.iter().any(|kind| {
                catalog.resolve(*kind, &locale, spelling).is_some()
                    || (locale != *primary && catalog.resolve(*kind, primary, spelling).is_some())
            })
        })
    }

    pub(crate) fn consume_phrase(&mut self, expected: &str) -> Result<()> {
        let (phrase, _, _) = self.phrase()?;
        if phrase != expected {
            return Err(self.malformed(&format!("expected '{expected}'"), self.previous()));
        }
        Ok(())
    }

    fn compute_structural_spellings(&self, id: &str) -> Vec<String> {
        structural_spellings(self.catalog, &self.locale, id)
    }

    fn match_spelling_tokens(&self, spelling: &str) -> Option<usize> {
        spelling_token_len(&self.tokens, self.pos, spelling)
    }

    pub(crate) fn keyword_prefix(&mut self, ids: &[&str]) -> Option<String> {
        let mut best: Option<(String, usize)> = None;
        for id in ids {
            if !self.structural_spellings.contains_key(*id) {
                let spellings = self.compute_structural_spellings(id);
                self.structural_spellings
                    .insert((*id).to_string(), spellings);
            }
            for spelling in &self.structural_spellings[*id] {
                if let Some(len) = self.match_spelling_tokens(spelling)
                    && best.as_ref().is_none_or(|(_, length)| len > *length)
                {
                    best = Some(((*id).to_string(), len));
                }
            }
        }
        if let Some((id, len)) = best {
            self.pos += len;
            Some(id)
        } else {
            None
        }
    }

    pub(crate) fn consume_keyword(&mut self, id: &str) -> Result<()> {
        if self.keyword_prefix(&[id]).is_some() {
            return Ok(());
        }
        match self.peek() {
            Some(token) => Err(self.malformed(&format!("expected '{id}'"), &token)),
            None => Err(self.malformed(&format!("expected '{id}'"), self.eof())),
        }
    }

    pub(crate) fn expect_keyword(&mut self, expected: &str) -> Result<Position> {
        let start = self.peek().map(|token| token.start);
        if self.keyword_prefix(&[expected]).is_some() {
            return Ok(start.unwrap_or_else(|| self.previous().start));
        }
        match self.next() {
            Some(token) => Err(self.malformed(&format!("expected '{expected}'"), &token)),
            None => Err(self.malformed(&format!("expected '{expected}'"), self.eof())),
        }
    }

    pub(crate) fn expect(&mut self, kind: TokenKind, message: &str) -> Result<()> {
        match self.next() {
            Some(token) if token.kind == kind => Ok(()),
            Some(token) => Err(self.malformed(message, &token)),
            None => Err(self.malformed(message, self.eof())),
        }
    }

    pub(crate) fn expect_semi(&mut self, message: &str) -> Result<()> {
        match self.peek() {
            Some(Token {
                kind: TokenKind::Semi,
                ..
            }) => {
                self.pos += 1;
                Ok(())
            }
            Some(token) if token.start.line > self.previous().end.line => {
                let end = self.previous().end;
                Err(WorkshopError::malformed(
                    message.to_string(),
                    Some(Span::new(self.file(), end, end)),
                ))
            }
            Some(token) => Err(self.malformed(message, &token)),
            None => Err(self.malformed(message, self.eof())),
        }
    }

    pub(crate) fn expect_string(&mut self, message: &str) -> Result<String> {
        match self.next() {
            Some(Token {
                kind: TokenKind::String(content),
                ..
            }) => Ok(content),
            Some(token) => Err(self.malformed(message, &token)),
            None => Err(self.malformed(message, self.eof())),
        }
    }

    pub(crate) fn malformed(&self, message: &str, token: &Token) -> WorkshopError {
        WorkshopError::malformed(
            message.to_string(),
            Some(Span::new(self.file(), token.start, token.end)),
        )
    }

    pub(crate) fn unknown(&self, kind: &'static str, spelling: &str) -> WorkshopError {
        WorkshopError::unknown(kind, spelling.to_string(), self.locale.clone(), None)
    }

    pub(crate) fn unknown_at(
        &self,
        kind: &'static str,
        spelling: &str,
        start: Position,
        end: Position,
    ) -> WorkshopError {
        WorkshopError::unknown(
            kind,
            spelling.to_string(),
            self.locale.clone(),
            Some(Span::new(self.file(), start, end)),
        )
    }

    pub(crate) fn accepted_spellings(
        &self,
        kind: Kind,
        include_primary: bool,
    ) -> impl Iterator<Item = &str> {
        let catalog = self.catalog;
        let primary = catalog.primary_locale();
        let include_primary = include_primary && self.locale != *primary;
        catalog.entries_of(kind).flat_map(move |entry| {
            let local: &[String] = entry.spellings(&self.locale);
            let fallback: &[String] = if include_primary {
                entry.spellings(primary)
            } else {
                &[]
            };
            local.iter().chain(fallback).map(String::as_str)
        })
    }

    pub(crate) fn accepted_member_spellings<'b>(
        &'b self,
        domain: &'b str,
        include_primary: bool,
    ) -> impl Iterator<Item = &'b str> {
        let catalog = self.catalog;
        let primary = catalog.primary_locale();
        let include_primary = include_primary && self.locale != *primary;
        catalog
            .enum_domain(domain)
            .into_iter()
            .flat_map(move |domain| {
                domain.members.iter().flat_map(move |member| {
                    let local: &[String] = member.spellings(&self.locale);
                    let fallback: &[String] = if include_primary {
                        member.spellings(primary)
                    } else {
                        &[]
                    };
                    local.iter().chain(fallback).map(String::as_str)
                })
            })
    }

    pub(crate) fn spelling_candidates(
        &self,
        kind: Kind,
        spelling: &str,
        include_primary: bool,
    ) -> Vec<String> {
        crate::core::suggest::nearest(
            spelling,
            self.accepted_spellings(kind, include_primary),
            crate::core::suggest::CANDIDATE_LIMIT,
        )
    }

    pub(crate) fn member_candidates(
        &self,
        domain: &str,
        spelling: &str,
        include_primary: bool,
    ) -> Vec<String> {
        crate::core::suggest::nearest(
            spelling,
            self.accepted_member_spellings(domain, include_primary),
            crate::core::suggest::CANDIDATE_LIMIT,
        )
    }

    pub(crate) fn peek(&self) -> Option<Token> {
        self.tokens.get(self.pos).cloned()
    }

    pub(crate) fn peek_at(&self, offset: usize) -> Option<Token> {
        self.tokens.get(self.pos + offset).cloned()
    }

    pub(crate) fn peek_kind_at(&self, offset: usize) -> Option<&TokenKind> {
        self.tokens.get(self.pos + offset).map(|token| &token.kind)
    }

    pub(crate) fn next(&mut self) -> Option<Token> {
        let token = self.tokens.get(self.pos).cloned();
        if token.is_some() {
            self.pos += 1;
        }
        token
    }

    pub(crate) fn previous(&self) -> &Token {
        self.tokens
            .get(self.pos.saturating_sub(1))
            .unwrap_or_else(|| self.tokens.last().unwrap())
    }

    pub(crate) fn previous_span(&self) -> (Position, Position) {
        let token = self.previous();
        (token.start, token.end)
    }

    pub(crate) fn span_here(&self) -> (Position, Position) {
        let token = self
            .peek()
            .unwrap_or_else(|| self.tokens.last().unwrap().clone());
        (token.start, token.end)
    }

    pub(crate) fn eof(&self) -> &Token {
        self.tokens.last().unwrap()
    }

    pub(crate) fn file(&self) -> crate::core::ids::Id<SourceFile> {
        crate::core::ids::Id::from_index(0)
    }
}

pub(super) fn structural_keyword_map() -> &'static HashMap<String, String> {
    static KEYWORDS: OnceLock<HashMap<String, String>> = OnceLock::new();
    KEYWORDS.get_or_init(|| {
        serde_json::from_str(include_str!("structural_keywords.json"))
            .expect("structural keyword data is valid JSON")
    })
}

pub(crate) fn canonical_keyword(keyword: &str) -> &str {
    structural_keyword_map()
        .get(keyword)
        .map(String::as_str)
        .unwrap_or(keyword)
}

pub(crate) fn join_as_written(parts: &[(String, Position, Position)]) -> String {
    let mut text = String::new();
    let mut prev_end: Option<Position> = None;
    for (fragment, start, end) in parts {
        if prev_end.is_some_and(|prev| prev != *start) {
            text.push(' ');
        }
        text.push_str(fragment);
        prev_end = Some(*end);
    }
    text
}

pub(crate) fn raw_token_text(kind: &TokenKind) -> String {
    match kind {
        TokenKind::Word(value) => value.clone(),
        TokenKind::Number { text, .. } => text.clone(),
        TokenKind::String(value) => format!("\"{}\"", crate::output::emitter::escape_string(value)),
        TokenKind::Op(value) => match value.as_str() {
            "not" => "!".to_string(),
            "and" => "&&".to_string(),
            "or" => "||".to_string(),
            _ => value.clone(),
        },
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
