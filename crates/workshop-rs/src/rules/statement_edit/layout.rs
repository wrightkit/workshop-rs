use crate::catalog::{Catalog, Locale};
use crate::core::source::{Position, Span, byte_offset};
use crate::frontend::keywords::{canonical_structural_id, structural_keyword_prefix};
use crate::frontend::lexer::{Token, TokenKind};

const RULE_SECTION_IDS: &[&str] = &["event", "conditions", "actions"];

#[derive(Debug, Clone, Copy)]
pub(super) struct StatementChunk {
    pub(super) first: usize,
    pub(super) first_content: usize,
    pub(super) semi: usize,
}

#[derive(Debug)]
pub(super) struct StatementSection {
    pub(super) id: &'static str,
    pub(super) name_start: Position,
    pub(super) close: usize,
    pub(super) chunks: Vec<StatementChunk>,
    pub(super) dangling: Option<usize>,
}

pub(super) struct RuleLayout {
    pub(super) close: usize,
    pub(super) sections: Vec<StatementSection>,
}

impl RuleLayout {
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

pub(super) fn newline_of(text: &str, position: Position) -> &'static str {
    let next = byte_offset(text, Position::new(position.line + 1, 1));
    match next {
        Some(offset) if text[..offset].ends_with("\r\n") => "\r\n",
        _ => "\n",
    }
}

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

pub(super) fn byte_is_whitespace(text: &str, position: Position) -> bool {
    byte_offset(text, position)
        .and_then(|offset| text[..offset].chars().next_back())
        .is_none_or(|character| character.is_whitespace())
}

pub(super) fn indent_unit(
    text: &str,
    tokens: &[Token],
    layout: &RuleLayout,
    rule_span: Span,
) -> String {
    let unit_of = |outer: &str, inner: &str| {
        (is_whitespace(outer)
            && is_whitespace(inner)
            && inner.len() > outer.len()
            && inner.starts_with(outer))
        .then(|| inner[outer.len()..].to_string())
    };
    layout
        .sections
        .iter()
        .find_map(|section| {
            let chunk = section.chunks.first()?;
            unit_of(
                line_prefix(text, section.name_start),
                line_prefix(text, tokens[chunk.first].start),
            )
        })
        .or_else(|| {
            let section = layout.sections.first()?;
            unit_of(
                line_prefix(text, rule_span.start),
                line_prefix(text, section.name_start),
            )
        })
        .unwrap_or_else(|| "    ".to_string())
}
