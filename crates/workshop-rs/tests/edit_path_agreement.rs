//! `SettingDefinition::write` and `SettingDefinition::source_edit` are
//! sibling edit paths over the same typed settings occurrence. For the same
//! target and input they must reach the same accept/reject outcome (#458):
//! every check `write` enforces — target applicability, occurrence
//! presence, the typed-leaf gate, supplied-value domain validation, the
//! source-supported value surface, and carrier shape — has to refuse the
//! same inputs before `source_edit` produces a replacement range, so a gate
//! dropped on either side fails here.
//!
//! `source_edit` additionally requires what `write` never needs: a
//! source-backed occurrence and a scalar `key: value` member spelling.
//! Where no scalar value bytes exist (brace-block lists, presence-only
//! flags) or no source location was recorded, it must fail closed without
//! producing a range — pinned below as its own contract, not shared
//! validation. The first gate `write` applies — `operation_id`, a
//! definition with no reviewed canonical identity — is unreachable through
//! the public schema (`validate_catalog` rejects identity-less
//! definitions), so no test input can exercise it here.

mod common;

use workshop_rs::gameplay::{HeroId, hero_ids};
use workshop_rs::parser;
use workshop_rs::settings::{
    SettingDefinition, SettingId, SettingOperationError, SettingSourceEdit, SettingTarget,
    SettingValue, Settings, SettingsNode, TeamId, definitions, definitions_by_id,
};

/// Parse a `settings { ... }` block to the carrier `write` mutates and
/// `source_edit` addresses through `source`.
fn parse_settings(source: &str) -> Settings {
    parser::parse(source, &common::catalog(), &common::en())
        .expect("settings source parses")
        .settings
        .expect("a settings block")
}

fn definition(path: &str) -> SettingDefinition {
    definitions()
        .find(|definition| definition.path() == path)
        .unwrap_or_else(|| panic!("no definition for path {path}"))
}

/// Both sibling paths facing the same input: `write` mutates a clone of
/// the carrier while `source_edit` derives a checked range from `source`
/// against the untouched tree.
fn both_paths(
    definition: &SettingDefinition,
    source: &str,
    settings: &Settings,
    target: &SettingTarget,
    value: SettingValue,
) -> (
    Result<(), SettingOperationError>,
    Result<SettingSourceEdit, SettingOperationError>,
) {
    let written = definition.write(&mut settings.clone(), target, value.clone());
    let edited = definition.source_edit(source, settings, "en-US", target, value);
    (written, edited)
}

/// `write` rejects ⟺ `source_edit` rejects — and because the refusal comes
/// from a shared check the diagnostic is identical on both paths.
#[track_caller]
fn assert_agree_reject(
    definition: &SettingDefinition,
    source: &str,
    settings: &Settings,
    target: &SettingTarget,
    value: SettingValue,
) {
    let (written, edited) = both_paths(definition, source, settings, target, value);
    let write_error = written.expect_err("write must reject this input");
    let edit_error =
        edited.expect_err("source_edit must reject this input and produce no edit range");
    assert_eq!(
        write_error, edit_error,
        "the same check must refuse the same input identically"
    );
}

/// `write` accepts ⟺ `source_edit` accepts — and both land on the same
/// resulting occurrence value: the written carrier reads back `value`, and
/// the applied edit reparses to the same read.
#[track_caller]
fn assert_agree_accept(
    definition: &SettingDefinition,
    source: &str,
    settings: &Settings,
    target: &SettingTarget,
    value: SettingValue,
    old_value_bytes: &str,
) {
    let mut carrier = settings.clone();
    definition
        .write(&mut carrier, target, value.clone())
        .expect("write must accept this input");
    assert_eq!(
        definition
            .read(&carrier, target)
            .expect("read the written value")
            .authored,
        value,
        "write leaves the supplied value in the carrier"
    );

    let edit = definition
        .source_edit(source, settings, "en-US", target, value.clone())
        .expect("source_edit must accept this input");
    assert_eq!(
        &source[edit.range()],
        old_value_bytes,
        "the edit range covers exactly the authored value bytes"
    );
    let edited = edit.apply(source).expect("the edit applies to its source");
    let reparsed = parse_settings(&edited);
    assert_eq!(
        definition
            .read(&reparsed, target)
            .expect("read the edited value")
            .authored,
        value,
        "the applied edit reads back the same value write leaves"
    );
}

