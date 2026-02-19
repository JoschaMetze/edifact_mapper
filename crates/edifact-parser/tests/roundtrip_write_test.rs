//! Port of C# `EdifactRoundtripTests.Roundtrip_RealEdifactFile_ProducesSameOutput`.
//!
//! Low-level parse-then-write roundtrip: parse an EDIFACT file into segments,
//! reconstruct the raw EDIFACT bytes from the parsed segments, and compare
//! with the original (after normalization).
//!
//! EDIFACT uses ISO-8859-1 encoding, so all byte-level operations work on
//! raw `&[u8]` rather than assuming UTF-8.

use edifact_parser::{EdifactHandler, EdifactStreamParser};
use edifact_types::{Control, EdifactDelimiters, RawSegment};
use std::path::Path;

/// Handler that collects all segments with their raw string representation.
struct SegmentCollector {
    delimiters: EdifactDelimiters,
    explicit_una: bool,
    segments: Vec<OwnedSegment>,
}

/// Owned copy of a parsed segment (since RawSegment borrows from input).
struct OwnedSegment {
    id: String,
    elements: Vec<Vec<String>>,
}

impl SegmentCollector {
    fn new() -> Self {
        Self {
            delimiters: EdifactDelimiters::default(),
            explicit_una: false,
            segments: Vec::new(),
        }
    }

    /// Reconstruct the EDIFACT bytes from collected segments.
    ///
    /// The tokenizer preserves escape sequences in component values
    /// (e.g. `?+` stays as `?+` in the parsed data), so we write
    /// components verbatim without re-escaping.
    fn reconstruct(&self) -> Vec<u8> {
        let mut output = Vec::new();

        // Write UNA if the original had one.
        // UNA is exactly 9 bytes: U N A + 6 delimiter definition chars.
        // The 6th char IS the segment terminator being defined.
        if self.explicit_una {
            output.extend_from_slice(b"UNA");
            output.push(self.delimiters.component);
            output.push(self.delimiters.element);
            output.push(self.delimiters.decimal);
            output.push(self.delimiters.release);
            output.push(self.delimiters.reserved);
            output.push(self.delimiters.segment);
        }

        for seg in &self.segments {
            output.extend_from_slice(seg.id.as_bytes());
            for element in &seg.elements {
                output.push(self.delimiters.element);
                for (j, component) in element.iter().enumerate() {
                    if j > 0 {
                        output.push(self.delimiters.component);
                    }
                    // Write verbatim — escape sequences are already in the data
                    output.extend_from_slice(component.as_bytes());
                }
            }
            output.push(self.delimiters.segment);
        }

        output
    }
}

impl EdifactHandler for SegmentCollector {
    fn on_delimiters(&mut self, delimiters: &EdifactDelimiters, explicit_una: bool) {
        self.delimiters = *delimiters;
        self.explicit_una = explicit_una;
    }

    fn on_segment(&mut self, seg: &RawSegment) -> Control {
        self.segments.push(OwnedSegment {
            id: seg.id.to_string(),
            elements: seg
                .elements
                .iter()
                .map(|e| e.iter().map(|c| c.to_string()).collect())
                .collect(),
        });
        Control::Continue
    }
}

/// Normalize EDIFACT bytes for comparison: strip \r and \n.
fn normalize_edifact(input: &[u8]) -> Vec<u8> {
    input
        .iter()
        .copied()
        .filter(|&b| b != b'\r' && b != b'\n')
        .collect()
}

/// Returns true if the file should be skipped for strict roundtrip comparison.
///
/// Reasons to skip:
/// - Non-standard escape usage (e.g. `?e` for ß)
/// - Missing trailing segment terminator (malformed but tolerated by parser)
fn should_skip_roundtrip(input: &[u8]) -> bool {
    // Non-standard escape sequences
    let release = b'?';
    let standard_escaped = b":+'.? ";
    for i in 0..input.len().saturating_sub(1) {
        if input[i] == release {
            let next = input[i + 1];
            if !standard_escaped.contains(&next) {
                return true;
            }
        }
    }

    // Missing trailing segment terminator (file doesn't end with ')
    let trimmed: Vec<u8> = input
        .iter()
        .rev()
        .copied()
        .skip_while(|&b| b == b'\r' || b == b'\n' || b == b' ')
        .collect();
    if let Some(&last) = trimmed.first() {
        if last != b'\'' {
            return true;
        }
    }

    false
}

/// Transcode ISO-8859-1 bytes to UTF-8 (mirrors parser's internal transcoding).
///
/// ISO-8859-1 code points 0x00–0xFF map directly to Unicode U+0000–U+00FF.
fn transcode_iso_8859_1_to_utf8(input: &[u8]) -> Vec<u8> {
    let mut output = Vec::with_capacity(input.len() + input.len() / 4);
    for &b in input {
        if b < 0x80 {
            output.push(b);
        } else {
            output.push(0xC0 | (b >> 6));
            output.push(0x80 | (b & 0x3F));
        }
    }
    output
}

/// Normalize input for roundtrip comparison.
///
/// For ISO-8859-1 inputs, the parser transcodes to UTF-8 internally, so
/// reconstructed output will be UTF-8. We transcode the original to match.
fn normalize_for_comparison(input: &[u8]) -> Vec<u8> {
    let normalized = normalize_edifact(input);
    if std::str::from_utf8(&normalized).is_ok() {
        normalized
    } else {
        transcode_iso_8859_1_to_utf8(&normalize_edifact(input))
    }
}

