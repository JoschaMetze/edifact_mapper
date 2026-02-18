//! Standard error codes for validation issues.
//!
//! Error codes follow a prefix convention:
//! - `STR0xx`: Structure validation
//! - `FMT0xx`: Format validation
//! - `COD0xx`: Code value validation
//! - `AHB0xx`: AHB condition rule validation

/// Standard error codes for validation issues.
pub struct ErrorCodes;

impl ErrorCodes {
    // --- Structure validation (STR001-STR099) ---

    /// A mandatory segment is missing.
    pub const MISSING_MANDATORY_SEGMENT: &'static str = "STR001";

    /// Segment repetitions exceed the MIG MaxRep count.
    pub const MAX_REPETITIONS_EXCEEDED: &'static str = "STR002";

    /// Unexpected segment found (not defined in MIG for this message type).
    pub const UNEXPECTED_SEGMENT: &'static str = "STR003";

    /// Segments are in wrong order according to the MIG.
    pub const WRONG_SEGMENT_ORDER: &'static str = "STR004";

    /// A mandatory segment group is missing.
    pub const MISSING_MANDATORY_GROUP: &'static str = "STR005";

    /// Segment group repetitions exceed MaxRep count.
    pub const GROUP_MAX_REP_EXCEEDED: &'static str = "STR006";

    // --- Format validation (FMT001-FMT099) ---

    /// Value exceeds maximum allowed length.
    pub const VALUE_TOO_LONG: &'static str = "FMT001";

    /// Value does not match required numeric format.
    pub const INVALID_NUMERIC_FORMAT: &'static str = "FMT002";

    /// Value does not match required alphanumeric format.
    pub const INVALID_ALPHANUMERIC_FORMAT: &'static str = "FMT003";

    /// Invalid date/time format.
    pub const INVALID_DATE_FORMAT: &'static str = "FMT004";

    /// Value is shorter than minimum required length.
    pub const VALUE_TOO_SHORT: &'static str = "FMT005";

    /// A required element is empty.
    pub const REQUIRED_ELEMENT_EMPTY: &'static str = "FMT006";

    // --- Code validation (COD001-COD099) ---

    /// Code value is not in the allowed code list for this element.
    pub const INVALID_CODE_VALUE: &'static str = "COD001";

    /// Code value is not allowed for this specific Pruefidentifikator.
    pub const CODE_NOT_ALLOWED_FOR_PID: &'static str = "COD002";

    // --- AHB validation (AHB001-AHB999) ---

    /// A field required by the AHB for this PID is missing.
    pub const MISSING_REQUIRED_FIELD: &'static str = "AHB001";

    /// A field is present but not allowed by the AHB for this PID.
    pub const FIELD_NOT_ALLOWED_FOR_PID: &'static str = "AHB002";

    /// A conditional AHB rule is violated.
    pub const CONDITIONAL_RULE_VIOLATION: &'static str = "AHB003";

    /// The Pruefidentifikator is unknown / not supported.
    pub const UNKNOWN_PRUEFIDENTIFIKATOR: &'static str = "AHB004";

    /// A condition expression could not be fully evaluated (Unknown result).
    pub const CONDITION_UNKNOWN: &'static str = "AHB005";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code_prefixes() {
        assert!(ErrorCodes::MISSING_MANDATORY_SEGMENT.starts_with("STR"));
        assert!(ErrorCodes::VALUE_TOO_LONG.starts_with("FMT"));
        assert!(ErrorCodes::INVALID_CODE_VALUE.starts_with("COD"));
        assert!(ErrorCodes::MISSING_REQUIRED_FIELD.starts_with("AHB"));
    }

    #[test]
    fn test_all_codes_are_unique() {
        let codes = [
            ErrorCodes::MISSING_MANDATORY_SEGMENT,
            ErrorCodes::MAX_REPETITIONS_EXCEEDED,
            ErrorCodes::UNEXPECTED_SEGMENT,
            ErrorCodes::WRONG_SEGMENT_ORDER,
            ErrorCodes::MISSING_MANDATORY_GROUP,
            ErrorCodes::GROUP_MAX_REP_EXCEEDED,
            ErrorCodes::VALUE_TOO_LONG,
            ErrorCodes::INVALID_NUMERIC_FORMAT,
            ErrorCodes::INVALID_ALPHANUMERIC_FORMAT,
            ErrorCodes::INVALID_DATE_FORMAT,
            ErrorCodes::VALUE_TOO_SHORT,
            ErrorCodes::REQUIRED_ELEMENT_EMPTY,
            ErrorCodes::INVALID_CODE_VALUE,
            ErrorCodes::CODE_NOT_ALLOWED_FOR_PID,
            ErrorCodes::MISSING_REQUIRED_FIELD,
            ErrorCodes::FIELD_NOT_ALLOWED_FOR_PID,
            ErrorCodes::CONDITIONAL_RULE_VIOLATION,
            ErrorCodes::UNKNOWN_PRUEFIDENTIFIKATOR,
            ErrorCodes::CONDITION_UNKNOWN,
        ];

        let unique: std::collections::HashSet<&str> = codes.iter().copied().collect();
        assert_eq!(
            codes.len(),
            unique.len(),
            "Duplicate error codes found"
        );
    }
}
