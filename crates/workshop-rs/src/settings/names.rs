pub(crate) fn names_eq(left: &str, right: &str) -> bool {
    // `str::to_lowercase` differs from per-character lowercasing only for the
    // context-sensitive final sigma, so every other pair compares without
    // allocating. Whitespace-insensitive equality implies the plain one.
    if !left.contains('\u{3a3}') && !right.contains('\u{3a3}') {
        fn folded(text: &str) -> impl Iterator<Item = char> + '_ {
            text.chars()
                .filter(|c| !c.is_whitespace())
                .flat_map(char::to_lowercase)
        }
        return left == right || folded(left).eq(folded(right));
    }
    left == right
        || left.to_lowercase() == right.to_lowercase()
        || crate::catalog::normalize_spelling(left).to_lowercase()
            == crate::catalog::normalize_spelling(right).to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::names_eq;

    #[test]
    fn names_eq_matches_allocating_comparison() {
        fn reference(left: &str, right: &str) -> bool {
            left == right
                || left.to_lowercase() == right.to_lowercase()
                || crate::catalog::normalize_spelling(left).to_lowercase()
                    == crate::catalog::normalize_spelling(right).to_lowercase()
        }
        let samples = [
            "",
            "Max Health",
            "maxhealth",
            "MAX\tHEALTH",
            "最大 生命值",
            "最大生命值",
            "\u{c9}clair",
            "\u{e9}clair",
            "\u{3a3}\u{3a3}",
            "\u{3c3}\u{3c2}",
            "\u{3c3}\u{3c3}",
            "\u{130}",
            "i\u{307}",
            "a b",
            "ab ",
        ];
        for left in samples {
            for right in samples {
                assert_eq!(
                    names_eq(left, right),
                    reference(left, right),
                    "{left:?} vs {right:?}"
                );
            }
        }
    }
}
