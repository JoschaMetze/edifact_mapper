use thiserror::Error;

#[derive(Error, Debug)]
pub enum AssemblyError {
    #[error(
        "Unexpected segment '{segment_id}' at position {position}, expected one of: {expected:?}"
    )]
    UnexpectedSegment {
        segment_id: String,
        position: usize,
        expected: Vec<String>,
    },

    #[error("Missing mandatory segment '{segment_id}' for PID {pid}")]
    MissingMandatory { segment_id: String, pid: String },

    #[error("Unknown PID: {0}")]
    UnknownPid(String),

    #[error("PID detection failed: could not determine PID from segments")]
    PidDetectionFailed,

    #[error("Segment cursor out of bounds at position {0}")]
    CursorOutOfBounds(usize),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Expected segment '{expected}' not found")]
    SegmentNotFound { expected: String },
}

impl From<mig_types::cursor::SegmentNotFound> for AssemblyError {
    fn from(e: mig_types::cursor::SegmentNotFound) -> Self {
        AssemblyError::SegmentNotFound {
            expected: e.expected,
        }
    }
}
