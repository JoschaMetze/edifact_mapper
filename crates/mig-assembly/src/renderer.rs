//! EDIFACT string renderer from disassembled segments.
//!
//! Converts a list of `DisassembledSegment` values back into a valid
//! EDIFACT string using the provided delimiters.

use crate::disassembler::DisassembledSegment;
use edifact_types::EdifactDelimiters;

/// Render a list of disassembled segments into an EDIFACT string.
///
/// Follows the same rendering rules as `RawSegment::to_raw_string`:
/// - Elements separated by element separator (`+`)
/// - Components separated by component separator (`:`)
/// - Trailing empty elements are trimmed
/// - Each segment terminated by segment terminator (`'`)
pub fn render_edifact(segments: &[DisassembledSegment], delimiters: &EdifactDelimiters) -> String {
    let mut out = String::new();

    for seg in segments {
        render_segment(seg, delimiters, &mut out);
    }

    out
}

fn render_segment(seg: &DisassembledSegment, delimiters: &EdifactDelimiters, out: &mut String) {
    let elem_sep = delimiters.element as char;
    let comp_sep = delimiters.component as char;
    let seg_term = delimiters.segment as char;

    out.push_str(&seg.tag);

    for element in &seg.elements {
        out.push(elem_sep);
        // Preserve ALL components including trailing empty ones for roundtrip fidelity.
        for (j, component) in element.iter().enumerate() {
            if j > 0 {
                out.push(comp_sep);
            }
            out.push_str(component);
        }
    }

    out.push(seg_term);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_segments_to_edifact() {
        let segments = vec![
            DisassembledSegment {
                tag: "UNH".to_string(),
                elements: vec![
                    vec!["1".to_string()],
                    vec!["UTILMD".to_string(), "D".to_string(), "11A".to_string()],
                ],
            },
            DisassembledSegment {
                tag: "BGM".to_string(),
                elements: vec![vec!["E01".to_string()]],
            },
        ];

        let delimiters = EdifactDelimiters::default();
        let rendered = render_edifact(&segments, &delimiters);

        assert_eq!(rendered, "UNH+1+UTILMD:D:11A'BGM+E01'");
    }

    #[test]
    fn test_render_empty_segments() {
        let delimiters = EdifactDelimiters::default();
        let rendered = render_edifact(&[], &delimiters);
        assert_eq!(rendered, "");
    }

    #[test]
    fn test_render_segment_with_empty_components() {
        let segments = vec![DisassembledSegment {
            tag: "CAV".to_string(),
            elements: vec![vec![
                "SA".to_string(),
                String::new(),
                String::new(),
                String::new(),
            ]],
        }];

        let delimiters = EdifactDelimiters::default();
        let rendered = render_edifact(&segments, &delimiters);

        // Trailing empty components should be preserved
        assert_eq!(rendered, "CAV+SA:::'");
    }

    #[test]
    fn test_render_multiple_elements() {
        let segments = vec![DisassembledSegment {
            tag: "DTM".to_string(),
            elements: vec![vec![
                "137".to_string(),
                "20250101".to_string(),
                "102".to_string(),
            ]],
        }];

        let delimiters = EdifactDelimiters::default();
        let rendered = render_edifact(&segments, &delimiters);

        assert_eq!(rendered, "DTM+137:20250101:102'");
    }
}
