//! Per-locale reverse lookup indexes (spelling to catalog identity).
//!
//! An index is built from the entries' alias lists the first time a lookup
//! needs its locale, so a run pays only for the locales it touches. Building
//! also validates the locale's aliases: [`Catalog::load`] builds every
//! declared locale, so supplied data is rejected at load; the embedded
//! catalog skips that at runtime and is checked by the test suite and
//! `workshop-catalog-gen check`.

use crate::core::error::{CatalogError, Result};
use crate::fast_hash::HashMap;

use super::{Catalog, Kind, Locale, normalize_spelling};

type Pair = (String, String);
type MemberIndexMap = HashMap<String, HashMap<String, (usize, usize)>>;

/// Spellings that differ from their whitespace-collapsed form, keyed by that
/// form; a collision between different identities maps to `None`.
type Collapsed<T> = HashMap<String, Option<T>>;

#[derive(Debug, Clone, Default)]
struct CollapsedIndex {
    alias_to_entry: [Collapsed<usize>; Kind::NUM_KINDS],
    localized_string_alias: Collapsed<usize>,
    enum_alias_to_domain: Collapsed<String>,
    enum_alias_to_member: HashMap<String, Collapsed<(usize, usize)>>,
    bare_member: HashMap<String, Vec<Pair>>,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct LocaleIndex {
    alias_to_entry: [HashMap<String, usize>; Kind::NUM_KINDS],
    localized_string_alias: HashMap<String, usize>,
    enum_alias_to_domain: HashMap<String, String>,
    enum_alias_to_member: MemberIndexMap,
    bare_member: HashMap<String, Vec<Pair>>,
    collapsed: CollapsedIndex,
}

fn collapse<T: PartialEq + Clone>(source: &HashMap<String, T>, target: &mut Collapsed<T>) {
    for (spelling, value) in source {
        let key = normalize_spelling(spelling);
        if key == spelling.as_str() {
            continue;
        }
        match target.get_mut(key.as_ref()) {
            Some(slot) if *slot != Some(value.clone()) => *slot = None,
            Some(_) => {}
            None => {
                target.insert(key.into_owned(), Some(value.clone()));
            }
        }
    }
}

fn register<T>(
    map: &mut HashMap<String, T>,
    spelling: &str,
    value: T,
    duplicate: impl FnOnce(&T) -> String,
) -> Result<()> {
    if let Some(existing) = map.get(spelling) {
        return Err(CatalogError::validation(duplicate(existing)));
    }
    map.insert(spelling.to_string(), value);
    Ok(())
}

impl LocaleIndex {
    /// Index the declared locale at `position` in `catalog.locales`.
    pub(super) fn build(catalog: &Catalog, position: usize) -> Result<LocaleIndex> {
        let locale = &catalog.locales[position];
        let mut index = LocaleIndex::default();

        for (entry_index, entry) in catalog.entries.iter().enumerate() {
            let map = &mut index.alias_to_entry[entry.kind.as_index()];
            for spelling in entry.spellings(locale) {
                register(map, spelling, entry_index, |_| {
                    format!(
                        "duplicate {} alias '{spelling}' for locale '{locale}'",
                        entry.kind.as_str()
                    )
                })?;
            }
        }
        for (string_index, entry) in catalog.localized_strings.iter().enumerate() {
            for spelling in entry.spellings(locale) {
                register(
                    &mut index.localized_string_alias,
                    spelling,
                    string_index,
                    |_| {
                        format!(
                            "duplicate localized string alias '{spelling}' for locale '{locale}'"
                        )
                    },
                )?;
            }
        }
        for (domain_index, domain) in catalog.enums.iter().enumerate() {
            // The primary locale also maps every domain's own name; that entry
            // is registered after the declared aliases, whatever their order.
            let primary = position == 0;
            for spelling in domain.aliases.get(locale).into_iter().flatten() {
                register(
                    &mut index.enum_alias_to_domain,
                    spelling,
                    domain.domain.clone(),
                    |existing| {
                        format!(
                            "duplicate enum domain alias '{spelling}' for '{existing}' and '{}' in locale '{locale}'",
                            domain.domain
                        )
                    },
                )?;
            }
            if primary {
                let spelling = &domain.domain;
                match index.enum_alias_to_domain.get(spelling) {
                    Some(existing) if *existing == domain.domain => {}
                    Some(existing) => {
                        return Err(CatalogError::validation(format!(
                            "duplicate enum domain alias '{spelling}' for '{existing}' and '{}' in locale '{locale}'",
                            domain.domain
                        )));
                    }
                    None => {
                        index
                            .enum_alias_to_domain
                            .insert(spelling.clone(), domain.domain.clone());
                    }
                }
            }
            let members = index
                .enum_alias_to_member
                .entry(domain.domain.clone())
                .or_default();
            for (member_index, member) in domain.members.iter().enumerate() {
                for spelling in member.spellings(locale) {
                    register(members, spelling, (domain_index, member_index), |_| {
                        format!(
                            "duplicate enum alias '{spelling}' in '{}' for locale '{locale}'",
                            domain.domain
                        )
                    })?;
                    index
                        .bare_member
                        .entry(spelling.clone())
                        .or_default()
                        .push((domain.domain.clone(), member.member.clone()));
                }
            }
        }

        let collapsed = &mut index.collapsed;
        for (map, norm) in index
            .alias_to_entry
            .iter()
            .zip(&mut collapsed.alias_to_entry)
        {
            collapse(map, norm);
        }
        collapse(
            &index.localized_string_alias,
            &mut collapsed.localized_string_alias,
        );
        collapse(
            &index.enum_alias_to_domain,
            &mut collapsed.enum_alias_to_domain,
        );
        for (domain, map) in &index.enum_alias_to_member {
            collapse(
                map,
                collapsed
                    .enum_alias_to_member
                    .entry(domain.clone())
                    .or_default(),
            );
        }
        for (spelling, matches) in &index.bare_member {
            let key = normalize_spelling(spelling);
            if key != spelling.as_str() {
                let merged = collapsed.bare_member.entry(key.into_owned()).or_default();
                for pair in matches {
                    if !merged.contains(pair) {
                        merged.push(pair.clone());
                    }
                }
            }
        }
        Ok(index)
    }

