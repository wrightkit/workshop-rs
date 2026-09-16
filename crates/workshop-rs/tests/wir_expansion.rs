//! WIR expansion and canonical-identity tests (#31): the supported Workshop surface is
//! representable in Workshop IR, and catalog-backed validation rejects
//! unknown or locale-tainted builtin references deterministically.

use workshop_rs::catalog::{Catalog, Locale};
use workshop_rs::source::{Position, SourceFile, Span};
use workshop_rs::validate;
use workshop_rs::wir::{self, Action, Event, Value, ValueNode};

fn catalog() -> Catalog {
    Catalog::builtin().expect("built-in catalog")
}

#[test]
fn member_access_has_a_canonical_shape_contract() {
    let catalog = catalog();
    let mut program = wir::Program::default();
    let receiver = program
        .values
        .push(ValueNode::new(Value::EventPlayer, None));
    let member = program.values.push(ValueNode::new(
        Value::Number {
            value: 1.0,
            text: "1".into(),
        },
        None,
    ));
    let access = program.values.push(ValueNode::new(
        Value::Call {
            name: "memberAccess".into(),
            args: vec![receiver, member],
        },
        None,
    ));
    program.rules.push(wir::Rule {
        name: "member-contract".into(),
        span: None,
        name_span: None,
        disabled: false,
        event: Event::Global,
        conditions: vec![access],
        actions: vec![],
    });
    let error =
        validate::validate_canonical_ids_wir(&program, &catalog).expect_err("invalid memberAccess");
    assert!(
        error
            .to_string()
            .contains("memberAccess member must be a string")
    );
}

#[test]
fn indexed_members_and_native_break_controls_have_one_canonical_contract() {
    let catalog = catalog();
    let locale = Locale::new("en-US");
    let source = r#"
        variables { global: 0: values }
        rule ("interop primitives") {
            event { Ongoing - Global; }
            actions {
                Global.values[1] = 2;
                Event Player.values[0] = 3;
                If(True);
                    Event Player.payload.member[0] = 4;
                    Skip If(True, 1);
                    While(True);
                        Break;
                        Continue;
                    End;
                Else;
                    Skip(2);
                End;
            }
        }
    "#;
    let program = workshop_rs::parser::parse_wir_with_context(source, &catalog, &locale, &catalog)
        .expect("native indexed/member and control-flow forms parse");
    program
        .validate()
        .expect("canonical WIR is structurally valid");
    validate::validate_canonical_ids_wir(&program, &catalog).expect("catalog ids resolve");

    let rule = program.rules.iter().next().expect("rule");
    assert!(matches!(
        program.actions.get(rule.actions[0]),
        Some(Action::Call { name, args, .. })
            if name == "setGlobalVariableAtIndex"
                && args.len() == 3
                && matches!(program.values.get(args[0]).map(|node| &node.value), Some(Value::GlobalVariable(_)))
    ));
    assert!(matches!(
        program.actions.get(rule.actions[1]),
        Some(Action::Call { name, args, .. })
            if name == "setPlayerVariableAtIndex"
                && args.len() == 3
                && matches!(program.values.get(args[0]).map(|node| &node.value), Some(Value::PlayerVariable { .. }))
    ));

    let Action::If {
        branches,
        else_body: Some(else_body),
        ..
    } = program.actions.get(rule.actions[2]).expect("if action")
    else {
        panic!("expected an if with an else branch");
    };
    let member_action = program
        .actions
        .get(branches[0].body[0])
        .expect("member assignment");
    let Action::AssignMember { target, .. } = member_action else {
        panic!("expected canonical member assignment");
    };
    assert!(matches!(
        program.values.get(*target).map(|node| &node.value),
        Some(Value::Call { name, args })
            if name == "memberAccess"
                && args.len() == 3
                && matches!(program.values.get(args[1]).map(|node| &node.value), Some(Value::String(member)) if member == "member")
    ));
    assert!(matches!(
        program.actions.get(branches[0].body[1]),
        Some(Action::Call { name, args, .. }) if name == "skipIf" && args.len() == 2
    ));
    let Action::While { body, .. } = program.actions.get(branches[0].body[2]).expect("while")
    else {
        panic!("expected nested while");
    };
    assert!(matches!(
        program.actions.get(body[0]),
        Some(Action::Call { name, args, .. }) if name == "break" && args.is_empty()
    ));
    assert!(matches!(
        program.actions.get(body[1]),
        Some(Action::Call { name, args, .. }) if name == "continue" && args.is_empty()
    ));
    assert!(matches!(
        program.actions.get(else_body[0]),
        Some(Action::Call { name, args, .. }) if name == "skip" && args.len() == 1
    ));

    let emitted = workshop_rs::emitter::emit_wir(&program, &catalog, &locale).expect("emits");
    let reparsed =
        workshop_rs::parser::parse_wir_with_context(&emitted, &catalog, &locale, &catalog)
            .expect("emitted native controls reparse");
    assert!(workshop_rs::roundtrip::equivalent_wir(&program, &reparsed));
    assert_eq!(
        emitted,
        workshop_rs::emitter::emit_wir(&reparsed, &catalog, &locale).expect("re-emits")
    );
}

