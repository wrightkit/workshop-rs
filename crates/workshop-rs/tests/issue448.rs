use crate::common;
use workshop_rs::catalog::Locale;
use workshop_rs::parser;
use workshop_rs::settings::SettingsNode;

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

fn hero_setting(hero: &str, member: &str) -> String {
    format!(
        "settings {{\n    heroes {{\n        General {{\n            {hero} {{\n                {member}\n            }}\n        }}\n    }}\n}}"
    )
}

fn leaf_names(children: &[SettingsNode]) -> Vec<&str> {
    let mut names = Vec::new();
    for node in children {
        match node {
            SettingsNode::Group { name, children, .. } => {
                names.push(name.as_str());
                names.extend(leaf_names(children));
            }
            SettingsNode::Workshop { children, .. } => {
                names.extend(leaf_names(children));
            }
            SettingsNode::Number { name, .. }
            | SettingsNode::Bool { name, .. }
            | SettingsNode::Flag { name, .. }
            | SettingsNode::String { name, .. }
            | SettingsNode::List { name, .. }
            | SettingsNode::Raw { name, .. }
            | SettingsNode::RawValue { name, .. }
            | SettingsNode::RawGroup { name, .. } => names.push(name.as_str()),
        }
    }
    names
}

#[test]
fn untranslated_markup_hero_names_parse() {
    let tr = common::locale("tr-TR");
    for hero in ["<en>Ana<en>", "<en>Soldier: 76<en>", "<en>Soldier : 76<en>"] {
        parse_in(&hero_setting(hero, "Ability 1: On"), &tr);
    }
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
fn settings_labels_containing_colons_parse() {
    let ru = common::locale("ru-RU");
    for label in [
        "Время восстановления: Удар щитом",
        "Удар щитом: сила отбрасывания",
    ] {
        let program = parse_in(&hero_setting("Brigitte", &format!("{label}: 100%")), &ru);
        let settings = program.settings.expect("settings");
        let names = leaf_names(&settings.children);
        assert!(
            names.iter().all(|name| *name != "Удар щитом"),
            "the ability label must not be truncated at the inner colon: {names:?}"
        );
    }
    parse_in(
        &hero_setting("Brigitte", "Shield Bash Cooldown Time: 100%"),
        &common::en(),
    );
}

#[test]
fn ability_display_name_binds_the_enable_key() {
    let en = common::en();
    let program = parse_in(&hero_setting("Brigitte", "Shield Bash: On"), &en);
    let settings = program.settings.expect("settings");
    assert!(
        leaf_names(&settings.children).contains(&"enableAbility3"),
        "expected enableAbility3, got {:?}",
        leaf_names(&settings.children)
    );
}

#[test]
fn localized_ability_templates_resolve() {
    let pt = common::locale("pt-BR");
    parse_in(
        &hero_setting("Ana", "Tempo de recarga de Dardos Tranquilizantes: 100%"),
        &pt,
    );
    parse_in(&hero_setting("Ana", "Habilidade 1: On"), &pt);
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

#[test]
fn dotted_settings_booleans_parse() {
    for (locale, boolean) in [("pl-PL", "Wł."), ("ru-RU", "Вкл."), ("ko-KR", "켜기")] {
        let src = format!("settings {{\n    lobby {{\n        Map Rotation: {boolean}\n    }}\n}}");
        parse_in(&src, &common::locale(locale));
    }
}

#[test]
fn digit_and_spaced_team_names_parse() {
    for (locale, team) in [("tr-TR", "1. Takım"), ("ko-KR", "1 팀")] {
        let src = format!(
            "settings {{\n    heroes {{\n        {team} {{\n            Ana {{\n                Ability 1: On\n            }}\n        }}\n    }}\n}}"
        );
        parse_in(&src, &common::locale(locale));
    }
}

#[test]
fn settings_name_errors_carry_a_span() {
    let catalog = common::catalog();
    let error = parser::parse_with_context(
        "settings {\n    heroes {\n        Bogus Team {\n            Ana {\n                Ability 1: On\n            }\n        }\n    }\n}",
        &catalog,
        &common::en(),
        &catalog,
    )
    .expect_err("an undeclared team name must be rejected");
    let span = match &error {
        workshop_rs::WorkshopError::Unknown { span, .. }
        | workshop_rs::WorkshopError::UnknownWithCandidates { span, .. } => span,
        _ => panic!("expected an unknown-spelling diagnostic, got {error}"),
    };
    let span = span.expect("the rejected spelling carries a span");
    assert_eq!(span.start.line, 3);
}

#[test]
fn localized_disabled_mode_prefixes_parse() {
    for (locale, source) in [
        ("tr-TR", "Devre Dışı Assault"),
        ("es-ES", "No permitir Assault"),
        ("ru-RU", "Откл. Assault"),
        ("en-US", "Disabled Assault"),
    ] {
        let src = format!(
            "settings {{\n    modes {{\n        {source} {{\n            enabled maps {{\n                Eichenwalde\n            }}\n        }}\n    }}\n}}"
        );
        parse_in(&src, &common::locale(locale));
    }
}