    pub(super) fn entry(&self, kind: Kind, spelling: &str) -> Option<usize> {
        self.alias_to_entry[kind.as_index()]
            .get(spelling)
            .copied()
            .or_else(|| {
                lookup_collapsed(&self.collapsed.alias_to_entry[kind.as_index()], spelling).copied()
            })
    }

    pub(super) fn localized_string(&self, spelling: &str) -> Option<usize> {
        self.localized_string_alias
            .get(spelling)
            .copied()
            .or_else(|| lookup_collapsed(&self.collapsed.localized_string_alias, spelling).copied())
    }

    pub(super) fn enum_domain(&self, spelling: &str) -> Option<&str> {
        self.enum_alias_to_domain
            .get(spelling)
            .map(String::as_str)
            .or_else(|| {
                lookup_collapsed(&self.collapsed.enum_alias_to_domain, spelling).map(String::as_str)
            })
    }

    pub(super) fn enum_member(&self, domain: &str, spelling: &str) -> Option<(usize, usize)> {
        self.enum_alias_to_member
            .get(domain)?
            .get(spelling)
            .copied()
            .or_else(|| {
                lookup_collapsed(self.collapsed.enum_alias_to_member.get(domain)?, spelling)
                    .copied()
            })
    }

    pub(super) fn bare_member(&self, spelling: &str) -> Option<&[Pair]> {
        self.bare_member
            .get(spelling)
            .or_else(|| {
                self.collapsed
                    .bare_member
                    .get(normalize_spelling(spelling).as_ref())
            })
            .map(Vec::as_slice)
    }
}

fn lookup_collapsed<'a, T>(index: &'a Collapsed<T>, spelling: &str) -> Option<&'a T> {
    index.get(normalize_spelling(spelling).as_ref())?.as_ref()
}

impl Catalog {
    /// The lookup index for a declared locale, built on first use. Indexes
    /// are validated at load for supplied data and by tests for the embedded
    /// catalog, so a build failure here is a broken embedded dataset.
    pub(super) fn locale_index(&self, locale: &Locale) -> Option<&LocaleIndex> {
        let position = self
            .locales
            .iter()
            .position(|declared| declared == locale)?;
        Some(self.locale_indexes[position].get_or_init(|| {
            LocaleIndex::build(self, position).expect("embedded catalog aliases are unique")
        }))
    }

    /// Build and store every declared locale's index, rejecting duplicate
    /// aliases. Reports the first failure.
    pub(super) fn validate_locale_indexes(&self) -> Result<()> {
        for position in 0..self.locales.len() {
            let index = LocaleIndex::build(self, position)?;
            let _ = self.locale_indexes[position].set(index);
        }
        Ok(())
    }
}
