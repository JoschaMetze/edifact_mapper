//! MIG-guided EDIFACT tree assembly.
//!
//! Two-pass approach:
//! 1. Tokenize EDIFACT into `Vec<RawSegment>` (existing parser)
//! 2. Assemble segments into typed MIG tree guided by MIG schema
//!
//! # Usage
//! ```ignore
//! let segments = parse_to_segments(input);
//! let tree = assemble_generic(&segments, &mig_schema)?;
//! ```

pub mod assembler;
pub mod cursor;
pub mod diagnostic;
pub mod disassembler;
pub mod error;
pub mod matcher;
pub mod navigator;
pub mod pid_detect;
pub mod pid_filter;
pub mod renderer;
pub mod roundtrip;
pub mod service;
pub mod tokenize;

pub use diagnostic::{StructureDiagnostic, StructureDiagnosticKind};
pub use error::AssemblyError;
pub use navigator::AssembledTreeNavigator;
pub use service::ConversionService;
pub use tokenize::{split_messages, InterchangeChunks, MessageChunk};
