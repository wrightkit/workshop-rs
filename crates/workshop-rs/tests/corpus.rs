//! Corpus-backed zh-CN conversion evidence.

use workshop_rs::catalog::{Catalog, Locale};
use workshop_rs::convert::{self, ConvertOptions};

fn catalog() -> Catalog {
    Catalog::builtin().expect("catalog validates")
}

fn en() -> Locale {
    Locale::new("en-US")
}

fn zh() -> Locale {
    Locale::new("zh-CN")
}

const REPRESENTATIVE: &str = "variables {
    global:
        0: result
}

rule (\"corpus\") {
    event {
        Ongoing - Global;
    }
    actions {
        Set Global Variable(result, Add(1, 2));
        Wait(1, Ignore Condition);
    }
}
";

#[test]
fn manifest_pins_the_export_and_exact_match_coverage() {
    let manifest: serde_json::Value =
        serde_json::from_str(include_str!("../../../tools/corpus/zh-cn-corpus.json"))
            .expect("generated corpus manifest is valid JSON");
    assert_eq!(manifest["locale"], "zh-CN");
    assert_eq!(
        manifest["source"]["commit"],
        "d854bf01fc7bbf3b2169f67408c07a8da8989ad6"
    );
    assert_eq!(manifest["coverage"]["total"]["matched"], 327);
    assert_eq!(manifest["coverage"]["total"]["total"], 344);
    assert_eq!(manifest["matches"].as_array().unwrap().len(), 327);
    assert_eq!(manifest["excluded"].as_array().unwrap().len(), 17);
}

#[test]
fn representative_corpus_converts_in_both_directions() {
    let catalog = catalog();
    let to_zh = convert::convert(
        REPRESENTATIVE,
        &catalog,
        &en(),
        &zh(),
        &ConvertOptions::default(),
    )
    .expect("en-US corpus converts to zh-CN");
    assert!(to_zh.fallback_ids.is_empty());
    assert!(to_zh.text.contains("持续 - 全局"), "{}", to_zh.text);
    assert!(to_zh.text.contains("设置全局变量"), "{}", to_zh.text);
    assert!(to_zh.text.contains("加(1, 2)"), "{}", to_zh.text);
    assert!(to_zh.text.contains("等待(1, 无视条件)"), "{}", to_zh.text);

    let back_to_en = convert::convert(
        &to_zh.text,
        &catalog,
        &zh(),
        &en(),
        &ConvertOptions::default(),
    )
    .expect("zh-CN corpus converts back to en-US");
    assert_eq!(back_to_en.fallback_ids, Vec::<String>::new());
    assert_eq!(back_to_en.text.trim_end(), REPRESENTATIVE.trim_end());
}
