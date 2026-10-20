//! Token layout of a parsed rule, re-derived from its retained source, and
//! the text geometry helpers statement edits use to place their bytes.

use crate::catalog::{Catalog, Locale};
use crate::core::source::{Position, Span, byte_offset};
use crate::frontend::keywords::{canonical_structural_id, structural_keyword_prefix};
use crate::frontend::lexer::{Token, TokenKind};

const RULE_SECTION_IDS: &[&str] = &["event", "conditions", "actions"];

// --- Source layout scan -----------------------------------------------

/// One `;`-terminated statement line inside a section: the statement plus
/// the standalone `"…"` annotation tokens directly attached to it.
#[derive(Debug, Clone, Copy)]
pub(super) struct StatementChunk {
    /// First token of the chunk (a `"…"` annotation when present).
    pub(super) first: usize,
    /// First token that is not a `"…"` annotation — the statement itself.
    pub(super) first_content: usize,
    /// The terminating `;` token.
    pub(super) semi: usize,
}

/// A `name { … }` block inside a rule body.
#[derive(Debug)]
pub(super) struct StatementSection {
    /// `event`, `conditions`, or `actions`.
    pub(super) id: &'static str,
    /// Start of the section keyword's first token (the create-section anchor).
    pub(super) name_start: Position,
    /// The section's closing `}` token index.
    pub(super) close: usize,
    /// `;`-terminated statement chunks (`conditions`/`actions` only use them).
    pub(super) chunks: Vec<StatementChunk>,
    /// First token of `"…"` annotations after the last statement, which
    /// annotate nothing.
    pub(super) dangling: Option<usize>,
}

/// The token layout of a parsed rule: its body braces and `name { … }`
/// sections, re-derived from the retained source in the session locale.
pub(super) struct RuleLayout {
    /// Token index of the rule body's closing `}`.
    pub(super) close: usize,
    pub(super) sections: Vec<StatementSection>,
}

impl RuleLayout {
    /// Re-read the rule's token layout from the retained source. `None`
    /// means the retained text does not carry the shape the model claims —
    /// a `SourceUnavailable` for the caller.
    pub(super) fn scan(
        tokens: &[Token],
        rule_span: Span,
        catalog: &Catalog,
        locale: &Locale,
    ) -> Option<Self> {
        let window: Vec<usize> = tokens
            .iter()
            .enumerate()
            .filter(|(_, token)| {
                (token.start.line, token.start.col) < (token.end.line, token.end.col)
                    && (token.start.line, token.start.col)
                        >= (rule_span.start.line, rule_span.start.col)
                    && (token.end.line, token.end.col) <= (rule_span.end.line, rule_span.end.col)
            })
            .map(|(index, _)| index)
            .collect();
        let open = *window
            .iter()
            .find(|&&index| matches!(tokens[index].kind, TokenKind::LBrace))?;
        let close = *window.last()?;
        if !matches!(tokens[close].kind, TokenKind::RBrace) {
            return None;
        }
        let mut sections = Vec::new();
        let mut i = open + 1;
        while i < close {
            match &tokens[i].kind {
                TokenKind::Word(word) => {
                    let id = canonical_structural_id(catalog, locale, word);
                    if !RULE_SECTION_IDS.contains(&id.as_str()) {
                        return None;
                    }
                    let id = RULE_SECTION_IDS
                        .iter()
                        .find(|known| **known == id)
                        .copied()?;
                    let (_, len) = structural_keyword_prefix(tokens, i, catalog, locale, &[id])?;
                    let brace = i + len;
                    if !matches!(
                        tokens.get(brace).map(|token| &token.kind),
                        Some(TokenKind::LBrace)
                    ) {
                        return None;
                    }
                    let (chunks, dangling, end) = scan_chunks(tokens, brace + 1)?;
                    sections.push(StatementSection {
                        id,
                        name_start: tokens[i].start,
                        close: end,
                        chunks,
                        dangling,
                    });
                    i = end + 1;
                }
                _ => return None,
            }
        }
        (i == close).then_some(Self { close, sections })
    }
}

/// Read a section body into `;`-terminated chunks, returning the chunks and
/// the index of its closing `}`.
pub(super) fn scan_chunks(
    tokens: &[Token],
    mut i: usize,
) -> Option<(Vec<StatementChunk>, Option<usize>, usize)> {
    let mut chunks = Vec::new();
    let mut dangling = None;
    loop {
        match tokens.get(i).map(|token| &token.kind) {
            Some(TokenKind::RBrace) => return Some((chunks, dangling, i)),
            Some(TokenKind::Semi) => i += 1,
            Some(_) => {
                let first = i;
                while matches!(
                    tokens.get(i).map(|token| &token.kind),
                    Some(TokenKind::String(_))
                ) {
                    i += 1;
                }
                let first_content = i;
                match tokens.get(i).map(|token| &token.kind) {
                    // A run of annotations (and stray semicolons) with no
                    // statement after them is a comment-only chunk.
                    Some(TokenKind::RBrace) => dangling = Some(first),
                    Some(TokenKind::Semi) => i += 1,
                    Some(_) => {
                        let mut semi = i;
                        while !matches!(
                            tokens.get(semi).map(|token| &token.kind),
                            Some(TokenKind::Semi) | None
                        ) {
                            semi += 1;
                        }
                        if semi == tokens.len() {
                            return None;
                        }
                        chunks.push(StatementChunk {
                            first,
                            first_content,
                            semi,
                        });
                        i = semi + 1;
                    }
                    None => return None,
                }
            }
            None => return None,
        }
    }
}

// --- Text geometry helpers ---------------------------------------------

/// The line ending of the line holding `position`, so an edit never mixes
/// endings inside a file that has both.
pub(super) fn newline_of(text: &str, position: Position) -> &'static str {
    let next = byte_offset(text, Position::new(position.line + 1, 1));
    match next {
        Some(offset) if text[..offset].ends_with("\r\n") => "\r\n",
        _ => "\n",
    }
}

/// The source text before `position` on its own line — the indentation when
/// it is all whitespace.
pub(super) fn line_prefix(text: &str, position: Position) -> &str {
    let line = Position::new(position.line, 1);
    match (byte_offset(text, line), byte_offset(text, position)) {
        (Some(start), Some(end)) if start <= end => &text[start..end],
        _ => "",
    }
}

pub(super) fn is_whitespace(text: &str) -> bool {
    text.chars()
        .all(|character| matches!(character, ' ' | '\t' | '\r'))
}

/// Whether the byte immediately before `position` is whitespace — when it
/// is not, an inline insert opens with a separating space.
pub(super) fn byte_is_whitespace(text: &str, position: Position) -> bool {
    byte_offset(text, position)
        .and_then(|offset| text[..offset].chars().next_back())
        .is_none_or(|character| character.is_whitespace())
}
