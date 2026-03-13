//! Format validation helpers for AHB 900-series conditions.
//!
//! These validate the FORMAT of data element values (decimal places, numeric ranges,
//! time patterns, ID formats, etc.). They operate on string values extracted from
//! EDIFACT segments and return `ConditionResult`.

use super::evaluator::ConditionResult;

// Re-export timezone helpers so generated code can use `use crate::eval::format_validators::*`
pub use super::timezone::{is_mesz_utc, is_mez_utc};

// --- Decimal/digit place validation ---

/// Validate that a numeric string has at most `max` decimal places.
///
/// Returns `True` if the value has <= max decimal places (or no decimal point),
/// `False` if it has more, `Unknown` if the value is empty.
///
/// Example: `validate_max_decimal_places("123.45", 2)` → True
/// Example: `validate_max_decimal_places("123.456", 2)` → False
/// Example: `validate_max_decimal_places("123", 2)` → True (no decimal → 0 places)
pub fn validate_max_decimal_places(value: &str, max: usize) -> ConditionResult {
    if value.is_empty() {
        return ConditionResult::Unknown;
    }
    let decimal_places = match value.find('.') {
        Some(pos) => value.len() - pos - 1,
        None => 0,
    };
    ConditionResult::from(decimal_places <= max)
}

/// Validate that a numeric string has at most `max` integer digits (before decimal point).
///
/// Ignores leading minus sign.
pub fn validate_max_integer_digits(value: &str, max: usize) -> ConditionResult {
    if value.is_empty() {
        return ConditionResult::Unknown;
    }
    let s = value.strip_prefix('-').unwrap_or(value);
    let integer_part = match s.find('.') {
        Some(pos) => &s[..pos],
        None => s,
    };
    ConditionResult::from(integer_part.len() <= max)
}

// --- Numeric range validation ---

/// Validate a numeric value against a comparison.
///
/// `op` is one of: "==", "!=", ">", ">=", "<", "<="
/// Returns `Unknown` if the value cannot be parsed as a number.
///
/// Example: `validate_numeric(value, ">=", 0.0)` for "Wert >= 0"
/// Example: `validate_numeric(value, "==", 1.0)` for "Wert = 1"
pub fn validate_numeric(value: &str, op: &str, threshold: f64) -> ConditionResult {
    let parsed = match value.parse::<f64>() {
        Ok(v) => v,
        Err(_) => return ConditionResult::Unknown,
    };
    let result = match op {
        "==" => (parsed - threshold).abs() < f64::EPSILON,
        "!=" => (parsed - threshold).abs() >= f64::EPSILON,
        ">" => parsed > threshold,
        ">=" => parsed >= threshold,
        "<" => parsed < threshold,
        "<=" => parsed <= threshold,
        _ => return ConditionResult::Unknown,
    };
    ConditionResult::from(result)
}

// --- DTM time/timezone validation ---

/// Validate that a DTM value's HHMM portion equals the expected value.
///
/// DTM format 303 is CCYYMMDDHHMM (12 chars) or CCYYMMDDHHMMZZZ (15 chars with timezone).
/// Extracts characters at positions 8..12 (HHMM) for comparison.
///
/// Example: `validate_hhmm_equals("202601012200+00", "2200")` → True
pub fn validate_hhmm_equals(dtm_value: &str, expected_hhmm: &str) -> ConditionResult {
    if dtm_value.len() < 12 {
        return ConditionResult::Unknown;
    }
    ConditionResult::from(&dtm_value[8..12] == expected_hhmm)
}

/// Validate that a DTM value's HHMM portion is within a range (inclusive).
///
/// Example: `validate_hhmm_range("202601011530+00", "0000", "2359")` → True
pub fn validate_hhmm_range(dtm_value: &str, min: &str, max: &str) -> ConditionResult {
    if dtm_value.len() < 12 {
        return ConditionResult::Unknown;
    }
    let hhmm = &dtm_value[8..12];
    ConditionResult::from(hhmm >= min && hhmm <= max)
}

/// Validate that a DTM value's MMDDHHMM portion equals the expected value.
///
/// Extracts characters at positions 4..12 for comparison.
///
/// Example: `validate_mmddhhmm_equals("202612312300+00", "12312300")` → True
pub fn validate_mmddhhmm_equals(dtm_value: &str, expected: &str) -> ConditionResult {
    if dtm_value.len() < 12 {
        return ConditionResult::Unknown;
    }
    ConditionResult::from(&dtm_value[4..12] == expected)
}

