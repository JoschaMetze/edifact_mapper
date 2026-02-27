use std::path::PathBuf;

/// Errors that can occur during code generation.
#[derive(Debug, thiserror::Error)]
pub enum GeneratorError {
    #[error("XML parsing error in {path}: {message}")]
    XmlParse {
        path: PathBuf,
        message: String,
        #[source]
        source: Option<quick_xml::Error>,
    },

    #[error("missing required attribute '{attribute}' on element '{element}' in {path}")]
    MissingAttribute {
        path: PathBuf,
        element: String,
        attribute: String,
    },

    #[error("invalid element '{element}' in {path} at line {line}")]
    InvalidElement {
        path: PathBuf,
        element: String,
        line: usize,
    },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("template rendering error: {0}")]
    Template(String),

    #[error("claude CLI error: {message}")]
    ClaudeCli { message: String },

    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("validation error: {message}")]
    Validation { message: String },

    #[error("file not found: {0}")]
    FileNotFound(PathBuf),

    #[error("assembly error: {0}")]
    Assembly(#[from] mig_assembly::AssemblyError),
}
