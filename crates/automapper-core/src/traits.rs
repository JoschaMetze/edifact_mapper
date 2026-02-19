//! Core trait definitions for the automapper pipeline.
//!
//! These traits mirror the C# `ISegmentHandler`, `IBuilder<T>`, `IEntityWriter`,
//! and `IMapper` interfaces. They define the contract for bidirectional
//! EDIFACT <-> BO4E mapping.

use edifact_types::RawSegment;

use crate::context::TransactionContext;

/// Reads EDIFACT segments into domain objects.
///
/// Implementations handle specific segment types (e.g., LOC+Z16 for Marktlokation).
/// The coordinator routes segments to handlers via `can_handle()` checks.
///
/// Mirrors C# `ISegmentHandler`.
pub trait SegmentHandler: Send {
    /// Determines whether this handler can process the given segment.
    ///
    /// Typically checks the segment ID and qualifier (e.g., `segment.is("LOC")`
    /// and the first element equals "Z16").
    fn can_handle(&self, segment: &RawSegment) -> bool;

    /// Processes the segment, accumulating state for later building.
    ///
    /// Called by the coordinator for every segment where `can_handle()` returned true.
    fn handle(&mut self, segment: &RawSegment, ctx: &mut TransactionContext);
}

/// Accumulates state across multiple segments and builds a domain object.
///
/// Builders are used by mappers to collect data from multiple segments
/// and produce a single business object once all relevant segments are processed.
///
/// Mirrors C# `IBuilder<T>` / `BusinessObjectBuilder<T>`.
pub trait Builder<T>: Send {
    /// Returns true if no data has been accumulated.
    fn is_empty(&self) -> bool;

    /// Consumes the accumulated state and produces the domain object.
    ///
    /// After calling `build()`, the builder should be considered consumed.
    /// Call `reset()` to reuse the builder for another object.
    fn build(&mut self) -> T;

    /// Resets the builder to its initial empty state for reuse.
    fn reset(&mut self);
}

/// Serializes domain objects back to EDIFACT segments.
///
/// Entity writers are the reverse of segment handlers: they take a domain
/// object and produce EDIFACT segment data. Used for the generation
/// (BO4E -> EDIFACT) direction.
///
/// Mirrors C# `IEntityWriter<TEntity>`.
pub trait EntityWriter: Send {
    /// Writes the entity as EDIFACT segments.
    ///
    /// The writer appends segments to the provided segment buffer. The
    /// `TransactionContext` provides shared state needed during writing
    /// (e.g., Zeitscheibe references, format version).
    fn write(&self, segments: &mut Vec<Vec<Vec<String>>>, ctx: &TransactionContext);
}

/// Bidirectional mapper combining reading + writing for one entity type.
///
/// A `Mapper` implements both `SegmentHandler` (for parsing) and `EntityWriter`
/// (for generation). This ensures that every entity that can be parsed can
/// also be written back, supporting roundtrip fidelity.
///
/// Mirrors C# `IMapper`.
pub trait Mapper: SegmentHandler + EntityWriter {
    /// Returns the format version this mapper targets.
    fn format_version(&self) -> FormatVersion;
}

/// EDIFACT format version identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FormatVersion {
    /// Format version April 2025.
    FV2504,
    /// Format version October 2025.
    FV2510,
}

impl FormatVersion {
    /// Returns the string representation (e.g., "FV2504").
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::FV2504 => "FV2504",
            Self::FV2510 => "FV2510",
        }
    }

    /// Returns the EDIFACT message type identifier for UNH segments.
    ///
    /// Derived from MIG XML: S_UNH / C_S009 composites.
    /// - FV2504 → S2.1 (Versionsnummer from FV2504 MIG)
    /// - FV2510 → S2.2 (Versionsnummer from FV2510 MIG)
    pub fn message_type_string(&self) -> &'static str {
        match self {
            Self::FV2504 => "UTILMD:D:11A:UN:S2.1",
            Self::FV2510 => "UTILMD:D:11A:UN:S2.2",
        }
    }
}

impl std::str::FromStr for FormatVersion {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "FV2504" => Ok(Self::FV2504),
            "FV2510" => Ok(Self::FV2510),
            _ => Err(format!("unknown format version: {s}")),
        }
    }
}

