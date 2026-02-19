//! High-level writer managing full EDIFACT interchange structure.
//!
//! Tracks segment counts for UNT and message counts for UNZ.
//! Mirrors C# `EdifactDocumentWriter.cs`.

use edifact_types::EdifactDelimiters;

use super::segment_writer::EdifactSegmentWriter;

/// Manages the full EDIFACT document structure with automatic
/// service segment handling (UNA, UNB, UNH, UNT, UNZ).
///
/// Tracks segment counts per message (for UNT) and message counts
/// per interchange (for UNZ).
pub struct EdifactDocumentWriter {
    writer: EdifactSegmentWriter,
    delimiters: EdifactDelimiters,
    write_una: bool,
    interchange_ref: Option<String>,
    message_ref: Option<String>,
    message_segment_count: u32,
    message_count: u32,
    in_interchange: bool,
    in_message: bool,
}

impl EdifactDocumentWriter {
    /// Creates a new document writer with default delimiters.
    pub fn new() -> Self {
        Self::with_delimiters(EdifactDelimiters::default(), true)
    }

    /// Creates a new document writer with the given delimiters.
    pub fn with_delimiters(delimiters: EdifactDelimiters, write_una: bool) -> Self {
        Self {
            writer: EdifactSegmentWriter::new(delimiters),
            delimiters,
            write_una,
            interchange_ref: None,
            message_ref: None,
            message_segment_count: 0,
            message_count: 0,
            in_interchange: false,
            in_message: false,
        }
    }

    /// Returns a mutable reference to the underlying segment writer.
    ///
    /// Use this to write content segments between `begin_message()` and
    /// `end_message()`. Each segment written via the returned writer
    /// is counted toward the UNT segment count.
    pub fn segment_writer(&mut self) -> &mut EdifactSegmentWriter {
        &mut self.writer
    }

    /// Increments the message segment count.
    /// Used by entity writers that write segments directly via the segment writer.
    pub fn message_segment_count_increment(&mut self) {
        self.message_segment_count += 1;
    }

    /// Writes the UNA service string advice.
    ///
    /// UNA is exactly 9 bytes: `UNA` + 6 delimiter characters.
    /// The 6th character is the segment terminator itself.
    fn write_una(&mut self) {
        // Use the canonical to_una_string() from EdifactDelimiters
        // which correctly includes all 6 service characters.
        // UNA is written directly (no extra segment terminator appended).
        self.writer
            .buffer
            .push_str(&self.delimiters.to_una_string());
    }

    /// Begins a new interchange with the UNB segment.
    pub fn begin_interchange(
        &mut self,
        sender: &str,
        recipient: &str,
        reference: &str,
        date: &str,
        time: &str,
    ) {
        if self.write_una {
            self.write_una();
        }

        self.interchange_ref = Some(reference.to_string());
        self.message_count = 0;
        self.in_interchange = true;

        self.writer.begin_segment("UNB");
        self.writer.begin_composite();
        self.writer.add_component("UNOC");
        self.writer.add_component("3");
        self.writer.end_composite();
        self.writer.add_element(sender);
        self.writer.add_element(recipient);
        self.writer.begin_composite();
        self.writer.add_component(date);
        self.writer.add_component(time);
        self.writer.end_composite();
        self.writer.add_element(reference);
        self.writer.end_segment();
    }

    /// Begins a new message with the UNH segment.
    ///
    /// The `message_type` is written as-is (not escaped), since it's a
    /// pre-formatted composite like `UTILMD:D:11A:UN:S2.1`.
    pub fn begin_message(&mut self, reference: &str, message_type: &str) {
        self.message_ref = Some(reference.to_string());
        self.message_segment_count = 0;
        self.in_message = true;

        self.writer.begin_segment("UNH");
        self.writer.add_element(reference);
        self.writer.add_raw_element(message_type);
        self.writer.end_segment();
        self.message_segment_count += 1; // Count UNH
    }

    /// Writes a content segment and increments the message segment count.
    ///
    /// This is the primary method for writing segments within a message.
    /// Elements are written as raw values (not escaped), so pre-formatted
    /// composites like `"9900123000002::293"` are preserved as-is.
    pub fn write_segment(&mut self, id: &str, elements: &[&str]) {
        self.writer.begin_segment(id);
        for element in elements {
            self.writer.add_raw_element(element);
        }
        self.writer.end_segment();
        self.message_segment_count += 1;
    }

