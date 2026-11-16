//! Localized display names of hero settings: the generated per-hero
//! projection and the reviewed aliases that override it.

use std::collections::HashMap;
use std::sync::OnceLock;

use serde::Deserialize;

use super::names::names_eq;

include!("data/generated_hero_settings.rs");

/// The generated names for one hero setting. The settings parser resolves
/// every member against every candidate key of its section, so this lookup is
/// indexed by hero, then key, instead of scanning the projection.
fn generated_names(hero: &str, key: &str) -> &'static [&'static GeneratedHeroSettingName] {
    type Index =
        HashMap<&'static str, HashMap<&'static str, Vec<&'static GeneratedHeroSettingName>>>;
    static INDEX: OnceLock<Index> = OnceLock::new();
    INDEX
        .get_or_init(|| {
            let mut index = Index::new();
            for entry in GENERATED_HERO_SETTING_NAMES {
                index
                    .entry(entry.hero)
                    .or_default()
                    .entry(entry.key)
                    .or_default()
                    .push(entry);
            }
            index
        })
        .get(hero)
        .and_then(|keys| keys.get(key))
        .map_or(&[], Vec::as_slice)
}

pub(crate) fn hero_setting_name(hero: &str, key: &str, locale: &str) -> Option<&'static str> {
    hero_setting_aliases()
        .iter()
        .find(|alias| {
            alias.hero == hero && alias.key == key && alias.locale.eq_ignore_ascii_case(locale)
        })
        .map(|alias| alias.display.as_str())
        .or_else(|| {
            generated_names(hero, key)
                .first()
                .and_then(|entry| entry.localized(locale))
        })
        .or_else(|| {
            if locale.eq_ignore_ascii_case("en-US") {
                generated_names(hero, key)
                    .first()
                    .filter(|entry| {
                        entry.locales.iter().any(|(known, value)| {
                            known.eq_ignore_ascii_case(locale)
                                && (value.trim().is_empty() || value.starts_with(' '))
                        })
                    })
                    .map(|entry| entry.key)
            } else {
                None
            }
        })
}

pub(crate) fn hero_setting_display_matches(
    hero: &str,
    key: &str,
    locale: &str,
    display: &str,
) -> bool {
    if hero_setting_has_alias(hero, key, locale) {
        return false;
    }
    generated_names(hero, key)
        .iter()
        .filter_map(|entry| entry.localized(locale))
        .any(|name| names_eq(name, display))
}

#[derive(Deserialize)]
struct HeroSettingAlias {
    hero: String,
    key: String,
    locale: String,
    display: String,
}

fn hero_setting_aliases() -> &'static [HeroSettingAlias] {
    static ALIASES: OnceLock<Vec<HeroSettingAlias>> = OnceLock::new();
    ALIASES.get_or_init(|| {
        serde_json::from_str(include_str!("data/hero_setting_aliases.json"))
            .expect("hero setting alias data is valid JSON")
    })
}

pub(crate) fn hero_setting_has_alias(hero: &str, key: &str, locale: &str) -> bool {
    hero_setting_aliases().iter().any(|alias| {
        alias.hero == hero && alias.key == key && alias.locale.eq_ignore_ascii_case(locale)
    })
}

pub(crate) fn hero_setting_alias(hero: &str, key: &str, locale: &str, display: &str) -> bool {
    hero_setting_aliases().iter().any(|alias| {
        alias.hero == hero
            && alias.key == key
            && alias.locale.eq_ignore_ascii_case(locale)
            && names_eq(&alias.display, display)
    })
}