fn group(name: &str, children: Vec<SettingsNode>) -> SettingsNode {
    SettingsNode::Group {
        name: name.to_string(),
        children,
        span: None,
    }
}

/// `ensure_write_target`: a target the definition does not apply to is
/// `NotApplicable` on both paths.
#[test]
fn rejects_not_applicable_targets_identically() {
    let settings = parse_settings("settings { main { Description: \"old\" } }");
    let description = definition("main.description");
    let enabled = definition("gamemodes.assault.enabled");
    let heroes = parse_settings("settings { heroes { General { enabled heroes { Genji Ana } } } }");
    let enabled_heroes = definition("heroes.<team>.enabledHeroes");

    for (definition, settings, target, label) in [
        // A global-scope definition does not apply to a mode.
        (
            &description,
            &settings,
            SettingTarget::Mode("skirmish".to_string()),
            "global definition with a mode target",
        ),
        (
            &description,
            &settings,
            SettingTarget::Team(TeamId::new("allTeams")),
            "global definition with a team target",
        ),
        (
            &description,
            &settings,
            SettingTarget::Hero {
                team: None,
                hero: HeroId::from(hero_ids::ANA),
            },
            "global definition with a hero target",
        ),
        // A mode-pinned definition does not apply to other targets.
        (
            &enabled,
            &settings,
            SettingTarget::Mode("skirmish".to_string()),
            "assault definition with a skirmish target",
        ),
        (
            &enabled,
            &settings,
            SettingTarget::Global,
            "mode definition with a global target",
        ),
        (
            &enabled_heroes,
            &heroes,
            SettingTarget::Mode("skirmish".to_string()),
            "team definition with a mode target",
        ),
    ] {
        let (written, edited) = both_paths(
            definition,
            "",
            settings,
            &target,
            SettingValue::Boolean(true),
        );
        let write_error = written.expect_err("{label}: write must reject");
        let edit_error = edited.expect_err("{label}: source_edit must reject");
        assert!(
            matches!(write_error, SettingOperationError::NotApplicable { .. }),
            "{label}: {write_error:?}"
        );
        assert_eq!(
            write_error, edit_error,
            "{label}: the applicability check is shared"
        );
    }
}

/// `ensure_write_target`: a target whose applicability cannot be proven is
/// `ApplicabilityUnknown` on both paths — writes fail closed.
#[test]
fn rejects_unknown_applicability_targets_identically() {
    let settings = parse_settings("settings { main { Description: \"old\" } }");
    let health = definitions()
        .find(|definition| {
            definition.path().ends_with("health%")
                && matches!(
                    definition.target_kind(),
                    workshop_rs::settings::SettingTargetKind::Hero
                )
        })
        .expect("hero health definition");
    let enabled_heroes = definition("heroes.<team>.enabledHeroes");

    for (definition, target, label) in [
        (
            &health,
            SettingTarget::Hero {
                team: None,
                hero: HeroId::new("futureHero"),
            },
            "hero definition with an unprovable hero target",
        ),
        (
            &health,
            SettingTarget::Hero {
                team: Some(TeamId::new("allTeams")),
                hero: HeroId::from(hero_ids::ANA),
            },
            "hero definition with a matching-but-unproven hero target",
        ),
        (
            &enabled_heroes,
            SettingTarget::Hero {
                team: Some(TeamId::new("allTeams")),
                hero: HeroId::from(hero_ids::ANA),
            },
            "team definition widening to a hero target",
        ),
    ] {
        let (written, edited) = both_paths(
            definition,
            "",
            &settings,
            &target,
            SettingValue::Boolean(true),
        );
        let write_error = written.expect_err("{label}: write must reject");
        let edit_error = edited.expect_err("{label}: source_edit must reject");
        assert!(
            matches!(
                write_error,
                SettingOperationError::ApplicabilityUnknown { .. }
            ),
            "{label}: {write_error:?}"
        );
        assert_eq!(
            write_error, edit_error,
            "{label}: the applicability check is shared"
        );
    }
}

