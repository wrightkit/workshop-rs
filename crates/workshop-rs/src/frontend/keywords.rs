use super::lexer::{Token, TokenKind};
use super::parser::{canonical_keyword, structural_keyword_map};
use crate::catalog::{Catalog, Kind, Locale};
use crate::settings::table;

pub(crate) fn structural_spellings(catalog: &Catalog, locale: &Locale, id: &str) -> Vec<String> {
    let mut spellings = Vec::new();
    if let Some(entry) = catalog.entry(Kind::Structural, id) {
        spellings.extend(entry.spellings(locale).iter().cloned());
        let primary = catalog.primary_locale();
        if locale != primary {
            spellings.extend(entry.spellings(primary).iter().cloned());
        }
    }
    for (spelling, english) in structural_keyword_map() {
        if catalog
            .resolve(Kind::Structural, catalog.primary_locale(), english)
            .is_some_and(|entry| entry.id == id)
        {
            spellings.push(spelling.clone());
        }
    }
    spellings
}

pub(crate) fn canonical_structural_id(
    catalog: &Catalog,
    locale: &Locale,
    spelling: &str,
) -> String {
    catalog
        .resolve(Kind::Structural, locale, spelling)
        .or_else(|| {
            (locale != catalog.primary_locale())
                .then(|| catalog.resolve(Kind::Structural, catalog.primary_locale(), spelling))?
        })
        .map(|entry| entry.id.clone())
        .unwrap_or_else(|| canonical_keyword(spelling).to_string())
}

pub(crate) fn spelling_token_len(tokens: &[Token], index: usize, spelling: &str) -> Option<usize> {
    let mut offset = 0;
    for piece in spelling.split(' ') {
        let (word, dotted) = match piece.strip_suffix('.') {
            Some(word) => (word, true),
            None => (piece, false),
        };
        if !word.is_empty() {
            match tokens.get(index + offset).map(|token| &token.kind) {
                Some(TokenKind::Word(candidate)) if table::names_eq(candidate, word) => {}
                Some(TokenKind::Number { text, .. }) if text == word => {}
                _ => return None,
            }
            offset += 1;
        }
        if dotted {
            match tokens.get(index + offset).map(|token| &token.kind) {
                Some(TokenKind::Dot) => offset += 1,
                _ => return None,
            }
        }
    }
    (offset > 0).then_some(offset)
}

pub(crate) fn structural_keyword_prefix(
    tokens: &[Token],
    index: usize,
    catalog: &Catalog,
    locale: &Locale,
    ids: &[&str],
) -> Option<(String, usize)> {
    let mut best: Option<(String, usize)> = None;
    for id in ids {
        for spelling in structural_spellings(catalog, locale, id) {
            if let Some(len) = spelling_token_len(tokens, index, &spelling)
                && best.as_ref().is_none_or(|(_, length)| len > *length)
            {
                best = Some(((*id).to_string(), len));
            }
        }
    }
    best
}
