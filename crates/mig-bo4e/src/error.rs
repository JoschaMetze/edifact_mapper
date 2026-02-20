use thiserror::Error;

#[derive(Error, Debug)]
pub enum MappingError {
    #[error("TOML parse error in {file}: {message}")]
    TomlParse { file: String, message: String },

    #[error("Invalid mapping path '{path}' in {file}: {reason}")]
    InvalidPath {
        path: String,
        file: String,
        reason: String,
    },

    #[error("Unknown handler '{name}' referenced in {file}")]
    UnknownHandler { name: String, file: String },

    #[error("Missing required field '{field}' during mapping")]
    MissingField { field: String },

    #[error("Type conversion error: {0}")]
    TypeConversion(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML deserialization error: {0}")]
    Toml(#[from] toml::de::Error),
}
