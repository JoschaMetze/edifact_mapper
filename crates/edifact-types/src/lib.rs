//! Shared EDIFACT primitive types.
//!
//! This crate defines the core data structures used across the EDIFACT parser
//! and automapper pipeline. It has zero external dependencies.
//!
//! # Types
//!
//! - [`EdifactDelimiters`] — the six delimiter characters (component, element, decimal, release, segment, reserved)
//! - [`SegmentPosition`] — byte offset and segment/message numbering
//! - [`RawSegment`] — zero-copy parsed segment borrowing from the input buffer
//! - [`Control`] — handler flow control (Continue / Stop)