#[test]
fn assign_member_rejects_non_lvalue_member_access_targets() {
    let catalog = catalog();
    let mut program = wir::Program::default();
    let receiver = program
        .values
        .push(ValueNode::new(Value::EventPlayer, None));
    let member = program
        .values
        .push(ValueNode::new(Value::String("payload".into()), None));
    let member_access = program.values.push(ValueNode::new(
        Value::Call {
            name: "memberAccess".into(),
            args: vec![receiver, member],
        },
        None,
    ));
    let index = program.values.push(ValueNode::new(
        Value::Number {
            value: 1.0,
            text: "1".into(),
        },
        None,
    ));
    let target = program.values.push(ValueNode::new(
        Value::Call {
            name: "valueInArray".into(),
            args: vec![member_access, index],
        },
        None,
    ));
    let value = program.values.push(ValueNode::new(Value::Bool(true), None));
    let action = program.actions.push(Action::AssignMember {
        target,
        op: None,
        value,
        span: None,
    });
    program.rules.push(wir::Rule {
        name: "invalid member target".into(),
        span: None,
        name_span: None,
        disabled: false,
        event: Event::Global,
        conditions: vec![],
        actions: vec![action],
    });

    let error = validate::validate_canonical_ids_wir(&program, &catalog)
        .expect_err("AssignMember must require a memberAccess target");
    assert!(
        error
            .to_string()
            .contains("AssignMember target must be a memberAccess value")
    );
}

fn span(file: workshop_rs::ids::Id<SourceFile>, line: u32, col: u32, end_col: u32) -> Span {
    Span::new(file, Position::new(line, col), Position::new(line, end_col))
}

