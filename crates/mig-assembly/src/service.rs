//! High-level conversion service that orchestrates the full pipeline.
//!
//! Loads a MIG schema once and provides methods for converting EDIFACT
//! input to assembled trees (as JSON) or performing roundtrip conversion.

use std::path::Path;

use crate::assembler::Assembler;
use crate::parsing::parse_mig;
use crate::tokenize::parse_to_segments;
use crate::AssemblyError;
use mig_types::schema::mig::MigSchema;

/// High-level service that holds a parsed MIG schema and provides
/// convenient methods for EDIFACT conversion.
pub struct ConversionService {
    mig: MigSchema,
}

impl ConversionService {
    /// Create a new `ConversionService` by loading a MIG XML file.
    pub fn new(
        mig_path: &Path,
        message_type: &str,
        variant: Option<&str>,
        format_version: &str,
    ) -> Result<Self, AssemblyError> {
        let mig = parse_mig(mig_path, message_type, variant, format_version)
            .map_err(|e| AssemblyError::ParseError(e.to_string()))?;
        Ok(Self { mig })
    }

    /// Create a `ConversionService` from an already-parsed MIG schema.
    pub fn from_mig(mig: MigSchema) -> Self {
        Self { mig }
    }

    /// Convert EDIFACT input to an assembled tree, serialized as JSON.
    pub fn convert_to_tree(&self, input: &str) -> Result<serde_json::Value, AssemblyError> {
        let segments = parse_to_segments(input.as_bytes())?;
        let assembler = Assembler::new(&self.mig);
        let tree = assembler.assemble_generic(&segments)?;
        serde_json::to_value(&tree).map_err(|e| AssemblyError::ParseError(e.to_string()))
    }

    /// Convert EDIFACT input to an `AssembledTree` (typed, not JSON).
    pub fn convert_to_assembled_tree(
        &self,
        input: &str,
    ) -> Result<crate::assembler::AssembledTree, AssemblyError> {
        let segments = parse_to_segments(input.as_bytes())?;
        let assembler = Assembler::new(&self.mig);
        assembler.assemble_generic(&segments)
    }

    /// Convert a complete interchange into per-message assembled trees.
    ///
    /// Steps:
    /// 1. Parse input to segments
    /// 2. Split at UNH/UNT boundaries
    /// 3. Assemble each message independently
    ///
    /// Returns the `InterchangeChunks` (for envelope access) and a `Vec<AssembledTree>`
    /// (one per message, in order).
    pub fn convert_interchange_to_trees(
        &self,
        input: &str,
    ) -> Result<
        (
            crate::tokenize::InterchangeChunks,
            Vec<crate::assembler::AssembledTree>,
        ),
        AssemblyError,
    > {
        let segments = parse_to_segments(input.as_bytes())?;
        let chunks = crate::tokenize::split_messages(segments)?;

        let mut trees = Vec::with_capacity(chunks.messages.len());
        for msg in &chunks.messages {
            let all_segments = msg.all_segments();
            let assembler = Assembler::new(&self.mig);
            let tree = assembler.assemble_generic(&all_segments)?;
            trees.push(tree);
        }

        Ok((chunks, trees))
    }

    /// Get a reference to the loaded MIG schema.
    pub fn mig(&self) -> &MigSchema {
        &self.mig
    }
}
