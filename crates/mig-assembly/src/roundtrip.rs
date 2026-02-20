//! Full roundtrip pipeline: EDIFACT → assemble → disassemble → render.
//!
//! Validates that the assembly/disassembly process preserves data by
//! comparing the rendered output with the original input.

use crate::assembler::Assembler;
use crate::disassembler::Disassembler;
use crate::renderer::render_edifact;
use crate::tokenize::parse_to_segments;
use crate::AssemblyError;
use automapper_generator::schema::mig::MigSchema;
use edifact_types::EdifactDelimiters;

/// Perform a full roundtrip: parse EDIFACT, assemble into tree, disassemble, render back.
///
/// Returns the rendered EDIFACT string. If the roundtrip is perfect,
/// the output should be byte-identical to the input.
pub fn roundtrip(input: &[u8], mig: &MigSchema) -> Result<String, AssemblyError> {
    // Detect delimiters from input (UNA or defaults)
    let (has_una, delimiters) = EdifactDelimiters::detect(input);

    // Detect if the input uses newlines after segment terminators
    let seg_term = delimiters.segment as char;
    let input_str = std::str::from_utf8(input)
        .map_err(|e| AssemblyError::ParseError(e.to_string()))?;
    let uses_newlines = detect_newline_style(input_str, seg_term);

    // Pass 1: tokenize
    let segments = parse_to_segments(input)?;

    // Pass 2: assemble
    let assembler = Assembler::new(mig);
    let tree = assembler.assemble_generic(&segments)?;

    // Disassemble
    let disassembler = Disassembler::new(mig);
    let dis_segments = disassembler.disassemble(&tree);

    // Render
    let mut output = String::new();
    if has_una {
        output.push_str(&delimiters.to_una_string());
    }
    let rendered = render_edifact(&dis_segments, &delimiters);

    // Inject newlines if the original used them
    if let Some(newline) = uses_newlines {
        for (i, ch) in rendered.char_indices() {
            output.push(ch);
            if ch == seg_term {
                // Check if this is not the last character
                if i + ch.len_utf8() < rendered.len() {
                    output.push_str(newline);
                }
            }
        }
    } else {
        output.push_str(&rendered);
    }

    Ok(output)
}

/// Detect if the input uses newlines after segment terminators.
/// Returns the newline string if found, None if no newlines.
fn detect_newline_style(input: &str, seg_term: char) -> Option<&'static str> {
    // Look for segment_terminator followed by \r\n or \n
    let pattern_crlf = format!("{}\r\n", seg_term);
    let pattern_lf = format!("{}\n", seg_term);

    if input.contains(&pattern_crlf) {
        Some("\r\n")
    } else if input.contains(&pattern_lf) {
        Some("\n")
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use automapper_generator::schema::mig::{MigSchema, MigSegment, MigSegmentGroup};

    fn make_mig_segment(id: &str) -> MigSegment {
        MigSegment {
            id: id.to_string(),
            name: id.to_string(),
            description: None,
            counter: None,
            level: 0,
            number: None,
            max_rep_std: 1,
            max_rep_spec: 1,
            status_std: Some("M".to_string()),
            status_spec: Some("M".to_string()),
            example: None,
            data_elements: vec![],
            composites: vec![],
        }
    }

    fn make_mig_group(
        id: &str,
        segments: Vec<&str>,
        nested: Vec<MigSegmentGroup>,
    ) -> MigSegmentGroup {
        MigSegmentGroup {
            id: id.to_string(),
            name: id.to_string(),
            description: None,
            counter: None,
            level: 1,
            max_rep_std: 99,
            max_rep_spec: 99,
            status_std: Some("M".to_string()),
            status_spec: Some("M".to_string()),
            segments: segments.into_iter().map(make_mig_segment).collect(),
            nested_groups: nested,
        }
    }

    #[test]
    fn test_roundtrip_minimal_utilmd() {
        let input = b"UNA:+.? 'UNH+1+UTILMD:D:11A:UN:S2.1'BGM+E01+MSG001+9'UNT+3+1'";

        let mig = MigSchema {
            message_type: "UTILMD".to_string(),
            variant: Some("Strom".to_string()),
            version: "S2.1".to_string(),
            publication_date: "2025-03-20".to_string(),
            author: "BDEW".to_string(),
            format_version: "FV2504".to_string(),
            source_file: "test".to_string(),
            segments: vec![
                make_mig_segment("UNH"),
                make_mig_segment("BGM"),
                make_mig_segment("UNT"),
            ],
            segment_groups: vec![],
        };

        let result = roundtrip(input, &mig);
        assert!(result.is_ok(), "Roundtrip failed: {:?}", result.err());

        let output = result.unwrap();
        assert!(output.contains("UNH+1+UTILMD"));
        assert!(output.contains("BGM+E01+MSG001+9"));
        assert!(output.contains("UNT+3+1"));
    }

    #[test]
    fn test_roundtrip_with_groups() {
        let input = b"UNA:+.? 'UNH+1+UTILMD:D:11A:UN:S2.1'BGM+E01'NAD+MS+9900123::293'NAD+MR+9900456::293'UNT+4+1'";

        let mig = MigSchema {
            message_type: "UTILMD".to_string(),
            variant: None,
            version: "S2.1".to_string(),
            publication_date: "".to_string(),
            author: "".to_string(),
            format_version: "FV2504".to_string(),
            source_file: "test".to_string(),
            segments: vec![
                make_mig_segment("UNH"),
                make_mig_segment("BGM"),
            ],
            segment_groups: vec![
                make_mig_group("SG2", vec!["NAD"], vec![]),
                make_mig_group("SG99", vec!["UNT"], vec![]),
            ],
        };

        let result = roundtrip(input, &mig).unwrap();
        assert!(result.contains("NAD+MS+9900123::293"));
        assert!(result.contains("NAD+MR+9900456::293"));
    }

    #[test]
    fn test_roundtrip_preserves_una() {
        let input = b"UNA:+.? 'UNH+1+TEST'";

        let mig = MigSchema {
            message_type: "TEST".to_string(),
            variant: None,
            version: "1".to_string(),
            publication_date: "".to_string(),
            author: "".to_string(),
            format_version: "FV2504".to_string(),
            source_file: "test".to_string(),
            segments: vec![make_mig_segment("UNH")],
            segment_groups: vec![],
        };

        let result = roundtrip(input, &mig).unwrap();
        assert!(result.starts_with("UNA:+.? '"));
    }

    #[test]
    fn test_roundtrip_byte_identical_simple() {
        // This simple case with synthetic MIG should be byte-identical
        let input = b"UNA:+.? 'UNH+1+UTILMD:D:11A:UN:S2.1'BGM+E01+MSG001+9'UNT+3+1'";

        let mig = MigSchema {
            message_type: "UTILMD".to_string(),
            variant: Some("Strom".to_string()),
            version: "S2.1".to_string(),
            publication_date: "2025-03-20".to_string(),
            author: "BDEW".to_string(),
            format_version: "FV2504".to_string(),
            source_file: "test".to_string(),
            segments: vec![
                make_mig_segment("UNH"),
                make_mig_segment("BGM"),
                make_mig_segment("UNT"),
            ],
            segment_groups: vec![],
        };

        let result = roundtrip(input, &mig).unwrap();
        let input_str = std::str::from_utf8(input).unwrap();
        assert_eq!(result, input_str, "Roundtrip should be byte-identical");
    }
}
