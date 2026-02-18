//! Coordinator discovery types.

use serde::{Deserialize, Serialize};

/// Information about an available coordinator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinatorInfo {
    /// EDIFACT message type (e.g., "UTILMD").
    pub message_type: String,

    /// Human-readable description.
    pub description: String,

    /// Format versions supported by this coordinator.
    pub supported_versions: Vec<String>,
}
