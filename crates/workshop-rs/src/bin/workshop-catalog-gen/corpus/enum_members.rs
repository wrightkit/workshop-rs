//! Member-specific rows for enum members whose shared English spelling
//! (`Enabled`) is ambiguous across domains: the export candidates whose key
//! ends in `<setting>.values.<member>` give that member its own spelling.

use super::{Index, settings_entry};
use serde_json::{Map, Value};

/// Records a member row for an ambiguous `enum.<setting>.<member>` surface
/// entry. Returns whether the surface entry is now resolved.
pub(super) fn resolve(
    index: &Index,
    (section, surface_id, en): (&str, &str, &str),
    locales: &[String],
    rows: &mut Map<String, Value>,
) -> Result<bool, String> {
    let Some((setting, member)) = (section == "enums")
        .then(|| surface_id.strip_prefix("enum.")?.split_once('.'))
        .flatten()
    else {
        return Ok(false);
    };
    let suffix = format!(".{setting}.values.{member}");
    let Some(candidates) = index.by_en.get(en) else {
        return Ok(false);
    };
    let found: Vec<_> = candidates
        .iter()
        .filter(|candidate| candidate.key.ends_with(&suffix))
        .cloned()
        .collect();
    if found.is_empty() {
        return Ok(false);
    }
    rows.insert(
        format!("{setting}.{member}"),
        settings_entry(&found, en, locales)?,
    );
    Ok(true)
}
