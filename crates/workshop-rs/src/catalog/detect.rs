use std::collections::HashMap;

use aho_corasick::AhoCorasick;

use crate::catalog::{Catalog, Kind, Locale};
use crate::core::error::{Result, WorkshopError};

/// A language-detection result with ranked evidence.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct Detection {
    /// The best-matching locale.
    pub locale: Locale,
    /// Confidence in `[0, 1)`; grows with the number of distinct matches.
    pub confidence: f64,
    /// Distinct catalog aliases found for the best locale.
    pub matches: usize,
    /// Every candidate locale with its match count, ranked descending.
    pub candidates: Vec<(Locale, usize)>,
}

/// The minimum distinct-match count required to trust a detection.
pub const MIN_MATCHES: usize = 2;

/// Detect the Workshop client language of the input.
pub fn detect(input: &str, catalog: &Catalog) -> Detection {
    let counts = catalog
        .detection_index
        .as_ref()
        .expect("validated catalog")
        .counts(input, catalog.locales().len());
    let mut candidates: Vec<(Locale, usize)> = catalog
        .locales()
        .iter()
        .enumerate()
        .map(|(index, locale)| (locale.clone(), counts[index]))
        .collect();
    candidates.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));

    let (locale, matches) = candidates
        .first()
        .cloned()
        .unwrap_or_else(|| (Locale::new("en-US"), 0));
    let confidence = matches as f64 / (matches as f64 + 1.0);
    Detection {
        locale,
        confidence,
        matches,
        candidates,
    }
}

/// Resolve a locale for parsing: an explicit override always wins; otherwise
/// auto-detect and require a confident, unambiguous match.
pub fn resolve_locale(
    input: &str,
    catalog: &Catalog,
    override_locale: Option<&Locale>,
) -> Result<Locale> {
    if let Some(locale) = override_locale {
        if !catalog.supports(locale) {
            return Err(WorkshopError::unknown(
                "locale",
                locale.to_string(),
                locale.clone(),
                None,
            ));
        }
        return Ok(locale.clone());
    }
    let detection = detect(input, catalog);
    if detection.matches == 0 {
        return Err(WorkshopError::unknown(
            "language",
            "<none>".to_string(),
            detection.locale,
            None,
        ));
    }
    if detection.matches < MIN_MATCHES {
        return Err(WorkshopError::unsupported(
            format!(
                "insufficient evidence to detect the Workshop client language ({} distinct match(es))",
                detection.matches
            ),
            None,
        ));
    }
    if detection.candidates.len() > 1
        && detection.candidates[0].1 == detection.candidates[1].1
        && detection.candidates[0].1 > 0
    {
        return Err(WorkshopError::unsupported(
            "ambiguous Workshop client language: multiple locales tie",
            None,
        ));
    }
    Ok(detection.locale)
}

#[derive(Debug, Clone)]
pub(super) struct AliasIndex {
    matcher: AhoCorasick,
    locale_indices: Vec<Vec<usize>>,
}

impl AliasIndex {
    pub(super) fn build(catalog: &Catalog) -> std::result::Result<Self, aho_corasick::BuildError> {
        let mut patterns = Vec::new();
        let mut by_spelling = HashMap::new();
        let mut locale_indices: Vec<Vec<usize>> = Vec::new();
        for (locale_index, locale) in catalog.locales().iter().enumerate() {
            let spellings = catalog
                .entries
                .iter()
                .filter(|entry| {
                    matches!(
                        entry.kind,
                        Kind::Structural
                            | Kind::Action
                            | Kind::Value
                            | Kind::Event
                            | Kind::Operator
                    )
                })
                .filter_map(|entry| {
                    Some((
                        entry.spelling(locale)?,
                        entry.spelling(catalog.primary_locale()),
                    ))
                })
                .chain(
                    catalog
                        .enum_domains()
                        .flat_map(|domain| &domain.members)
                        .filter_map(|member| {
                            Some((
                                member.spelling(locale)?,
                                member.spelling(catalog.primary_locale()),
                            ))
                        }),
                );
            for (spelling, primary) in spellings {
                if spelling.is_empty()
                    || (locale != catalog.primary_locale() && primary == Some(spelling))
                {
                    continue;
                }
                let pattern = *by_spelling.entry(spelling).or_insert_with(|| {
                    let index = patterns.len();
                    patterns.push(spelling);
                    locale_indices.push(Vec::new());
                    index
                });
                locale_indices[pattern].push(locale_index);
            }
        }
        Ok(Self {
            matcher: AhoCorasick::new(patterns)?,
            locale_indices,
        })
    }

    fn counts(&self, input: &str, locale_count: usize) -> Vec<usize> {
        let mut counts = vec![0; locale_count];
        let mut seen = vec![false; self.locale_indices.len()];
        let mut next_start = vec![0; self.locale_indices.len()];
        for found in self.matcher.find_overlapping_iter(input) {
            let pattern = found.pattern().as_usize();
            if seen[pattern] || found.start() < next_start[pattern] {
                continue;
            }
            // str::match_indices skips overlapping occurrences of the same spelling,
            // even when the first occurrence fails the word-boundary check.
            next_start[pattern] = found.end();
            let before_ok = !input[..found.start()]
                .chars()
                .next_back()
                .is_some_and(is_word_char);
            let after_ok = !input[found.end()..]
                .chars()
                .next()
                .is_some_and(is_word_char);
            if before_ok && after_ok {
                seen[pattern] = true;
                for &locale in &self.locale_indices[pattern] {
                    counts[locale] += 1;
                }
            }
        }
        counts
    }
}

fn is_word_char(ch: char) -> bool {
    unicode_ident::is_xid_continue(ch) || ch == '_' || ch == '-'
}
