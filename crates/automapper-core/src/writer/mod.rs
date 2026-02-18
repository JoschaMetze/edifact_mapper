//! EDIFACT writer layer for serializing domain objects back to EDIFACT format.
//!
//! - `EdifactSegmentWriter` — builds individual segment strings from field data
//! - `EdifactDocumentWriter` — manages full interchange structure (UNA/UNB...UNZ)
//! - Entity writers — serialize domain objects using the segment writer

pub mod document_writer;
pub mod entity_writers;
pub mod segment_writer;

pub use document_writer::EdifactDocumentWriter;
pub use entity_writers::*;
pub use segment_writer::EdifactSegmentWriter;