/// `find_node`: no occurrence at the concrete path — missing leaf, missing
/// group, and an intermediate leaf that blocks descent — is `NotFound` on
/// both paths.
#[test]
fn rejects_missing_occurrences_identically() {
    let description = definition("main.description");
    let spectators = definition("lobby.spectatorSlots");

    // The leaf's group is absent entirely.
    let no_lobby = parse_settings("settings { main { Description: \"old\" } }");
    assert_agree_reject(
        &spectators,
        "",
        &no_lobby,
        &SettingTarget::Global,
        SettingValue::Number(4.0),
    );

    // The group exists but the leaf is absent.
    let no_spectators = parse_settings("settings { lobby { Max FFA Players: 4 } }");
    assert_agree_reject(
        &spectators,
        "",
        &no_spectators,
        &SettingTarget::Global,
        SettingValue::Number(4.0),
    );

    // An intermediate node that is a leaf, not a group: descent ends, so
    // the occurrence is missing rather than non-leaf.
    let leaf_main = Settings {
        span: None,
        children: vec![SettingsNode::Number {
            name: "main".to_string(),
            value: 1.0,
            span: None,
        }],
    };
    assert_agree_reject(
        &description,
        "",
        &leaf_main,
        &SettingTarget::Global,
        SettingValue::String("new".to_string()),
    );

    // A parsed occurrence under a different mode does not satisfy the
    // requested mode's concrete path.
    let kill_cam = definitions_by_id(&SettingId::from("setting.gameMode.enableKillCam"))
        .find(|definition| definition.path() == "gamemodes.general.enableKillCam")
        .expect("enableKillCam definition");
    let skirmish_only = parse_settings("settings { modes { Skirmish { Kill Cam: Off } } }");
    assert_agree_reject(
        &kill_cam,
        "",
        &skirmish_only,
        &SettingTarget::Mode("assault".to_string()),
        SettingValue::Boolean(true),
    );
}

/// The shared typed-leaf gate (the #435 fix): every non-leaf variant that
/// can sit at the leaf's path — a parsed `Group`, a `RawValue` left by an
/// undeclared enum spelling, a programmatically carried `Raw`/`RawGroup` —
/// is refused with the identical `InvalidValue` on both paths instead of
/// yielding a range carved out of the block's interior.
#[test]
fn rejects_non_leaf_occurrences_identically() {
    let description = definition("main.description");
    let map_rotation = definition("lobby.mapRotation");

    // A parsed block under a catalogued scalar key is a `Group`.
    let grouped = parse_settings("settings { main { Description { Custom: X } } }");
    assert_agree_reject(
        &description,
        "settings { main { Description { Custom: X } } }",
        &grouped,
        &SettingTarget::Global,
        SettingValue::String("new".to_string()),
    );

    // A catalogued enum key carrying an undeclared member is kept as a
    // `RawValue` pass-through, not a scalar leaf.
    let raw_value_source = "settings { lobby { Map Rotation: Not A Member } }";
    let raw_value = parse_settings(raw_value_source);
    assert_agree_reject(
        &map_rotation,
        raw_value_source,
        &raw_value,
        &SettingTarget::Global,
        SettingValue::Enum("afterAGame".to_string()),
    );

    // Programmatically carried pass-through occurrences — `Raw` and
    // `RawGroup` — refuse identically.
    for leaf in [
        SettingsNode::Raw {
            name: "description".to_string(),
            value: "anything".to_string(),
            span: None,
        },
        SettingsNode::RawGroup {
            name: "description".to_string(),
            children: vec![SettingsNode::Raw {
                name: "Custom".to_string(),
                value: "X".to_string(),
                span: None,
            }],
            span: None,
        },
    ] {
        let settings = Settings {
            span: None,
            children: vec![group("main", vec![leaf])],
        };
        assert_agree_reject(
            &description,
            "",
            &settings,
            &SettingTarget::Global,
            SettingValue::String("new".to_string()),
        );
    }
}

