mod common;

use crate::common::{catalog, en};
use workshop_rs::settings::SettingsNode;
use workshop_rs::source::{FileId, Position, Span};
use workshop_rs::{MappedText, Program, SourceMap, emitter, parser};

const WORKSHOP_TEXT: &str = r#"variables {
    global:
        0: counter
    player:
        1: score
}

subroutines {
    0: tick
}

rule ("invocation") {
    event { Ongoing - Global; }
    conditions {
        Global.counter > 0;
    }
    actions {
        Set Global Variable(counter, Add(Global.counter, Event Player.score));
        Call Subroutine(tick);
    }
}

rule ("expansion") {
    event {
        Subroutine;
        tick;
    }
    actions {
        Wait(1);
    }
}
"#;

const MAIN_OPY: &str = "// main.opy\nglobalvar 0: counter\nplayervar 1: score\nrule invocation {\n    Global.counter > 0\n    counter = Add(counter, EventPlayer.score)\n    CallSubroutine(tick)\n}\n";

const HUD_OPY: &str =
    "// hud.opy\nplayervar 1: score\n#!define SUM Add(counter, EventPlayer.score)\n";

const GENERATED_OPY: &str = "// generated.opy — macro expansion body\ndef tick\nrule expansion {\n    event Subroutine tick\n    Wait(1)\n}\n";

const LATE_OPY: &str = "// late.opy — the provider never attaches this file's text\ndef tick\nrule expansion {\n    event Subroutine tick\n}\n";

fn wire(file: usize, line: u32, start: u32, end: u32) -> serde_json::Value {
    serde_json::json!({
        "file": file,
        "start": {"line": line, "column": start},
        "end": {"line": line, "column": end},
    })
}

fn at(file: usize, line: u32, start: u32, end: u32) -> Span {
    Span::new(
        FileId::from_index(file),
        Position::new(line, start),
        Position::new(line, end),
    )
}

fn provider_program(files: &[&str], spans: serde_json::Value) -> Program {
    let artifact = serde_json::json!({
        "format": "workshop-rs/mapped-text-v1",
        "text": WORKSHOP_TEXT,
        "files": files
            .iter()
            .map(|path| serde_json::json!({"path": path}))
            .collect::<Vec<_>>(),
        "shape": {
            "global_variables": 1,
            "player_variables": 1,
            "subroutines": 1,
            "rules": [
                {"conditions": 1, "actions": 2},
                {"conditions": 0, "actions": 1},
            ],
        },
        "spans": spans,
    });
    let decoded = MappedText::from_json(&artifact.to_string()).expect("decodes");
    let mut program = parser::parse(&decoded.text, &catalog(), &en()).expect("parses");
    decoded.map.apply(&mut program).expect("map applies");
    program
}

#[test]
fn apply_drops_settings_spans_the_map_does_not_carry() {
    let catalog = catalog();
    let en = en();
    let source = "settings { lobby { Match Voice Chat: Enabled } }\n\nrule (\"one\") {\n    event { Ongoing - Global; }\n    actions { Wait(1, Ignore Condition); }\n}\n";
    let program = parser::parse(source, &catalog, &en).expect("parses");
    let artifact = MappedText::new(
        emitter::emit(&program, &catalog, &en).expect("emits"),
        SourceMap::extract(&program),
    );
    let mut reparsed = parser::parse(&artifact.text, &catalog, &en).expect("reparses");
    let settings = reparsed.settings.as_ref().expect("settings");
    assert!(settings.span.is_some());
    assert!(settings.children.iter().any(|node| node.span().is_some()));

    artifact.map.apply(&mut reparsed).expect("applies");

    fn assert_unmapped(nodes: &[SettingsNode]) {
        for node in nodes {
            assert!(node.span().is_none(), "{} kept a span", node.name());
            match node {
                SettingsNode::Workshop { children, .. }
                | SettingsNode::Group { children, .. }
                | SettingsNode::RawGroup { children, .. } => assert_unmapped(children),
                SettingsNode::List { elements, .. } => {
                    for element in elements {
                        assert!(element.span.is_none());
                    }
                }
                _ => {}
            }
        }
    }
    let settings = reparsed.settings.as_ref().expect("settings survive");
    assert!(settings.span.is_none());
    assert_unmapped(&settings.children);
}