    /// Writes a segment with composite elements.
    pub fn write_segment_with_composites(&mut self, id: &str, composites: &[&[&str]]) {
        self.writer.begin_segment(id);
        for composite in composites {
            if composite.len() == 1 {
                // Simple element
                self.writer.add_element(composite[0]);
            } else {
                // Composite element
                self.writer.begin_composite();
                for component in *composite {
                    self.writer.add_component(component);
                }
                self.writer.end_composite();
            }
        }
        self.writer.end_segment();
        self.message_segment_count += 1;
    }

    /// Ends the current message with the UNT segment.
    pub fn end_message(&mut self) {
        self.message_segment_count += 1; // Count UNT itself

        let reference = self.message_ref.take().unwrap_or_default();
        self.writer.begin_segment("UNT");
        self.writer
            .add_element(&self.message_segment_count.to_string());
        self.writer.add_element(&reference);
        self.writer.end_segment();

        self.message_count += 1;
        self.in_message = false;
    }

    /// Ends the current interchange with the UNZ segment.
    pub fn end_interchange(&mut self) {
        let reference = self.interchange_ref.take().unwrap_or_default();
        self.writer.begin_segment("UNZ");
        self.writer.add_element(&self.message_count.to_string());
        self.writer.add_element(&reference);
        self.writer.end_segment();

        self.in_interchange = false;
    }

    /// Returns the accumulated output.
    pub fn output(&self) -> &str {
        self.writer.output()
    }

    /// Consumes the writer and returns the output as bytes.
    pub fn into_bytes(self) -> Vec<u8> {
        self.writer.into_output().into_bytes()
    }
}

impl Default for EdifactDocumentWriter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_writer_minimal_interchange() {
        let mut w = EdifactDocumentWriter::new();
        w.begin_interchange("SENDER", "RECEIVER", "REF001", "251217", "1229");
        w.begin_message("MSG001", "UTILMD:D:11A:UN:S2.1");
        w.write_segment("BGM", &["E03", "DOC001"]);
        w.end_message();
        w.end_interchange();

        let output = w.output();
        assert!(output.starts_with("UNA:+.? "));
        assert!(output.contains("UNB+UNOC:3+SENDER+RECEIVER+251217:1229+REF001'"));
        assert!(output.contains("UNH+MSG001+UTILMD:D:11A:UN:S2.1'"));
        assert!(output.contains("BGM+E03+DOC001'"));
        assert!(output.contains("UNT+3+MSG001'")); // UNH + BGM + UNT = 3
        assert!(output.contains("UNZ+1+REF001'")); // 1 message
    }

    #[test]
    fn test_document_writer_segment_count() {
        let mut w = EdifactDocumentWriter::new();
        w.begin_interchange("S", "R", "REF", "D", "T");
        w.begin_message("M", "TYPE");
        w.write_segment("BGM", &["E03"]);
        w.write_segment("DTM", &["137:20250701:102"]);
        w.write_segment("NAD", &["MS", "ID"]);
        w.end_message();
        w.end_interchange();

        // UNH + BGM + DTM + NAD + UNT = 5
        let output = w.output();
        assert!(output.contains("UNT+5+M'"));
    }

    #[test]
    fn test_document_writer_multi_message() {
        let mut w = EdifactDocumentWriter::new();
        w.begin_interchange("S", "R", "REF", "D", "T");

        w.begin_message("M1", "TYPE");
        w.write_segment("BGM", &["E03"]);
        w.end_message();

        w.begin_message("M2", "TYPE");
        w.write_segment("BGM", &["E03"]);
        w.end_message();

        w.end_interchange();

        let output = w.output();
        assert!(output.contains("UNZ+2+REF'")); // 2 messages
    }

    #[test]
    fn test_document_writer_composite_segments() {
        let mut w = EdifactDocumentWriter::new();
        w.begin_interchange("S", "R", "REF", "D", "T");
        w.begin_message("M", "TYPE");
        w.write_segment_with_composites("DTM", &[&["137", "202507010000", "303"]]);
        w.end_message();
        w.end_interchange();

        let output = w.output();
        assert!(output.contains("DTM+137:202507010000:303'"));
    }

    #[test]
    fn test_document_writer_without_una() {
        let mut w = EdifactDocumentWriter::with_delimiters(EdifactDelimiters::default(), false);
        w.begin_interchange("S", "R", "REF", "D", "T");
        w.begin_message("M", "TYPE");
        w.end_message();
        w.end_interchange();

        let output = w.output();
        assert!(output.starts_with("UNB"));
    }

    #[test]
    fn test_document_writer_into_bytes() {
        let mut w = EdifactDocumentWriter::new();
        w.begin_interchange("S", "R", "REF", "D", "T");
        w.begin_message("M", "TYPE");
        w.end_message();
        w.end_interchange();

        let bytes = w.into_bytes();
        assert!(!bytes.is_empty());
    }
}
