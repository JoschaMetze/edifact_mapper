//! Builds individual EDIFACT segment strings from structured data.
//!
//! Mirrors C# `EdifactSegmentSerializer` and the `SegmentBuilder` used by writers.

use edifact_types::EdifactDelimiters;

/// Builds EDIFACT segment strings from elements and components.
///
/// Handles proper delimiter escaping (release character), trailing
/// empty element suppression, and component separator insertion.
///
/// # Example
///
/// ```ignore
/// let mut writer = EdifactSegmentWriter::new(EdifactDelimiters::default());
/// writer.begin_segment("LOC");
/// writer.add_element("Z16");
/// writer.begin_composite();
/// writer.add_component("DE00014545768S0000000000000003054");
/// writer.end_composite();
/// let segment = writer.end_segment();
/// assert_eq!(segment, "LOC+Z16+DE00014545768S0000000000000003054");
/// ```
pub struct EdifactSegmentWriter {
    delimiters: EdifactDelimiters,
    pub(crate) buffer: String,
    segment_id: Option<String>,
    elements: Vec<String>,
    current_composite: Vec<String>,
    in_composite: bool,
    segment_count: u32,
}

impl EdifactSegmentWriter {
    /// Creates a new segment writer with the given delimiters.
    pub fn new(delimiters: EdifactDelimiters) -> Self {
        Self {
            delimiters,
            buffer: String::new(),
            segment_id: None,
            elements: Vec::new(),
            current_composite: Vec::new(),
            in_composite: false,
            segment_count: 0,
        }
    }

    /// Creates a writer with default delimiters.
    pub fn with_defaults() -> Self {
        Self::new(EdifactDelimiters::default())
    }

    /// Returns the total number of segments written.
    pub fn segment_count(&self) -> u32 {
        self.segment_count
    }

    /// Returns the accumulated output buffer.
    pub fn output(&self) -> &str {
        &self.buffer
    }

    /// Consumes the writer and returns the output buffer.
    pub fn into_output(self) -> String {
        self.buffer
    }

    /// Begins a new segment with the given ID.
    pub fn begin_segment(&mut self, id: &str) {
        self.segment_id = Some(id.to_string());
        self.elements.clear();
        self.current_composite.clear();
        self.in_composite = false;
    }

    /// Adds a simple (non-composite) element.
    pub fn add_element(&mut self, value: &str) {
        self.elements.push(self.escape(value));
    }

    /// Adds a pre-formatted element without escaping.
    ///
    /// Use this for values that already contain proper EDIFACT formatting,
    /// e.g. composite identifiers like `UTILMD:D:11A:UN:S2.1`.
    pub fn add_raw_element(&mut self, value: &str) {
        self.elements.push(value.to_string());
    }

    /// Adds an empty element placeholder.
    pub fn add_empty_element(&mut self) {
        self.elements.push(String::new());
    }

    /// Begins a composite element.
    pub fn begin_composite(&mut self) {
        self.current_composite.clear();
        self.in_composite = true;
    }

    /// Adds a component to the current composite.
    pub fn add_component(&mut self, value: &str) {
        if self.in_composite {
            self.current_composite.push(self.escape(value));
        }
    }

    /// Adds an empty component to the current composite.
    pub fn add_empty_component(&mut self) {
        if self.in_composite {
            self.current_composite.push(String::new());
        }
    }

    /// Ends the current composite element and adds it to the segment.
    pub fn end_composite(&mut self) {
        if self.in_composite {
            // Strip trailing empty components
            while self.current_composite.last().is_some_and(|c| c.is_empty()) {
                self.current_composite.pop();
            }
            let component_sep = self.delimiters.component as char;
            let composite = self.current_composite.join(&component_sep.to_string());
            self.elements.push(composite);
            self.in_composite = false;
        }
    }

    /// Ends the current segment, appends it to the output buffer,
    /// and returns the segment string (without terminator).
    pub fn end_segment(&mut self) -> String {
        let id = self.segment_id.take().unwrap_or_default();

        // Strip trailing empty elements
        while self.elements.last().is_some_and(|e| e.is_empty()) {
            self.elements.pop();
        }

        let element_sep = self.delimiters.element as char;
        let segment_term = self.delimiters.segment as char;

        let segment = if self.elements.is_empty() {
            id.clone()
        } else {
            format!(
                "{}{}{}",
                id,
                element_sep,
                self.elements.join(&element_sep.to_string())
            )
        };

        // Append to buffer with segment terminator
        self.buffer.push_str(&segment);
        self.buffer.push(segment_term);

        self.segment_count += 1;
        self.elements.clear();

        segment
    }

