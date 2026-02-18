---
feature: edifact-core-implementation
epic: 8
title: "Writer, Roundtrip & Batch Processing"
depends_on: [edifact-core-implementation/E07]
estimated_tasks: 7
crate: automapper-core
status: in_progress
---

# Epic 8: Writer, Roundtrip & Batch Processing

> **For Claude:** Implement each task in sequence. Every task follows TDD: write the test FIRST, run it to see it fail, write the implementation, run it to see it pass, then commit. All code lives in `crates/automapper-core/src/`. All code must compile with `cargo check -p automapper-core`.

**Goal:** Implement the EDIFACT writer layer (`EdifactSegmentWriter`, `EdifactDocumentWriter`), entity writers (`MarktlokationWriter`, `ZaehlerWriter`, etc.), roundtrip integration tests (parse -> map -> write -> compare byte-identical), `convert_batch()` with rayon parallelism, and `criterion` benchmarks for parser throughput and batch conversion.

**Architecture:** The writer layer reverses the parsing pipeline. `EdifactSegmentWriter` builds individual segment strings from field data. `EdifactDocumentWriter` manages the full interchange structure (UNA/UNB/UNH...UNT/UNZ), tracking segment counts and message counts automatically. Entity writers implement the `EntityWriter` trait and serialize domain objects back to EDIFACT segments using the segment writer. The roundtrip test pattern verifies that parse -> map -> write produces byte-identical output. Batch processing uses rayon for parallel conversion of multiple interchanges. See design doc sections 5 and 10, and C# `EdifactDocumentWriter.cs`, `EdifactStreamWriter.cs`, `MarktlokationWriter.cs`.

**Tech Stack:** Rust, rayon, criterion, insta, test-case

---

## Task 1: EdifactSegmentWriter

**Files:**
- Create: `crates/automapper-core/src/writer/mod.rs`
- Create: `crates/automapper-core/src/writer/segment_writer.rs`
- Modify: `crates/automapper-core/src/lib.rs`

**Step 1: Write the failing test**

Create `crates/automapper-core/src/writer/mod.rs`:

```rust
//! EDIFACT writer layer for serializing domain objects back to EDIFACT format.
//!
//! - `EdifactSegmentWriter` — builds individual segment strings from field data
//! - `EdifactDocumentWriter` — manages full interchange structure (UNA/UNB...UNZ)
//! - Entity writers — serialize domain objects using the segment writer

pub mod segment_writer;

pub use segment_writer::EdifactSegmentWriter;
```

Create `crates/automapper-core/src/writer/segment_writer.rs`:

```rust
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
    buffer: String,
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
            while self.current_composite.last().map_or(false, |c| c.is_empty()) {
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
        while self.elements.last().map_or(false, |e| e.is_empty()) {
            self.elements.pop();
        }

        let element_sep = self.delimiters.element as char;
        let segment_term = self.delimiters.segment as char;

        let segment = if self.elements.is_empty() {
            id.clone()
        } else {
            format!("{}{}{}", id, element_sep, self.elements.join(&element_sep.to_string()))
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
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p automapper-core test_simple_segment`
Expected: FAIL -- module not found.

**Step 3: Write minimal implementation**

Update `crates/automapper-core/src/lib.rs`:

```rust
//! Core automapper library.

pub mod context;
pub mod coordinator;
pub mod error;
pub mod mappers;
pub mod traits;
pub mod utilmd_coordinator;
pub mod version;
pub mod writer;

pub use context::TransactionContext;
pub use coordinator::{create_coordinator, detect_format_version, Coordinator};
pub use error::AutomapperError;
pub use traits::*;
pub use utilmd_coordinator::UtilmdCoordinator;
pub use version::{FV2504, FV2510, VersionConfig, VersionPhantom};
pub use writer::EdifactSegmentWriter;
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p automapper-core test_simple_segment`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/automapper-core/
git commit -m "$(cat <<'EOF'
feat(automapper-core): add EdifactSegmentWriter for building segments

