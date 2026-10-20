//! Localized description coverage contract (#517): every catalog action,
//! value, and parameter position resolves a description in each shipped
//! locale or names an explicit entry in the dataset's `gaps` list. A
//! requested locale without text falls back to the primary locale's text
//! with `fallback` marked on the returned [`Description`].

use std::collections::BTreeSet;

use serde_json::Value;

use workshop_rs::catalog::descriptions::DESCRIPTIONS_DATA;
use workshop_rs::catalog::{Catalog, Kind};
use workshop_rs::lookup::LookupMatch;

use crate::common::{catalog, en, zh};

/// The gap keys the committed dataset declares for `locale`, split into
/// entry-level ids and `(id, position)` parameter gaps.
fn gaps(locale: &str) -> (BTreeSet<String>, BTreeSet<(String, usize)>) {
    let raw: Value = serde_json::from_str(DESCRIPTIONS_DATA).expect("descriptions data is JSON");
    let mut entries = BTreeSet::new();
    let mut params = BTreeSet::new();
    for key in raw["gaps"][locale].as_array().into_iter().flatten() {
        let key = key.as_str().expect("gap keys are strings");
        if let Some(without_bracket) = key.strip_suffix(']') {
            let (id, index) = without_bracket
                .split_once(".params[")
                .expect("parameter gaps are '<id>.params[<index>]'");
            params.insert((id.to_string(), index.parse().expect("index is a number")));
        } else {
            entries.insert(key.to_string());
        }
    }
    (entries, params)
}

#[test]
fn every_action_and_value_resolves_en_us_description_or_names_a_gap() {
    let catalog = catalog();
    let (entry_gaps, param_gaps) = gaps("en-US");
    let mut described = 0usize;
    let mut gapped = 0usize;
    for kind in [Kind::Action, Kind::Value] {
        for entry in catalog.entries_of(kind) {
            let in_gap = entry_gaps.contains(&entry.id);
            let resolved = catalog.description(kind, &entry.id, &en()).is_some();
            assert_ne!(
                resolved,
                in_gap,
                "{} '{id}': en-US coverage must be text or a gap, not both or neither",
                kind.as_str(),
                id = entry.id
            );
            if resolved {
                described += 1;
            } else {
                gapped += 1;
            }
            for position in 0..entry.param_count() {
                let in_gap = param_gaps.contains(&(entry.id.clone(), position));
                let resolved = catalog
                    .param_description(kind, &entry.id, position, &en())
                    .is_some();
                assert_ne!(
                    resolved,
                    in_gap,
                    "{} '{}' parameter {position}: en-US coverage must be text or a gap",
                    kind.as_str(),
                    entry.id
                );
            }
        }
    }
    assert_eq!(gapped, entry_gaps.len(), "every declared gap is needed");
    assert!(described > 0);
}

#[test]
fn every_action_and_value_resolves_zh_cn_description_or_names_a_gap() {
    let catalog = catalog();
    let (en_entry_gaps, en_param_gaps) = gaps("en-US");
    let (entry_gaps, param_gaps) = gaps("zh-CN");
    let mut described = 0usize;
    let mut fallen_back = 0usize;
    // zh-CN text stands alone: a position covered in zh-CN resolves zh-CN
    // even where en-US is gapped. A zh-CN gap falls back to en-US when en-US
    // text exists, and resolves to None only when neither locale has text.
    for kind in [Kind::Action, Kind::Value] {
        for entry in catalog.entries_of(kind) {
            let zh_gapped = entry_gaps.contains(&entry.id);
            let resolved = catalog.description(kind, &entry.id, &zh());
            match (zh_gapped, en_entry_gaps.contains(&entry.id), resolved) {
                (false, _, Some(description)) => {
                    described += 1;
                    assert!(
                        !description.fallback,
                        "{} '{}': covered zh-CN text must not be marked fallback",
                        kind.as_str(),
                        entry.id
                    );
                    assert_eq!(description.locale, zh());
                }
                (true, false, Some(description)) => {
                    fallen_back += 1;
                    assert!(
                        description.fallback,
                        "{} '{}': a zh-CN gap falls back visibly",
                        kind.as_str(),
                        entry.id
                    );
                    assert_eq!(description.locale, en());
                }
                (true, true, None) => {}
                (zh_gapped, en_gapped, resolved) => panic!(
                    "{} '{}': zh gap {zh_gapped}, en gap {en_gapped}, resolved {resolved:?}",
                    kind.as_str(),
                    entry.id
                ),
            }
            for position in 0..entry.param_count() {
                let key = (entry.id.clone(), position);
                let zh_gapped = param_gaps.contains(&key);
                let en_gapped = en_param_gaps.contains(&key);
                let resolved = catalog.param_description(kind, &entry.id, position, &zh());
                match (zh_gapped, en_gapped, resolved) {
                    (false, _, Some(description)) => {
                        assert!(
                            !description.fallback,
                            "{} '{}' parameter {position}: covered zh-CN text must not \
                             be marked fallback",
                            kind.as_str(),
                            entry.id
                        );
                        assert_eq!(description.locale, zh());
                    }
                    (true, false, Some(description)) => {
                        assert!(
                            description.fallback,
                            "{} '{}' parameter {position}: a zh-CN gap falls back visibly",
                            kind.as_str(),
                            entry.id
                        );
                        assert_eq!(description.locale, en());
                    }
                    (true, true, None) => {}
                    (zh_gapped, en_gapped, resolved) => panic!(
                        "{} '{}' parameter {position}: zh gap {zh_gapped}, en gap \
                         {en_gapped}, resolved {resolved:?}",
                        kind.as_str(),
                        entry.id
                    ),
                }
            }
        }
    }
    assert!(described > 0);
    assert!(fallen_back > 0, "the zh-CN gap list is exercised");
}

