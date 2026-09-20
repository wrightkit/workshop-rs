//! Tokenizer for localized vanilla Workshop text.
//!
//! Tokens carry 1-based line/column spans so parser diagnostics and the
//! produced WIR preserve source locations. Workshop identifiers are
//! multi-word phrases; the tokenizer emits single [`Word`] tokens and the
//! parser groups them, which keeps locale spellings data-driven.

use crate::core::source::Position;

/// A lexical token kind.
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    /// A run of identifier characters (`[A-Za-z_][A-Za-z0-9_]*`).
    Word(String),
    /// A numeric literal with its source spelling.
    Number {
        value: f64,
        text: String,
    },
    /// A string literal (content, unescaped).
    String(String),
    /// An operator token: `= == != < <= > >= && || ! + - * / %`.
    Op(String),
    LParen,
    RParen,
    Comma,
    Semi,
    LBrace,
    RBrace,
    Colon,
    Dot,
    LBracket,
    RBracket,
    Eof,
}

/// A token with its source span.
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub start: Position,
    pub end: Position,
}

/// A lexing error with a source position.
#[derive(Debug, Clone, PartialEq)]
pub struct LexError {
    pub message: String,
    pub position: Position,
}

struct Cursor<'a> {
    rest: &'a str,
    line: u32,
    col: u32,
}

impl<'a> Cursor<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            rest: input,
            line: 1,
            col: 1,
        }
    }

    fn pos(&self) -> Position {
        Position::new(self.line, self.col)
    }

    fn peek(&self) -> Option<char> {
        self.rest.chars().next()
    }

    fn peek_at(&self, n: usize) -> Option<char> {
        self.rest.chars().nth(n)
    }

    fn advance(&mut self) -> Option<char> {
        let mut chars = self.rest.chars();
        let ch = chars.next()?;
        self.rest = chars.as_str();
        if ch == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        Some(ch)
    }
}

