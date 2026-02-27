use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum RendererError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("generator error: {0}")]
    Generator(#[from] automapper_generator::GeneratorError),

    #[error("assembly error: {0}")]
    Assembly(#[from] mig_assembly::AssemblyError),

    #[error("mapping error: {0}")]
    Mapping(String),

    #[error("PID {pid} not found in AHB")]
    PidNotFound { pid: String },

    #[error("no messages found in interchange")]
    NoMessages,

    #[error("file not found: {0}")]
    FileNotFound(PathBuf),
}
