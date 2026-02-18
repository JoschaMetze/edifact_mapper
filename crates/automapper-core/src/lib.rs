//! Core automapper library.
//!
//! Coordinators, mappers, builders, and writers for bidirectional
//! EDIFACT <-> BO4E conversion. Supports streaming parsing with
//! format-version-dispatched mappers and parallel batch processing.

pub mod context;
pub mod coordinator;
pub mod error;
pub mod traits;
pub mod version;

pub use context::TransactionContext;
pub use coordinator::{create_coordinator, detect_format_version, Coordinator};
pub use error::AutomapperError;
pub use traits::*;
pub use version::{VersionConfig, VersionPhantom, FV2504, FV2510};
