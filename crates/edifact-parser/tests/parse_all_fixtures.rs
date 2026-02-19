//! Port of C# `EdifactRoundtripTests.Parse_AllEdifactFiles_ShouldNotThrow`.
//!
//! Scans all .edi files in the fixture submodule and verifies that
//! every single one parses without panicking or returning an error.

use edifact_parser::{EdifactHandler, EdifactStreamParser};
use edifact_types::{Control, RawSegment};
use std::path::Path;

/// Minimal handler that counts segments and tracks structural validity.
struct CollectingHandler {
    segment_count: usize,
    segment_ids: Vec<String>,
    has_unb: bool,
    has_unz: bool,
    has_unh: bool,
    has_unt: bool,
    message_count: usize,
}

impl CollectingHandler {
    fn new() -> Self {
        Self {
            segment_count: 0,
            segment_ids: Vec::new(),
            has_unb: false,
            has_unz: false,
            has_unh: false,
            has_unt: false,
            message_count: 0,
        }
    }
}

impl EdifactHandler for CollectingHandler {
    fn on_interchange_start(&mut self, _unb: &RawSegment) -> Control {
        self.has_unb = true;
        Control::Continue
    }

    fn on_interchange_end(&mut self, _unz: &RawSegment) {
        self.has_unz = true;
    }

    fn on_message_start(&mut self, _unh: &RawSegment) -> Control {
        self.has_unh = true;
        Control::Continue
    }

    fn on_message_end(&mut self, _unt: &RawSegment) {
        self.has_unt = true;
        self.message_count += 1;
    }

    fn on_segment(&mut self, seg: &RawSegment) -> Control {
        self.segment_count += 1;
        self.segment_ids.push(seg.id.to_string());
        Control::Continue
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

#[test]
fn test_parse_all_edi_files_should_not_fail() {
    let fixture_path = match fixture_dir() {
        Some(p) => p,
        None => {
            eprintln!("Skipping: fixture submodule not initialized");
            return;
        }
    };

    let files = collect_edi_files(&fixture_path);
    assert!(
        !files.is_empty(),
        "No .edi files found in fixture directory"
    );

    let mut failures: Vec<String> = Vec::new();
    let mut success_count = 0;

    for file_path in &files {
        let content = match std::fs::read(file_path) {
            Ok(c) => c,
            Err(e) => {
                failures.push(format!("{}: read error: {}", file_path.display(), e));
                continue;
            }
        };

        let mut handler = CollectingHandler::new();
        match EdifactStreamParser::parse(&content, &mut handler) {
            Ok(()) => {
                if handler.segment_count == 0 {
                    failures.push(format!("{}: parsed OK but 0 segments", file_path.display()));
                } else {
                    success_count += 1;
                }
            }
            Err(e) => {
                failures.push(format!("{}: parse error: {}", file_path.display(), e));
            }
        }
    }

    eprintln!(
        "Parsed {}/{} files successfully",
        success_count,
        files.len()
    );

    if !failures.is_empty() {
        panic!(
            "{} of {} files failed to parse:\n{}",
            failures.len(),
            files.len(),
            failures.join("\n")
        );
    }
}

#[test]
fn test_all_edi_files_have_valid_structure() {
    let fixture_path = match fixture_dir() {
        Some(p) => p,
        None => {
            eprintln!("Skipping: fixture submodule not initialized");
            return;
        }
    };

    let files = collect_edi_files(&fixture_path);
    let mut failures: Vec<String> = Vec::new();

    for file_path in &files {
        let content = match std::fs::read(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let mut handler = CollectingHandler::new();
        if EdifactStreamParser::parse(&content, &mut handler).is_err() {
            continue; // Parse failures caught by the other test
        }

        let rel = file_path.strip_prefix(&fixture_path).unwrap_or(file_path);

        if !handler.has_unb {
            failures.push(format!("{}: missing UNB", rel.display()));
        }
        if !handler.has_unz {
            failures.push(format!("{}: missing UNZ", rel.display()));
        }
        if !handler.has_unh {
            failures.push(format!("{}: missing UNH", rel.display()));
        }
        if !handler.has_unt {
            failures.push(format!("{}: missing UNT", rel.display()));
        }

        // First segment should be UNB (or UNA handled before)
        if let Some(first) = handler.segment_ids.first() {
            if first != "UNB" {
                failures.push(format!(
                    "{}: first segment is {} (expected UNB)",
                    rel.display(),
                    first
                ));
            }
        }
        // Last segment should be UNZ
        if let Some(last) = handler.segment_ids.last() {
            if last != "UNZ" {
                failures.push(format!(
                    "{}: last segment is {} (expected UNZ)",
                    rel.display(),
                    last
                ));
            }
        }
    }

    if !failures.is_empty() {
        panic!(
            "{} structural issues found:\n{}",
            failures.len(),
            failures.join("\n")
        );
    }
}