/// Build a WIR program representing the corpus Workshop surface: one global
/// variable, a rule with a condition, a For loop, an If, a Modify action, a
/// generic action, and the catalog-backed values.
fn build_surface_program() -> wir::Program {
    let mut program = wir::Program::default();
    let file = program.files.push(SourceFile::new("workshop.txt"));
    let s = |line, col, end| span(file, line, col, end);

    let index = program.global_variables.push(wir::WorkshopVariable {
        name: "index".into(),
        index: 0,
        span: Some(s(1, 15, 20)),
        name_span: Some(s(1, 15, 20)),
    });

    let zero = program.values.push(ValueNode::new(
        Value::Number {
            value: 0.0,
            text: "0".to_string(),
        },
        Some(s(3, 24, 25)),
    ));
    let stop = program.values.push(ValueNode::new(
        Value::Number {
            value: 3.0,
            text: "3".to_string(),
        },
        Some(s(3, 27, 28)),
    ));
    let one = program.values.push(ValueNode::new(
        Value::Number {
            value: 1.0,
            text: "1".to_string(),
        },
        Some(s(3, 30, 31)),
    ));
    let index_ref = program.values.push(ValueNode::new(
        Value::GlobalVariable(index),
        Some(s(4, 18, 23)),
    ));
    let compare = program.values.push(ValueNode::new(
        Value::Call {
            name: "==".into(),
            args: vec![index_ref, zero],
        },
        Some(s(4, 10, 24)),
    ));
    let modified = program.values.push(ValueNode::new(
        Value::Call {
            name: "add".into(),
            args: vec![index_ref, one],
        },
        Some(s(5, 26, 33)),
    ));
    let yellow = program.values.push(ValueNode::new(
        Value::Enum {
            value_type: "Color".into(),
            value: "YELLOW".into(),
        },
        Some(s(5, 41, 47)),
    ));
    let beam_type = program.values.push(ValueNode::new(
        Value::Enum {
            value_type: "Beam".into(),
            value: "GRAPPLE".into(),
        },
        Some(s(5, 35, 40)),
    ));
    let start_position = program.values.push(ValueNode::new(
        Value::Vector {
            x: zero,
            y: zero,
            z: zero,
        },
        Some(s(5, 48, 55)),
    ));
    let end_position = program.values.push(ValueNode::new(
        Value::Vector {
            x: one,
            y: one,
            z: one,
        },
        Some(s(5, 56, 63)),
    ));
    let effect_reeval = program.values.push(ValueNode::new(
        Value::Enum {
            value_type: "EffectReeval".into(),
            value: "NONE".into(),
        },
        Some(s(5, 64, 68)),
    ));
    let all_teams = program.values.push(ValueNode::new(
        Value::Enum {
            value_type: "Team".into(),
            value: "ALL".into(),
        },
        Some(s(6, 14, 23)),
    ));
    let players = program.values.push(ValueNode::new(
        Value::Call {
            name: "allPlayers".into(),
            args: vec![all_teams],
        },
        Some(s(6, 14, 24)),
    ));

    let wait_duration = program.values.push(ValueNode::new(
        Value::Number {
            value: 1.0,
            text: "1".to_string(),
        },
        Some(s(7, 11, 12)),
    ));
    let wait = program.actions.push(Action::Call {
        name: "wait".into(),
        args: vec![wait_duration],
        span: Some(s(7, 9, 13)),
    });
    let if_body = vec![wait];
    let if_action = program.actions.push(Action::If {
        branches: vec![wir::IfBranch {
            condition: compare,
            body: if_body,
        }],
        else_body: None,
        span: Some(s(4, 5, 8)),
    });
    let modify = program.actions.push(Action::ModifyGlobalVariable {
        variable: index,
        op: wir::ModifyOp::Add,
        value: modified,
        span: Some(s(5, 5, 34)),
        target_span: Some(s(5, 5, 10)),
    });
    let beam = program.actions.push(Action::Call {
        name: "createBeamEffect".into(),
        args: vec![
            players,
            beam_type,
            start_position,
            end_position,
            yellow,
            effect_reeval,
        ],
        span: Some(s(6, 5, 25)),
    });
    let for_action = program.actions.push(Action::ForGlobalVariable {
        variable: index,
        start: zero,
        stop,
        step: one,
        body: vec![if_action, modify, beam],
        span: Some(s(3, 5, 31)),
        target_span: Some(s(3, 5, 10)),
    });

    program.rules.push(wir::Rule {
        name: "surface".into(),
        span: Some(s(2, 1, 6)),
        name_span: Some(s(2, 5, 6)),
        disabled: false,
        event: Event::Global,
        conditions: vec![],
        actions: vec![for_action],
    });
    program
}

#[test]
fn corpus_surface_is_representable_and_validates() {
    let program = build_surface_program();
    program.validate().expect("WIR is structurally valid");
    validate::validate_canonical_ids_wir(&program, &catalog()).expect("canonical ids resolve");

    let dump = program.dump();
    assert!(dump.contains("forGlobalVariable index in 0, 3, 1"));
    assert!(dump.contains("modifyGlobalVariable index Add"));
    assert!(dump.contains("call createBeamEffect"));
    assert!(dump.contains("Color.YELLOW"));
    assert!(dump.contains("allPlayers(Team.ALL)"));
}