/// Validate that a DTM value's timezone portion is "+00" (UTC).
///
/// DTM format 303 with timezone: CCYYMMDDHHMM+ZZ or CCYYMMDDHHMM-ZZ (15 chars).
/// Checks that the last 3 characters are "+00".
///
/// Example: `validate_timezone_utc("202601012200+00")` → True
/// Example: `validate_timezone_utc("202601012200+01")` → False
pub fn validate_timezone_utc(dtm_value: &str) -> ConditionResult {
    if dtm_value.len() < 15 {
        return ConditionResult::Unknown;
    }
    ConditionResult::from(&dtm_value[12..] == "+00")
}

// --- Contact format validation ---

/// Validate email format: must contain both '@' and '.'.
///
/// Example: `validate_email("user@example.com")` → True
pub fn validate_email(value: &str) -> ConditionResult {
    if value.is_empty() {
        return ConditionResult::Unknown;
    }
    ConditionResult::from(value.contains('@') && value.contains('.'))
}

/// Validate phone format: must start with '+' followed by only digits.
///
/// Example: `validate_phone("+4930123456")` → True
/// Example: `validate_phone("030123456")` → False
pub fn validate_phone(value: &str) -> ConditionResult {
    if value.is_empty() {
        return ConditionResult::Unknown;
    }
    if !value.starts_with('+') || value.len() < 2 {
        return ConditionResult::from(false);
    }
    ConditionResult::from(value[1..].chars().all(|c| c.is_ascii_digit()))
}

// --- ID format validation ---

/// Validate Marktlokations-ID (MaLo-ID): exactly 11 digits.
///
/// The 11th digit is a check digit (Luhn algorithm modulo 10).
pub fn validate_malo_id(value: &str) -> ConditionResult {
    if value.len() != 11 {
        return ConditionResult::from(false);
    }
    if !value.chars().all(|c| c.is_ascii_digit()) {
        return ConditionResult::from(false);
    }
    // Luhn check digit validation
    let digits: Vec<u32> = value.chars().filter_map(|c| c.to_digit(10)).collect();
    let check = digits[10];
    let mut sum = 0u32;
    for (i, &d) in digits[..10].iter().enumerate() {
        let multiplied = if i % 2 == 0 { d } else { d * 2 };
        sum += if multiplied > 9 {
            multiplied - 9
        } else {
            multiplied
        };
    }
    let expected = (10 - (sum % 10)) % 10;
    ConditionResult::from(check == expected)
}

/// Validate Transaktionsreferenz-ID (TR-ID): 1-35 alphanumeric characters.
pub fn validate_tr_id(value: &str) -> ConditionResult {
    if value.is_empty() {
        return ConditionResult::Unknown;
    }
    ConditionResult::from(value.len() <= 35 && value.chars().all(|c| c.is_ascii_alphanumeric()))
}

/// Validate Steuerbare-Ressource-ID (SR-ID): same format as MaLo-ID (11 digits, Luhn check).
pub fn validate_sr_id(value: &str) -> ConditionResult {
    validate_malo_id(value)
}

/// Validate Zahlpunktbezeichnung: exactly 33 alphanumeric characters.
pub fn validate_zahlpunkt(value: &str) -> ConditionResult {
    if value.len() != 33 {
        return ConditionResult::from(false);
    }
    ConditionResult::from(value.chars().all(|c| c.is_ascii_alphanumeric()))
}

/// Validate either MaLo-ID or Zahlpunktbezeichnung format.
pub fn validate_malo_or_zahlpunkt(value: &str) -> ConditionResult {
    if value.len() == 11 && validate_malo_id(value).is_true() {
        return ConditionResult::True;
    }
    if value.len() == 33 && validate_zahlpunkt(value).is_true() {
        return ConditionResult::True;
    }
    ConditionResult::False
}

// --- Artikelnummer pattern validation ---

/// Validate a dash-separated digit pattern like "n1-n2-n1-n3".
///
/// `segment_lengths` defines expected digit counts per dash-separated segment.
///
/// Example: `validate_artikel_pattern("1-23-4-567", &[1, 2, 1, 3])` → True
/// Example: `validate_artikel_pattern("1-23-4", &[1, 2, 1])` → True
pub fn validate_artikel_pattern(value: &str, segment_lengths: &[usize]) -> ConditionResult {
    if value.is_empty() {
        return ConditionResult::Unknown;
    }
    let parts: Vec<&str> = value.split('-').collect();
    if parts.len() != segment_lengths.len() {
        return ConditionResult::from(false);
    }
    let valid = parts
        .iter()
        .zip(segment_lengths.iter())
        .all(|(part, &expected_len)| {
            part.len() == expected_len && part.chars().all(|c| c.is_ascii_digit())
        });
    ConditionResult::from(valid)
}

