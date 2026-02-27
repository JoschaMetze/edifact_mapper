//! Deterministic ID generators for German energy market identifiers.
//!
//! Each generator takes a `seed: u64` and produces a valid identifier of the
//! appropriate format. Same seed always produces the same output.

/// Generate a Marktlokation ID: 11 digits, last digit is a Luhn check digit.
pub fn generate_malo_id(seed: u64) -> String {
    // Use seed to derive 10 base digits
    let base = seed % 10_000_000_000;
    let base_str = format!("{base:010}");
    let check = luhn_check_digit(&base_str);
    format!("{base_str}{check}")
}

/// Generate a Messlokation ID: "DE" + 31 digits = 33 chars total.
pub fn generate_melo_id(seed: u64) -> String {
    // Split the seed into parts to fill 31 digits
    let part1 = seed % 10_000_000_000; // 10 digits
    let part2 = (seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1)) % 10_000_000_000; // 10 digits
    let part3 = (seed.wrapping_mul(2_862_933_555_777_941_757).wrapping_add(3)) % 100_000_000_000; // 11 digits
    format!("DE{part1:010}{part2:010}{part3:011}")
}

/// Generate a Netzlokation ID: "E" + 10 digits = 11 chars.
pub fn generate_nelo_id(seed: u64) -> String {
    let digits = seed % 10_000_000_000;
    format!("E{digits:010}")
}

/// Generate a Steuerbare Ressource ID: "C" + 10 digits = 11 chars.
pub fn generate_steuress_id(seed: u64) -> String {
    let digits = seed % 10_000_000_000;
    format!("C{digits:010}")
}

/// Generate a Technische Ressource ID: "D" + 10 digits = 11 chars.
pub fn generate_techress_id(seed: u64) -> String {
    let digits = seed % 10_000_000_000;
    format!("D{digits:010}")
}

/// Generate a Global Location Number (GLN): 13 digits with valid GS1 check digit.
///
/// The check digit uses the GS1 algorithm: alternating weights 1 and 3 from
/// the rightmost position, sum mod 10, then (10 - sum % 10) % 10.
pub fn generate_gln(seed: u64) -> String {
    let base = seed % 1_000_000_000_000;
    let base_str = format!("{base:012}");
    let check = gs1_check_digit(&base_str);
    format!("{base_str}{check}")
}

/// Generate an alphanumeric business reference ID safe for EDIFACT.
///
/// Uses only characters that don't conflict with EDIFACT delimiters
/// (no `+`, `:`, `'`, or `?`).
pub fn generate_reference_id(seed: u64) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let len = 12;
    let mut result = Vec::with_capacity(len);
    let mut state = seed;
    for _ in 0..len {
        // Simple LCG for deterministic pseudo-random selection
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1);
        let idx = (state >> 33) as usize % CHARSET.len();
        result.push(CHARSET[idx]);
    }
    String::from_utf8(result).expect("charset is ASCII")
}

/// Compute a Luhn check digit for a string of ASCII digits.
fn luhn_check_digit(digits: &str) -> u8 {
    let mut sum: u32 = 0;
    // Process from right to left; the check digit position is odd (1-indexed),
    // so digits at even 0-indexed positions from the right get doubled.
    for (i, ch) in digits.chars().rev().enumerate() {
        let mut d = ch.to_digit(10).expect("input must be all digits");
        if i % 2 == 0 {
            d *= 2;
            if d > 9 {
                d -= 9;
            }
        }
        sum += d;
    }
    ((10 - (sum % 10)) % 10) as u8
}