#[test]
fn positions_without_text_never_fabricate_a_description() {
    let catalog = catalog();
    // en-US gaps resolve to None in en-US.
    assert!(
        catalog
            .description(Kind::Value, "customString", &en())
            .is_none()
    );
    for (kind, id, position) in [
        (Kind::Value, "customString", 0),
        (Kind::Value, "compare", 1),
        (Kind::Value, "string", 0),
        (Kind::Action, "setHealingDealt", 1),
    ] {
        assert!(
            catalog
                .param_description(kind, id, position, &en())
                .is_none(),
            "{id} parameter {position}"
        );
    }
    // `compare.params[1]` is gapped in both locales: zh-CN has no en-US text
    // to fall back to and resolves to None.
    assert!(
        catalog
            .param_description(Kind::Value, "compare", 1, &zh())
            .is_none()
    );
    // A position uncovered in en-US but covered in zh-CN still resolves the
    // zh-CN text: an en-US gap never hides another locale's text.
    let zh_text = catalog
        .description(Kind::Value, "customString", &zh())
        .expect("customString carries zh-CN text");
    assert_eq!(zh_text.locale, zh());
    assert!(!zh_text.fallback);
    assert!(
        catalog
            .param_description(Kind::Action, "setHealingDealt", 1, &zh())
            .is_some_and(|description| description.locale == zh()),
        "setHealingDealt parameter 1 resolves its zh-CN text"
    );
    // `customString`'s remaining positions carry text.
    assert!(
        catalog
            .param_description(Kind::Value, "customString", 1, &en())
            .is_some()
    );
}

#[test]
fn gapped_locale_falls_back_to_primary_marked() {
    let catalog = catalog();
    // `getSpawnPoints` carries en-US text while zh-CN is an explicit gap.
    let direct = catalog
        .description(Kind::Value, "getSpawnPoints", &en())
        .expect("en-US text exists");
    assert!(!direct.fallback);
    assert_eq!(direct.locale, en());
    let fallen_back = catalog
        .description(Kind::Value, "getSpawnPoints", &zh())
        .expect("zh-CN falls back to en-US");
    assert!(fallen_back.fallback, "the fallback is visibly marked");
    assert_eq!(fallen_back.locale, en(), "the returned text is en-US");
    assert_eq!(fallen_back.text, direct.text);
    let param = catalog
        .param_description(Kind::Value, "getSpawnPoints", 0, &zh())
        .expect("parameter zh-CN falls back to en-US");
    assert!(param.fallback);
    assert_eq!(param.locale, en());
}

#[test]
fn lookup_surfaces_entry_and_parameter_descriptions() {
    let catalog = catalog();
    let matches = catalog
        .lookup(&en(), "Create HUD Text")
        .expect("lookup answers");
    let LookupMatch::Builtin {
        description,
        signature,
        ..
    } = matches.first().expect("a match exists")
    else {
        panic!("expected a builtin match: {matches:?}")
    };
    let description = description.as_ref().expect("entry description exists");
    assert!(!description.fallback);
    assert!(description.text.contains("hud text"));
    let signature = signature.as_ref().expect("createHudText is callable");
    for param in &signature.params {
        assert!(
            param.description.is_some(),
            "parameter '{}' resolves a description",
            param.name
        );
    }
}

#[test]
fn lookup_in_another_shipped_locale_surfaces_text() {
    let catalog = catalog();
    let matches = catalog
        .lookup(&zh(), "创建HUD文本")
        .expect("lookup answers");
    let description = matches
        .iter()
        .find_map(|lookup| match lookup {
            LookupMatch::Builtin {
                id, description, ..
            } if id == "createHudText" => description.as_ref(),
            _ => None,
        })
        .expect("zh-CN lookup resolves the entry");
    assert!(!description.fallback);
    assert_eq!(description.locale, zh());
    let signature = matches
        .iter()
        .find_map(|lookup| match lookup {
            LookupMatch::Builtin { id, signature, .. } if id == "createHudText" => {
                signature.as_ref()
            }
            _ => None,
        })
        .expect("signature is present");
    assert!(
        signature
            .params
            .iter()
            .all(|param| param.description.as_ref().is_some_and(|d| !d.fallback)),
        "parameter descriptions resolve zh-CN text as well"
    );
}

