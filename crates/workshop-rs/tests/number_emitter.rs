use super::common::catalog;
use workshop_rs::{Action, Event, Program, Rule, Value, Variable};

#[test]
fn computed_number_emission_matches_pinned_overpy_program() {
    // OverPy 9.7.17, gitHead ec763c1d245aa5485ef61af5b2bda4975bce4877.
    let reference = r#"variables {
    global:
        0: a
        1: b
        2: c
        3: d
        4: e
        5: f
        6: g
        7: h
}

rule ("precision") {
    event {
        Ongoing - Global;
    }
    actions {
        Set Global Variable(a, 100.01666666666667);
        Set Global Variable(b, 33.333333333333336);
        Set Global Variable(c, 142857.14285714287);
        Set Global Variable(d, 12.345678901234567);
        Set Global Variable(e, 9.999999999999999);
        Set Global Variable(f, 1.234567890123456);
        Set Global Variable(g, 0.012345678901234);
        Set Global Variable(h, -12.345678901234567);
    }
}
"#;
    let catalog = catalog();
    let locale = workshop_rs::catalog::Locale::new("en-US");
    let expected = workshop_rs::parser::parse(reference, &catalog, &locale).expect("oracle parses");

    let mut program = Program::new();
    let mut rule = Rule::new("precision", Event::Global);
    for (name, value) in [
        ("a", "100.01666666666667"),
        ("b", "33.333333333333336"),
        ("c", "142857.14285714287"),
        ("d", "12.345678901234567"),
        ("e", "9.999999999999999"),
        ("f", "1.2345678901234567"),
        ("g", "0.012345678901234567"),
        ("h", "-12.345678901234567"),
    ] {
        program.global_variable(Variable::new(name));
        rule = rule.action(Action::SetGlobalVariable {
            variable: name.into(),
            value: Value::number(value.parse().expect("number")),
        });
    }
    program.rule(rule);

    let emitted = workshop_rs::emitter::emit(&program, &catalog, &locale).expect("emits");
    let reparsed = workshop_rs::parser::parse(&emitted, &catalog, &locale).expect("reparses");
    assert!(
        workshop_rs::roundtrip::equivalent(&expected, &reparsed),
        "computed numbers differ from the oracle: {emitted}"
    );

    let emitted_reference =
        workshop_rs::emitter::emit(&expected, &catalog, &locale).expect("parsed oracle emits");
    assert!(
        emitted_reference.contains("Set Global Variable(a, 100.01666666666667);"),
        "source number changed during emission: {emitted_reference}"
    );
    let reparsed_reference = workshop_rs::parser::parse(&emitted_reference, &catalog, &locale)
        .expect("reparse emitted oracle");
    assert!(workshop_rs::roundtrip::equivalent(
        &expected,
        &reparsed_reference
    ));
}