/// `validate_value`: a supplied value whose kind is not the domain's kind
/// is `WrongValueKind` on both paths.
#[test]
fn rejects_wrong_value_kinds_identically() {
    let string_source = "settings { main { Description: \"old\" } }";
    let description = definition("main.description");
    assert_agree_reject(
        &description,
        string_source,
        &parse_settings(string_source),
        &SettingTarget::Global,
        SettingValue::Number(1.0),
    );

    let number_source = "settings { lobby { Max Spectators: 2 } }";
    let spectators = definition("lobby.spectatorSlots");
    assert_agree_reject(
        &spectators,
        number_source,
        &parse_settings(number_source),
        &SettingTarget::Global,
        SettingValue::String("four".to_string()),
    );

    let bool_source = "settings { modes { Skirmish { Kill Cam: Off } } }";
    let kill_cam = definition("gamemodes.general.enableKillCam");
    assert_agree_reject(
        &kill_cam,
        bool_source,
        &parse_settings(bool_source),
        &SettingTarget::Mode("skirmish".to_string()),
        SettingValue::Number(1.0),
    );

    let enum_source = "settings { lobby { Map Rotation: After A Game } }";
    let map_rotation = definition("lobby.mapRotation");
    assert_agree_reject(
        &map_rotation,
        enum_source,
        &parse_settings(enum_source),
        &SettingTarget::Global,
        SettingValue::Number(1.0),
    );

    let flag_source = "settings { extensions { Beam Effects } }";
    let beam_effects = definition("extensions.beamEffects");
    assert_agree_reject(
        &beam_effects,
        flag_source,
        &parse_settings(flag_source),
        &SettingTarget::Global,
        SettingValue::Boolean(true),
    );

    let list_source = "settings { modes { Skirmish { enabled maps { King's Row Winter } } } }";
    let enabled_maps = definition("gamemodes.skirmish.enabledMaps");
    assert_agree_reject(
        &enabled_maps,
        list_source,
        &parse_settings(list_source),
        &SettingTarget::Mode("skirmish".to_string()),
        SettingValue::Boolean(true),
    );
}

/// `validate_value`: domain-invalid supplied values — non-finite numbers,
/// undeclared enum members, unknown heroes and maps — are `InvalidValue`
/// on both paths.
#[test]
fn rejects_invalid_values_identically() {
    let number_source = "settings { lobby { Max Spectators: 2 } }";
    let spectators = definition("lobby.spectatorSlots");
    for value in [
        SettingValue::Number(f64::NAN),
        SettingValue::Number(f64::INFINITY),
        SettingValue::Number(f64::NEG_INFINITY),
    ] {
        assert_agree_reject(
            &spectators,
            number_source,
            &parse_settings(number_source),
            &SettingTarget::Global,
            value,
        );
    }

    let percent_source = "settings { modes { Skirmish { Respawn Time Scalar: 30% } } }";
    let respawn = definition("gamemodes.general.respawnTime%");
    assert_agree_reject(
        &respawn,
        percent_source,
        &parse_settings(percent_source),
        &SettingTarget::Mode("skirmish".to_string()),
        SettingValue::Percent(f64::NAN),
    );

    let enum_source = "settings { lobby { Map Rotation: After A Game } }";
    let map_rotation = definition("lobby.mapRotation");
    assert_agree_reject(
        &map_rotation,
        enum_source,
        &parse_settings(enum_source),
        &SettingTarget::Global,
        SettingValue::Enum("noSuchMember".to_string()),
    );

    let heroes_source = "settings { heroes { General { enabled heroes { Genji Ana } } } }";
    let enabled_heroes = definition("heroes.<team>.enabledHeroes");
    assert_agree_reject(
        &enabled_heroes,
        heroes_source,
        &parse_settings(heroes_source),
        &SettingTarget::Team(TeamId::new("allTeams")),
        SettingValue::HeroList(vec!["genji".to_string(), "notAHero".to_string()]),
    );

    let maps_source = "settings { modes { Skirmish { enabled maps { King's Row Winter } } } }";
    let enabled_maps = definition("gamemodes.skirmish.enabledMaps");
    assert_agree_reject(
        &enabled_maps,
        maps_source,
        &parse_settings(maps_source),
        &SettingTarget::Mode("skirmish".to_string()),
        SettingValue::MapList(vec!["kingsRowWinter".to_string(), "notAMap".to_string()]),
    );
}

/// The boolean-enum gate: `Match Voice Chat` renders `true` through a
/// source-supported enabled-member token; `false` has no authored spelling
/// and the emitter rejects the carrier. Both edit paths must refuse it
/// with the identical `InvalidValue` — `write` must not accept a value its
/// carrier cannot emit.
#[test]
fn rejects_unsupported_boolean_enum_false_identically() {
    let source = "settings { lobby { Match Voice Chat: Enabled } }";
    let voice_chat = definition("lobby.enableMatchVoiceChat");
    assert_agree_reject(
        &voice_chat,
        source,
        &parse_settings(source),
        &SettingTarget::Global,
        SettingValue::Boolean(false),
    );
}