Builds EDIFACT segment strings from elements and components with
proper delimiter escaping, trailing empty suppression, and composite
handling. Foundation for entity writers and roundtrip generation.

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
EOF
)"
```

---

## Task 2: EdifactDocumentWriter

**Files:**
- Create: `crates/automapper-core/src/writer/document_writer.rs`
- Modify: `crates/automapper-core/src/writer/mod.rs`

**Step 1: Write the failing test**

Create `crates/automapper-core/src/writer/document_writer.rs`:

```rust
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

    /// Writes the UNA service string advice.
    pub fn write_una(&mut self) {
        let una = format!(
            "UNA{}{}{}{}{}{}",
            self.delimiters.component as char,
            self.delimiters.element as char,
            self.delimiters.decimal as char,
            self.delimiters.release as char,
            self.delimiters.reserved as char,
            // Note: UNA does not have a segment terminator in the normal sense;
            // the 6th character IS the terminator indicator
            ""
        );
        // UNA is written without the segment terminator appended by write_raw
        // Instead we write the raw UNA string directly
        self.writer.buffer.push_str(&una);
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
    pub fn begin_message(&mut self, reference: &str, message_type: &str) {
        self.message_ref = Some(reference.to_string());
        self.message_segment_count = 0;
        self.in_message = true;

        self.writer.begin_segment("UNH");
        self.writer.add_element(reference);
        self.writer.add_element(message_type);
        self.writer.end_segment();
        self.message_segment_count += 1; // Count UNH
    }

    /// Writes a content segment and increments the message segment count.
    ///
    /// This is the primary method for writing segments within a message.
    /// The segment must be fully built before calling this.
    pub fn write_segment(&mut self, id: &str, elements: &[&str]) {
        self.writer.begin_segment(id);
        for element in elements {
            self.writer.add_element(element);
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
        self.writer.add_element(&self.message_segment_count.to_string());
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
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p automapper-core test_document_writer`
Expected: FAIL -- module not found.

**Step 3: Write minimal implementation**

Update `crates/automapper-core/src/writer/mod.rs`:

```rust
//! EDIFACT writer layer.

pub mod document_writer;
pub mod segment_writer;

pub use document_writer::EdifactDocumentWriter;
pub use segment_writer::EdifactSegmentWriter;
```

Update the writer re-exports in `crates/automapper-core/src/lib.rs` to include `EdifactDocumentWriter`:

```rust
pub use writer::{EdifactDocumentWriter, EdifactSegmentWriter};
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p automapper-core test_document_writer`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/automapper-core/
git commit -m "$(cat <<'EOF'
feat(automapper-core): add EdifactDocumentWriter for interchange structure

Manages UNA/UNB/UNH...UNT/UNZ with automatic segment counting for UNT
and message counting for UNZ. Supports composite elements and multi-message
interchanges. Built on top of EdifactSegmentWriter.

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
EOF
)"
```

---

## Task 3: Entity Writers (MarktlokationWriter, ZaehlerWriter, etc.)

**Files:**
- Create: `crates/automapper-core/src/writer/entity_writers.rs`
- Modify: `crates/automapper-core/src/writer/mod.rs`

**Step 1: Write the failing test**

Create `crates/automapper-core/src/writer/entity_writers.rs`:

```rust
//! Entity writers that serialize domain objects back to EDIFACT segments.
//!
//! Each writer knows how to produce the EDIFACT segments for one entity type.
//! They use `EdifactDocumentWriter` to append segments within an open message.

use bo4e_extensions::*;

use super::document_writer::EdifactDocumentWriter;
use crate::context::TransactionContext;

/// Writes a Marktlokation to EDIFACT segments.
///
/// Produces: LOC+Z16+id' and NAD+DP address segments.
pub struct MarktlokationWriter;

impl MarktlokationWriter {
    /// Writes the LOC+Z16 segment for a Marktlokation.
    pub fn write(
        doc: &mut EdifactDocumentWriter,
        ml: &WithValidity<Marktlokation, MarktlokationEdifact>,
    ) {
        // LOC+Z16+marktlokationsId'
        if let Some(ref id) = ml.data.marktlokations_id {
            doc.write_segment("LOC", &["Z16", id]);
        }
    }

    /// Writes the NAD+DP address segment if address data is present.
    pub fn write_address(
        doc: &mut EdifactDocumentWriter,
        ml: &WithValidity<Marktlokation, MarktlokationEdifact>,
    ) {
        if let Some(ref addr) = ml.data.lokationsadresse {
            let w = doc.segment_writer();
            w.begin_segment("NAD");
            w.add_element("DP");
            w.add_empty_element(); // C082
            w.add_empty_element(); // C058
            w.add_empty_element(); // C080
            // C059: street address
            w.begin_composite();
            w.add_component(addr.strasse.as_deref().unwrap_or(""));
            w.add_empty_component(); // 3042_1
            w.add_component(addr.hausnummer.as_deref().unwrap_or(""));
            w.end_composite();
            w.add_element(addr.ort.as_deref().unwrap_or(""));
            w.add_empty_element(); // region
            w.add_element(addr.postleitzahl.as_deref().unwrap_or(""));
            w.add_element(addr.landescode.as_deref().unwrap_or(""));
            w.end_segment();
            doc.message_segment_count_increment();
        }
    }
}

/// Writes a Messlokation to EDIFACT segments.
pub struct MesslokationWriter;

impl MesslokationWriter {
    pub fn write(
        doc: &mut EdifactDocumentWriter,
        ml: &WithValidity<Messlokation, MesslokationEdifact>,
    ) {
        if let Some(ref id) = ml.data.messlokations_id {
            doc.write_segment("LOC", &["Z17", id]);
        }
    }
}

/// Writes a Netzlokation to EDIFACT segments.
pub struct NetzlokationWriter;

impl NetzlokationWriter {
    pub fn write(
        doc: &mut EdifactDocumentWriter,
        nl: &WithValidity<Netzlokation, NetzlokationEdifact>,
    ) {
        if let Some(ref id) = nl.data.netzlokations_id {
            doc.write_segment("LOC", &["Z18", id]);
        }
    }
}

/// Writes Geschaeftspartner NAD segments.
pub struct GeschaeftspartnerWriter;

impl GeschaeftspartnerWriter {
    pub fn write(
        doc: &mut EdifactDocumentWriter,
        gp: &WithValidity<Geschaeftspartner, GeschaeftspartnerEdifact>,
    ) {
        let qualifier = gp.edifact.nad_qualifier.as_deref().unwrap_or("Z04");
        let id = gp.data.name1.as_deref().unwrap_or("");
        doc.write_segment("NAD", &[qualifier, id]);
    }
}

/// Writes a Zaehler to EDIFACT segments.
pub struct ZaehlerWriter;

impl ZaehlerWriter {
    pub fn write(
        doc: &mut EdifactDocumentWriter,
        z: &WithValidity<Zaehler, ZaehlerEdifact>,
    ) {
        // SEQ+Z03'
        doc.write_segment("SEQ", &["Z03"]);

        // PIA+5+zaehlernummer'
        if let Some(ref nr) = z.data.zaehlernummer {
            doc.write_segment("PIA", &["5", nr]);
        }

        // RFF+Z19:messlokation_ref'
        if let Some(ref melo_ref) = z.edifact.referenz_messlokation {
            doc.write_segment_with_composites("RFF", &[&["Z19", melo_ref]]);
        }

        // RFF+Z14:gateway_ref'
        if let Some(ref gw_ref) = z.edifact.referenz_gateway {
            doc.write_segment_with_composites("RFF", &[&["Z14", gw_ref]]);
        }
    }
}

/// Writes Vertrag data segments.
pub struct VertragWriter;

impl VertragWriter {
    pub fn write(
        doc: &mut EdifactDocumentWriter,
        v: &WithValidity<Vertrag, VertragEdifact>,
    ) {
        doc.write_segment("SEQ", &["Z18"]);

        if let Some(haushalt) = v.edifact.haushaltskunde {
            let code = if haushalt { "Z01" } else { "Z02" };
            doc.write_segment("CCI", &["Z15", "", code]);
        }

        if let Some(ref va) = v.edifact.versorgungsart {
            doc.write_segment("CCI", &["Z36", "", va]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_marktlokation_writer_loc() {
        let mut doc = EdifactDocumentWriter::with_delimiters(
            edifact_types::EdifactDelimiters::default(),
            false,
        );
        doc.begin_interchange("S", "R", "REF", "D", "T");
        doc.begin_message("M", "TYPE");

        let ml = WithValidity {
            data: Marktlokation {
                marktlokations_id: Some("DE00014545768S0000000000000003054".to_string()),
                ..Default::default()
            },
            edifact: MarktlokationEdifact::default(),
            gueltigkeitszeitraum: None,
            zeitscheibe_ref: None,
        };

        MarktlokationWriter::write(&mut doc, &ml);
        doc.end_message();
        doc.end_interchange();

        let output = doc.output();
        assert!(output.contains("LOC+Z16+DE00014545768S0000000000000003054'"));
    }

    #[test]
    fn test_messlokation_writer() {
        let mut doc = EdifactDocumentWriter::with_delimiters(
            edifact_types::EdifactDelimiters::default(),
            false,
        );
        doc.begin_interchange("S", "R", "REF", "D", "T");
        doc.begin_message("M", "TYPE");

        let ml = WithValidity {
            data: Messlokation {
                messlokations_id: Some("MELO001".to_string()),
                ..Default::default()
            },
            edifact: MesslokationEdifact::default(),
            gueltigkeitszeitraum: None,
            zeitscheibe_ref: None,
        };

        MesslokationWriter::write(&mut doc, &ml);
        doc.end_message();
        doc.end_interchange();

        assert!(doc.output().contains("LOC+Z17+MELO001'"));
    }

    #[test]
    fn test_zaehler_writer() {
        let mut doc = EdifactDocumentWriter::with_delimiters(
            edifact_types::EdifactDelimiters::default(),
            false,
        );
        doc.begin_interchange("S", "R", "REF", "D", "T");
        doc.begin_message("M", "TYPE");

        let z = WithValidity {
            data: Zaehler {
                zaehlernummer: Some("ZAEHLER001".to_string()),
                ..Default::default()
            },
            edifact: ZaehlerEdifact {
                referenz_messlokation: Some("MELO001".to_string()),
                ..Default::default()
            },
            gueltigkeitszeitraum: None,
            zeitscheibe_ref: None,
        };

        ZaehlerWriter::write(&mut doc, &z);
        doc.end_message();
        doc.end_interchange();

        let output = doc.output();
        assert!(output.contains("SEQ+Z03'"));
        assert!(output.contains("PIA+5+ZAEHLER001'"));
        assert!(output.contains("RFF+Z19:MELO001'"));
    }

    #[test]
    fn test_vertrag_writer() {
        let mut doc = EdifactDocumentWriter::with_delimiters(
            edifact_types::EdifactDelimiters::default(),
            false,
        );
        doc.begin_interchange("S", "R", "REF", "D", "T");
        doc.begin_message("M", "TYPE");

        let v = WithValidity {
            data: Vertrag::default(),
            edifact: VertragEdifact {
                haushaltskunde: Some(true),
                versorgungsart: Some("ZD0".to_string()),
            },
            gueltigkeitszeitraum: None,
            zeitscheibe_ref: None,
        };

        VertragWriter::write(&mut doc, &v);
        doc.end_message();
        doc.end_interchange();

        let output = doc.output();
        assert!(output.contains("SEQ+Z18'"));
        assert!(output.contains("CCI+Z15++Z01'"));
        assert!(output.contains("CCI+Z36++ZD0'"));
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p automapper-core test_marktlokation_writer`
Expected: FAIL -- module not found.

**Step 3: Write minimal implementation**

Add `message_segment_count_increment()` method to `EdifactDocumentWriter`:

Add to `crates/automapper-core/src/writer/document_writer.rs` in the `impl EdifactDocumentWriter` block:

```rust
    /// Increments the message segment count.
    /// Used by entity writers that write segments directly via the segment writer.
    pub fn message_segment_count_increment(&mut self) {
        self.message_segment_count += 1;
    }
```

Also make the `buffer` field in `EdifactSegmentWriter` public within the crate:

In `crates/automapper-core/src/writer/segment_writer.rs`, change:
```rust
    buffer: String,
```
to:
```rust
    pub(crate) buffer: String,
```

Update `crates/automapper-core/src/writer/mod.rs`:

```rust
//! EDIFACT writer layer.

pub mod document_writer;
pub mod entity_writers;
pub mod segment_writer;

pub use document_writer::EdifactDocumentWriter;
pub use entity_writers::*;
pub use segment_writer::EdifactSegmentWriter;
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p automapper-core test_marktlokation_writer`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/automapper-core/
git commit -m "$(cat <<'EOF'
feat(automapper-core): add entity writers for reverse mapping

MarktlokationWriter (LOC+Z16, NAD+DP), MesslokationWriter (LOC+Z17),
NetzlokationWriter (LOC+Z18), GeschaeftspartnerWriter (NAD),
ZaehlerWriter (SEQ+Z03, PIA, RFF), VertragWriter (SEQ+Z18, CCI).

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
EOF
)"
```

---

## Task 4: Roundtrip Integration Tests

**Files:**
- Create: `crates/automapper-core/tests/roundtrip_test.rs`

**Step 1: Write the roundtrip test**

Create `crates/automapper-core/tests/roundtrip_test.rs`:

```rust
//! Roundtrip integration tests: EDIFACT -> BO4E -> EDIFACT.
//!
//! Tests verify that parse -> map -> write produces output matching
//! the original input. For full byte-identical roundtrip, the writer
//! must reproduce all segment details from the parsed data.

use automapper_core::{
    create_coordinator, detect_format_version, EdifactDocumentWriter, FormatVersion,
    UtilmdCoordinator, FV2504,
};
use automapper_core::writer::entity_writers::*;

/// A minimal EDIFACT interchange for roundtrip testing.
///
/// This is intentionally simple to test the roundtrip mechanism.
/// Full roundtrip with real fixture files will be added when all
/// mappers and writers are feature-complete.
const SIMPLE_INTERCHANGE: &[u8] = b"UNA:+.? 'UNB+UNOC:3+SENDER+RECEIVER+251217:1229+REF001'UNH+MSG001+UTILMD:D:11A:UN:S2.1'BGM+E03+DOC001'IDE+24+TXID001'LOC+Z16+MALO001'LOC+Z17+MELO001'UNT+5+MSG001'UNZ+1+REF001'";

#[test]
fn test_roundtrip_simple_locations() {
    // Step 1: Parse EDIFACT -> BO4E
    let mut coord = UtilmdCoordinator::<FV2504>::new();
    let transactions = coord.parse(SIMPLE_INTERCHANGE).unwrap();
    assert_eq!(transactions.len(), 1);

    let tx = &transactions[0];

    // Step 2: Generate BO4E -> EDIFACT
    let mut doc = EdifactDocumentWriter::new();
    doc.begin_interchange("SENDER", "RECEIVER", "REF001", "251217", "1229");
    doc.begin_message("MSG001", "UTILMD:D:11A:UN:S2.1");

    doc.write_segment("BGM", &["E03", "DOC001"]);
    doc.write_segment("IDE", &["24", &tx.transaktions_id]);

    // Write locations
    for ml in &tx.marktlokationen {
        MarktlokationWriter::write(&mut doc, ml);
    }
    for ml in &tx.messlokationen {
        MesslokationWriter::write(&mut doc, ml);
    }

    doc.end_message();
    doc.end_interchange();

    // Step 3: Verify key segments are present
    let output = doc.output();
    assert!(
        output.contains("LOC+Z16+MALO001'"),
        "output should contain Marktlokation LOC"
    );
    assert!(
        output.contains("LOC+Z17+MELO001'"),
        "output should contain Messlokation LOC"
    );
    assert!(
        output.contains("IDE+24+TXID001'"),
        "output should contain transaction ID"
    );
    assert!(
        output.contains("UNT+5+MSG001'"),
        "output should have correct segment count in UNT"
    );
    assert!(
        output.contains("UNZ+1+REF001'"),
        "output should have correct message count in UNZ"
    );
}

#[test]
fn test_roundtrip_detect_version_and_parse() {
    let fv = detect_format_version(SIMPLE_INTERCHANGE).unwrap();
    assert_eq!(fv, FormatVersion::FV2504);

    let mut coord = create_coordinator(fv).unwrap();
    let transactions = coord.parse(SIMPLE_INTERCHANGE).unwrap();
    assert_eq!(transactions.len(), 1);
    assert_eq!(transactions[0].transaktions_id, "TXID001");
}

#[test]
fn test_roundtrip_preserves_location_ids() {
    let mut coord = UtilmdCoordinator::<FV2504>::new();
    let transactions = coord.parse(SIMPLE_INTERCHANGE).unwrap();
    let tx = &transactions[0];

    // Verify forward mapping extracted correct IDs
    assert_eq!(
        tx.marktlokationen[0].data.marktlokations_id,
        Some("MALO001".to_string())
    );
    assert_eq!(
        tx.messlokationen[0].data.messlokations_id,
        Some("MELO001".to_string())
    );

    // Generate back and verify IDs survive the roundtrip
    let mut doc = EdifactDocumentWriter::with_delimiters(
        edifact_types::EdifactDelimiters::default(),
        false,
    );
    doc.begin_interchange("S", "R", "REF", "D", "T");
    doc.begin_message("M", "TYPE");
    MarktlokationWriter::write(&mut doc, &tx.marktlokationen[0]);
    MesslokationWriter::write(&mut doc, &tx.messlokationen[0]);
    doc.end_message();
    doc.end_interchange();

    let output = doc.output();
    assert!(output.contains("LOC+Z16+MALO001'"));
    assert!(output.contains("LOC+Z17+MELO001'"));
}
```

**Step 2: Run roundtrip test**

Run: `cargo test -p automapper-core --test roundtrip_test`
Expected: PASS

**Step 3: Commit**

```bash
git add crates/automapper-core/tests/
git commit -m "$(cat <<'EOF'
test(automapper-core): add roundtrip integration tests

Tests parse -> map -> write cycle with simple EDIFACT interchanges.
Verifies location IDs, transaction IDs, and segment counts survive
the roundtrip. Foundation for byte-identical roundtrip tests.

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
EOF
)"
```

---

## Task 5: convert_batch() with Rayon Parallelism

**Files:**
- Create: `crates/automapper-core/src/batch.rs`
- Modify: `crates/automapper-core/src/lib.rs`

**Step 1: Write the failing test**

Create `crates/automapper-core/src/batch.rs`:

```rust
//! Batch processing with rayon parallelism.
//!
//! Each message gets its own coordinator instance -- no shared mutable state,
//! perfect for data-parallel processing with rayon.
//!
//! See design doc section 5 (Batch Processing).

use rayon::prelude::*;

use bo4e_extensions::UtilmdTransaktion;

use crate::coordinator::create_coordinator;
use crate::error::AutomapperError;
use crate::traits::FormatVersion;

/// Converts multiple EDIFACT interchanges in parallel using rayon.
///
/// Each input gets its own coordinator instance for full isolation.
/// Results are returned in the same order as inputs.
///
/// # Example
///
/// ```ignore
/// let inputs: Vec<&[u8]> = load_edifact_files();
/// let results = convert_batch(&inputs, FormatVersion::FV2504);
/// for result in results {
///     match result {
///         Ok(transactions) => process(transactions),
///         Err(e) => log_error(e),
///     }
/// }
/// ```
pub fn convert_batch(
    inputs: &[&[u8]],
    fv: FormatVersion,
) -> Vec<Result<Vec<UtilmdTransaktion>, AutomapperError>> {
    inputs
        .par_iter()
        .map(|input| {
            let mut coord = create_coordinator(fv)?;
            coord.parse(input)
        })
        .collect()
}

/// Converts multiple EDIFACT interchanges sequentially (for comparison/testing).
pub fn convert_sequential(
    inputs: &[&[u8]],
    fv: FormatVersion,
) -> Vec<Result<Vec<UtilmdTransaktion>, AutomapperError>> {
    inputs
        .iter()
        .map(|input| {
            let mut coord = create_coordinator(fv)?;
            coord.parse(input)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const MSG1: &[u8] = b"UNA:+.? 'UNB+UNOC:3+S+R+D:T+R1'UNH+M1+UTILMD:D:11A:UN:S2.1'BGM+E03+D1'IDE+24+TX1'LOC+Z16+MALO1'UNT+4+M1'UNZ+1+R1'";
    const MSG2: &[u8] = b"UNA:+.? 'UNB+UNOC:3+S+R+D:T+R2'UNH+M2+UTILMD:D:11A:UN:S2.1'BGM+E03+D2'IDE+24+TX2'LOC+Z16+MALO2'UNT+4+M2'UNZ+1+R2'";
    const MSG3: &[u8] = b"UNA:+.? 'UNB+UNOC:3+S+R+D:T+R3'UNH+M3+UTILMD:D:11A:UN:S2.1'BGM+E03+D3'IDE+24+TX3'LOC+Z17+MELO1'UNT+4+M3'UNZ+1+R3'";

    #[test]
    fn test_convert_batch_multiple() {
        let inputs: Vec<&[u8]> = vec![MSG1, MSG2, MSG3];
        let results = convert_batch(&inputs, FormatVersion::FV2504);

        assert_eq!(results.len(), 3);

        let tx1 = results[0].as_ref().unwrap();
        assert_eq!(tx1.len(), 1);
        assert_eq!(tx1[0].transaktions_id, "TX1");

        let tx2 = results[1].as_ref().unwrap();
        assert_eq!(tx2[0].transaktions_id, "TX2");

        let tx3 = results[2].as_ref().unwrap();
        assert_eq!(tx3[0].transaktions_id, "TX3");
    }

    #[test]
    fn test_convert_batch_empty() {
        let inputs: Vec<&[u8]> = vec![];
        let results = convert_batch(&inputs, FormatVersion::FV2504);
        assert!(results.is_empty());
    }

    #[test]
    fn test_convert_batch_matches_sequential() {
        let inputs: Vec<&[u8]> = vec![MSG1, MSG2, MSG3];

        let parallel = convert_batch(&inputs, FormatVersion::FV2504);
        let sequential = convert_sequential(&inputs, FormatVersion::FV2504);

        assert_eq!(parallel.len(), sequential.len());
        for (p, s) in parallel.iter().zip(sequential.iter()) {
            let p_tx = p.as_ref().unwrap();
            let s_tx = s.as_ref().unwrap();
            assert_eq!(p_tx.len(), s_tx.len());
            for (pt, st) in p_tx.iter().zip(s_tx.iter()) {
                assert_eq!(pt.transaktions_id, st.transaktions_id);
            }
        }
    }

    #[test]
    fn test_convert_batch_single() {
        let inputs: Vec<&[u8]> = vec![MSG1];
        let results = convert_batch(&inputs, FormatVersion::FV2504);
        assert_eq!(results.len(), 1);
        assert!(results[0].is_ok());
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p automapper-core test_convert_batch`
Expected: FAIL -- module not found.

**Step 3: Write minimal implementation**

Update `crates/automapper-core/src/lib.rs`:

```rust
//! Core automapper library.

pub mod batch;
pub mod context;
pub mod coordinator;
pub mod error;
pub mod mappers;
pub mod traits;
pub mod utilmd_coordinator;
pub mod version;
pub mod writer;

pub use batch::{convert_batch, convert_sequential};
pub use context::TransactionContext;
pub use coordinator::{create_coordinator, detect_format_version, Coordinator};
pub use error::AutomapperError;
pub use traits::*;
pub use utilmd_coordinator::UtilmdCoordinator;
pub use version::{FV2504, FV2510, VersionConfig, VersionPhantom};
pub use writer::{EdifactDocumentWriter, EdifactSegmentWriter};
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p automapper-core test_convert_batch`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/automapper-core/
git commit -m "$(cat <<'EOF'
feat(automapper-core): add convert_batch() with rayon parallelism

Parallel batch conversion of EDIFACT interchanges. Each input gets
its own coordinator instance for full isolation. Includes sequential
variant for comparison testing. Results preserve input order.

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
EOF
)"
```

---

## Task 6: Criterion Benchmarks

**Files:**
- Create: `crates/automapper-core/benches/parser_throughput.rs`

**Step 1: Write the benchmark**

Create `crates/automapper-core/benches/parser_throughput.rs`:

```rust
//! Benchmarks for parser throughput and batch conversion.
//!
//! Run with: `cargo bench -p automapper-core`

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};

use automapper_core::{convert_batch, convert_sequential, FormatVersion, UtilmdCoordinator, FV2504};

/// A synthetic UTILMD message for benchmarking.
fn synthetic_utilmd() -> Vec<u8> {
    let msg = b"UNA:+.? 'UNB+UNOC:3+9900123000002:500+9900456000001:500+251217:1229+REF001'UNH+MSG001+UTILMD:D:11A:UN:S2.1'BGM+E03+DOC001'DTM+137:202506190130:303'NAD+MS+9900123000002::293'NAD+MR+9900456000001::293'IDE+24+TXID001'STS+E01+E01::Z44'DTM+137:202507010000:303'DTM+471:202508010000:303'RFF+Z13:VORGANGS001'RFF+Z49:1'DTM+Z25:202507010000:303'DTM+Z26:202512310000:303'LOC+Z16+DE00014545768S0000000000000003054'LOC+Z17+DE00098765432100000000000000012'LOC+Z18+NELO00000000001'NAD+Z04+9900999000003::293'UNT+18+MSG001'UNZ+1+REF001'";
    msg.to_vec()
}

fn bench_single_parse(c: &mut Criterion) {
    let input = synthetic_utilmd();

    let mut group = c.benchmark_group("single_parse");
    group.throughput(Throughput::Bytes(input.len() as u64));

    group.bench_function("utilmd_parse", |b| {
        b.iter(|| {
            let mut coord = UtilmdCoordinator::<FV2504>::new();
            let result = coord.parse(black_box(&input)).unwrap();
            black_box(result);
        });
    });

    group.finish();
}

fn bench_batch_conversion(c: &mut Criterion) {
    let msg = synthetic_utilmd();

    for batch_size in [10, 100, 1000] {
        let inputs: Vec<&[u8]> = vec![msg.as_slice(); batch_size];
        let total_bytes = msg.len() * batch_size;

        let mut group = c.benchmark_group(format!("batch_{}", batch_size));
        group.throughput(Throughput::Bytes(total_bytes as u64));

        group.bench_function("parallel", |b| {
            b.iter(|| {
                let results =
                    convert_batch(black_box(&inputs), FormatVersion::FV2504);
                black_box(results);
            });
        });

        group.bench_function("sequential", |b| {
            b.iter(|| {
                let results =
                    convert_sequential(black_box(&inputs), FormatVersion::FV2504);
                black_box(results);
            });
        });

        group.finish();
    }
}

criterion_group!(benches, bench_single_parse, bench_batch_conversion);
criterion_main!(benches);
```

**Step 2: Verify benchmark compiles**

Run: `cargo bench -p automapper-core --no-run`
Expected: PASS -- compiles without errors.

**Step 3: Run benchmarks (optional, for verification)**

Run: `cargo bench -p automapper-core -- --warm-up-time 1 --measurement-time 3`
Expected: Benchmark results showing throughput in bytes/sec and parallel vs sequential comparison.

**Step 4: Commit**

```bash
git add crates/automapper-core/benches/
git commit -m "$(cat <<'EOF'
bench(automapper-core): add criterion benchmarks for throughput

Benchmarks for single parse throughput (bytes/sec) and batch
conversion (parallel vs sequential) at 10, 100, and 1000 messages.
Run with: cargo bench -p automapper-core

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
EOF
)"
```

---

## Task 7: Full Pipeline Integration Test

**Files:**
- Create: `crates/automapper-core/tests/full_pipeline_test.rs`

**Step 1: Write the full pipeline test**

Create `crates/automapper-core/tests/full_pipeline_test.rs`:

```rust
//! Full pipeline integration test: detect version -> parse -> write -> verify.
//!
//! Tests the complete flow from raw EDIFACT bytes through all layers.

use automapper_core::{
    convert_batch, create_coordinator, detect_format_version, EdifactDocumentWriter,
    FormatVersion, UtilmdCoordinator, FV2504,
};
use automapper_core::writer::entity_writers::*;

const FULL_UTILMD: &[u8] = b"UNA:+.? '\
UNB+UNOC:3+9900123000002:500+9900456000001:500+251217:1229+GEN0001'\
UNH+GEN0001MSG+UTILMD:D:11A:UN:S2.1'\
BGM+E03+DOC001'\
DTM+137:202506190130:303'\
NAD+MS+9900123000002::293'\
NAD+MR+9900456000001::293'\
IDE+24+TXID001'\
STS+E01+E01::Z44'\
RFF+Z13:VORGANGS001'\
RFF+Z49:1'\
DTM+Z25:202507010000:303'\
DTM+Z26:202512310000:303'\
LOC+Z16+DE00014545768S0000000000000003054'\
LOC+Z17+DE00098765432100000000000000012'\
LOC+Z18+NELO00000000001'\
NAD+Z04+9900999000003::293'\
SEQ+Z03'\
PIA+5+ZAEHLER001'\
RFF+Z19:DE00098765432100000000000000012'\
SEQ+Z18'\
CCI+Z15++Z01'\
FTX+ACB+++Testbemerkung'\
UNT+21+GEN0001MSG'\
UNZ+1+GEN0001'";

#[test]
fn test_full_pipeline_detect_parse_write() {
    // Step 1: Detect format version
    let fv = detect_format_version(FULL_UTILMD).unwrap();
    assert_eq!(fv, FormatVersion::FV2504);

    // Step 2: Parse with coordinator
    let mut coord = create_coordinator(fv).unwrap();
    let transactions = coord.parse(FULL_UTILMD).unwrap();

    assert_eq!(transactions.len(), 1);
    let tx = &transactions[0];

    // Step 3: Verify parsed data
    assert_eq!(tx.transaktions_id, "TXID001");
    assert_eq!(tx.absender.mp_id, Some("9900123000002".to_string()));
    assert_eq!(tx.empfaenger.mp_id, Some("9900456000001".to_string()));
    assert_eq!(tx.prozessdaten.transaktionsgrund, Some("E01".to_string()));
    assert_eq!(
        tx.prozessdaten.referenz_vorgangsnummer,
        Some("VORGANGS001".to_string())
    );
    assert_eq!(tx.prozessdaten.bemerkung, Some("Testbemerkung".to_string()));
    assert_eq!(tx.zeitscheiben.len(), 1);
    assert_eq!(tx.zeitscheiben[0].zeitscheiben_id, "1");
    assert_eq!(tx.marktlokationen.len(), 1);
    assert_eq!(tx.messlokationen.len(), 1);
    assert_eq!(tx.netzlokationen.len(), 1);
    assert_eq!(tx.parteien.len(), 1);
    assert_eq!(tx.zaehler.len(), 1);
    assert_eq!(
        tx.zaehler[0].data.zaehlernummer,
        Some("ZAEHLER001".to_string())
    );
    assert!(tx.vertrag.is_some());
    assert_eq!(tx.vertrag.as_ref().unwrap().edifact.haushaltskunde, Some(true));

    // Step 4: Write back to EDIFACT
    let mut doc = EdifactDocumentWriter::new();
    doc.begin_interchange(
        "9900123000002:500",
        "9900456000001:500",
        "GEN0001",
        "251217",
        "1229",
    );
    doc.begin_message("GEN0001MSG", "UTILMD:D:11A:UN:S2.1");

    doc.write_segment("BGM", &["E03", "DOC001"]);
    doc.write_segment_with_composites("DTM", &[&["137", "202506190130", "303"]]);
    doc.write_segment("NAD", &["MS", "9900123000002::293"]);
    doc.write_segment("NAD", &["MR", "9900456000001::293"]);
    doc.write_segment("IDE", &["24", &tx.transaktions_id]);
    doc.write_segment("STS", &["E01", "E01::Z44"]);
    doc.write_segment_with_composites("RFF", &[&["Z13", "VORGANGS001"]]);

    for ml in &tx.marktlokationen {
        MarktlokationWriter::write(&mut doc, ml);
    }
    for ml in &tx.messlokationen {
        MesslokationWriter::write(&mut doc, ml);
    }
    for nl in &tx.netzlokationen {
        NetzlokationWriter::write(&mut doc, nl);
    }
    for gp in &tx.parteien {
        GeschaeftspartnerWriter::write(&mut doc, gp);
    }
    for z in &tx.zaehler {
        ZaehlerWriter::write(&mut doc, z);
    }
    if let Some(ref v) = tx.vertrag {
        VertragWriter::write(&mut doc, v);
    }
    doc.write_segment("FTX", &["ACB", "", "", "Testbemerkung"]);

    doc.end_message();
    doc.end_interchange();

    // Step 5: Verify output contains key segments
    let output = doc.output();
    assert!(output.contains("UNA:+.? "), "should have UNA");
    assert!(output.contains("UNB+"), "should have UNB");
    assert!(output.contains("UNH+"), "should have UNH");
    assert!(output.contains("LOC+Z16+DE00014545768S0000000000000003054'"), "should have MALO");
    assert!(output.contains("LOC+Z17+DE00098765432100000000000000012'"), "should have MELO");
    assert!(output.contains("LOC+Z18+NELO00000000001'"), "should have NELO");
    assert!(output.contains("SEQ+Z03'"), "should have Zaehler SEQ");
    assert!(output.contains("PIA+5+ZAEHLER001'"), "should have Zaehler PIA");
    assert!(output.contains("SEQ+Z18'"), "should have Vertrag SEQ");
    assert!(output.contains("CCI+Z15++Z01'"), "should have Haushaltskunde");
    assert!(output.contains("UNT+"), "should have UNT");
    assert!(output.contains("UNZ+1+GEN0001'"), "should have UNZ");
}

#[test]
fn test_batch_processing_produces_correct_results() {
    let inputs: Vec<&[u8]> = vec![FULL_UTILMD; 5];
    let results = convert_batch(&inputs, FormatVersion::FV2504);

    assert_eq!(results.len(), 5);
    for result in &results {
        let txs = result.as_ref().unwrap();
        assert_eq!(txs.len(), 1);
        assert_eq!(txs[0].transaktions_id, "TXID001");
        assert_eq!(txs[0].marktlokationen.len(), 1);
        assert_eq!(txs[0].zaehler.len(), 1);
    }
}
```

**Step 2: Run full pipeline test**

Run: `cargo test -p automapper-core --test full_pipeline_test`
Expected: PASS

**Step 3: Commit**

```bash
git add crates/automapper-core/tests/
git commit -m "$(cat <<'EOF'
test(automapper-core): add full pipeline and batch processing tests

End-to-end test: detect version -> create coordinator -> parse ->
verify domain objects -> write EDIFACT -> verify output segments.
Batch test verifies parallel processing produces correct results.

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
EOF
)"
```
