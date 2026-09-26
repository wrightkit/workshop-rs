//! Constant wrapper and settings-boolean forms (docs/wrapper-forms.md).

use std::collections::BTreeSet;

use workshop_rs::catalog::{Catalog, Locale};
use workshop_rs::emitter;
use workshop_rs::parser;

use super::common;

fn round_trip(text: &str, locale: &str) -> String {
    let catalog = Catalog::builtin().unwrap();
    let locale = Locale::new(locale);
    let program = parser::parse_wir_with_context(text, &catalog, &locale, &catalog)
        .unwrap_or_else(|error| panic!("parse failed: {error:?}\n{text}"));
    emitter::emit_wir(&program, &catalog, &locale)
        .unwrap_or_else(|error| panic!("emit failed: {error:?}"))
}

fn assert_lines_preserved(actions: &[&str], locale: &str, event: &str, section: &str) {
    let body: String = actions
        .iter()
        .map(|line| ["        ", line, "\n"].concat())
        .collect();
    let text = format!(
        "{section} (\"t\") {{\n    {event} {{\n        {}\n    }}\n    actions {{\n{body}    }}\n}}\n",
        if locale == "zh-CN" {
            "持续 - 全局;"
        } else {
            "Ongoing - Global;"
        }
    );
    let emitted = round_trip(&text, locale);
    for line in actions {
        assert!(
            emitted.contains(&format!("        {line}\n")),
            "{locale}: `{line}` was not preserved:\n{emitted}"
        );
    }
}

#[test]
fn overpy_constant_forms_round_trip_in_en_us() {
    assert_lines_preserved(
        &[
            "Set Global Variable(A, All Players(Team 1));",
            "Set Global Variable(A, Players On Hero(Hero(Mercy), Team 1));",
            "Set Global Variable(A, Array(Hero(D.Va), Hero(Reinhardt), Hero(Winston)));",
            "Set Global Variable(A, Array(Team 1, Team 2));",
            "Set Global Variable(A, Array(Color(Red), Color(Blue)));",
            "Set Global Variable(A, Array(Map(Hanamura)));",
            "Set Global Variable(A, Array(Button(Reload), Button(Jump)));",
            "Set Global Variable(A, Compare(Global.A, ==, Team 1));",
            "Small Message(Event Player, Custom String(\"x{0}\", Button(Reload)));",
        ],
        "en-US",
        "event",
        "rule",
    );
}

#[test]
fn overpy_constant_forms_round_trip_in_zh_cn() {
    assert_lines_preserved(
        &[
            "设置全局变量(A, 所有玩家(队伍1));",
            "设置全局变量(A, 选择英雄的玩家(英雄(天使), 队伍1));",
            "设置全局变量(A, 数组(英雄(D.Va), 英雄(莱因哈特), 英雄(温斯顿)));",
            "设置全局变量(A, 数组(队伍1, 队伍2));",
            "设置全局变量(A, 数组(颜色(红色), 颜色(蓝色)));",
            "设置全局变量(A, 数组(地图(花村)));",
            "设置全局变量(A, 数组(按钮(装填), 按钮(跳跃)));",
            "设置全局变量(A, 比较(全局.A, ==, 队伍1));",
            "小字体信息(事件玩家, 自定义字符串(\"x{0}\", 按钮(装填)));",
        ],
        "zh-CN",
        "事件",
        "规则",
    );
}

#[test]
fn a_team_wrapper_in_the_input_is_written_bare_in_every_locale() {
    let text = "rule (\"t\") {\n    event {\n        Ongoing - Global;\n    }\n    actions {\n        Set Global Variable(A, All Players(Team(Team 1)));\n    }\n}\n";
    let catalog = Catalog::builtin().unwrap();
    let program =
        parser::parse_wir_with_context(text, &catalog, &Locale::new("en-US"), &catalog).unwrap();
    for (locale, expected) in [
        ("en-US", "All Players(Team 1)"),
        ("zh-CN", "所有玩家(队伍1)"),
    ] {
        let emitted = emitter::emit_wir(&program, &catalog, &Locale::new(locale)).unwrap();
        assert!(emitted.contains(expected), "{locale}:\n{emitted}");
    }
}

#[test]
fn yes_no_settings_are_written_with_the_client_word() {
    for (locale, text) in [
        (
            "en-US",
            "settings {\n    lobby {\n        Allow Players Who Are In Queue: No\n        Swap Teams After Match: Yes\n    }\n}\n",
        ),
        (
            "zh-CN",
            "设置 {\n    大厅 {\n        队列中的玩家可以加入: 否\n        比赛结束后转换队伍: 是\n    }\n}\n",
        ),
    ] {
        assert_eq!(round_trip(text, locale).trim_end(), text.trim_end());
    }
}

fn wrapper_counts(text: &str) -> Vec<usize> {
    ["Team", "队伍", "Hero", "英雄", "Button", "按钮"]
        .iter()
        .map(|name| {
            let pattern = format!(r"(^|[^\p{{L}}\p{{N}}_]){name}\(");
            regex::Regex::new(&pattern).unwrap().find_iter(text).count()
        })
        .collect()
}

fn tokens(text: &str) -> BTreeSet<String> {
    regex::Regex::new(r"[\p{L}\p{N}_.]+")
        .unwrap()
        .find_iter(text)
        .map(|token| token.as_str().to_string())
        .collect()
}

#[test]
fn overpy_generated_projects_keep_wrapper_counts_and_add_no_tokens() {
    for case in common::cases()
        .iter()
        .filter(|case| matches!(case.id, "ai-pve-zh-CN" | "bastion-en-US"))
    {
        let (source, locale) = common::source(case);
        let emitted = round_trip(&source, locale.as_str());
        assert_eq!(
            wrapper_counts(&source),
            wrapper_counts(&emitted),
            "{} wrapper call counts changed",
            case.id
        );
        let added: Vec<_> = tokens(&emitted)
            .difference(&tokens(&source))
            .cloned()
            .collect();
        assert!(
            added.is_empty(),
            "{} emission added tokens: {added:?}",
            case.id
        );
    }
}

#[test]
fn zh_cn_action_spellings_follow_the_pinned_overpy_emission() {
    let spellings = [
        "消除所有图标;",
        "消除所有地图文本;",
        "消除所有效果;",
        "消除所有HUD文本;",
        "关闭游戏预设通告模式;",
        "关闭游戏预设完成条件;",
        "关闭游戏预设音乐模式;",
        "关闭游戏预设计分模式;",
        "开启游戏预设音乐模式;",
        "比赛时间继续;",
    ];
    let body: String = spellings
        .iter()
        .map(|line| ["        ", line, "\n"].concat())
        .collect();
    let text = format!(
        "规则 (\"t\") {{\n    事件 {{\n        持续 - 全局;\n    }}\n    动作 {{\n{body}    }}\n}}\n"
    );
    let emitted = round_trip(&text, "zh-CN");
    for line in spellings {
        assert!(
            emitted.contains(&format!("        {line}\n")),
            "`{line}` was not preserved:\n{emitted}"
        );
    }
}