/// The carrier-shape agreement `apply_value` enforces inside `write`: a
/// leaf whose authored value kind is not the domain's is `WrongValueKind`
/// on both paths.
#[test]
fn rejects_mismatched_leaf_shape_identically() {
    let description = definition("main.description");
    for leaf in [
        SettingsNode::Bool {
            name: "description".to_string(),
            value: true,
            span: None,
        },
        SettingsNode::Number {
            name: "description".to_string(),
            value: 1.0,
            span: None,
        },
        SettingsNode::Flag {
            name: "description".to_string(),
            span: None,
        },
        SettingsNode::List {
            name: "description".to_string(),
            elements: vec![],
            span: None,
        },
    ] {
        let settings = Settings {
            span: None,
            children: vec![group("main", vec![leaf])],
        };
        assert_agree_reject(
            &description,
            "",
            &settings,
            &SettingTarget::Global,
            SettingValue::String("new".to_string()),
        );
    }
}

/// The occurrence's current value is replaced wholesale, so neither path
/// judges it: a leaf holding a value its domain does not declare — the
/// carrier can only reach that state programmatically, since parsing falls
/// back to `RawValue` — is repaired identically by both paths.
#[test]
fn repairs_a_leaf_holding_an_undeclared_value_identically() {
    let source = "settings { lobby { Map Rotation: After A Game } }";
    let mut settings = parse_settings(source);
    let leaf = match &mut settings.children[0] {
        SettingsNode::Group { children, .. } => children
            .iter_mut()
            .find(|node| node.name() == "mapRotation")
            .expect("the parsed leaf"),
        node => panic!("expected the lobby group, got {node:?}"),
    };
    match leaf {
        SettingsNode::String { value, .. } => *value = "unreviewedMember".to_string(),
        node => panic!("expected the enum String leaf, got {node:?}"),
    }

    let map_rotation = definition("lobby.mapRotation");
    let value = SettingValue::Enum("afterAGame".to_string());
    // `write` accepts: the carrier's stale member is overwritten.
    // `source_edit` must accept too: the edit rewrites the value bytes.
    assert_agree_accept(
        &map_rotation,
        source,
        &settings,
        &SettingTarget::Global,
        value,
        "After A Game",
    );
}

/// `write` accepts ⟺ `source_edit` accepts across the scalar leaf surface:
/// every domain/kind spelling both paths support. The edit covers exactly
/// the authored value bytes, and both paths converge on the same
/// observable occurrence value.
#[test]
fn accepts_the_same_scalar_edits() {
    let cases = [
        (
            "lobby.mapRotation",
            "settings { lobby { Map Rotation: After A Game } }",
            SettingTarget::Global,
            SettingValue::Enum("afterMirrorMatch".to_string()),
            "After A Game",
        ),
        (
            "lobby.mapRotation",
            "settings { lobby { Map Rotation: After A Game } }",
            SettingTarget::Global,
            SettingValue::Enum("paused".to_string()),
            "After A Game",
        ),
        (
            "lobby.spectatorSlots",
            "settings { lobby { Max Spectators: 2 } }",
            SettingTarget::Global,
            SettingValue::Number(4.0),
            "2",
        ),
        (
            "main.description",
            "settings { main { Description: \"old\" } }",
            SettingTarget::Global,
            SettingValue::String("new".to_string()),
            "\"old\"",
        ),
        (
            "lobby.allowPlayersInQueue",
            "settings { lobby { Allow Players Who Are In Queue: No } }",
            SettingTarget::Global,
            SettingValue::Boolean(true),
            "No",
        ),
        (
            "lobby.allowPlayersInQueue",
            "settings { lobby { Allow Players Who Are In Queue: Yes } }",
            SettingTarget::Global,
            SettingValue::Boolean(false),
            "Yes",
        ),
        (
            "lobby.enableMatchVoiceChat",
            "settings { lobby { Match Voice Chat: Enabled } }",
            SettingTarget::Global,
            SettingValue::Boolean(true),
            "Enabled",
        ),
        (
            "gamemodes.general.enableKillCam",
            "settings { modes { Skirmish { Kill Cam: Off } } }",
            SettingTarget::Mode("skirmish".to_string()),
            SettingValue::Boolean(true),
            "Off",
        ),
        (
            "gamemodes.general.respawnTime%",
            "settings { modes { Skirmish { Respawn Time Scalar: 30% } } }",
            SettingTarget::Mode("skirmish".to_string()),
            SettingValue::Percent(45.0),
            "30%",
        ),
        (
            "gamemodes.assault.enabled",
            "settings { modes { Assault { enabled: On } } }",
            SettingTarget::Mode("assault".to_string()),
            SettingValue::Boolean(false),
            "On",
        ),
    ];
    for (path, source, target, value, old_value_bytes) in cases {
        let settings = parse_settings(source);
        assert_agree_accept(
            &definition(path),
            source,
            &settings,
            &target,
            value,
            old_value_bytes,
        );
    }
}

