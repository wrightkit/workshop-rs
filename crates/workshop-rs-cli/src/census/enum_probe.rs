//! Localized enum-member probes. `enums` rows are shared by English name;
//! `enumMembers` rows (`<setting>.<member>`) belong to one setting.

use super::*;

pub(super) fn probe(
    catalog: &Catalog,
    index: &SettingsSpellings,
    name: &settings_schema::LocalizedSettingName,
    locale: &Locale,
) -> Result<String, String> {
    let (english, spelling) = (name.english, name.spelling);
    let definition = match name.section {
        "enumMembers" => {
            let suffix = english
                .split_once('.')
                .map(|(setting, _)| format!(".{setting}"));
            index
                .by_member
                .values()
                .flatten()
                .find(|definition| {
                    suffix
                        .as_ref()
                        .is_some_and(|s| definition.path().ends_with(s))
                })
                .ok_or_else(|| format!("no settings definition for {english:?}"))?
        }
        _ => &index
            .by_member
            .get(english)
            .ok_or_else(|| format!("no settings definition takes member {english:?}"))?[0],
    };
    let hero = settings_probe_hero(definition, definition.presentation().english_name);
    let label = index.name(
        locale.as_str(),
        "labels",
        definition.presentation().english_name,
    );
    Ok(localized_settings_context(
        catalog,
        index,
        locale,
        definition,
        &format!("{label}: {spelling}"),
        hero.as_ref().and_then(|hero| hero.as_deref()),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn census_probes_the_member_specific_zh_cn_spelling() {
        let catalog = Catalog::builtin().expect("builtin catalog");
        let census = Census::builtin(&catalog).expect("census");
        let case = census
            .cases()
            .find(|case| {
                case.case_id
                    .starts_with("localization/zh-cn/settings/enumMembers/")
                    && case.source.contains("Spawn Health Packs: 开启")
            })
            .expect("a zh-CN probe of the spawnHealthPacks enabled spelling");
        assert!(!matches!(case.support, CensusSupport::KnownGap { .. }));
    }
}