// --- General string validation ---

/// Validate exact character length.
pub fn validate_exact_length(value: &str, expected: usize) -> ConditionResult {
    if value.is_empty() {
        return ConditionResult::Unknown;
    }
    ConditionResult::from(value.len() == expected)
}

/// Validate maximum character length.
pub fn validate_max_length(value: &str, max: usize) -> ConditionResult {
    if value.is_empty() {
        return ConditionResult::Unknown;
    }
    ConditionResult::from(value.len() <= max)
}

/// Validate that a string contains only digits (positive integer check).
pub fn validate_all_digits(value: &str) -> ConditionResult {
    if value.is_empty() {
        return ConditionResult::Unknown;
    }
    ConditionResult::from(value.chars().all(|c| c.is_ascii_digit()))
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Decimal places ---

    #[test]
    fn test_max_decimal_places() {
        assert_eq!(
            validate_max_decimal_places("123.45", 2),
            ConditionResult::True
        );
        assert_eq!(
            validate_max_decimal_places("123.456", 2),
            ConditionResult::False
        );
        assert_eq!(validate_max_decimal_places("123", 2), ConditionResult::True);
        assert_eq!(validate_max_decimal_places("0.1", 3), ConditionResult::True);
        assert_eq!(validate_max_decimal_places("", 2), ConditionResult::Unknown);
    }

    #[test]
    fn test_no_decimal_places() {
        assert_eq!(validate_max_decimal_places("100", 0), ConditionResult::True);
        assert_eq!(
            validate_max_decimal_places("100.5", 0),
            ConditionResult::False
        );
    }

    #[test]
    fn test_max_integer_digits() {
        assert_eq!(
            validate_max_integer_digits("1234", 4),
            ConditionResult::True
        );
        assert_eq!(
            validate_max_integer_digits("12345", 4),
            ConditionResult::False
        );
        assert_eq!(
            validate_max_integer_digits("-123.45", 4),
            ConditionResult::True
        );
        assert_eq!(validate_max_integer_digits("", 4), ConditionResult::Unknown);
    }

    // --- Numeric range ---

    #[test]
    fn test_validate_numeric() {
        assert_eq!(validate_numeric("5.0", ">=", 0.0), ConditionResult::True);
        assert_eq!(validate_numeric("-1.0", ">=", 0.0), ConditionResult::False);
        assert_eq!(validate_numeric("1", "==", 1.0), ConditionResult::True);
        assert_eq!(validate_numeric("2", "==", 1.0), ConditionResult::False);
        assert_eq!(validate_numeric("0", ">", 0.0), ConditionResult::False);
        assert_eq!(validate_numeric("1", ">", 0.0), ConditionResult::True);
        assert_eq!(validate_numeric("abc", ">=", 0.0), ConditionResult::Unknown);
    }

    // --- DTM validation ---

    #[test]
    fn test_hhmm_equals() {
        assert_eq!(
            validate_hhmm_equals("202601012200+00", "2200"),
            ConditionResult::True
        );
        assert_eq!(
            validate_hhmm_equals("202601012300+00", "2200"),
            ConditionResult::False
        );
        assert_eq!(
            validate_hhmm_equals("short", "2200"),
            ConditionResult::Unknown
        );
    }

    #[test]
    fn test_hhmm_range() {
        assert_eq!(
            validate_hhmm_range("202601011530+00", "0000", "2359"),
            ConditionResult::True
        );
        assert_eq!(
            validate_hhmm_range("202601010000+00", "0000", "2359"),
            ConditionResult::True
        );
        assert_eq!(
            validate_hhmm_range("202601012359+00", "0000", "2359"),
            ConditionResult::True
        );
    }

    #[test]
    fn test_mmddhhmm_equals() {
        assert_eq!(
            validate_mmddhhmm_equals("202612312300+00", "12312300"),
            ConditionResult::True
        );
        assert_eq!(
            validate_mmddhhmm_equals("202601012200+00", "12312300"),
            ConditionResult::False
        );
    }

    #[test]
    fn test_timezone_utc() {
        assert_eq!(
            validate_timezone_utc("202601012200+00"),
            ConditionResult::True
        );
        assert_eq!(
            validate_timezone_utc("202601012200+01"),
            ConditionResult::False
        );
        assert_eq!(
            validate_timezone_utc("202601012200"),
            ConditionResult::Unknown
        );
    }

    // --- Contact validation ---

    #[test]
    fn test_email() {
        assert_eq!(validate_email("user@example.com"), ConditionResult::True);
        assert_eq!(validate_email("nope"), ConditionResult::False);
        assert_eq!(validate_email("has@but-no-dot"), ConditionResult::False);
        assert_eq!(validate_email(""), ConditionResult::Unknown);
    }

    #[test]
    fn test_phone() {
        assert_eq!(validate_phone("+4930123456"), ConditionResult::True);
        assert_eq!(validate_phone("030123456"), ConditionResult::False);
        assert_eq!(validate_phone("+"), ConditionResult::False);
        assert_eq!(validate_phone("+49 30 123"), ConditionResult::False); // spaces not allowed
        assert_eq!(validate_phone(""), ConditionResult::Unknown);
    }

    // --- ID validation ---

    #[test]
    fn test_malo_id() {
        // Valid MaLo-ID: 50820849851 (example with valid Luhn check)
        // Let's compute: digits 5,0,8,2,0,8,4,9,8,5 → check digit
        // Manual Luhn: pos 0(x1)=5, 1(x2)=0, 2(x1)=8, 3(x2)=4, 4(x1)=0, 5(x2)=16→7, 6(x1)=4, 7(x2)=18→9, 8(x1)=8, 9(x2)=10→1
        // sum = 5+0+8+4+0+7+4+9+8+1 = 46, check = (10 - 46%10) % 10 = 4
        assert_eq!(validate_malo_id("50820849854"), ConditionResult::True);
        assert_eq!(validate_malo_id("50820849855"), ConditionResult::False); // wrong check digit
        assert_eq!(validate_malo_id("1234567890"), ConditionResult::False); // too short
        assert_eq!(validate_malo_id("abcdefghijk"), ConditionResult::False); // not digits
    }

    #[test]
    fn test_zahlpunkt() {
        let valid = "DE0001234567890123456789012345678";
        assert_eq!(valid.len(), 33);
        assert_eq!(validate_zahlpunkt(valid), ConditionResult::True);
        assert_eq!(validate_zahlpunkt("tooshort"), ConditionResult::False);
    }

    // --- Artikelnummer pattern ---

    #[test]
    fn test_artikel_pattern() {
        assert_eq!(
            validate_artikel_pattern("1-23-4", &[1, 2, 1]),
            ConditionResult::True
        );
        assert_eq!(
            validate_artikel_pattern("1-23-4-567", &[1, 2, 1, 3]),
            ConditionResult::True
        );
        assert_eq!(
            validate_artikel_pattern("1-23-4-56", &[1, 2, 1, 3]),
            ConditionResult::False
        );
        assert_eq!(
            validate_artikel_pattern("1-AB-4", &[1, 2, 1]),
            ConditionResult::False
        );
        assert_eq!(
            validate_artikel_pattern("", &[1, 2, 1]),
            ConditionResult::Unknown
        );
    }

    // --- TR-ID / SR-ID validation ---

    #[test]
    fn test_tr_id() {
        assert_eq!(validate_tr_id("ABC123"), ConditionResult::True);
        assert_eq!(validate_tr_id("A"), ConditionResult::True);
        assert_eq!(validate_tr_id(&"A".repeat(35)), ConditionResult::True);
        assert_eq!(validate_tr_id(&"A".repeat(36)), ConditionResult::False);
        assert_eq!(validate_tr_id("has spaces"), ConditionResult::False);
        assert_eq!(validate_tr_id("has-dash"), ConditionResult::False);
        assert_eq!(validate_tr_id(""), ConditionResult::Unknown);
    }

    #[test]
    fn test_sr_id() {
        assert_eq!(validate_sr_id("50820849854"), ConditionResult::True);
        assert_eq!(validate_sr_id("50820849855"), ConditionResult::False);
        assert_eq!(validate_sr_id("1234567890"), ConditionResult::False);
        assert_eq!(validate_sr_id(""), ConditionResult::False);
    }

    // --- String validation ---

    #[test]
    fn test_exact_length() {
        assert_eq!(
            validate_exact_length("1234567890123456", 16),
            ConditionResult::True
        );
        assert_eq!(validate_exact_length("123", 16), ConditionResult::False);
        assert_eq!(validate_exact_length("", 16), ConditionResult::Unknown);
    }

    #[test]
    fn test_all_digits() {
        assert_eq!(validate_all_digits("12345"), ConditionResult::True);
        assert_eq!(validate_all_digits("123a5"), ConditionResult::False);
        assert_eq!(validate_all_digits(""), ConditionResult::Unknown);
    }
}