#[test]
fn attaching_sources_in_any_order_keeps_every_files_provenance() {
    const FILES: [&str; 3] = ["src/main.opy", "src/hud.opy", "src/generated.opy"];
    const AUTHORED: [&str; 3] = [MAIN_OPY, HUD_OPY, GENERATED_OPY];
    let spans = serde_json::json!([
        {"node": "global_variable", "index": 0, "span": wire(0, 2, 1, 21), "name_span": wire(0, 2, 14, 21)},
        {"node": "player_variable", "index": 0, "span": wire(1, 2, 1, 19), "name_span": wire(1, 2, 14, 19)},
        {"node": "subroutine", "index": 0, "span": wire(2, 2, 1, 9), "name_span": wire(2, 2, 5, 9)},
        {"node": "rule", "rule": 0, "span": wire(0, 4, 1, 18), "name_span": wire(0, 4, 6, 16)},
        {"node": "condition", "rule": 0, "condition": 0, "span": wire(0, 5, 5, 23),
            "children": [{"identifier_span": wire(0, 5, 12, 19)}, {"span": wire(0, 5, 22, 23)}]},
        {"node": "action", "rule": 0, "action": 0, "span": wire(0, 6, 5, 46), "identifier_span": wire(0, 6, 5, 12)},
        {"node": "action_argument", "rule": 0, "action": 0, "argument": 0, "span": wire(0, 6, 15, 46),
            "children": [{"identifier_span": wire(1, 3, 18, 25)}, {"identifier_span": wire(1, 3, 38, 43)}]},
        {"node": "action", "rule": 0, "action": 1, "span": wire(0, 7, 5, 25), "identifier_span": wire(2, 4, 22, 26)},
        {"node": "rule", "rule": 1, "span": wire(2, 3, 1, 16), "name_span": wire(2, 3, 6, 15), "event_name_span": wire(2, 4, 22, 26)},
        {"node": "action", "rule": 1, "action": 0, "span": wire(2, 5, 5, 12)},
    ]);

    for order in [
        [0usize, 1, 2],
        [2, 1, 0],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
    ] {
        let mut program = provider_program(&FILES, spans.clone());
        let mapped = SourceMap::extract(&program);
        for &file in &order {
            assert!(
                program.set_file_source(FileId::from_index(file), AUTHORED[file]),
                "order {order:?}: attaching {} must not fail",
                FILES[file]
            );
            assert_eq!(
                SourceMap::extract(&program),
                mapped,
                "order {order:?}: attaching {} dropped provenance",
                FILES[file]
            );
        }
        program
            .validate()
            .unwrap_or_else(|error| panic!("order {order:?}: {error}"));
    }
}

#[test]
fn spans_outside_the_attached_text_survive_until_their_own_files_audit() {
    const FILES: [&str; 3] = ["src/main.opy", "src/generated.opy", "src/late.opy"];
    const AUTHORED: [&str; 3] = [MAIN_OPY, GENERATED_OPY, LATE_OPY];
    const GENERATED: usize = 1;
    let spans = |expansion_span: bool| {
        let mut entries = serde_json::json!([
            {"node": "global_variable", "index": 0, "span": wire(0, 2, 1, 21), "name_span": wire(0, 2, 14, 21)},
            {"node": "player_variable", "index": 0, "span": wire(0, 3, 1, 19), "name_span": wire(0, 3, 14, 19)},
            {"node": "subroutine", "index": 0, "span": wire(1, 2, 1, 9), "name_span": wire(1, 2, 5, 9)},
            {"node": "rule", "rule": 0, "span": wire(0, 4, 1, 18), "name_span": wire(0, 4, 6, 16)},
            {"node": "condition", "rule": 0, "condition": 0, "span": wire(0, 5, 5, 23),
                "children": [{"identifier_span": wire(0, 5, 12, 19)}, {"span": wire(0, 5, 22, 23)}]},
            {"node": "action", "rule": 0, "action": 0, "span": wire(0, 6, 5, 46), "identifier_span": wire(0, 6, 5, 12)},
            {"node": "action", "rule": 0, "action": 1, "span": wire(0, 7, 5, 25), "identifier_span": wire(2, 2, 5, 9)},
            {"node": "rule", "rule": 1, "span": wire(1, 3, 1, 16), "name_span": wire(1, 3, 6, 15), "event_name_span": wire(2, 4, 22, 26)},
            {"node": "action", "rule": 1, "action": 0, "identifier_span": wire(0, 7, 19, 23)},
        ]);
        if expansion_span {
            entries[8]["span"] = wire(1, 40, 5, 40);
        }
        entries
    };
    let stale = at(GENERATED, 40, 5, 40);

    for order in [[0usize, 1usize], [1, 0]] {
        let mut program = provider_program(&FILES, spans(true));
        let full_map = SourceMap::extract(&program);
        let pruned_map = SourceMap::extract(&provider_program(&FILES, spans(false)));
        assert_eq!(program.action_span(1, 0), Some(stale));

        let mut generated_attached = false;
        for &file in &order {
            assert!(
                program.set_file_source(FileId::from_index(file), AUTHORED[file]),
                "order {order:?}: attaching {} must not fail on a span outside its text",
                FILES[file]
            );
            if file == GENERATED {
                generated_attached = true;
                assert_eq!(program.action_span(1, 0), None);
                assert_eq!(SourceMap::extract(&program), pruned_map);
            } else {
                if !generated_attached {
                    assert_eq!(program.action_span(1, 0), Some(stale));
                }
                let expected = if generated_attached {
                    &pruned_map
                } else {
                    &full_map
                };
                assert_eq!(
                    SourceMap::extract(&program),
                    *expected,
                    "order {order:?}: attaching {} dropped provenance",
                    FILES[file]
                );
            }
            assert_eq!(
                program.action_identifier_span(0, 1),
                Some(at(2, 2, 5, 9)),
                "order {order:?}: late.opy callee span pruned"
            );
            assert_eq!(
                program.rule_event_name_span(1),
                Some(at(2, 4, 22, 26)),
                "order {order:?}: late.opy event-name span pruned"
            );
        }
        program.validate().unwrap_or_else(|error| {
            panic!("order {order:?}: load fails on a pruned expansion span: {error}")
        });
    }
}
