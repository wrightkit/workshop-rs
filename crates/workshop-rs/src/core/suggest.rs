use std::ops::RangeInclusive;

use unicode_normalization::UnicodeNormalization;
use unicode_normalization::char::is_combining_mark;

pub(crate) const CANDIDATE_LIMIT: usize = 3;

pub(crate) fn fold(spelling: &str) -> String {
    spelling
        .nfd()
        .filter(|character| !is_combining_mark(*character))
        .collect::<String>()
        .to_lowercase()
}

pub(crate) fn fold_loose(spelling: &str) -> String {
    fold(spelling)
        .chars()
        .filter(|character| character.is_alphanumeric())
        .collect()
}

pub(crate) fn edit_distance(left: &[char], right: &[char], max: usize) -> usize {
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

pub(crate) fn max_distance(folded_len: usize) -> usize {
    if folded_len >= 6 { 2 } else { 1 }
}

const HANGUL_JAMO: RangeInclusive<char> = '\u{1100}'..='\u{11FF}';
const HIRAGANA: RangeInclusive<char> = '\u{3040}'..='\u{309F}';
const KATAKANA: RangeInclusive<char> = '\u{30A0}'..='\u{30FF}';
const CJK_UNIFIED_IDEOGRAPHS_EXTENSION_A: RangeInclusive<char> = '\u{3400}'..='\u{4DBF}';
const CJK_UNIFIED_IDEOGRAPHS: RangeInclusive<char> = '\u{4E00}'..='\u{9FFF}';
const HANGUL_SYLLABLES: RangeInclusive<char> = '\u{AC00}'..='\u{D7AF}';
const CJK_COMPATIBILITY_IDEOGRAPHS: RangeInclusive<char> = '\u{F900}'..='\u{FAFF}';
const HALFWIDTH_KATAKANA: RangeInclusive<char> = '\u{FF66}'..='\u{FF9D}';

pub(crate) fn is_unspaced_script(character: char) -> bool {
    [
        HANGUL_JAMO,
        HIRAGANA,
        KATAKANA,
        CJK_UNIFIED_IDEOGRAPHS_EXTENSION_A,
        CJK_UNIFIED_IDEOGRAPHS,
        HANGUL_SYLLABLES,
        CJK_COMPATIBILITY_IDEOGRAPHS,
        HALFWIDTH_KATAKANA,
    ]
    .iter()
    .any(|range| range.contains(&character))
}

pub(crate) fn suggest<'a>(
    input: &str,
    candidates: impl Iterator<Item = &'a str>,
) -> Option<String> {
    let folded_input: Vec<char> = fold(input).chars().collect();
    let max = max_distance(folded_input.len());
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

pub(crate) fn nearest<'a>(
    input: &str,
    candidates: impl IntoIterator<Item = &'a str>,
    limit: usize,
) -> Vec<String> {
    let folded_input: Vec<char> = fold(input).chars().collect();
    let max = max_distance(folded_input.len());
    let mut seen = std::collections::HashSet::new();
    let mut ranked: Vec<(usize, &str)> = candidates
        .into_iter()
        .filter_map(|candidate| {
            let folded = fold(candidate);
            if !seen.insert(folded.clone()) {
                return None;
            }
            let folded_candidate: Vec<char> = folded.chars().collect();
            let distance = edit_distance(&folded_input, &folded_candidate, max);
            (distance <= max).then_some((distance, candidate))
        })
        .collect();
    ranked.sort_by_key(|(distance, _)| *distance);
    ranked.truncate(limit);
    ranked
        .into_iter()
        .map(|(_, candidate)| candidate.to_string())
        .collect()
}

pub(crate) fn with_suggestion_text(message: String, suggestion: Option<&str>) -> String {
    match suggestion {
        Some(spelling) => format!("{message} (did you mean '{spelling}'?)"),
        None => message,
    }
}

pub(crate) fn with_candidates_text(message: String, candidates: &[String]) -> String {
    match candidates {
        [] => message,
        [one] => format!("{message} (did you mean '{one}'?)"),
        [first, rest @ ..] => {
            let others = rest
                .iter()
                .map(|spelling| format!("'{spelling}'"))
                .collect::<Vec<_>>()
                .join(" or ");
            format!("{message} (did you mean '{first}', {others}?)")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unspaced_scripts_cover_each_named_block() {
        for character in ['ᄀ', 'あ', 'カ', '㐀', '一', '가', '\u{F900}', 'ﾊ'] {
            assert!(is_unspaced_script(character), "{character}");
        }
        for character in ['a', '⺀', 'ힰ'] {
            assert!(!is_unspaced_script(character), "{character}");
        }
    }

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
        assert_eq!(suggest("Team X", ["Team 1", "Team 2"].into_iter()), None);
    }

    #[test]
    fn nearest_ranks_and_limits_candidates() {
        let candidates = ["Create HUD Text", "Create In-World Text", "Wait"];
        assert_eq!(
            nearest("Create HUD Txt", candidates.into_iter(), 3),
            vec!["Create HUD Text".to_string()]
        );
        assert_eq!(
            nearest("Team X", ["Team 1", "Team 2"].into_iter(), 3),
            vec!["Team 1".to_string(), "Team 2".to_string()]
        );
        assert_eq!(
            nearest("Team 1", ["Team 1", "team 1", "Team 2"].into_iter(), 3),
            vec!["Team 1".to_string(), "Team 2".to_string()]
        );
    }
}
