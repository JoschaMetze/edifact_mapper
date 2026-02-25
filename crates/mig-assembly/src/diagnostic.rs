//! Structure diagnostics emitted during MIG-guided assembly.

use serde::{Deserialize, Serialize};

/// A structure-level issue found during MIG-guided assembly.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructureDiagnostic {
    pub kind: StructureDiagnosticKind,
    pub segment_id: String,
    pub position: usize,
    pub message: String,
}

/// Classification of structure-level diagnostic issues.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StructureDiagnosticKind {
    /// A segment appeared where it was not expected by the MIG schema.
    UnexpectedSegment,
    /// A mandatory segment defined in the MIG schema was not found.
    MissingRequiredSegment,
    /// A segment or group exceeded its maximum allowed repetitions.
    MaxRepetitionsExceeded,
    /// A qualifier value was not recognized for the current MIG context.
    UnrecognizedQualifier,
}

impl std::fmt::Display for StructureDiagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{:?}] {} at position {}: {}",
            self.kind, self.segment_id, self.position, self.message
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_format_includes_all_fields() {
        let diag = StructureDiagnostic {
            kind: StructureDiagnosticKind::UnexpectedSegment,
            segment_id: "BGM".to_string(),
            position: 3,
            message: "BGM not expected after UNH".to_string(),
        };
        let display = diag.to_string();
        assert!(
            display.contains("BGM"),
            "display should contain segment_id"
        );
        assert!(
            display.contains("3"),
            "display should contain position"
        );
        assert!(
            display.contains("BGM not expected after UNH"),
            "display should contain message"
        );
        assert!(
            display.contains("UnexpectedSegment"),
            "display should contain kind"
        );
    }

    #[test]
    fn display_format_missing_required() {
        let diag = StructureDiagnostic {
            kind: StructureDiagnosticKind::MissingRequiredSegment,
            segment_id: "IDE".to_string(),
            position: 5,
            message: "mandatory IDE segment missing in SG4".to_string(),
        };
        let display = diag.to_string();
        assert_eq!(
            display,
            "[MissingRequiredSegment] IDE at position 5: mandatory IDE segment missing in SG4"
        );
    }

    #[test]
    fn serialization_roundtrip() {
        let diag = StructureDiagnostic {
            kind: StructureDiagnosticKind::MaxRepetitionsExceeded,
            segment_id: "RFF".to_string(),
            position: 12,
            message: "RFF exceeded max repetitions of 5".to_string(),
        };
        let json = serde_json::to_string(&diag).expect("serialize");
        let roundtripped: StructureDiagnostic =
            serde_json::from_str(&json).expect("deserialize");
        assert_eq!(roundtripped.kind, StructureDiagnosticKind::MaxRepetitionsExceeded);
        assert_eq!(roundtripped.segment_id, "RFF");
        assert_eq!(roundtripped.position, 12);
        assert_eq!(roundtripped.message, "RFF exceeded max repetitions of 5");
    }

    #[test]
    fn serialization_roundtrip_unrecognized_qualifier() {
        let diag = StructureDiagnostic {
            kind: StructureDiagnosticKind::UnrecognizedQualifier,
            segment_id: "LOC".to_string(),
            position: 7,
            message: "qualifier Z99 not recognized for LOC in SG5".to_string(),
        };
        let json = serde_json::to_string(&diag).expect("serialize");
        let roundtripped: StructureDiagnostic =
            serde_json::from_str(&json).expect("deserialize");
        assert_eq!(roundtripped.kind, StructureDiagnosticKind::UnrecognizedQualifier);
        assert_eq!(roundtripped.segment_id, "LOC");
        assert_eq!(roundtripped.position, 7);
        assert_eq!(
            roundtripped.message,
            "qualifier Z99 not recognized for LOC in SG5"
        );
    }
}