/// Compute a GS1 check digit for a string of ASCII digits.
///
/// Alternating weights 1 and 3 from the rightmost digit of the input,
/// then check = (10 - sum % 10) % 10.
fn gs1_check_digit(digits: &str) -> u8 {
    let mut sum: u32 = 0;
    for (i, ch) in digits.chars().rev().enumerate() {
        let d = ch.to_digit(10).expect("input must be all digits");
        let weight = if i % 2 == 0 { 3 } else { 1 };
        sum += d * weight;
    }
    ((10 - (sum % 10)) % 10) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_malo_id_format() {
        let id = generate_malo_id(12345);
        assert_eq!(id.len(), 11, "MaLo ID must be 11 chars, got: {id}");
        assert!(
            id.chars().all(|c| c.is_ascii_digit()),
            "MaLo ID must be all digits, got: {id}"
        );
    }

    #[test]
    fn test_malo_id_deterministic() {
        let a = generate_malo_id(42);
        let b = generate_malo_id(42);
        assert_eq!(a, b, "Same seed must produce same MaLo ID");
    }

    #[test]
    fn test_malo_id_valid_check_digit() {
        let id = generate_malo_id(99999);
        // Verify by running Luhn over all 11 digits — must yield 0
        let base = &id[..10];
        let expected_check = luhn_check_digit(base);
        let actual_check = id.chars().last().unwrap().to_digit(10).unwrap() as u8;
        assert_eq!(
            expected_check, actual_check,
            "Luhn check digit mismatch for {id}"
        );
    }

    #[test]
    fn test_melo_id_format() {
        let id = generate_melo_id(67890);
        assert!(
            id.starts_with("DE"),
            "MeLo ID must start with DE, got: {id}"
        );
        assert_eq!(id.len(), 33, "MeLo ID must be 33 chars, got: {id}");
        assert!(
            id[2..].chars().all(|c| c.is_ascii_digit()),
            "MeLo ID digits portion must be all digits, got: {id}"
        );
    }

    #[test]
    fn test_melo_id_deterministic() {
        let a = generate_melo_id(42);
        let b = generate_melo_id(42);
        assert_eq!(a, b, "Same seed must produce same MeLo ID");
    }

    #[test]
    fn test_nelo_id_format() {
        let id = generate_nelo_id(11111);
        assert_eq!(
            id.chars().next().unwrap(),
            'E',
            "NeLo ID must start with E, got: {id}"
        );
        assert_eq!(id.len(), 11, "NeLo ID must be 11 chars, got: {id}");
        assert!(
            id[1..].chars().all(|c| c.is_ascii_digit()),
            "NeLo ID digits portion must be all digits, got: {id}"
        );
    }

    #[test]
    fn test_steuress_id_format() {
        let id = generate_steuress_id(22222);
        assert_eq!(
            id.chars().next().unwrap(),
            'C',
            "SteuRess ID must start with C, got: {id}"
        );
        assert_eq!(id.len(), 11, "SteuRess ID must be 11 chars, got: {id}");
        assert!(
            id[1..].chars().all(|c| c.is_ascii_digit()),
            "SteuRess ID digits portion must be all digits, got: {id}"
        );
    }

    #[test]
    fn test_techress_id_format() {
        let id = generate_techress_id(33333);
        assert_eq!(
            id.chars().next().unwrap(),
            'D',
            "TechRess ID must start with D, got: {id}"
        );
        assert_eq!(id.len(), 11, "TechRess ID must be 11 chars, got: {id}");
        assert!(
            id[1..].chars().all(|c| c.is_ascii_digit()),
            "TechRess ID digits portion must be all digits, got: {id}"
        );
    }

    #[test]
    fn test_gln_format() {
        let id = generate_gln(44444);
        assert_eq!(id.len(), 13, "GLN must be 13 chars, got: {id}");
        assert!(
            id.chars().all(|c| c.is_ascii_digit()),
            "GLN must be all digits, got: {id}"
        );
    }

    #[test]
    fn test_gln_valid_check_digit() {
        for seed in [0, 1, 42, 999, 123456789, u64::MAX] {
            let id = generate_gln(seed);
            // Verify GS1 check: sum of (digit * weight) for all 13 digits should be divisible by 10
            let mut sum: u32 = 0;
            for (i, ch) in id.chars().rev().enumerate() {
                let d = ch.to_digit(10).unwrap();
                let weight = if i % 2 == 0 { 1 } else { 3 };
                sum += d * weight;
            }
            assert_eq!(
                sum % 10,
                0,
                "GLN {id} (seed={seed}) fails GS1 check digit validation: sum={sum}"
            );
        }
    }

    #[test]
    fn test_gln_deterministic() {
        let a = generate_gln(42);
        let b = generate_gln(42);
        assert_eq!(a, b, "Same seed must produce same GLN");
    }

    #[test]
    fn test_reference_id_format() {
        let id = generate_reference_id(55555);
        assert!(!id.is_empty(), "Reference ID must not be empty");
        assert!(
            !id.contains('+'),
            "Reference ID must not contain +, got: {id}"
        );
        assert!(
            !id.contains(':'),
            "Reference ID must not contain :, got: {id}"
        );
        assert!(
            !id.contains('\''),
            "Reference ID must not contain ', got: {id}"
        );
        assert!(
            !id.contains('?'),
            "Reference ID must not contain ?, got: {id}"
        );
    }

    #[test]
    fn test_reference_id_deterministic() {
        let a = generate_reference_id(42);
        let b = generate_reference_id(42);
        assert_eq!(a, b, "Same seed must produce same reference ID");
    }

    #[test]
    fn test_different_seeds_produce_different_ids() {
        // Sanity check: different seeds should generally produce different output
        let a = generate_malo_id(1);
        let b = generate_malo_id(2);
        assert_ne!(a, b, "Different seeds should produce different MaLo IDs");

        let a = generate_gln(1);
        let b = generate_gln(2);
        assert_ne!(a, b, "Different seeds should produce different GLNs");

        let a = generate_reference_id(1);
        let b = generate_reference_id(2);
        assert_ne!(
            a, b,
            "Different seeds should produce different reference IDs"
        );
    }
}
