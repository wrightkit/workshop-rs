//! The hero-values-v1 schema accepts the example file and rejects each invalid example.

use serde_json::Value;

fn read(path: &str) -> Value {
    let root = env!("CARGO_MANIFEST_DIR");
    let text = std::fs::read_to_string(format!("{root}/{path}")).expect("file reads");
    serde_json::from_str(&text).expect("file parses as JSON")
}

fn validator() -> jsonschema::JSONSchema {
    let schema = read("schemas/hero-values-v1.schema.json");
    jsonschema::JSONSchema::compile(&schema).expect("schema compiles")
}

fn fixture(name: &str) -> Value {
    read(&format!("tests/fixtures/hero-values/{name}.json"))
}

#[test]
fn example_file_validates() {
    let document = fixture("valid-example");
    if let Err(errors) = validator().validate(&document) {
        let errors = errors.map(|error| error.to_string()).collect::<Vec<_>>();
        panic!("example does not validate: {errors:?}");
    }
}

#[test]
fn invalid_examples_are_rejected() {
    let validator = validator();
    for name in [
        "invalid-missing-source",
        "invalid-unknown-verification",
        "invalid-unknown-kind",
    ] {
        assert!(
            validator.validate(&fixture(name)).is_err(),
            "{name} unexpectedly validates"
        );
    }
}