#[test]
fn unknown_action_id_is_rejected_with_location() {
    let mut program = build_surface_program();
    let file = program
        .files
        .iter()
        .next()
        .map(|_| workshop_rs::ids::Id::from_index(0))
        .expect("one file");
    let action = program
        .actions
        .get_mut(wir::ActionId::from_index(3))
        .expect("beam action");
    if let Action::Call { name, span, .. } = action {
        *name = "createLaserEffect".into();
        *span = Some(Span::new(file, Position::new(9, 5), Position::new(9, 20)));
    }
    let error =
        validate::validate_canonical_ids_wir(&program, &catalog()).expect_err("unknown action");
    assert!(error.to_string().contains("createLaserEffect"), "{error}");
}

#[test]
fn unknown_enum_member_is_rejected() {
    let mut program = build_surface_program();
    // The Color enum node is at a known value id; replace its member.
    for node in program.values.iter() {
        if let Value::Enum {
            value_type, value, ..
        } = &node.value
        {
            if value_type == "Color" {
                let _ = value;
            }
        }
    }
    // Rebuild a bad enum node: find the Color enum and mutate it.
    let color_id = program
        .values
        .iter()
        .enumerate()
        .find(|(_, node)| {
            matches!(&node.value, Value::Enum { value_type, .. } if value_type == "Color")
        })
        .map(|(index, _)| workshop_rs::wir::ValueId::from_index(index))
        .expect("color node");
    if let Value::Enum { value, .. } = &mut program.values.get_mut(color_id).unwrap().value {
        *value = "NEON".into();
    }
    let error =
        validate::validate_canonical_ids_wir(&program, &catalog()).expect_err("unknown member");
    assert!(error.to_string().contains("NEON"), "{error}");
}

#[test]
fn unknown_value_id_is_rejected() {
    let mut program = build_surface_program();
    for node in program.values.iter() {
        if let Value::Call { name, .. } = &node.value {
            if name == "add" {
                let _ = name;
            }
        }
    }
    let add_id = program
        .values
        .iter()
        .enumerate()
        .find(|(_, node)| matches!(&node.value, Value::Call { name, .. } if name == "add"))
        .map(|(index, _)| workshop_rs::wir::ValueId::from_index(index))
        .expect("add node");
    if let Value::Call { name, .. } = &mut program.values.get_mut(add_id).unwrap().value {
        *name = "plus".into();
    }
    let error =
        validate::validate_canonical_ids_wir(&program, &catalog()).expect_err("unknown value");
    assert!(error.to_string().contains("plus"), "{error}");
}

#[test]
fn canonical_validation_is_locale_independent() {
    // Resolution uses canonical ids; locale spelling never appears in WIR.
    let program = build_surface_program();
    let _ = Locale::new("en-US");
    validate::validate_canonical_ids_wir(&program, &catalog()).expect("valid");
    // No WIR dump contains a localized spelling.
    let dump = program.dump();
    assert!(!dump.contains("Disable Inspector Recording"));
}

