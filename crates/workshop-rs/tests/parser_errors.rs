use crate::common::en;
use workshop_rs::WorkshopError;
use workshop_rs::catalog::Catalog;
use workshop_rs::parser;

#[test]
fn malformed_number_literals_are_reported_by_the_public_parser() {
    let catalog = Catalog::builtin().expect("catalog");
    let locale = en();

    for source in ["0X", "1.2.3"] {
        let error = parser::parse(source, &catalog, &locale).expect_err("invalid number");
        let WorkshopError::Malformed { message, span } = error else {
            panic!("invalid number should be a malformed Workshop diagnostic");
        };
        assert_eq!(message, format!("invalid number '{source}'"));
        assert_eq!(span.expect("source span").start.col, 1);
    }
}
