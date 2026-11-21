//! Matching of a written settings name against catalogued entries.

use crate::frontend::parser::*;

impl ParseContext<'_> {
    pub(super) fn settings_name_candidates<'b>(
        &'b self,
        names: impl IntoIterator<Item = &'b str>,
        section: &'b str,
        display: &str,
    ) -> Vec<String> {
        crate::core::suggest::nearest(
            display,
            names.into_iter().map(|name| {
                table::localized_name(self.locale.as_str(), section, name).unwrap_or(name)
            }),
            crate::core::suggest::CANDIDATE_LIMIT,
        )
    }

    /// Whether `display` is the key itself or a spelling the hero's own
    /// settings projection or aliases declare for it.
    pub(super) fn hero_setting_name_matches(
        &self,
        candidate: &table::TableEntry,
        display: &str,
        hero: Option<&str>,
    ) -> bool {
        if let Some(PathPart::Part(key)) = candidate.path.last()
            && table::names_eq(display, key)
        {
            return true;
        }
        if let (Some(hero), Some(PathPart::Part(key))) = (hero, candidate.path.last()) {
            if table::hero_setting_name(hero, key, self.locale.as_str())
                .is_some_and(|name| table::names_eq(name, display))
            {
                return true;
            }
            if table::hero_setting_display_matches(hero, key, self.locale.as_str(), display) {
                return true;
            }
            if table::hero_setting_alias(hero, key, self.locale.as_str(), display) {
                return true;
            }
        }
        false
    }

    pub(crate) fn settings_name_matches_for_path(
        &self,
        candidate: &table::TableEntry,
        display: &str,
        hero: Option<&str>,
    ) -> bool {
        if self.hero_setting_name_matches(candidate, display, hero) {
            return true;
        }
        if let (Some(hero), Some(slot)) = (hero, table::ability_slot_for_path(candidate.path)) {
            let aliased = candidate
                .path
                .last()
                .and_then(|part| match part {
                    PathPart::Part(key) => Some(*key),
                    _ => None,
                })
                .is_some_and(|key| table::hero_setting_has_alias(hero, key, self.locale.as_str()));
            let unshadowed_key = candidate.path.last().and_then(|part| match part {
                PathPart::Part(key) if !aliased => Some(*key),
                _ => None,
            });
            if let Some(key) = unshadowed_key
                && let Some(template) = table::hero_key_template(key).or_else(|| {
                    candidate
                        .workshop_name
                        .contains("%1$s")
                        .then_some(candidate.workshop_name)
                })
            {
                let ability_name = |locale: &str| {
                    crate::gameplay::data::builtin_ref()
                        .ok()?
                        .query()
                        .ability_name(hero, slot, None, locale)
                        .ok()
                        .map(str::to_string)
                };
                let substitute = |template: &str, name: &str| {
                    table::names_eq(template.replace("%1$s", name).as_str(), display)
                };
                let localized_template =
                    table::localized_name(self.locale.as_str(), "labels", template)
                        .unwrap_or(template);
                let matched = ability_name(self.locale.as_str()).is_some_and(|name| {
                    substitute(localized_template, &name) || substitute(template, &name)
                }) || (self.locale.as_str() != "en-US"
                    && ability_name("en-US").is_some_and(|name| substitute(template, &name)));
                if matched {
                    return true;
                }
            }
        }
        self.settings_name_matches("labels", candidate.workshop_name, display)
    }

    pub(crate) fn settings_name_matches(
        &self,
        section: &str,
        english: &str,
        display: &str,
    ) -> bool {
        let localized = table::localized_name(self.locale.as_str(), section, english);
        localized
            .is_some_and(|localized| table::names_eq(localized, display))
            // Real Workshop exports can mix the selected locale with
            // primary-locale labels when a reviewed mapping is absent.
            // Accept that source spelling for parsing, while emission
            // still fails explicitly if the target mapping is missing.
            || table::names_eq(display, english)
    }
}