#[test]
fn lookup_in_a_shipped_locale_marks_gapped_fallback() {
    let catalog = catalog();
    // `getSpawnPoints` is a zh-CN gap: lookup still answers, with the en-US
    // text visibly marked as a fallback.
    let matches = catalog.lookup(&zh(), "重生点").expect("lookup answers");
    let description = matches
        .iter()
        .find_map(|lookup| match lookup {
            LookupMatch::Builtin {
                id, description, ..
            } if id == "getSpawnPoints" => description.as_ref(),
            _ => None,
        })
        .expect("zh-CN lookup resolves the entry");
    assert!(description.fallback);
    assert_eq!(description.locale, en());
}

#[test]
fn descriptions_dataset_reports_version_digest_and_stages() {
    let catalog = catalog();
    assert_eq!(catalog.description_locales(), &[en(), zh()]);
    assert!(catalog.pending_description_locales().is_empty());
    assert!(catalog.descriptions_digest().is_some());
    assert_ne!(catalog.descriptions_version(), "dev");
}

/// `data` mutated by `mutate` must still parse, but attaching it must fail
/// with a message matching `expected`.
fn rejected(mutate: impl FnOnce(&mut Value), expected: &str) {
    let mut catalog = catalog();
    let mut raw: Value =
        serde_json::from_str(DESCRIPTIONS_DATA).expect("descriptions data is JSON");
    mutate(&mut raw);
    let json = serde_json::to_string(&raw).expect("reserializes");
    let error = catalog
        .load_descriptions_unverified(&json)
        .expect_err("mutated descriptions must be rejected");
    assert!(
        format!("{error}").contains(expected),
        "expected '{expected}' in '{error}'"
    );
}

#[test]
fn digest_mismatch_fails_verified_load() {
    let mut catalog = catalog();
    let tampered = DESCRIPTIONS_DATA.replacen("Creates hud text", "Creates hud text!", 1);
    assert_ne!(tampered, DESCRIPTIONS_DATA);
    let error = catalog
        .load_descriptions(&tampered)
        .expect_err("a tampered file must fail the declared digest");
    assert!(
        format!("{error}").contains("digest mismatch"),
        "unexpected error: {error}"
    );
}

#[test]
fn gap_naming_no_catalog_entry_fails() {
    rejected(
        |raw| {
            raw["gaps"]["en-US"]
                .as_array_mut()
                .unwrap()
                .push(Value::String("noSuchEntry".to_string()));
        },
        "names no catalog action or value",
    );
}

#[test]
fn gap_and_text_is_rejected() {
    rejected(
        |raw| {
            raw["gaps"]["en-US"]
                .as_array_mut()
                .unwrap()
                .push(Value::String("createHudText".to_string()));
        },
        "must be text or a gap, not both or neither",
    );
}

#[test]
fn unshipped_locale_text_is_rejected() {
    rejected(
        |raw| {
            raw["entries"]["actions"]["createHudText"]["description"]["de-DE"] =
                Value::String("Erstellt HUD-Text.".to_string());
        },
        "which the dataset does not ship",
    );
}

#[test]
fn wrong_param_arity_is_rejected() {
    rejected(
        |raw| {
            raw["entries"]["actions"]["createHudText"]["params"]
                .as_array_mut()
                .unwrap()
                .pop();
        },
        "parameter descriptions for",
    );
}

#[test]
fn entry_with_no_text_is_rejected() {
    rejected(
        |raw| {
            raw["entries"]["values"]["countOf"]
                .as_object_mut()
                .unwrap()
                .clear();
        },
        "carries no description text",
    );
}

#[test]
fn pending_locale_ships_neither_text_nor_gaps() {
    rejected(
        |raw| {
            raw["pendingLocales"] = Value::Array(vec![Value::String("ko-KR".to_string())]);
            raw["entries"]["actions"]["createHudText"]["description"]["ko-KR"] =
                Value::String("HUD 텍스트를 만든다.".to_string());
        },
        "which the dataset does not ship",
    );
    rejected(
        |raw| {
            raw["pendingLocales"] = Value::Array(vec![Value::String("ko-KR".to_string())]);
            raw["gaps"]["ko-KR"] = Value::Array(vec![Value::String("compare".to_string())]);
        },
        "which the dataset does not ship",
    );
}

#[test]
fn builtin_attaches_descriptions() {
    let catalog = Catalog::builtin().expect("built-in catalog validates");
    let description = catalog
        .description(Kind::Action, "createHudText", &en())
        .expect("the embedded dataset attaches");
    assert!(!description.text.is_empty());
}
