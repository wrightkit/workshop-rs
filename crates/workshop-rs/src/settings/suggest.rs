//! Canonical-spelling suggestions for rejected settings input.
//!
//! Acceptance stays strict on accents (only case differences parse); this
//! module only ranks candidates for diagnostics. A rejected key, map, hero,
//! team, or enum value suggests the single closest canonical spelling when
//! one exists: a case or accent difference, or a small edit distance. No
//! candidate or a tie between candidates yields no suggestion.

use unicode_normalization::UnicodeNormalization;
use unicode_normalization::char::is_combining_mark;

/// Comparison fold for suggestion ranking: NFD decomposition drops accent
/// marks (`â` -> `a`), then lowercase drops case. Whitespace and punctuation
/// are significant, matching the parser's phrase handling.
fn fold(spelling: &str) -> String {
    spelling
        .nfd()
        .filter(|character| !is_combining_mark(*character))
        .collect::<String>()
        .to_lowercase()
}

/// Levenshtein distance between folded spellings, early-exiting past `max`.
fn edit_distance(left: &[char], right: &[char], max: usize) -> usize {
    if left.len().abs_diff(right.len()) > max {
        return max + 1;
    }
    let mut row: Vec<usize> = (0..=right.len()).collect();
    for (i, a) in left.iter().enumerate() {
        let mut previous = row[0];
        row[0] = i + 1;
        let mut row_min = row[0];
        for (j, b) in right.iter().enumerate() {
            let cost = usize::from(a != b);
            let cell = (row[j + 1] + 1).min(row[j] + 1).min(previous + cost);
            previous = row[j + 1];
            row[j + 1] = cell;
            row_min = row_min.min(cell);
        }
        if row_min > max {
            return max + 1;
        }
    }
    row[right.len()]
}

/// The single closest canonical spelling to rejected `input`, or `None` when
/// no candidate is close or the closest is ambiguous. Candidates that fold to
/// the same comparison form are considered once.
pub(crate) fn suggest<'a>(
    input: &str,
    candidates: impl Iterator<Item = &'a str>,
) -> Option<String> {
    let folded_input: Vec<char> = fold(input).chars().collect();
    // Short spellings allow one edit; longer ones allow two. That is enough
    // for case/accent differences (distance 0 after folding) and single
    // typos, while keeping distant names from suggesting.
    let max = if folded_input.len() >= 6 { 2 } else { 1 };
    let mut seen = std::collections::HashSet::new();
    let mut best: Option<(usize, &str)> = None;
    let mut tied = false;
    for candidate in candidates {
        let folded = fold(candidate);
        if !seen.insert(folded.clone()) {
            continue;
        }
        let folded_candidate: Vec<char> = folded.chars().collect();
        let distance = edit_distance(&folded_input, &folded_candidate, max);
        if distance > max {
            continue;
        }
        match best {
            Some((best_distance, _)) if distance == best_distance => tied = true,
            Some((best_distance, _)) if distance > best_distance => {}
            _ => {
                best = Some((distance, candidate));
                tied = false;
            }
        }
    }
    if tied {
        None
    } else {
        best.map(|(_, candidate)| candidate.to_string())
    }
}

/// Append the `did you mean` suffix diagnostics render for a suggestion.
pub(crate) fn with_suggestion_text(message: String, suggestion: Option<&str>) -> String {
    match suggestion {
        Some(spelling) => format!("{message} (did you mean '{spelling}'?)"),
        None => message,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accent_difference_suggests_canonical() {
        let candidates = ["Château Guillard", "King's Row"];
        assert_eq!(
            suggest("Chateau Guillard", candidates.into_iter()).as_deref(),
            Some("Château Guillard")
        );
    }

    #[test]
    fn distant_and_ambiguous_spellings_suggest_nothing() {
        let candidates = ["Château Guillard", "King's Row"];
        assert_eq!(suggest("Nepal", candidates.into_iter()), None);
        // Two candidates at the same small distance tie, yielding none.
        assert_eq!(suggest("Team X", ["Team 1", "Team 2"].into_iter()), None);
    }
}
