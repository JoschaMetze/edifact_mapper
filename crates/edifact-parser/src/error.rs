use edifact_types::SegmentPosition;

/// Errors that can occur during EDIFACT parsing.
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    /// The UNA service string advice header is invalid.
    #[error("invalid UNA header at byte {offset}")]
    InvalidUna { offset: usize },

    /// A segment was not properly terminated.
    #[error("unterminated segment at byte {offset}")]
    UnterminatedSegment { offset: usize },

    /// The input ended unexpectedly.
    #[error("unexpected end of input")]
    UnexpectedEof,

    /// The input contains invalid UTF-8.
    #[error("invalid UTF-8 at byte {offset}: {source}")]
    InvalidUtf8 {
        offset: usize,
        #[source]
        source: std::str::Utf8Error,
    },

    /// A segment ID could not be determined.
    #[error("empty segment ID at byte {offset}")]
    EmptySegmentId { offset: usize },

    /// Handler returned Control::Stop.
    #[error("parsing stopped by handler at {position}")]
    StoppedByHandler { position: SegmentPosition },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_error_display_invalid_una() {
        let err = ParseError::InvalidUna { offset: 0 };
        assert_eq!(err.to_string(), "invalid UNA header at byte 0");
    }

    #[test]
    fn test_parse_error_display_unterminated() {
        let err = ParseError::UnterminatedSegment { offset: 42 };
        assert_eq!(err.to_string(), "unterminated segment at byte 42");
    }

    #[test]
    fn test_parse_error_display_unexpected_eof() {
        let err = ParseError::UnexpectedEof;
        assert_eq!(err.to_string(), "unexpected end of input");
    }

    #[test]
    fn test_parse_error_display_stopped() {
        let err = ParseError::StoppedByHandler {
            position: SegmentPosition::new(3, 100, 1),
        };
        assert_eq!(
            err.to_string(),
            "parsing stopped by handler at segment 3 at byte 100 (message 1)"
        );
    }

    #[test]
    fn test_parse_error_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        // ParseError contains Utf8Error which is Send+Sync
        // This ensures our error type can be used across threads
        assert_send_sync::<ParseError>();
    }
}
