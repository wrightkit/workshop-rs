/// Format a Workshop number to 15 fractional digits unless truncation makes it integral;
/// `Infinity`, `-Infinity`, and `NaN` are emission-only spellings.
pub fn format_number(value: f64) -> String {
    if !value.is_finite() {
        return if value.is_nan() {
            "NaN"
        } else if value.is_sign_negative() {
            "-Infinity"
        } else {
            "Infinity"
        }
        .to_string();
    }
    if value == 0.0 {
        return "0".to_string();
    }
    if value.fract() == 0.0 && value.abs() < 1e15 {
        return format!("{}", value as i64);
    }
    let shortest = format!("{value}");
    if value.fract() == 0.0 {
        return truncate_significant(&shortest, 16);
    }
    let mut truncated = shortest.clone();
    if let Some(point) = truncated.find('.') {
        truncated.truncate(point + 16);
    }
    if truncated
        .parse::<f64>()
        .is_ok_and(|formatted| formatted.fract() == 0.0)
    {
        shortest
    } else {
        truncated
    }
}

/// Format a settings number: the shortest decimal that round-trips, so a
/// value read from a project is written back unchanged.
pub fn format_setting_number(value: f64) -> String {
    if value.fract() == 0.0 && value.abs() < 1e15 {
        return format_number(value);
    }
    format!("{value}")
}

fn truncate_significant(text: &str, max_digits: usize) -> String {
    let (mantissa, exponent) = match text.find('e').or_else(|| text.find('E')) {
        Some(index) => {
            let exponent: i32 = text[index + 1..].parse().unwrap_or(0);
            (&text[..index], exponent)
        }
        None => (text, 0),
    };
    let (sign, mantissa) = mantissa
        .strip_prefix('-')
        .map_or(("", mantissa), |rest| ("-", rest));
    let digits: Vec<char> = mantissa.chars().filter(|c| c.is_ascii_digit()).collect();
    let before_dot = mantissa.find('.').unwrap_or(mantissa.len());
    let point = before_dot as i32 + exponent;
    let first_nonzero = digits
        .iter()
        .position(|c| *c != '0')
        .unwrap_or(digits.len());
    let mut digits = digits;
    if digits.len() - first_nonzero > max_digits {
        digits.truncate(first_nonzero + max_digits);
    }
    let mut out = String::from(sign);
    if point <= 0 {
        out.push_str("0.");
        for _ in 0..(-point) {
            out.push('0');
        }
        for c in &digits {
            out.push(*c);
        }
    } else if point as usize >= digits.len() {
        for c in &digits {
            out.push(*c);
        }
        for _ in 0..(point as usize - digits.len()) {
            out.push('0');
        }
    } else {
        for (index, c) in digits.iter().enumerate() {
            if index == point as usize {
                out.push('.');
            }
            out.push(*c);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::format_number;

    #[test]
    fn integers_print_without_decimals() {
        assert_eq!(format_number(0.0), "0");
        assert_eq!(format_number(100.0), "100");
        assert_eq!(format_number(-3.0), "-3");
        assert_eq!(
            format_number(100_000_000_000_000_032.0),
            "100000000000000000"
        );
    }

    #[test]
    fn floats_match_reference_precision() {
        assert_eq!(format_number(1.8106601717798212), "1.810660171779821");
        assert_eq!(format_number(-1.2803300858899105), "-1.280330085889910");
        assert_eq!(format_number(0.016), "0.016");
        assert_eq!(format_number(0.125), "0.125");
        assert_eq!(format_number(1.5), "1.5");
    }

    #[test]
    fn non_integer_precision_matches_pinned_overpy() {
        // OverPy 9.7.17, gitHead ec763c1d245aa5485ef61af5b2bda4975bce4877.
        for (value, expected) in [
            (100.01666666666667, "100.01666666666667"),
            (100.0 / 3.0, "33.333333333333336"),
            (1_000_000.0 / 7.0, "142857.14285714287"),
            (12.345678901234567, "12.345678901234567"),
            (9.999_999_999_999_998, "9.999999999999998"),
            (1.2345678901234567, "1.234567890123456"),
            (0.012345678901234567, "0.012345678901234"),
            (-12.345678901234567, "-12.345678901234567"),
        ] {
            assert_eq!(format_number(value), expected, "input {value}");
        }
    }

    #[test]
    fn non_finite_values_take_the_reference_spellings() {
        // Pinned OverPy 9.7.10 emits `-Infinity` and `NaN` for `log(0)` and
        // `log(-1)` folds; `String(n)` spells positive infinity `Infinity`.
        assert_eq!(format_number(f64::INFINITY), "Infinity");
        assert_eq!(format_number(f64::NEG_INFINITY), "-Infinity");
        assert_eq!(format_number(f64::NAN), "NaN");
    }

    #[test]
    fn near_integer_non_integers_keep_their_fractional_value() {
        let value = 1.0000000000000004;
        let text = format_number(value);

        assert_eq!(text, "1.0000000000000004");
        assert_eq!(text.parse::<f64>(), Ok(value));
        assert_eq!(format_number(0.000_000_000_000_000_4), "0.0000000000000004");
        assert_eq!(
            format_number(-0.000_000_000_000_000_4),
            "-0.0000000000000004"
        );
    }
}