    /// Writes a pre-formatted segment string directly.
    pub fn write_raw(&mut self, segment: &str) {
        let segment_term = self.delimiters.segment as char;
        self.buffer.push_str(segment);
        self.buffer.push(segment_term);
        self.segment_count += 1;
    }

    /// Escapes special characters in a value using the release character.
    fn escape(&self, value: &str) -> String {
        let release = self.delimiters.release as char;
        let specials = [
            self.delimiters.component as char,
            self.delimiters.element as char,
            self.delimiters.segment as char,
            release,
        ];

        let mut result = String::with_capacity(value.len());
        for ch in value.chars() {
            if specials.contains(&ch) {
                result.push(release);
            }
            result.push(ch);
        }
        result
    }

    /// Resets the writer, clearing the output buffer and segment count.
    pub fn reset(&mut self) {
        self.buffer.clear();
        self.segment_id = None;
        self.elements.clear();
        self.current_composite.clear();
        self.in_composite = false;
        self.segment_count = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_segment() {
        let mut w = EdifactSegmentWriter::with_defaults();
        w.begin_segment("BGM");
        w.add_element("E03");
        w.add_element("DOC001");
        let seg = w.end_segment();
        assert_eq!(seg, "BGM+E03+DOC001");
        assert_eq!(w.output(), "BGM+E03+DOC001'");
    }

    #[test]
    fn test_composite_segment() {
        let mut w = EdifactSegmentWriter::with_defaults();
        w.begin_segment("DTM");
        w.begin_composite();
        w.add_component("137");
        w.add_component("202507010000");
        w.add_component("303");
        w.end_composite();
        let seg = w.end_segment();
        assert_eq!(seg, "DTM+137:202507010000:303");
    }

    #[test]
    fn test_loc_segment() {
        let mut w = EdifactSegmentWriter::with_defaults();
        w.begin_segment("LOC");
        w.add_element("Z16");
        w.begin_composite();
        w.add_component("DE00014545768S0000000000000003054");
        w.end_composite();
        let seg = w.end_segment();
        assert_eq!(seg, "LOC+Z16+DE00014545768S0000000000000003054");
    }

    #[test]
    fn test_trailing_empty_elements_stripped() {
        let mut w = EdifactSegmentWriter::with_defaults();
        w.begin_segment("NAD");
        w.add_element("MS");
        w.begin_composite();
        w.add_component("9900123000002");
        w.add_empty_component();
        w.add_component("293");
        w.end_composite();
        w.add_empty_element();
        w.add_empty_element();
        let seg = w.end_segment();
        assert_eq!(seg, "NAD+MS+9900123000002::293");
    }

    #[test]
    fn test_escape_special_characters() {
        let mut w = EdifactSegmentWriter::with_defaults();
        w.begin_segment("FTX");
        w.add_element("ACB");
        w.add_empty_element();
        w.add_empty_element();
        w.begin_composite();
        w.add_component("Text with + and : chars");
        w.end_composite();
        let seg = w.end_segment();
        assert_eq!(seg, "FTX+ACB+++Text with ?+ and ?: chars");
    }

    #[test]
    fn test_multiple_segments() {
        let mut w = EdifactSegmentWriter::with_defaults();

        w.begin_segment("BGM");
        w.add_element("E03");
        w.end_segment();

        w.begin_segment("DTM");
        w.begin_composite();
        w.add_component("137");
        w.add_component("202507010000");
        w.add_component("303");
        w.end_composite();
        w.end_segment();

        assert_eq!(w.segment_count(), 2);
        assert_eq!(w.output(), "BGM+E03'DTM+137:202507010000:303'");
    }

    #[test]
    fn test_write_raw() {
        let mut w = EdifactSegmentWriter::with_defaults();
        w.write_raw("UNA:+.? ");
        assert_eq!(w.output(), "UNA:+.? '");
        assert_eq!(w.segment_count(), 1);
    }

    #[test]
    fn test_reset() {
        let mut w = EdifactSegmentWriter::with_defaults();
        w.begin_segment("BGM");
        w.add_element("E03");
        w.end_segment();

        w.reset();
        assert_eq!(w.segment_count(), 0);
        assert_eq!(w.output(), "");
    }

    #[test]
    fn test_into_output() {
        let mut w = EdifactSegmentWriter::with_defaults();
        w.begin_segment("BGM");
        w.add_element("E03");
        w.end_segment();

        let output = w.into_output();
        assert_eq!(output, "BGM+E03'");
    }
}
