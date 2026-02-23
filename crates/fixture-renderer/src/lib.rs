//! Fixture renderer — renders EDIFACT fixtures from BO4E via TOML mappings.
//!
//! Provides two operations:
//! 1. `render_fixture()` — source .edi → forward map → reverse map → rendered .edi
//! 2. `generate_canonical_bo4e()` — source .edi → forward map → .mig.bo.json

pub mod error;
pub mod renderer;

pub use error::RendererError;
pub use renderer::{generate_canonical_bo4e, render_fixture, RenderInput};
