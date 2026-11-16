use workshop_rs::{parser, validate};

use crate::common::{catalog, en, fixture_text};

/// Each probe isolates one argument whose semantic type does not match the
/// declared parameter type. Under ADR-0014 these stay rejected until a client
/// capture of the probe proves acceptance; a recorded acceptance moves the
/// probe to the accepted side here together with the catalog fact.
const REJECTED: &[(&str, &str)] = &[
    (
        "string-slice-number-literal.ws",
        "'stringSlice' argument 1 must have semantic type 'String', got Number",
    ),
    (
        "string-slice-number-expression.ws",
        "'stringSlice' argument 1 must have semantic type 'String', got Number",
    ),
    (
        "string-slice-vector.ws",
        "'stringSlice' argument 1 must have semantic type 'String', got Vector",
    ),
    (
        "string-slice-player.ws",
        "'stringSlice' argument 1 must have semantic type 'String', got Player",
    ),
    (
        "combined.ws",
        "'stringSlice' argument 1 must have semantic type 'String', got Number",
    ),
    (
        "square-root-player.ws",
        "'squareRoot' argument 1 must have semantic type 'Number', got Player",
    ),
];

#[test]
fn the_control_probe_validates() {
    validate::validate_canonical_ids(&probe("control-string-slice-string.ws"), &catalog())
        .expect("the control must validate");
}

#[test]
fn type_mismatch_probes_are_rejected_until_client_evidence_exists() {
    for &(name, message) in REJECTED {
        let error = validate::validate_canonical_ids(&probe(name), &catalog())
            .err()
            .unwrap_or_else(|| panic!("{name} must be rejected"));
        assert!(
            error.to_string().contains(message),
            "{name}: unexpected error {error}"
        );
    }
}

fn probe(name: &str) -> workshop_rs::Program {
    let text = fixture_text("slot-acceptance", name);
    parser::parse(&text, &catalog(), &en())
        .unwrap_or_else(|error| panic!("{name} must parse: {error}"))
}
