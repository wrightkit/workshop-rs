use workshop_rs::lexer::{TokenKind, tokenize};

#[test]
fn tokenizes_the_evidenced_uppercase_workshop_hex_literal() {
    let tokens = tokenize("0X20").expect("hexadecimal Workshop number tokenizes");

    assert_eq!(
        tokens[0].kind,
        TokenKind::Number {
            value: 32.0,
            text: "0X20".to_string(),
        }
    );
    assert_eq!(tokens[0].start.line, 1);
    assert_eq!(tokens[0].start.col, 1);
    assert_eq!(tokens[0].end.col, 5);
}

#[test]
fn malformed_hexadecimal_number_keeps_the_number_diagnostic() {
    let error = tokenize("0X").expect_err("missing hexadecimal digits must fail");

    assert_eq!(error.message, "invalid number '0X'");
    assert_eq!(error.position.line, 1);
    assert_eq!(error.position.col, 1);
}

#[test]
fn malformed_decimal_number_keeps_the_number_diagnostic() {
    let error = tokenize("1.2.3").expect_err("multiple decimal points must fail");

    assert_eq!(error.message, "invalid number '1.2.3'");
    assert_eq!(error.position.line, 1);
    assert_eq!(error.position.col, 1);
}
