//! PID detection from EDIFACT segments.
//!
//! Determines the Pruefidentifikator (PID) from a list of parsed EDIFACT
//! segments. For UTILMD messages, the PID is encoded in a combination of
//! the BGM document name code and the RFF+Z13 reference in SG6.

use crate::tokenize::OwnedSegment;
use crate::AssemblyError;

/// Detect the PID (Pruefidentifikator) from a list of parsed EDIFACT segments.
///
/// For UTILMD messages, the PID is determined by examining:
/// - BGM document name code (first element, first component — e.g., "E01")
/// - RFF+Z13 references in SG6 which directly contain the PID number
///
/// Returns the PID as a string (e.g., "55001").
pub fn detect_pid(segments: &[OwnedSegment]) -> Result<String, AssemblyError> {
    // Strategy 1: Look for RFF+Z13 which directly contains the PID number
    // In UTILMD, the PID reference is in SG6 as RFF+Z13:<pid_number>
    for seg in segments {
        if seg.is("RFF") {
            let qualifier = seg.get_component(0, 0);
            let reference = seg.get_component(0, 1);
            if qualifier == "Z13" && !reference.is_empty() {
                return Ok(reference.to_string());
            }
        }
    }

    // Strategy 2: Derive PID from BGM document code + STS transaction reason
    let bgm = segments.iter().find(|s| s.is("BGM"));
    let sts = segments.iter().find(|s| s.is("STS"));

    match (bgm, sts) {
        (Some(bgm_seg), Some(sts_seg)) => {
            let doc_code = bgm_seg.get_element(0);
            let reason = sts_seg.get_component(1, 0);
            resolve_utilmd_pid(doc_code, reason)
        }
        (Some(bgm_seg), None) => {
            // Some PIDs can be determined from BGM alone
            let doc_code = bgm_seg.get_element(0);
            resolve_utilmd_pid_from_bgm(doc_code)
        }
        _ => Err(AssemblyError::PidDetectionFailed),
    }
}

/// Resolve PID from BGM document code + STS transaction reason.
///
/// The mapping table is derived from the AHB. Common combinations:
/// - E01 (Anmeldung) + various STS reasons -> 55001-55009
/// - E02 (Abmeldung) + various STS reasons -> 55101-55109
/// - E03 (Bestellung) -> 55201-55209
fn resolve_utilmd_pid(doc_code: &str, reason: &str) -> Result<String, AssemblyError> {
    // For UTILMD, common PID mappings based on BGM doc code + STS reason
    // These are the most common ones; this can be extended as needed
    match (doc_code, reason) {
        ("E01", "Z33") => Ok("55001".to_string()),
        ("E01", "Z34") => Ok("55002".to_string()),
        ("E01", "Z35") => Ok("55003".to_string()),
        ("E02", "Z33") => Ok("55101".to_string()),
        ("E02", "Z34") => Ok("55102".to_string()),
        ("E03", "Z33") => Ok("55201".to_string()),
        _ => Err(AssemblyError::PidDetectionFailed),
    }
}

/// Resolve PID from BGM document code alone (fallback).
fn resolve_utilmd_pid_from_bgm(doc_code: &str) -> Result<String, AssemblyError> {
    match doc_code {
        "E01" => Ok("55001".to_string()),
        "E02" => Ok("55101".to_string()),
        "E03" => Ok("55201".to_string()),
        _ => Err(AssemblyError::PidDetectionFailed),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_segment(id: &str, elements: Vec<Vec<&str>>) -> OwnedSegment {
        OwnedSegment {
            id: id.to_string(),
            elements: elements
                .into_iter()
                .map(|e| e.into_iter().map(|c| c.to_string()).collect())
                .collect(),
            segment_number: 0,
        }
    }

    #[test]
    fn test_detect_pid_from_rff_z13() {
        let segments = vec![
            make_segment("UNH", vec![vec!["001"]]),
            make_segment("BGM", vec![vec!["E01"]]),
            make_segment("RFF", vec![vec!["Z13", "55001"]]),
            make_segment("UNT", vec![vec!["3", "001"]]),
        ];
        let pid = detect_pid(&segments).unwrap();
        assert_eq!(pid, "55001");
    }

    #[test]
    fn test_detect_pid_from_bgm_and_sts() {
        let segments = vec![
            make_segment("UNH", vec![vec!["001"]]),
            make_segment("BGM", vec![vec!["E01"]]),
            make_segment("STS", vec![vec![""], vec!["Z33"]]),
            make_segment("UNT", vec![vec!["3", "001"]]),
        ];
        let pid = detect_pid(&segments).unwrap();
        assert_eq!(pid, "55001");
    }

    #[test]
    fn test_detect_pid_from_bgm_only() {
        let segments = vec![
            make_segment("UNH", vec![vec!["001"]]),
            make_segment("BGM", vec![vec!["E03"]]),
            make_segment("UNT", vec![vec!["2", "001"]]),
        ];
        let pid = detect_pid(&segments).unwrap();
        assert_eq!(pid, "55201");
    }

    #[test]
    fn test_detect_pid_fails_no_bgm() {
        let segments = vec![
            make_segment("UNH", vec![vec!["001"]]),
            make_segment("UNT", vec![vec!["1", "001"]]),
        ];
        let result = detect_pid(&segments);
        assert!(result.is_err());
    }

    #[test]
    fn test_detect_pid_prefers_rff_z13_over_bgm() {
        // If both RFF+Z13 and BGM are present, RFF+Z13 wins
        let segments = vec![
            make_segment("UNH", vec![vec!["001"]]),
            make_segment("BGM", vec![vec!["E01"]]),
            make_segment("STS", vec![vec![""], vec!["Z33"]]),
            make_segment("RFF", vec![vec!["Z13", "99999"]]),
            make_segment("UNT", vec![vec!["4", "001"]]),
        ];
        let pid = detect_pid(&segments).unwrap();
        assert_eq!(pid, "99999"); // RFF+Z13 takes priority
    }

    #[test]
    fn test_detect_pid_from_parsed_edifact() {
        // Test with actual parsed EDIFACT input
        let input = b"UNA:+.? 'UNB+UNOC:3+SENDER+RECEIVER+210101:1200+REF001'UNH+MSG001+UTILMD:D:11A:UN:S2.1'BGM+E01+DOC001'RFF+Z13:55001'UNT+3+MSG001'UNZ+1+REF001'";
        let segments = crate::tokenize::parse_to_segments(input).unwrap();
        let pid = detect_pid(&segments).unwrap();
        assert_eq!(pid, "55001");
    }
}
