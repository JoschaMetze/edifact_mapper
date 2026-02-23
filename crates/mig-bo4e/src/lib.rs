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
//! let engine = MappingEngine::load("mappings/FV2504/UTILMD_Strom/pid_55001")?;
//! let def = engine.definition_for_entity("Marktlokation").unwrap();
//! let bo4e = engine.map_forward(&tree, def, 0);
//! let instance = engine.map_reverse(&bo4e, def);
//! ```

pub mod definition;
pub mod engine;
pub mod error;
pub mod handlers;
pub mod model;
pub mod segment_structure;

pub use engine::MappingEngine;
pub use error::MappingError;
pub use handlers::HandlerRegistry;
pub use model::{Interchange, Nachricht, Transaktion};
