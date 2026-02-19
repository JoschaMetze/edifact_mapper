//! BO4E extension types for EDIFACT mapping.
//!
//! Bridges the standard BO4E types with EDIFACT-specific functional domain data.
//! Provides `WithValidity<T, E>` wrappers, companion `*Edifact` structs,
//! and container types like `UtilmdTransaktion`.

pub mod bo4e_types;
pub mod data_quality;
pub mod edifact_types;
pub mod link_registry;
pub mod passthrough;
pub mod prozessdaten;
pub mod transaction;
pub mod uri;
pub mod with_validity;
pub mod zeitraum;

pub use bo4e_types::*;
pub use data_quality::DataQuality;
pub use edifact_types::*;
pub use link_registry::LinkRegistry;
pub use passthrough::{PassthroughSegment, SegmentZone};
pub use prozessdaten::*;
pub use transaction::*;
pub use uri::Bo4eUri;
pub use with_validity::WithValidity;
pub use zeitraum::Zeitraum;
