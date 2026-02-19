//! Passthrough segment storage for roundtrip fidelity.
//!
//! Segments not handled by any mapper are stored verbatim and replayed
//! during EDIFACT generation to achieve byte-identical output.

use serde::{Deserialize, Serialize};

/// Zone within an EDIFACT transaction where a segment appears.
/// Used to interleave passthrough segments in the correct position during generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SegmentZone {
    /// Message header (before first IDE): DTM, IMD, etc.
    MessageHeader,
    /// Transaction header (after IDE, before LOC): DTM, STS, FTX area
    TransactionHeader,
    /// SG5: LOC area
    Locations,
    /// SG6: RFF area (after LOC, before SEQ)
    References,
    /// SG8: SEQ groups area
    Sequences,
    /// SG12: NAD parties area
    Parties,
}

/// A raw segment preserved for roundtrip fidelity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PassthroughSegment {
    /// The full raw segment text (without terminator), e.g. "CCI+Z30++Z07"
    pub raw: String,
    /// Which zone this segment belongs to
    pub zone: SegmentZone,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_passthrough_segment_serialization() {
        let ps = PassthroughSegment {
            raw: "CCI+Z30++Z07".to_string(),
            zone: SegmentZone::Sequences,
        };
        let json = serde_json::to_string(&ps).unwrap();
        let deserialized: PassthroughSegment = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.raw, "CCI+Z30++Z07");
        assert_eq!(deserialized.zone, SegmentZone::Sequences);
    }

    #[test]
    fn test_segment_zone_all_variants_serialize() {
        let zones = [
            SegmentZone::MessageHeader,
            SegmentZone::TransactionHeader,
            SegmentZone::Locations,
            SegmentZone::References,
            SegmentZone::Sequences,
            SegmentZone::Parties,
        ];
        for zone in &zones {
            let json = serde_json::to_string(zone).unwrap();
            let deserialized: SegmentZone = serde_json::from_str(&json).unwrap();
            assert_eq!(&deserialized, zone);
        }
    }

    #[test]
    fn test_passthrough_segment_clone() {
        let ps = PassthroughSegment {
            raw: "FTX+ACB+++Text".to_string(),
            zone: SegmentZone::TransactionHeader,
        };
        let cloned = ps.clone();
        assert_eq!(cloned.raw, ps.raw);
        assert_eq!(cloned.zone, ps.zone);
    }
}
