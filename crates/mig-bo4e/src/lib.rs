//! Declarative TOML-based MIG-tree <-> BO4E mapping engine.
//!
//! # Architecture
//!
//! - **TOML mapping files** define simple 1:1 field mappings
//! - **Complex handlers** are Rust functions for non-trivial logic
//! - **MappingEngine** loads all definitions and provides bidirectional conversion
//!
//! # Usage
//! ```ignore
//! let engine = MappingEngine::load("mappings/")?;
//! let bo4e = engine.to_bo4e(&assembled_tree)?;
//! let tree = engine.from_bo4e(&bo4e, "55001")?;
//! ```

pub mod definition;
pub mod engine;
pub mod error;
pub mod handlers;

pub use engine::MappingEngine;
pub use error::MappingError;
pub use handlers::HandlerRegistry;
