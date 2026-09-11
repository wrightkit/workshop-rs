use workshop_rs::{Action, Value};

#[test]
fn typed_constructors_preserve_canonical_ids_and_order() {
    let action = Action::set_slow_motion(0.5);
    assert!(matches!(
        action,
        Action::Call { name, args } if name == "setSlowMotion"
            && matches!(&args[..], [Value::Number(value)] if *value == 0.5)
    ));

    let value = Value::get_max_health(Value::event_player());
    assert!(matches!(
        value,
        Value::Call { name, args } if name == "getMaxHealth"
            && matches!(&args[..], [Value::Call { name, args }]
                if name == "eventPlayer" && args.is_empty())
    ));
}

#[test]
fn typed_values_accept_obvious_rust_literals() {
    let value = Value::vector(1, 2, 3);
    assert!(matches!(
        value,
        Value::Call { name, args } if name == "vector"
            && matches!(&args[..], [
                Value::Number(x), Value::Number(y), Value::Number(z)
            ] if (*x, *y, *z) == (1.0, 2.0, 3.0))
    ));

    let array = Value::array(["first", "second"]);
    assert!(matches!(
        array,
        Value::Call { name, args } if name == "array"
            && matches!(&args[..], [Value::String(first), Value::String(second)]
                if first == "first" && second == "second")
    ));
}

#[test]
fn generic_calls_remain_available_for_dynamic_consumers() {
    let action = Action::call("runtimeAction", [Value::from(1)]);
    let value = Value::call("runtimeValue", [Value::from("input")]);

    assert!(matches!(
        action,
        Action::Call { name, args } if name == "runtimeAction" && args.len() == 1
    ));
    assert!(matches!(
        value,
        Value::Call { name, args } if name == "runtimeValue" && args.len() == 1
    ));
}
