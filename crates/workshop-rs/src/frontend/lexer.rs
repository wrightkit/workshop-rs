use crate::core::source::Position;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum TokenKind {
    Word(String),
    Number { value: f64, text: String },
    String(String),
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

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Token {
    pub(crate) kind: TokenKind,
    pub(crate) start: Position,
    pub(crate) end: Position,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct LexError {
    pub(crate) message: String,
    pub(crate) position: Position,
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

pub(crate) fn tokenize(input: &str) -> Result<Vec<Token>, LexError> {
    let mut cursor = Cursor::new(input);
    let mut tokens = Vec::new();

    while let Some(ch) = cursor.peek() {
        match ch {
            ' ' | '\t' | '\r' | '\n' | '\u{FEFF}' => {
                cursor.advance();
            }
            '(' | ')' | ',' | ';' | '{' | '}' | ':' | '.' | '[' | ']' | '?' | '-' => {
                let start = cursor.pos();
                cursor.advance();
                let kind = match ch {
                    '(' => TokenKind::LParen,
                    ')' => TokenKind::RParen,
                    ',' => TokenKind::Comma,
                    ';' => TokenKind::Semi,
                    '{' => TokenKind::LBrace,
                    '}' => TokenKind::RBrace,
                    ':' => TokenKind::Colon,
                    '.' => TokenKind::Dot,
                    '[' => TokenKind::LBracket,
                    ']' => TokenKind::RBracket,
                    other => TokenKind::Op(other.to_string()),
                };
                tokens.push(Token {
                    kind,
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
            '<' if starts_markup_word(&cursor)
                || (markup_tag_len(cursor.rest).is_some()
                    && tokens.last().is_some_and(|token: &Token| {
                        token.end == cursor.pos()
                            && matches!(token.kind, TokenKind::Word(_) | TokenKind::Number { .. })
                    })) =>
            {
                tokens.push(take_word(&mut cursor));
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
            c if is_word_start(c) => {
                tokens.push(take_word(&mut cursor));
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

/// Consume one word token: identifier characters, interior dashes, and
/// localization markup sequences (`<en>`, `<br>`, `<hy>`), which the
/// reviewed catalog carries verbatim inside member spellings such as
/// `Meis Schneeball<hy>schlacht` and `<en>Orisa<en>` (#448).
fn take_word(cursor: &mut Cursor) -> Token {
    let start = cursor.pos();
    let mut word = String::new();
    while let Some(c) = cursor.peek() {
        if c == '<' {
            if let Some(len) = markup_tag_len(cursor.rest) {
                for _ in 0..len {
                    word.push(cursor.advance().unwrap());
                }
                continue;
            }
        }
        let interior_dash = c == '-'
            && cursor
                .peek_at(1)
                .is_some_and(|next| unicode_ident::is_xid_continue(next) || next == '_');
        if is_word_character(c) || interior_dash {
            word.push(cursor.advance().unwrap());
        } else {
            break;
        }
    }
    Token {
        kind: TokenKind::Word(word),
        start,
        end: cursor.pos(),
    }
}

/// Whether the `<` at the cursor opens a markup-led word such as
/// `<en>Orisa<en>` rather than a comparison operator: a `<tag>` sequence
/// directly followed by a word character.
fn starts_markup_word(cursor: &Cursor) -> bool {
    match markup_tag_len(cursor.rest) {
        Some(len) => cursor.rest[len..].chars().next().is_some_and(is_word_start),
        None => false,
    }
}

/// The byte length of a `<tag>` markup sequence at the start of `rest`
/// (`<en>`, `<br>`, `<hy>`), or `None` when `rest` does not open one.
fn markup_tag_len(rest: &str) -> Option<usize> {
    let inner = rest.strip_prefix('<')?;
    let close = inner.find('>')?;
    let name = &inner[..close];
    if !name.is_empty() && name.chars().all(|c| c.is_ascii_lowercase()) {
        Some(1 + close + 1)
    } else {
        None
    }
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
    // Punctuation found inside reviewed catalog spellings that is not
    // Workshop syntax (#448): typographic quotes/brackets, the en dash, and
    // the fullwidth/ideographic marks used by member and value names such as
    // es-ES `¡Alto!`, fr-FR `Portrait « en feu »`, ja-JP
    // `条件が「TRUE」の場合中止`, or ja-JP `対面・垂直角`.
    matches!(
        ch,
        '¡' | '«'
            | '»'
            | '–'
            | '“'
            | '”'
            | '、'
            | '。'
            | '〈'
            | '〉'
            | '「'
            | '」'
            | '・'
            | '！'
            | '（'
            | '）'
            | '，'
            | '：'
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(source: &str) -> Vec<TokenKind> {
        tokenize(source)
            .expect("tokenizes")
            .into_iter()
            .map(|token| token.kind)
            .collect()
    }

    /// Catalog spellings carry locale punctuation the lexer must accept as
    /// word content (#448): `¡`, `«»`, `「」`, `・`, `。`, `–`, `〈〉`.
    #[test]
    fn localized_punctuation_is_word_content() {
        for (spelling, expected) in [
            ("¡Alto", "¡Alto"),
            ("«Горизонт»", "«Горизонт»"),
            ("条件が「TRUE」の場合ループ", "条件が「TRUE」の場合ループ"),
            ("イベント・プレイヤー", "イベント・プレイヤー"),
            ("最も近いプレイヤー。基準", "最も近いプレイヤー。基準"),
            ("Orisa–Stać", "Orisa–Stać"),
            (
                "ソルジャー76の〈スプリント〉のエフェクト",
                "ソルジャー76の〈スプリント〉のエフェクト",
            ),
        ] {
            assert_eq!(
                kinds(spelling),
                vec![TokenKind::Word(expected.to_string()), TokenKind::Eof],
                "{spelling:?} must lex as one word"
            );
        }
    }

    /// Localization markup sequences (`<en>`, `<br>`, `<hy>`) stay inside
    /// the word they interrupt or open (#448).
    #[test]
    fn markup_tags_join_their_word() {
        assert_eq!(
            kinds("Meis Schneeball<hy>schlacht"),
            vec![
                TokenKind::Word("Meis".to_string()),
                TokenKind::Word("Schneeball<hy>schlacht".to_string()),
                TokenKind::Eof,
            ]
        );
        assert_eq!(
            kinds("ジャンケン<br>シュタインの<br>復讐"),
            vec![
                TokenKind::Word("ジャンケン<br>シュタインの<br>復讐".to_string()),
                TokenKind::Eof,
            ]
        );
        assert_eq!(
            kinds("<en>Orisa<en>"),
            vec![TokenKind::Word("<en>Orisa<en>".to_string()), TokenKind::Eof]
        );
    }

    /// A `<` that does not open a `<tag>` sequence keeps its operator
    /// reading, so `a < b` and `a<b` still lex as comparisons.
    #[test]
    fn less_than_still_lexes_as_operator() {
        for source in ["a < b", "a<b", "a<=b", "a <b "] {
            assert!(
                kinds(source)
                    .iter()
                    .any(|kind| matches!(kind, TokenKind::Op(op) if op.starts_with('<'))),
                "{source:?} lost its `<` operator"
            );
        }
    }
}
