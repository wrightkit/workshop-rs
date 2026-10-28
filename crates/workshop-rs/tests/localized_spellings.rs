use crate::common;
use workshop_rs::catalog::Locale;
use workshop_rs::parser;

fn parse_in(source: &str, locale: &Locale) -> workshop_rs::Program {
    let catalog = common::catalog();
    parser::parse_with_context(source, &catalog, locale, &catalog)
        .unwrap_or_else(|error| panic!("{locale} source must parse: {error}\n{source}"))
}

fn value_rule(action: &str) -> String {
    format!(
        "variables {{\n    global:\n        0: probe\n    player:\n        0: probe\n}}\n\nrule (\"probe\") {{\n    event {{\n        Ongoing - Global;\n    }}\n    actions {{\n        {action}\n    }}\n}}\n"
    )
}

#[test]
fn hero_value_call_with_enum_constructor_argument_parses() {
    for (locale, source) in [
        ("en-US", "Hero(Hero(D.Va))"),
        ("en-US", "Hero(D.Va)"),
        ("zh-CN", "英雄(英雄(D.Va))"),
        ("zh-CN", "英雄(D.Va)"),
        ("zh-CN", "英雄(Hero(D.Va))"),
        ("fr-FR", "Personnage(D.Va)"),
    ] {
        parse_in(
            &value_rule(&format!("Set Global Variable(probe, {source});")),
            &common::locale(locale),
        );
    }
}

#[test]
fn punctuated_enum_member_spellings_parse() {
    for (locale, expr) in [
        ("tr-TR", "Button(1. Kabiliyet)"),
        ("fr-FR", "Map(Observatoire : Gibraltar)"),
        ("pt-BR", "Spray - Baixo"),
        ("ja-JP", "対ワールド:"),
    ] {
        parse_in(
            &value_rule(&format!("Set Global Variable(probe, {expr});")),
            &common::locale(locale),
        );
    }
}

#[test]
fn punctuated_action_and_operator_spellings_parse() {
    for (locale, action) in [
        ("fr-FR", "Forcer le nom de l’I.A.(Event Player)"),
        ("es-ES", "Resurrección:(Event Player)"),
        ("tr-TR", "Bekle:(Ignore Condition)"),
        (
            "fr-FR",
            "Modify Global Variable(probe, Élévation à une puissance, 2)",
        ),
        ("th-TH", "Modify Global Variable(probe, เพิ่มเข้าในอาร์เรย์, 0)"),
    ] {
        parse_in(&value_rule(&format!("{action};")), &common::locale(locale));
    }
}

#[test]
fn markup_inside_member_spellings_parses() {
    for (locale, expr) in [
        ("de-DE", "Hero(Meis Schneeball<hy>schlacht)"),
        ("ja-JP", "Map(ジャンケン<br>シュタインの<br>復讐)"),
    ] {
        parse_in(
            &value_rule(&format!("Set Global Variable(probe, {expr});")),
            &common::locale(locale),
        );
    }
}

#[test]
fn locale_punctuation_inside_spellings_parses() {
    for (locale, expr) in [
        ("es-ES", "Haz de enganche de ¡Alto! de Orisa"),
        ("ja-JP", "「FALSE」の場合中止"),
        ("ru-RU", "Map(Лунная колония «Горизонт»)"),
    ] {
        parse_in(
            &value_rule(&format!("Set Global Variable(probe, {expr});")),
            &common::locale(locale),
        );
    }
}
