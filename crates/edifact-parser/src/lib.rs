//! Streaming EDIFACT tokenizer and SAX-style event-driven parser.
//!
//! This crate provides a standalone EDIFACT parser with no BO4E dependency.
//! It can be used by anyone in the Rust ecosystem for generic EDIFACT parsing.
//!
//! # Architecture
//!
//! The parser uses a SAX-style streaming model:
//! 1. Tokenizer splits raw bytes into segments
//! 2. Parser routes segments to handler callbacks
//! 3. Handler accumulates state as needed

mod error;
mod handler;
mod parser;
mod segment_builder;
mod tokenizer;

pub use error::ParseError;
pub use handler::EdifactHandler;
pub use parser::EdifactStreamParser;
pub use segment_builder::SegmentBuilder;
pub use tokenizer::EdifactTokenizer;
