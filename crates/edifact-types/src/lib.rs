//! Shared EDIFACT primitive types.
//!
//! This crate defines the core data structures used across the EDIFACT parser
//! and automapper pipeline. It has zero external dependencies.
//!
//! # Types
//!
//! - [`EdifactDelimiters`] — the six delimiter characters
//! - [`SegmentPosition`] — byte offset and segment/message numbering
//! - [`RawSegment`] — zero-copy parsed segment borrowing from the input buffer
//! - [`Control`] — handler flow control (Continue / Stop)

mod control;
mod delimiters;
mod position;
mod segment;

pub use control::Control;
pub use delimiters::{EdifactDelimiters, UnaParseError};
pub use position::SegmentPosition;
pub use segment::RawSegment;
