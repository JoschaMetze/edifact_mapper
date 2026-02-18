//! Error types for the automapper-validation crate.

/// Errors that can occur during condition expression parsing.
#[derive(Debug, Clone, thiserror::Error)]
pub enum ParseError {
    /// Unexpected token encountered during parsing.
    #[error("unexpected token at position {position}: expected {expected}, found '{found}'")]
    UnexpectedToken {
        position: usize,
        expected: String,
        found: String,
    },

    /// Unmatched closing parenthesis.
    #[error("unmatched closing parenthesis at position {position}")]
    UnmatchedCloseParen { position: usize },

    /// Empty expression after stripping prefix.
    #[error("empty expression after stripping status prefix")]
    EmptyExpression,

    /// Invalid condition reference content.
    #[error("invalid condition reference: '{content}'")]
    InvalidConditionRef { content: String },
}

/// Errors that can occur during validation.
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    /// EDIFACT parse error.
    #[error(transparent)]
    Parse(#[from] edifact_parser::ParseError),

    /// Condition expression parse error.
    #[error("condition expression parse error: {0}")]
    ConditionParse(#[from] ParseError),

    /// Unknown Pruefidentifikator.
    #[error("unknown Pruefidentifikator: '{0}'")]
    UnknownPruefidentifikator(String),

    /// No evaluator registered for message type and format version.
    #[error("no condition evaluator registered for {message_type}/{format_version}")]
    NoEvaluator {
        message_type: String,
        format_version: String,
    },
}
