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

impl LexError {
    fn invalid_number(text: &str, position: Position) -> Self {
        Self {
            message: format!("invalid number '{text}'"),
            position,
        }
    }

    pub(crate) fn missing_mantissa(text: &str, position: Position) -> Option<Self> {
        let exponent = text.strip_prefix(['e', 'E'])?;
        (!exponent.is_empty() && exponent.bytes().all(|b| b.is_ascii_digit()))
            .then(|| Self::invalid_number(text, position))
    }
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
        let start = cursor.pos();
        let kind = match ch {
            ' ' | '\t' | '\r' | '\n' | '\u{FEFF}' => {
                cursor.advance();
                continue;
            }
            '(' | ')' | ',' | ';' | '{' | '}' | ':' | '.' | '[' | ']' | '?' | '-' => {
                cursor.advance();
                match ch {
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
                }
            }
            '"' => {
                cursor.advance();
                let mut content = String::new();
                let mut closed = false;
                while let Some(c) = cursor.peek() {
                    if c == '\\'
                        && let Some(escaped) = cursor.peek_at(1)
                    {
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
                TokenKind::String(content)
            }
            '0'..='9' => {
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
                        return Err(LexError::invalid_number(&text, start));
                    }
                    u64::from_str_radix(&text[2..], 16)
                        .map(|value| value as f64)
                        .map_err(|_| LexError::invalid_number(&text, start))?
                } else {
                    while let Some(c) = cursor.peek() {
                        if c.is_ascii_digit() || c == '.' {
                            text.push(cursor.advance().unwrap());
                        } else {
                            break;
                        }
                    }
                    if matches!(cursor.peek(), Some('e' | 'E'))
                        && !cursor.peek_at(1).is_some_and(is_word_start)
                    {
                        text.push(cursor.advance().unwrap());
                        if matches!(cursor.peek(), Some('+' | '-')) {
                            text.push(cursor.advance().unwrap());
                        }
                        while matches!(cursor.peek(), Some('0'..='9')) {
                            text.push(cursor.advance().unwrap());
                        }
                    }
                    text.parse()
                        .map_err(|_| LexError::invalid_number(&text, start))?
                };
                TokenKind::Number { value, text }
            }
            '<' if starts_markup_word(&cursor)
                || (markup_tag_len(cursor.rest).is_some()
                    && tokens.last().is_some_and(|token: &Token| {
                        token.end == cursor.pos()
                            && matches!(token.kind, TokenKind::Word(_) | TokenKind::Number { .. })
                    })) =>
            {
                tokens.push(take_word(&mut cursor));
                continue;
            }
            '=' | '!' | '<' | '>' => {
                cursor.advance();
                let spelling = if cursor.peek() == Some('=') {
                    cursor.advance();
                    format!("{ch}=")
                } else if ch == '!' {
                    "not".to_string()
                } else {
                    ch.to_string()
                };
                TokenKind::Op(spelling)
            }
            '&' | '|' => {
                cursor.advance();
                if cursor.peek() != Some(ch) {
                    return Err(LexError {
                        message: format!("unexpected '{ch}'; expected '{ch}{ch}'"),
                        position: start,
                    });
                }
                cursor.advance();
                TokenKind::Op(if ch == '&' { "and" } else { "or" }.to_string())
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
                cursor.advance();
                TokenKind::Op(ch.to_string())
            }
            c if is_word_start(c) => {
                tokens.push(take_word(&mut cursor));
                continue;
            }
            other => {
                return Err(LexError {
                    message: format!("unexpected character '{other}'"),
                    position: start,
                });
            }
        };
        tokens.push(Token {
            kind,
            start,
            end: cursor.pos(),
        });
    }

    let end = cursor.pos();
    tokens.push(Token {
        kind: TokenKind::Eof,
        start: end,
        end,
    });
    Ok(tokens)
}

fn take_word(cursor: &mut Cursor) -> Token {
    let start = cursor.pos();
    let mut word = String::new();
    while let Some(c) = cursor.peek() {
        if c == '<'
            && let Some(len) = markup_tag_len(cursor.rest)
        {
            for _ in 0..len {
                word.push(cursor.advance().unwrap());
            }
            continue;
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

fn starts_markup_word(cursor: &Cursor) -> bool {
    match markup_tag_len(cursor.rest) {
        Some(len) => cursor.rest[len..].chars().next().is_some_and(is_word_start),
        None => false,
    }
}

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
