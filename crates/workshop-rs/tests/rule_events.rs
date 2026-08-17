use workshop_rs::catalog::{Catalog, Kind, Locale};
use workshop_rs::wir::{self, EventTarget, EventTeam, PlayerEventKind};
use workshop_rs::{emitter, parser, roundtrip, validate};

const RULE_EVENTS: &str = r#"
rule ("each player filtered") {
    event {
        Ongoing - Each Player;
        Team 1;
        Slot 3;
    }
    actions {
        Disable Inspector Recording;
    }
}

rule ("dealt damage") {
    event {
        Player Dealt Damage;
        All;
        Ana;
    }
    actions {
        Disable Inspector Recording;
    }
}

rule ("dealt final blow") {
    event {
        Player Dealt Final Blow;
        Team 2;
        All;
    }
    actions {
        Disable Inspector Recording;
    }
}

rule ("dealt healing") {
    event {
        Player Dealt Healing;
        All;
        Slot 0;
    }
    actions {
        Disable Inspector Recording;
    }
}

rule ("died") {
    event {
        Player Died;
        Team 1;
        All;
    }
    actions {
        Disable Inspector Recording;
    }
}

rule ("earned elimination") {
    event {
        Player Earned Elimination;
        All;
        Slot 11;
    }
    actions {
        Disable Inspector Recording;
    }
}

rule ("joined") {
    event {
        Player Joined Match;
        Team 2;
        Ana;
    }
    actions {
        Disable Inspector Recording;
    }
}

rule ("left") {
    event {
        Player Left Match;
        All;
        All;
    }
    actions {
        Disable Inspector Recording;
    }
}

rule ("received healing") {
    event {
        Player Received Healing;
        Team 1;
        Slot 4;
    }
    actions {
        Disable Inspector Recording;
    }
}

rule ("took damage") {
    event {
        Player Took Damage;
        All;
        All;
    }
    actions {
        Disable Inspector Recording;
    }
}
"#;

fn catalog() -> Catalog {
    Catalog::builtin().expect("built-in catalog")
}

fn en() -> Locale {
    Locale::new("en-US")
}

fn zh() -> Locale {
    Locale::new("zh-CN")
}

#[test]
fn all_player_events_parse_validate_emit_and_round_trip() {
    let catalog = catalog();
    let program = parser::parse_with_context(RULE_EVENTS, &catalog, &en(), &catalog)
        .expect("all canonical player events parse");
    program.validate().expect("event filters validate");
    validate::validate_canonical_ids(&program, &catalog).expect("event catalog ids validate");

    let player_kinds: Vec<_> = program
        .rules
        .iter()
        .filter_map(|rule| match &rule.event {
            wir::Event::Player { kind, .. } => Some(*kind),
            _ => None,
        })
        .collect();
    assert_eq!(
        player_kinds,
        vec![
            PlayerEventKind::DealtDamage,
            PlayerEventKind::DealtFinalBlow,
            PlayerEventKind::DealtHealing,
            PlayerEventKind::Died,
            PlayerEventKind::EarnedElimination,
            PlayerEventKind::Joined,
            PlayerEventKind::Left,
            PlayerEventKind::ReceivedHealing,
            PlayerEventKind::TookDamage,
        ]
    );
    assert!(matches!(
        program.rules.get(wir::RuleId::from_index(0)).unwrap().event,
        wir::Event::EachPlayerWithFilters {
            team: EventTeam::Team1,
            target: EventTarget::Slot(3),
        }
    ));

    let en_text = emitter::emit(&program, &catalog, &en()).expect("en-US event emission");
    let reparsed_en = parser::parse_with_context(&en_text, &catalog, &en(), &catalog)
        .expect("en-US event reparse");
    assert!(roundtrip::equivalent(&program, &reparsed_en));

    let zh_text = emitter::emit(&program, &catalog, &zh()).expect("zh-CN event emission");
    assert!(zh_text.contains("持续 - 每名玩家"));
    assert!(zh_text.contains("玩家造成伤害"));
    assert!(zh_text.contains("队伍1;"));
    assert!(zh_text.contains("栏位 3;"));
    let reparsed_zh = parser::parse_with_context(&zh_text, &catalog, &zh(), &catalog)
        .expect("zh-CN event reparse");
    assert!(roundtrip::equivalent(&program, &reparsed_zh));
}

#[test]
fn event_catalog_declares_parameter_and_filter_provenance_surface() {
    let catalog = catalog();
    let event = catalog
        .entry(Kind::Event, "playerDied")
        .expect("playerDied catalog entry");
    assert_eq!(event.params, vec!["Team", "Player"]);
    assert_eq!(
        event.param_domains,
        vec![Some("EventTeam".to_string()), None]
    );
    assert_eq!(
        catalog.enum_spelling("EventTeam", &zh(), "ALL"),
        Some("双方")
    );
    assert_eq!(
        catalog.enum_spelling("EventPlayer", &zh(), "SLOT_3"),
        Some("栏位 3")
    );
}

#[test]
fn invalid_event_filter_is_rejected_and_invalid_slot_fails_wir_validation() {
    let catalog = catalog();
    let invalid_text = RULE_EVENTS.replace("Slot 3;", "Unknown Player;");
    let error = parser::parse_with_context(&invalid_text, &catalog, &en(), &catalog)
        .expect_err("unknown event player filter must fail");
    assert!(error.to_string().contains("unknown event player"));

    let mut program = wir::Program::default();
    program
        .files
        .push(workshop_rs::source::SourceFile::new("events.ws"));
    program.rules.push(wir::Rule {
        name: "invalid slot".into(),
        span: None,
        name_span: None,
        disabled: false,
        event: wir::Event::Player {
            kind: PlayerEventKind::Died,
            team: EventTeam::All,
            target: EventTarget::Slot(12),
        },
        conditions: vec![],
        actions: vec![],
    });
    let error = program
        .validate()
        .expect_err("slot 12 is outside Workshop range");
    assert_eq!(error.code(), "invalid-event-slot");
}
