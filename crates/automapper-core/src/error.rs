//! Error types for the automapper-core crate.

use edifact_parser::ParseError;

/// Errors that can occur during automapping operations.
#[derive(Debug, thiserror::Error)]
pub enum AutomapperError {
    /// A parse error occurred in the EDIFACT parser.
    #[error(transparent)]
    Parse(#[from] ParseError),

    /// An unknown format version was specified.
    #[error("unknown format version: {0}")]
    UnknownFormatVersion(String),

    /// A mapping error occurred while processing a segment.
    #[error("mapping error in {segment} at position {position}: {message}")]
    Mapping {
        segment: String,
        position: u32,
        message: String,
    },

    /// A roundtrip mismatch was detected.
    #[error("roundtrip mismatch: {message}")]
    RoundtripMismatch { message: String },

    /// A required field was missing during building.
    #[error("missing required field '{field}' in {entity}")]
    MissingField { entity: String, field: String },

    /// A writer error occurred during EDIFACT generation.
    #[error("writer error: {message}")]
    WriterError { message: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_automapper_error_display_unknown_version() {
        let err = AutomapperError::UnknownFormatVersion("FV9999".to_string());
        assert_eq!(err.to_string(), "unknown format version: FV9999");
    }

    #[test]
    fn test_automapper_error_display_mapping() {
        let err = AutomapperError::Mapping {
            segment: "LOC".to_string(),
            position: 42,
            message: "invalid qualifier".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "mapping error in LOC at position 42: invalid qualifier"
        );
    }

    #[test]
    fn test_automapper_error_display_roundtrip() {
        let err = AutomapperError::RoundtripMismatch {
            message: "segment count differs".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "roundtrip mismatch: segment count differs"
        );
    }

    #[test]
    fn test_automapper_error_display_missing_field() {
        let err = AutomapperError::MissingField {
            entity: "Marktlokation".to_string(),
            field: "marktlokations_id".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "missing required field 'marktlokations_id' in Marktlokation"
        );
    }

    #[test]
    fn test_automapper_error_display_writer() {
        let err = AutomapperError::WriterError {
            message: "buffer full".to_string(),
        };
        assert_eq!(err.to_string(), "writer error: buffer full");
    }

    #[test]
    fn test_automapper_error_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<AutomapperError>();
    }
}
