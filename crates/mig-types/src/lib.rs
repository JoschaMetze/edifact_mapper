//! Generated MIG-tree types for EDIFACT messages.
//!
//! Types are organized by format version and message type:
//! - `generated::fv2504::utilmd` — UTILMD types for FV2504
//!
//! Each message type module contains:
//! - `segments` — segment structs (SegNad, SegLoc, etc.)
//! - `composites` — composite data element structs
//! - `enums` — code list enums (NadQualifier, LocQualifier, etc.)
//! - `groups` — segment group structs (Sg2Party, Sg8SeqGroup, etc.)
//! - `pids` — per-PID composition structs
//!
//! Also provides crate-independent primitives:
//! - `segment` — `OwnedSegment` for parsed EDIFACT segments
//! - `cursor` — `SegmentCursor` and helpers for sequential segment consumption

pub mod cursor;
pub mod generated;
pub mod navigator;
pub mod segment;
pub mod traits;