#[test]
fn canonical_validation_returns_first_error_and_short_circuits() {
    let catalog = catalog();
    let mut program = wir::Program::default();

    // Action 1: Unknown action "unknownActionOne"
    let action_1 = program.actions.push(Action::Call {
        name: "unknownActionOne".into(),
        args: vec![],
        span: None,
    });
    // Action 2: Unknown action "unknownActionTwo"
    let action_2 = program.actions.push(Action::Call {
        name: "unknownActionTwo".into(),
        args: vec![],
        span: None,
    });
    // Rule 1 has action_1 then action_2
    program.rules.push(wir::Rule {
        name: "rule_one".into(),
        span: None,
        name_span: None,
        disabled: false,
        event: Event::Global,
        conditions: vec![],
        actions: vec![action_1, action_2],
    });

    // Action 3: Unknown action "unknownActionThree" in Rule 2
    let action_3 = program.actions.push(Action::Call {
        name: "unknownActionThree".into(),
        args: vec![],
        span: None,
    });
    program.rules.push(wir::Rule {
        name: "rule_two".into(),
        span: None,
        name_span: None,
        disabled: false,
        event: Event::Global,
        conditions: vec![],
        actions: vec![action_3],
    });

    let error = validate::validate_canonical_ids_wir(&program, &catalog)
        .expect_err("multi-error program must fail validation on first error");
    assert!(
        error.to_string().contains("unknownActionOne"),
        "expected first error for unknownActionOne, got: {error}"
    );

    // Also verify event error precedes action error
    let mut program_event = wir::Program::default();
    let bad_action = program_event.actions.push(Action::Call {
        name: "unknownActionInEventTest".into(),
        args: vec![],
        span: None,
    });
    program_event.rules.push(wir::Rule {
        name: "rule_event".into(),
        span: None,
        name_span: None,
        disabled: false,
        event: Event::Player {
            kind: wir::PlayerEventKind::Died,
            team: wir::EventTeam::All,
            target: wir::EventTarget::Hero("NonExistentHero".into()),
        },
        conditions: vec![],
        actions: vec![bad_action],
    });
    let event_error = validate::validate_canonical_ids_wir(&program_event, &catalog)
        .expect_err("event error must precede action error");
    assert!(
        event_error.to_string().contains("NonExistentHero"),
        "expected event error, got: {event_error}"
    );

    // Action error precedes Condition error in the same rule
    let mut program_action_cond = wir::Program::default();
    let bad_action = program_action_cond.actions.push(Action::Call {
        name: "unknownActionBeforeCondition".into(),
        args: vec![],
        span: None,
    });
    let bad_cond = program_action_cond.values.push(ValueNode::new(
        Value::Call {
            name: "unknownConditionValue".into(),
            args: vec![],
        },
        None,
    ));
    program_action_cond.rules.push(wir::Rule {
        name: "rule_action_cond".into(),
        span: None,
        name_span: None,
        disabled: false,
        event: Event::Global,
        conditions: vec![bad_cond],
        actions: vec![bad_action],
    });
    let action_cond_error = validate::validate_canonical_ids_wir(&program_action_cond, &catalog)
        .expect_err("action error must precede condition error");
    assert!(
        action_cond_error
            .to_string()
            .contains("unknownActionBeforeCondition"),
        "expected action error before condition error, got: {action_cond_error}"
    );

    // Call signature error precedes argument value error
    let mut program_sig = wir::Program::default();
    // "Wait" requires at least 1 argument; passing empty args causes arity error before arg evaluation
    let bad_call = program_sig.actions.push(Action::Call {
        name: "wait".into(),
        args: vec![],
        span: None,
    });
    program_sig.rules.push(wir::Rule {
        name: "rule_sig".into(),
        span: None,
        name_span: None,
        disabled: false,
        event: Event::Global,
        conditions: vec![],
        actions: vec![bad_call],
    });
    let sig_error = validate::validate_canonical_ids_wir(&program_sig, &catalog)
        .expect_err("signature error must trigger");
    assert!(
        sig_error.to_string().contains("expects"),
        "expected arity error, got: {sig_error}"
    );

    // Argument 1 error precedes Argument 2 error
    let mut program_args = wir::Program::default();
    let bad_arg1 = program_args.values.push(ValueNode::new(
        Value::Call {
            name: "unknownArgOne".into(),
            args: vec![],
        },
        None,
    ));
    let bad_arg2 = program_args.values.push(ValueNode::new(
        Value::Call {
            name: "unknownArgTwo".into(),
            args: vec![],
        },
        None,
    ));
    let call_args = program_args.actions.push(Action::Call {
        name: "modifyGlobalVariable".into(),
        args: vec![bad_arg1, bad_arg2],
        span: None,
    });
    program_args.rules.push(wir::Rule {
        name: "rule_args".into(),
        span: None,
        name_span: None,
        disabled: false,
        event: Event::Global,
        conditions: vec![],
        actions: vec![call_args],
    });
    let args_error = validate::validate_canonical_ids_wir(&program_args, &catalog)
        .expect_err("argument 1 error must precede argument 2 error");
    assert!(
        args_error.to_string().contains("unknownArgOne"),
        "expected arg 1 error, got: {args_error}"
    );
}