fn fixture_dir() -> Option<std::path::PathBuf> {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let fixture_path = manifest.join("../../example_market_communication_bo4e_transactions");
    if fixture_path.exists() {
        Some(fixture_path)
    } else {
        None
    }
}

fn collect_edi_files(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                files.extend(collect_edi_files(&path));
            } else if path.extension().is_some_and(|ext| ext == "edi") {
                files.push(path);
            }
        }
    }
    files.sort();
    files
}

/// Roundtrip test using a representative sample (first file per message type).
#[test]
fn test_roundtrip_sample_files() {
    let fixture_path = match fixture_dir() {
        Some(p) => p,
        None => {
            eprintln!("Skipping: fixture submodule not initialized");
            return;
        }
    };

    let files = collect_edi_files(&fixture_path);
    assert!(!files.is_empty(), "No .edi files found");

    // Take a sample: first file from each message type directory
    let mut seen_types = std::collections::HashSet::new();
    let mut sample_files = Vec::new();
    for file in &files {
        if let Some(rel) = file.strip_prefix(&fixture_path).ok() {
            if let Some(msg_type) = rel.components().next() {
                let msg_type_str = msg_type.as_os_str().to_string_lossy().to_string();
                if seen_types.insert(msg_type_str) {
                    sample_files.push(file.clone());
                }
            }
        }
    }

    let mut failures: Vec<String> = Vec::new();
    let mut roundtrip_ok = 0;
    let mut skipped_nonstandard = 0;

    for file_path in &sample_files {
        let content = std::fs::read(file_path).unwrap();

        if should_skip_roundtrip(&content) {
            skipped_nonstandard += 1;
            continue;
        }

        let mut handler = SegmentCollector::new();
        if let Err(e) = EdifactStreamParser::parse(&content, &mut handler) {
            failures.push(format!("{}: parse error: {}", file_path.display(), e));
            continue;
        }

        let reconstructed = handler.reconstruct();
        let expected = normalize_for_comparison(&content);

        if reconstructed != expected {
            let rel = file_path.strip_prefix(&fixture_path).unwrap_or(file_path);
            failures.push(format!(
                "{}: roundtrip mismatch\n  original len: {}\n  reconstructed len: {}\n  first diff at: {}",
                rel.display(),
                expected.len(),
                reconstructed.len(),
                find_first_diff(&expected, &reconstructed),
            ));
        } else {
            roundtrip_ok += 1;
        }
    }

    eprintln!(
        "Roundtrip: {} OK, {} skipped (non-ASCII/non-standard/malformed), {} failed",
        roundtrip_ok, skipped_nonstandard, failures.len()
    );

    if !failures.is_empty() {
        panic!(
            "{} roundtrip failures:\n{}",
            failures.len(),
            failures.join("\n\n")
        );
    }
}

/// Full roundtrip test across ALL UTILMD files.
#[test]
fn test_roundtrip_all_utilmd_files() {
    let fixture_path = match fixture_dir() {
        Some(p) => p,
        None => {
            eprintln!("Skipping: fixture submodule not initialized");
            return;
        }
    };

    let utilmd_dir = fixture_path.join("UTILMD");
    if !utilmd_dir.exists() {
        eprintln!("Skipping: UTILMD directory not found");
        return;
    }

    let files = collect_edi_files(&utilmd_dir);
    assert!(!files.is_empty(), "No UTILMD .edi files found");

    let mut failures: Vec<String> = Vec::new();
    let mut roundtrip_ok = 0;
    let mut skipped_nonstandard = 0;

    for file_path in &files {
        let content = std::fs::read(file_path).unwrap();

        if should_skip_roundtrip(&content) {
            skipped_nonstandard += 1;
            continue;
        }

        let mut handler = SegmentCollector::new();
        if EdifactStreamParser::parse(&content, &mut handler).is_err() {
            continue; // Parse failures caught by parse_all_fixtures test
        }

        let reconstructed = handler.reconstruct();
        let expected = normalize_for_comparison(&content);

        if reconstructed != expected {
            let rel = file_path.strip_prefix(&fixture_path).unwrap_or(file_path);
            failures.push(format!(
                "{}: roundtrip mismatch (first diff at {})",
                rel.display(),
                find_first_diff(&expected, &reconstructed),
            ));
        } else {
            roundtrip_ok += 1;
        }
    }

    eprintln!(
        "UTILMD roundtrip: {} OK, {} skipped (non-ASCII/non-standard/malformed), {} failed out of {}",
        roundtrip_ok,
        skipped_nonstandard,
        failures.len(),
        files.len()
    );

    if !failures.is_empty() {
        panic!(
            "{} of {} UTILMD roundtrip failures:\n{}",
            failures.len(),
            files.len(),
            failures.join("\n")
        );
    }
}

fn find_first_diff(a: &[u8], b: &[u8]) -> String {
    for (i, (ab, bb)) in a.iter().zip(b.iter()).enumerate() {
        if ab != bb {
            let context_start = i.saturating_sub(20);
            let context_end_a = (i + 20).min(a.len());
            let context_end_b = (i + 20).min(b.len());
            return format!(
                "byte {} (original: {:?} vs reconstructed: {:?})",
                i,
                String::from_utf8_lossy(&a[context_start..context_end_a]),
                String::from_utf8_lossy(&b[context_start..context_end_b]),
            );
        }
    }
    if a.len() != b.len() {
        format!("lengths differ: {} vs {}", a.len(), b.len())
    } else {
        "no diff found".to_string()
    }
}