impl std::fmt::Display for FormatVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use edifact_types::SegmentPosition;

    // --- Test helpers ---

    struct TestHandler {
        handled_ids: Vec<String>,
    }

    impl TestHandler {
        fn new() -> Self {
            Self {
                handled_ids: Vec::new(),
            }
        }
    }

    impl SegmentHandler for TestHandler {
        fn can_handle(&self, segment: &RawSegment) -> bool {
            segment.id == "LOC"
        }

        fn handle(&mut self, segment: &RawSegment, _ctx: &mut TransactionContext) {
            self.handled_ids.push(segment.id.to_string());
        }
    }

    struct TestBuilder {
        value: Option<String>,
    }

    impl TestBuilder {
        fn new() -> Self {
            Self { value: None }
        }
    }

    impl Builder<String> for TestBuilder {
        fn is_empty(&self) -> bool {
            self.value.is_none()
        }

        fn build(&mut self) -> String {
            self.value.take().unwrap_or_default()
        }

        fn reset(&mut self) {
            self.value = None;
        }
    }

    // --- Tests ---

    #[test]
    fn test_segment_handler_can_handle() {
        let handler = TestHandler::new();
        let pos = SegmentPosition::new(1, 0, 1);
        let loc = RawSegment::new("LOC", vec![], pos);
        let bgm = RawSegment::new("BGM", vec![], pos);

        assert!(handler.can_handle(&loc));
        assert!(!handler.can_handle(&bgm));
    }

    #[test]
    fn test_segment_handler_handle() {
        let mut handler = TestHandler::new();
        let mut ctx = TransactionContext::new("FV2504");
        let pos = SegmentPosition::new(1, 0, 1);
        let seg = RawSegment::new("LOC", vec![], pos);

        handler.handle(&seg, &mut ctx);
        assert_eq!(handler.handled_ids, vec!["LOC"]);
    }

    #[test]
    fn test_builder_lifecycle() {
        let mut builder = TestBuilder::new();
        assert!(builder.is_empty());

        builder.value = Some("test".to_string());
        assert!(!builder.is_empty());

        let result = builder.build();
        assert_eq!(result, "test");

        // After build, value is consumed
        assert!(builder.is_empty());
    }

    #[test]
    fn test_builder_reset() {
        let mut builder = TestBuilder::new();
        builder.value = Some("data".to_string());

        builder.reset();
        assert!(builder.is_empty());
        assert_eq!(builder.build(), "");
    }

    #[test]
    fn test_format_version_as_str() {
        assert_eq!(FormatVersion::FV2504.as_str(), "FV2504");
        assert_eq!(FormatVersion::FV2510.as_str(), "FV2510");
    }

    #[test]
    fn test_format_version_from_str() {
        assert_eq!(
            "FV2504".parse::<FormatVersion>().unwrap(),
            FormatVersion::FV2504
        );
        assert_eq!(
            "FV2510".parse::<FormatVersion>().unwrap(),
            FormatVersion::FV2510
        );
        assert!("FV9999".parse::<FormatVersion>().is_err());
        assert!("".parse::<FormatVersion>().is_err());
    }

    #[test]
    fn test_format_version_message_type_string() {
        assert_eq!(
            FormatVersion::FV2504.message_type_string(),
            "UTILMD:D:11A:UN:S2.1"
        );
        assert_eq!(
            FormatVersion::FV2510.message_type_string(),
            "UTILMD:D:11A:UN:S2.2"
        );
    }

    #[test]
    fn test_format_version_display() {
        assert_eq!(format!("{}", FormatVersion::FV2504), "FV2504");
        assert_eq!(format!("{}", FormatVersion::FV2510), "FV2510");
    }

    #[test]
    fn test_format_version_equality() {
        assert_eq!(FormatVersion::FV2504, FormatVersion::FV2504);
        assert_ne!(FormatVersion::FV2504, FormatVersion::FV2510);
    }

    #[test]
    fn test_segment_handler_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<TestHandler>();
    }

    #[test]
    fn test_builder_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<TestBuilder>();
    }
}