/// Tokenize Workshop text.
pub fn tokenize(input: &str) -> Result<Vec<Token>, LexError> {
    let mut cursor = Cursor::new(input);
    let mut tokens = Vec::new();

    while let Some(ch) = cursor.peek() {
        match ch {
            ' ' | '\t' | '\r' | '\n' => {
                cursor.advance();
            }
            '(' => {
                let start = cursor.pos();
                cursor.advance();
                tokens.push(Token {
                    kind: TokenKind::LParen,
                    start,
                    end: cursor.pos(),
                });
            }
            ')' => {
                let start = cursor.pos();
                cursor.advance();
                tokens.push(Token {
                    kind: TokenKind::RParen,
                    start,
                    end: cursor.pos(),
                });
            }
            ',' => {
                let start = cursor.pos();
                cursor.advance();
                tokens.push(Token {
                    kind: TokenKind::Comma,
                    start,
                    end: cursor.pos(),
                });
            }
            ';' => {
                let start = cursor.pos();
                cursor.advance();
                tokens.push(Token {
                    kind: TokenKind::Semi,
                    start,
                    end: cursor.pos(),
                });
            }
            '{' => {
                let start = cursor.pos();
                cursor.advance();
                tokens.push(Token {
                    kind: TokenKind::LBrace,
                    start,
                    end: cursor.pos(),
                });
            }
            '}' => {
                let start = cursor.pos();
                cursor.advance();
                tokens.push(Token {
                    kind: TokenKind::RBrace,
                    start,
                    end: cursor.pos(),
                });
            }
            ':' => {
                let start = cursor.pos();
                cursor.advance();
                tokens.push(Token {
                    kind: TokenKind::Colon,
                    start,
                    end: cursor.pos(),
                });
            }
            '.' => {
                let start = cursor.pos();
                cursor.advance();
                tokens.push(Token {
                    kind: TokenKind::Dot,
                    start,
                    end: cursor.pos(),
                });
            }
            '[' => {
                let start = cursor.pos();
                cursor.advance();
                tokens.push(Token {
                    kind: TokenKind::LBracket,
                    start,
                    end: cursor.pos(),
                });
            }
            ']' => {
                let start = cursor.pos();
                cursor.advance();
                tokens.push(Token {
                    kind: TokenKind::RBracket,
                    start,
                    end: cursor.pos(),
                });
            }
            '"' => {
                let start = cursor.pos();
                cursor.advance();
                let mut content = String::new();
                let mut closed = false;
                while let Some(c) = cursor.peek() {
                    // Decode the escape spellings the emitter produces so a
                    // settings/value string round-trips byte-identically
                    // (`\"`, `\\`, `\n`, `\r`, `\t`; #87).
                    if c == '\\' {
                        if let Some(escaped) = cursor.peek_at(1) {
                            let decoded = match escaped {
                                '"' => Some('"'),
                                '\\' => Some('\\'),
                                'n' => Some('\n'),
                                'r' => Some('\r'),
                                't' => Some('\t'),
                                _ => None,
                            };
                            if let Some(decoded) = decoded {
                                cursor.advance();
                                cursor.advance();
                                content.push(decoded);
                                continue;
                            }
                        }
                    }
                    if c == '"' {
                        cursor.advance();
                        closed = true;
                        break;
                    }
                    cursor.advance();
                    content.push(c);
                }
                if !closed {
                    return Err(LexError {
                        message: "unterminated string literal".to_string(),
                        position: start,
                    });
                }
                tokens.push(Token {
                    kind: TokenKind::String(content),
                    start,
                    end: cursor.pos(),
                });
            }
            '0'..='9' => {
                let start = cursor.pos();
                let mut text = String::new();
                let value = if ch == '0' && cursor.peek_at(1) == Some('X') {
                    text.push(cursor.advance().unwrap());
                    text.push(cursor.advance().unwrap());
                    let mut digit_count = 0;
                    while let Some(c) = cursor.peek() {
                        if c.is_ascii_hexdigit() {
                            text.push(cursor.advance().unwrap());
                            digit_count += 1;
                        } else {
                            break;
                        }
                    }
                    if digit_count == 0 {
                        return Err(LexError {
                            message: format!("invalid number '{text}'"),
                            position: start,
                        });
                    }
                    u64::from_str_radix(&text[2..], 16)
                        .map(|value| value as f64)
                        .map_err(|_| LexError {
                            message: format!("invalid number '{text}'"),
                            position: start,
                        })?
                } else {
                    while let Some(c) = cursor.peek() {
                        if c.is_ascii_digit() || c == '.' {
                            text.push(cursor.advance().unwrap());
                        } else {
                            break;
                        }
                    }
                    text.parse().map_err(|_| LexError {
                        message: format!("invalid number '{text}'"),
                        position: start,
                    })?
                };
                tokens.push(Token {
                    kind: TokenKind::Number { value, text },
                    start,
                    end: cursor.pos(),
                });
            }
            '=' => {
                let start = cursor.pos();
                cursor.advance();
                if cursor.peek() == Some('=') {
                    cursor.advance();
                    tokens.push(Token {
                        kind: TokenKind::Op("==".to_string()),
                        start,
                        end: cursor.pos(),
                    });
                } else {
                    tokens.push(Token {
                        kind: TokenKind::Op("=".to_string()),
                        start,
                        end: cursor.pos(),
                    });
                }
            }
            '!' => {
                let start = cursor.pos();
                cursor.advance();
                if cursor.peek() == Some('=') {
                    cursor.advance();
                    tokens.push(Token {
                        kind: TokenKind::Op("!=".to_string()),
                        start,
                        end: cursor.pos(),
                    });
                } else {
                    tokens.push(Token {
                        kind: TokenKind::Op("not".to_string()),
                        start,
                        end: cursor.pos(),
                    });
                }
            }
            '?' => {
                let start = cursor.pos();
                cursor.advance();
                tokens.push(Token {
                    kind: TokenKind::Op("?".to_string()),
                    start,
                    end: cursor.pos(),
                });
            }
            '&' | '|' => {
                let start = cursor.pos();
                let operator = cursor.advance().unwrap();
                if cursor.peek() == Some(operator) {
                    cursor.advance();
                    tokens.push(Token {
                        kind: TokenKind::Op(if operator == '&' {
                            "and".to_string()
                        } else {
                            "or".to_string()
                        }),
                        start,
                        end: cursor.pos(),
                    });
                } else {
                    return Err(LexError {
                        message: format!(
                            "unexpected '{operator}'; expected '{operator}{operator}'"
                        ),
                        position: start,
                    });
                }
            }
            '<' | '>' => {
                let start = cursor.pos();
                let op = cursor.advance().unwrap();
                if cursor.peek() == Some('=') {
                    cursor.advance();
                    tokens.push(Token {
                        kind: TokenKind::Op(format!("{op}=")),
                        start,
                        end: cursor.pos(),
                    });
                } else {
                    tokens.push(Token {
                        kind: TokenKind::Op(op.to_string()),
                        start,
                        end: cursor.pos(),
                    });
                }
            }
            '+' | '*' | '/' | '%' => {
                if ch == '/' && cursor.peek_at(1) == Some('/') {
                    // `//` line comments: the reference's Workshop export
                    // format carries element-count comments between rules
                    // (#119 differential evidence); skip to end of line.
                    while let Some(c) = cursor.peek() {
                        if c == '\n' {
                            break;
                        }
                        cursor.advance();
                    }
                    continue;
                }
                let start = cursor.pos();
                let op = cursor.advance().unwrap();
                tokens.push(Token {
                    kind: TokenKind::Op(op.to_string()),
                    start,
                    end: cursor.pos(),
                });
            }
            '-' => {
                let start = cursor.pos();
                cursor.advance();
                tokens.push(Token {
                    kind: TokenKind::Op("-".to_string()),
                    start,
                    end: cursor.pos(),
                });
            }
            c if is_word_start(c) => {
                let start = cursor.pos();
                let mut word = String::new();
                while let Some(c) = cursor.peek() {
                    let interior_dash = c == '-'
                        && cursor.peek_at(1).is_some_and(|next| {
                            unicode_ident::is_xid_continue(next) || next == '_'
                        });
                    if is_word_character(c) || interior_dash {
                        word.push(cursor.advance().unwrap());
                    } else {
                        break;
                    }
                }
                tokens.push(Token {
                    kind: TokenKind::Word(word),
                    start,
                    end: cursor.pos(),
                });
            }
            other => {
                return Err(LexError {
                    message: format!("unexpected character '{other}'"),
                    position: cursor.pos(),
                });
            }
        }
    }

    let end = cursor.pos();
    tokens.push(Token {
        kind: TokenKind::Eof,
        start: end,
        end,
    });
    Ok(tokens)
}

fn is_word_start(ch: char) -> bool {
    unicode_ident::is_xid_start(ch) || ch == '_' || is_raw_label_punctuation(ch)
}

fn is_word_character(ch: char) -> bool {
    unicode_ident::is_xid_continue(ch)
        || ch == '_'
        || matches!(ch, '\'' | '’')
        || is_raw_label_punctuation(ch)
}

fn is_raw_label_punctuation(ch: char) -> bool {
    matches!(
        ch,
        '“' | '”' | '（' | '）' | '，' | '、' | '！' | '？' | '：' | '【' | '】' | '［' | '］'
    )
}
