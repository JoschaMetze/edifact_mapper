//! Common traits for MIG tree types.

/// Trait implemented by all generated PID-specific tree types.
///
/// Provides common operations that work across any PID structure:
/// identification, metadata access, and serialization.
pub trait PidTree: std::fmt::Debug + Send + Sync {
    /// The PID identifier (e.g., "55001").
    fn pid_id(&self) -> &str;

    /// Human-readable description of this PID.
    fn beschreibung(&self) -> &str;

    /// Communication direction (e.g., "NB an LF").
    fn kommunikation_von(&self) -> Option<&str>;

    /// The message type (e.g., "UTILMD").
    fn message_type(&self) -> &str;

    /// The format version (e.g., "FV2504").
    fn format_version(&self) -> &str;
}