/// `write`'s `apply_value` also refuses a list write that changes the
/// element count; `source_edit` fails the same input closed — a list
/// member is a brace block with no `key: value` value bytes, so it has no
/// scalar edit to make at all (`SourceUnavailable`). Both paths reject;
/// the diagnostics differ because the refusal is the shared list-length
/// rule on one side and the scalar-edit boundary on the other.
#[test]
fn rejects_list_length_changes() {
    let source = "settings { modes { Skirmish { enabled maps { King's Row Winter } } } }";
    let enabled_maps = definition("gamemodes.skirmish.enabledMaps");
    let value = SettingValue::MapList(vec![
        "kingsRowWinter".to_string(),
        "workshopIsland".to_string(),
    ]);
    let (written, edited) = both_paths(
        &enabled_maps,
        source,
        &parse_settings(source),
        &SettingTarget::Mode("skirmish".to_string()),
        value,
    );
    assert!(
        matches!(written, Err(SettingOperationError::InvalidValue { .. })),
        "write refuses the length change: {written:?}"
    );
    assert!(
        matches!(edited, Err(SettingOperationError::SourceUnavailable { .. })),
        "source_edit fails closed at the scalar-edit boundary: {edited:?}"
    );
}

/// `source_edit`'s own contract — never fabricate a range where no scalar
/// `key: value` bytes exist — is not a check `write` shares: mutating the
/// carrier in place is legal for lists and presence-only flags, and a
/// carrier without recorded spans has no source to edit. These pins keep
/// the boundary explicit.
#[test]
fn source_edit_fails_closed_where_no_scalar_edit_exists() {
    // A same-length list write is a legal carrier mutation, but the
    // brace-block member carries no value bytes for an edit to replace.
    let maps_source = "settings { modes { Skirmish { enabled maps { King's Row Winter } } } }";
    let enabled_maps = definition("gamemodes.skirmish.enabledMaps");
    let (written, edited) = both_paths(
        &enabled_maps,
        maps_source,
        &parse_settings(maps_source),
        &SettingTarget::Mode("skirmish".to_string()),
        SettingValue::MapList(vec!["workshopIsland".to_string()]),
    );
    assert!(written.is_ok(), "write accepts the carrier mutation");
    assert!(
        matches!(edited, Err(SettingOperationError::SourceUnavailable { .. })),
        "no scalar value bytes exist to replace: {edited:?}"
    );

    // A presence-only flag is itself the value — there is no `:` to
    // address a replacement through.
    let flag_source = "settings { extensions { Beam Effects } }";
    let beam_effects = definition("extensions.beamEffects");
    let (written, edited) = both_paths(
        &beam_effects,
        flag_source,
        &parse_settings(flag_source),
        &SettingTarget::Global,
        SettingValue::PresenceOnly,
    );
    assert!(written.is_ok(), "write accepts the carrier mutation");
    assert!(
        matches!(edited, Err(SettingOperationError::SourceUnavailable { .. })),
        "no scalar value bytes exist to replace: {edited:?}"
    );

    // A carrier with no recorded spans gives the edit nothing to anchor.
    let unspanned = Settings {
        span: None,
        children: vec![group(
            "lobby",
            vec![SettingsNode::Number {
                name: "spectatorSlots".to_string(),
                value: 2.0,
                span: None,
            }],
        )],
    };
    let spectators = definition("lobby.spectatorSlots");
    let (written, edited) = both_paths(
        &spectators,
        "settings { lobby { Max Spectators: 2 } }",
        &unspanned,
        &SettingTarget::Global,
        SettingValue::Number(4.0),
    );
    assert!(written.is_ok(), "write accepts the carrier mutation");
    assert!(
        matches!(edited, Err(SettingOperationError::SourceUnavailable { .. })),
        "no source location exists to anchor an edit: {edited:?}"
    );
}
