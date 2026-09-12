//! Shared number formatting for emission.
//!
//! [`format_number`] renders a Workshop number the way the pinned oracle
//! does for computed values: integers print without a decimal point, and
//! non-integers print the shortest round-trip representation truncated to 16
//! significant digits (OverPy behavior; evidence: the pinned oracle
//! snapshots), unless that truncation would discard a non-zero fractional
//! component. Literal source spellings (e.g. `0.0`, `5.0`) are preserved by
//! the frontends' number nodes and take precedence over this formatter.

/// Format a float like the reference frontend: integers print without a
/// decimal point, and non-integers print the shortest round-trip
/// representation truncated to 16 significant digits (OverPy behavior;
/// evidence: the pinned oracle snapshots). A non-integer keeps its shortest
/// representation when truncation would make the emitted value integral.
pub fn format_number(value: f64) -> String {
    if !value.is_finite() {
        return format!("{value}");
    }
    if value == 0.0 {
        return "0".to_string();
    }
    if value.fract() == 0.0 && value.abs() < 1e15 {
        return format!("{}", value as i64);
    }
    let shortest = format!("{value}");
    let truncated = truncate_significant(&shortest, 16);
    if value.fract() != 0.0
        && truncated
            .parse::<f64>()
            .is_ok_and(|formatted| formatted.fract() == 0.0)
    {
        shortest
    } else {
        truncated
    }
}

/// Keep at most `max_digits` significant digits of a decimal string,
/// truncating (not rounding) and expanding any exponent form.
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
    fn near_integer_non_integers_keep_their_fractional_value() {
        let value = 1.0000000000000004;
        let text = format_number(value);

        assert_eq!(text, "1.0000000000000004");
        assert_eq!(text.parse::<f64>(), Ok(value));
    }
}
