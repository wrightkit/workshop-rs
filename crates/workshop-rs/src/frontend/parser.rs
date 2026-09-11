//! Native localized Workshop parser.
//!
//! Parses vanilla Workshop text directly into validated, locale-independent
//! the canonical public [`Program`]. Localized actions, values, events, enums, and structural
//! keywords resolve through the canonical catalog; malformed input,
//! unknown spellings, and recognized-but-unsupported constructs are reported
//! as distinct structured diagnostics with source spans.

pub(crate) use std::collections::HashMap;
use std::sync::OnceLock;

pub(crate) use crate::core::signatures::ExpectedDomain;
pub(crate) use crate::core::source::{Position, SourceFile, Span};
use crate::program::Program;
pub(crate) use crate::settings::table::{self, KeyKind, PathPart};
pub(crate) use crate::settings::{Settings, SettingsListElement, SettingsNode};
pub(crate) use crate::wir::{
    self, Action, Event, EventTarget, EventTeam, ModifyOp, PlayerEventKind, Value, ValueNode,
};

pub(crate) use super::lexer::{Token, TokenKind, tokenize};
pub(crate) use crate::catalog::{Catalog, Kind, Locale, ParamCoercions};
pub(crate) use crate::core::error::{Result, WorkshopError};

/// Where action parsing stopped.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Stop {
    /// The enclosing `}` was consumed.
    SectionClosed,
    /// The `End` keyword is next (not consumed).
    End,
    /// The `Else If` keyword is next (not consumed).
    ElseIf,
    /// The `Else` keyword is next (not consumed).
    Else,
}

pub(crate) enum AssignmentOperator {
    Set,
    Modify(ModifyOp),
}

/// Parse localized Workshop text into Workshop IR using the catalog's
/// canonical call-signature context. Ambiguous bare enum members resolve when
/// their enclosing call pins one matching domain; unpinned ambiguity remains a
/// structured unsupported diagnostic. See [`parse_with_context`] when a
/// consumer needs to provide additional signature context.
pub fn parse(input: &str, catalog: &Catalog, locale: &Locale) -> Result<Program> {
    parse_with_context(input, catalog, locale, catalog)
}

/// Compatibility entry point for crate-internal storage tests and migration
/// checks. Ordinary consumers should use [`parse`], which returns [`Program`].
#[doc(hidden)]
pub fn parse_wir(input: &str, catalog: &Catalog, locale: &Locale) -> Result<wir::Program> {
    parse_wir_with_context(input, catalog, locale, catalog)
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
) -> Result<Program> {
    parse_wir_with_context(input, catalog, locale, context).and_then(Program::from_wir)
}

#[doc(hidden)]
pub fn parse_wir_with_context(
    input: &str,
    catalog: &Catalog,
    locale: &Locale,
    context: &dyn ExpectedDomain,
) -> Result<wir::Program> {
    let tokens = tokenize(input).map_err(|error| WorkshopError::Malformed {
        message: error.message,
        span: Some(synthetic_span(error.position)),
    })?;
    ParseContext {
        tokens,
        pos: 0,
        source: input,
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
pub(crate) fn synthetic_span(position: Position) -> Span {
    Span::new(crate::core::ids::Id::from_index(0), position, position)
}

pub(crate) struct ParseContext<'a> {
    pub(crate) tokens: Vec<Token>,
    pub(crate) pos: usize,
    pub(crate) source: &'a str,
    pub(crate) catalog: &'a Catalog,
    pub(crate) locale: Locale,
    /// Canonical signature context (#111): supplies the expected enum domain
    /// for the call argument currently being parsed.
    pub(crate) context: &'a dyn ExpectedDomain,
    /// The expected enum domain for the value currently being parsed, set by
    /// [`ParseContext::value_args`] from the enclosing call's signature.
    pub(crate) expected_domain: Option<&'a str>,
    pub(crate) call_stack: Vec<String>,
    pub(crate) target: wir::Program,
    pub(crate) globals: HashMap<String, wir::GlobalVarId>,
    pub(crate) players: HashMap<String, wir::PlayerVarId>,
    pub(crate) subroutines: HashMap<String, wir::SubroutineId>,
}

impl ParseContext<'_> {
    pub(crate) fn resolve_entry(
        &self,
        kind: Kind,
        spelling: &str,
    ) -> Option<crate::catalog::CatalogEntry> {
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

    pub(crate) fn canonical_keyword(&self, spelling: &str) -> String {
        self.catalog
            .resolve(Kind::Structural, &self.locale, spelling)
            .or_else(|| {
                (self.locale != *self.catalog.primary_locale()).then(|| {
                    self.catalog
                        .resolve(Kind::Structural, self.catalog.primary_locale(), spelling)
                })?
            })
            .map(|entry| entry.id.clone())
            .unwrap_or_else(|| canonical_keyword(spelling).to_string())
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

    /// Read the maximal phrase of consecutive words (space-joined). Phrases
    /// may span lines because long Workshop action arguments wrap mid-phrase.
    pub(crate) fn phrase(&mut self) -> Result<(String, Position, Position)> {
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
                    if matches!(
                        self.peek_at(1).map(|token| token.kind),
                        Some(TokenKind::Op(equal)) if equal == "="
                    ) {
                        break;
                    }
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
    pub(crate) fn phrase_on_line(&mut self) -> Result<(String, Position, Position)> {
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
                .replace(" : ", ":")
                .replace(" %", "%"),
            start,
            end,
        ))
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
    pub(crate) fn line_text(&mut self) -> Result<String> {
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
    pub(crate) fn consume_phrase(&mut self, expected: &str) -> Result<()> {
        let (phrase, _, _) = self.phrase()?;
        if phrase != expected {
            return Err(self.malformed(&format!("expected '{expected}'"), self.previous()));
        }
        Ok(())
    }

    pub(crate) fn expect_keyword(&mut self, expected: &str) -> Result<Position> {
        match self.next() {
            Some(Token {
                kind: TokenKind::Word(word),
                start,
                ..
            }) if self.canonical_keyword(&word) == expected => Ok(start),
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
        WorkshopError::Malformed {
            message: message.to_string(),
            span: Some(Span::new(self.file(), token.start, token.end)),
        }
    }

    pub(crate) fn unknown(&self, kind: &'static str, spelling: &str) -> WorkshopError {
        WorkshopError::Unknown {
            kind,
            spelling: spelling.to_string(),
            locale: self.locale.clone(),
            span: None,
        }
    }

    pub(crate) fn peek(&self) -> Option<Token> {
        self.tokens.get(self.pos).cloned()
    }

    pub(crate) fn peek_at(&self, offset: usize) -> Option<Token> {
        self.tokens.get(self.pos + offset).cloned()
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

pub(crate) fn is_comparison(op: &str) -> bool {
    matches!(op, "==" | "!=" | "<" | "<=" | ">" | ">=")
}

pub(crate) fn canonical_keyword(keyword: &str) -> &str {
    static KEYWORDS: OnceLock<HashMap<String, String>> = OnceLock::new();
    KEYWORDS
        .get_or_init(|| {
            serde_json::from_str(include_str!("structural_keywords.json"))
                .expect("structural keyword data is valid JSON")
        })
        .get(keyword)
        .map(String::as_str)
        .unwrap_or(keyword)
}

pub(crate) fn raw_token_text(kind: &TokenKind) -> String {
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
