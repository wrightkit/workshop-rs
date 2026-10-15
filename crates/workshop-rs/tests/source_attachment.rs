//! Regression coverage for wrightkit/workshop-rs#462 (parent #454): attaching
//! authored source to a registered file must not disturb provenance recorded
//! by a provider map — whatever order the file texts arrive in, and wherever
//! the provider's spans resolve. The two defects this guards:
//!
//! - #410: attaching source left every recorded span to be bounds-checked
//!   against the retained text, so provider spans outside authored extents —
//!   macro expansion attributes expanded tokens to the invocation site
//!   (wrightkit/wright#583) — failed the whole load with `invalid-span`.
//! - #418: the #410 audit then judged every file's spans against the newly
//!   attached document, clearing earlier files' provenance under the
//!   documented sequential-attach pattern (wrightkit/wright#602).

mod common;

use crate::common::{catalog, en};
use workshop_rs::source::{FileId, Position, Span};
use workshop_rs::{MappedText, Program, SourceMap, parser};

/// The emitted Workshop text the provider maps below were authored against:
/// one global and one player variable, one subroutine, an "invocation" rule
/// carrying a condition and two actions, and an "expansion" rule bound to
/// the subroutine — as a macro invocation would produce.
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

/// `late.opy` is registered in the file table but its text never arrives —
/// the provider only retains sources it owns mappings for.
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

/// Build a provider-mapped program: parse the emitted Workshop text, rebind
/// the authored file table and spans a source-language provider recorded,
/// and retain no source yet — `set_file_source` attaches each file's text
/// afterwards in whatever order the provider walks its inputs.
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

/// Attaching each file's text in sequence is the documented consumer pattern
/// for a provider-mapped program (wrightkit/wright#602). Every recorded span
/// must survive every other file's attach: a span cannot be judged against a
/// text that is not its own, so attaching sources in any order leaves every
/// file's provenance intact. Under the #418 audit each attach also cleared
/// the other files' spans, which this catches on the second attach.
#[test]
fn attaching_sources_in_any_order_keeps_every_files_provenance() {
    const FILES: [&str; 3] = ["src/main.opy", "src/hud.opy", "src/generated.opy"];
    const AUTHORED: [&str; 3] = [MAIN_OPY, HUD_OPY, GENERATED_OPY];
    // Provenance spread across all three files and every mapped slot kind:
    // declaration and name spans, rule/name/event-name spans, condition and
    // action spans with identifiers, and nested value children — including
    // children attributed to a different file than their node.
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

/// A provider span that does not resolve inside the attached authored text
/// must neither fail attachment nor be pruned by another file's attach. The
/// expansion rule's action records a span at an invocation-site offset past
/// `generated.opy`'s text — macro expansion shifts expanded tokens onto the
/// invocation site, so a recorded extent can run outside the authored member
/// (wrightkit/wright#583). It stays recorded while other files attach, and
/// only `generated.opy`'s own audit judges it stale: the node then keeps no
/// span rather than a wrong one, so the load still completes (#410). Spans
/// in `late.opy`, whose text never arrives, are never judged at all.
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
            // The invoked name stays attributed to the call site in main.opy.
            {"node": "action", "rule": 0, "action": 1, "span": wire(0, 7, 5, 25), "identifier_span": wire(2, 2, 5, 9)},
            {"node": "rule", "rule": 1, "span": wire(1, 3, 1, 16), "name_span": wire(1, 3, 6, 15), "event_name_span": wire(2, 4, 22, 26)},
            {"node": "action", "rule": 1, "action": 0, "identifier_span": wire(0, 7, 19, 23)},
        ]);
        if expansion_span {
            // The expanded action's own span is shifted onto the invocation
            // site past generated.opy's authored extent: it does not resolve
            // in any attached text.
            entries[8]["span"] = wire(1, 40, 5, 40);
        }
        entries
    };
    let stale = at(GENERATED, 40, 5, 40);

    for order in [[0usize, 1usize], [1, 0]] {
        let mut program = provider_program(&FILES, spans(true));
        let full_map = SourceMap::extract(&program);
        // The same map without the expansion span: what the extraction must
        // look like once generated.opy's own audit has pruned exactly that
        // span and nothing else.
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
                // The span's own audit: it does not resolve inside
                // generated.opy's text, so it is pruned — and nothing else.
                assert_eq!(program.action_span(1, 0), None);
                assert_eq!(SourceMap::extract(&program), pruned_map);
            } else {
                // A foreign attach cannot judge the expansion span: it lies
                // outside this attached text and stays recorded. Under the
                // #410 bounds-check it failed the load here; under the #418
                // audit it was cleared here.
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
            // late.opy's text never arrives, so its spans are never judged:
            // they survive every attach unchanged.
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
