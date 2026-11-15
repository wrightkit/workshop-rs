//! Member-specific localized enum spellings. The shared `enums` row of an
//! English name (`Enabled`) can disagree across enum domains, so the
//! `enumMembers` rows, keyed `<setting>.<member>`, take precedence.

use super::schema::{SettingEnumMember, SettingId, SettingOperationError};
use super::table;
use serde_json::Value;

/// The localized spelling of one enum member. Generated table domains
/// (`setting_gamemodes_general_spawnHealthPacks`) end in the setting key.
pub(crate) fn localized(
    locale: &str,
    domain: &str,
    member: &str,
    english: &str,
) -> Option<&'static str> {
    let setting = domain.rsplit('_').next().unwrap_or(domain);
    table::locale_data()
        .get("enumMembers")
        .and_then(|rows| rows.get(format!("{setting}.{member}")))
        .and_then(Value::as_object)
        .and_then(|row| {
            row.iter()
                .find(|(known, _)| known.eq_ignore_ascii_case(locale))
        })
        .and_then(|(_, value)| value.as_str())
        .or_else(|| table::localized_name(locale, "enums", english))
}

pub(crate) fn for_member(locale: &str, member: &SettingEnumMember) -> Option<&'static str> {
    localized(locale, member.domain(), member.id(), member.english_name())
}

pub(crate) fn matches(locale: &str, member: &table::EnumMember, display: &str) -> bool {
    let (domain, id, english) = (member.domain, member.member, member.name);
    localized(locale, domain, id, english).is_some_and(|name| table::names_eq(name, display))
        || table::names_eq(display, english)
}

/// The spelling `source_edit` writes for an enum member in `locale`.
pub(crate) fn source_spelling(
    locale: &str,
    domain: &str,
    member: &str,
    english: &str,
    setting: &SettingId,
) -> Result<String, SettingOperationError> {
    if locale.eq_ignore_ascii_case("en-US") {
        return Ok(english.to_string());
    }
    localized(locale, domain, member, english)
        .map(str::to_string)
        .ok_or_else(|| SettingOperationError::InvalidValue {
            setting: setting.clone(),
            message: format!("missing enums locale mapping for '{english}' in {locale}"),
            span: None,
        })
}
