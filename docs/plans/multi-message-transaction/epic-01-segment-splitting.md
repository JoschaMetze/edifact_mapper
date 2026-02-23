---
feature: multi-message-transaction
epic: 1
title: "Segment Splitting at UNH/UNT Boundaries"
depends_on: []
estimated_tasks: 4
crate: mig-assembly
status: complete
---

# Epic 1: Segment Splitting at UNH/UNT Boundaries

> **For Claude:** Implement each task in sequence. Every task follows TDD: write the test FIRST, run it to see it fail, write the implementation, run it to see it pass, then commit. The workspace must compile with `cargo check --workspace` after every task.

**Goal:** Add a `split_messages()` function to `mig-assembly::tokenize` that takes a flat `Vec<OwnedSegment>` and splits it into per-message chunks at UNH/UNT boundaries. Each chunk includes the interchange envelope segments (UNA, UNB) so it can be independently assembled.

**Architecture:** Post-processing step after `parse_to_segments()`. Walks the segment list, collects UNA/UNB as envelope, then creates a new chunk for each UNH...UNT pair. Each chunk gets a copy of the envelope segments prepended so the existing assembler works unchanged.

**Tech Stack:** Rust, `mig-assembly::tokenize`, `mig_types::segment::OwnedSegment`

---

## Task 1: Define InterchangeChunks and MessageChunk Types

**Files:**
- Modify: `crates/mig-assembly/src/tokenize.rs`

**Step 1: Write the failing test**

Add to the `#[cfg(test)] mod tests` block in `crates/mig-assembly/src/tokenize.rs`:

```rust
#[test]
fn test_message_chunk_struct_exists() {
    let chunk = MessageChunk {
        envelope: vec![],
        unh: OwnedSegment { id: "UNH".to_string(), elements: vec![], segment_number: 0 },
        body: vec![],
        unt: OwnedSegment { id: "UNT".to_string(), elements: vec![], segment_number: 1 },
    };
    assert_eq!(chunk.unh.id, "UNH");
    assert_eq!(chunk.unt.id, "UNT");
    assert!(chunk.envelope.is_empty());
    assert!(chunk.body.is_empty());
}

#[test]
fn test_interchange_chunks_struct_exists() {
    let chunks = InterchangeChunks {
        envelope: vec![],
        messages: vec![],
        unz: None,
    };
    assert!(chunks.messages.is_empty());
    assert!(chunks.unz.is_none());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p mig-assembly test_message_chunk_struct_exists`
Expected: FAIL — `MessageChunk` not found

**Step 3: Write minimal implementation**

Add the following types above the `SegmentCollector` struct in `crates/mig-assembly/src/tokenize.rs`:

```rust
/// A single EDIFACT message (UNH...UNT) with its interchange envelope.
#[derive(Debug, Clone)]
pub struct MessageChunk {
    /// Interchange envelope segments (UNA, UNB) — shared across all messages.
    pub envelope: Vec<OwnedSegment>,
    /// The UNH segment itself.
    pub unh: OwnedSegment,
    /// Segments between UNH and UNT (exclusive of both).
    pub body: Vec<OwnedSegment>,
    /// The UNT segment itself.
    pub unt: OwnedSegment,
}

/// A complete EDIFACT interchange split into per-message chunks.
#[derive(Debug, Clone)]
pub struct InterchangeChunks {
    /// Interchange envelope segments (UNA, UNB) — shared across all messages.
    pub envelope: Vec<OwnedSegment>,
    /// One entry per UNH/UNT pair.
    pub messages: Vec<MessageChunk>,
    /// The UNZ segment (interchange trailer), if present.
    pub unz: Option<OwnedSegment>,
}
```

Also add a helper method on `MessageChunk`:

```rust
impl MessageChunk {
    /// Reconstruct the full segment list for this message (envelope + UNH + body + UNT).
    /// This is the input format expected by `Assembler::assemble_generic()`.
    pub fn all_segments(&self) -> Vec<OwnedSegment> {
        let mut segs = self.envelope.clone();
        segs.push(self.unh.clone());
        segs.extend(self.body.iter().cloned());
        segs.push(self.unt.clone());
        segs
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p mig-assembly test_message_chunk_struct_exists && cargo test -p mig-assembly test_interchange_chunks_struct_exists`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/mig-assembly/src/tokenize.rs
git commit -m "feat(mig-assembly): add MessageChunk and InterchangeChunks types"
```

---

## Task 2: Implement split_messages() for Single-Message Input

**Files:**
- Modify: `crates/mig-assembly/src/tokenize.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn test_split_messages_single_message() {
    let input = b"UNA:+.? 'UNB+UNOC:3+SENDER+RECEIVER+210101:1200+REF001'UNH+MSG001+UTILMD:D:11A:UN:S2.1'BGM+E03+DOC001'UNT+3+MSG001'UNZ+1+REF001'";
    let segments = parse_to_segments(input).unwrap();
    let chunks = split_messages(segments).unwrap();

    assert_eq!(chunks.messages.len(), 1);
    assert_eq!(chunks.envelope.len(), 1); // UNB only (UNA not emitted by parser)
    assert!(chunks.unz.is_some());

    let msg = &chunks.messages[0];
    assert!(msg.unh.is("UNH"));
    assert!(msg.unt.is("UNT"));
    assert_eq!(msg.body.len(), 1); // BGM only
    assert!(msg.body[0].is("BGM"));

    // all_segments() should reconstruct: UNB, UNH, BGM, UNT
    let all = msg.all_segments();
    assert_eq!(all.len(), 4);
    assert!(all[0].is("UNB"));
    assert!(all[1].is("UNH"));
    assert!(all[2].is("BGM"));
    assert!(all[3].is("UNT"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p mig-assembly test_split_messages_single_message`
Expected: FAIL — `split_messages` not found

**Step 3: Write implementation**

Add after the `parse_to_segments()` function:

```rust
/// Split a flat segment list into per-message chunks at UNH/UNT boundaries.
///
/// Each message gets a copy of the interchange envelope (UNB and any segments
/// before the first UNH) so it can be independently assembled.
///
/// # Errors
///
/// Returns an error if no UNH/UNT pairs are found.
pub fn split_messages(segments: Vec<OwnedSegment>) -> Result<InterchangeChunks, crate::AssemblyError> {
    let mut envelope: Vec<OwnedSegment> = Vec::new();
    let mut messages: Vec<MessageChunk> = Vec::new();
    let mut unz: Option<OwnedSegment> = None;

    // State machine
    let mut current_unh: Option<OwnedSegment> = None;
    let mut current_body: Vec<OwnedSegment> = Vec::new();
    let mut seen_first_unh = false;

    for seg in segments {
        let id_upper = seg.id.to_uppercase();
        match id_upper.as_str() {
            "UNH" => {
                // If we were in a message without UNT (shouldn't happen), discard it
                seen_first_unh = true;
                current_unh = Some(seg);
                current_body.clear();
            }
            "UNT" => {
                if let Some(unh) = current_unh.take() {
                    messages.push(MessageChunk {
                        envelope: envelope.clone(),
                        unh,
                        body: std::mem::take(&mut current_body),
                        unt: seg,
                    });
                }
                // else: UNT without UNH — ignore
            }
            "UNZ" => {
                unz = Some(seg);
            }
            _ => {
                if seen_first_unh {
                    // Inside a message
                    current_body.push(seg);
                } else {
                    // Before first UNH — part of envelope
                    envelope.push(seg);
                }
            }
        }
    }

    if messages.is_empty() {
        return Err(crate::AssemblyError::ParseError(
            "No UNH/UNT message pairs found in interchange".to_string(),
        ));
    }

    Ok(InterchangeChunks {
        envelope,
        messages,
        unz,
    })
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p mig-assembly test_split_messages_single_message`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/mig-assembly/src/tokenize.rs
git commit -m "feat(mig-assembly): implement split_messages() for single-message input"
```

---

## Task 3: Handle Multi-Message Interchanges

**Files:**
- Modify: `crates/mig-assembly/src/tokenize.rs`

**Step 1: Write the failing tests**

```rust
#[test]
fn test_split_messages_two_messages() {
    let input = b"UNA:+.? 'UNB+UNOC:3+SENDER+RECEIVER+210101:1200+REF001'UNH+001+UTILMD:D:11A:UN:S2.1'BGM+E01+DOC001'UNT+2+001'UNH+002+UTILMD:D:11A:UN:S2.1'BGM+E03+DOC002'DTM+137:20250101:102'UNT+3+002'UNZ+2+REF001'";
    let segments = parse_to_segments(input).unwrap();
    let chunks = split_messages(segments).unwrap();

    assert_eq!(chunks.messages.len(), 2);

    // First message: UNH, BGM, UNT
    let msg1 = &chunks.messages[0];
    assert_eq!(msg1.unh.get_element(0), "001");
    assert_eq!(msg1.body.len(), 1);
    assert!(msg1.body[0].is("BGM"));

    // Second message: UNH, BGM, DTM, UNT
    let msg2 = &chunks.messages[1];
    assert_eq!(msg2.unh.get_element(0), "002");
    assert_eq!(msg2.body.len(), 2);
    assert!(msg2.body[0].is("BGM"));
    assert!(msg2.body[1].is("DTM"));

    // Both messages share the same envelope
    assert_eq!(msg1.envelope.len(), msg2.envelope.len());
    assert!(msg1.envelope[0].is("UNB"));
}

#[test]
fn test_split_messages_envelope_preserved_per_message() {
    // Each message's all_segments() should start with envelope
    let input = b"UNA:+.? 'UNB+UNOC:3+SEND+RECV+210101:1200+REF'UNH+001+UTILMD:D:11A:UN:S2.1'UNT+1+001'UNH+002+UTILMD:D:11A:UN:S2.1'UNT+1+002'UNZ+2+REF'";
    let segments = parse_to_segments(input).unwrap();
    let chunks = split_messages(segments).unwrap();

    for msg in &chunks.messages {
        let all = msg.all_segments();
        assert!(all[0].is("UNB"), "First segment should be UNB");
        assert!(all[1].is("UNH"), "Second segment should be UNH");
        assert!(all.last().unwrap().is("UNT"), "Last segment should be UNT");
    }
}

#[test]
fn test_split_messages_no_messages_errors() {
    let input = b"UNA:+.? 'UNB+UNOC:3+S+R+210101:1200+REF'UNZ+0+REF'";
    let segments = parse_to_segments(input).unwrap();
    let result = split_messages(segments);
    assert!(result.is_err());
}
```

**Step 2: Run tests to verify they pass**

These should already pass with the Task 2 implementation since it handles multiple UNH/UNT pairs.

Run: `cargo test -p mig-assembly test_split_messages`
Expected: ALL PASS

If any fail, adjust the implementation to handle edge cases.

**Step 3: Commit**

```bash
git add crates/mig-assembly/src/tokenize.rs
git commit -m "test(mig-assembly): add multi-message and edge case tests for split_messages"
```

---

## Task 4: Export split_messages and Types from Crate

**Files:**
- Modify: `crates/mig-assembly/src/lib.rs`

**Step 1: Add public re-exports**

In `crates/mig-assembly/src/lib.rs`, add re-exports after the existing ones:

```rust
pub use tokenize::{split_messages, InterchangeChunks, MessageChunk};
```

**Step 2: Verify workspace compiles**

Run: `cargo check --workspace`
Expected: OK

Run: `cargo clippy --workspace -- -D warnings`
Expected: No warnings

**Step 3: Commit**

```bash
git add crates/mig-assembly/src/lib.rs
git commit -m "feat(mig-assembly): export split_messages and chunk types"
```

## Test Summary

| Metric | Value |
|--------|-------|
| Tests | 45 |
| Passed | 45 |
| Failed | 0 |
| Skipped | 0 |

Files tested:
- `crates/mig-assembly/src/tokenize.rs` (6 new tests: struct existence, single-message split, two-message split, envelope preservation, no-messages error, plus existing tests)
- `crates/mig-assembly/src/lib.rs` (re-exports verified via `cargo check --workspace`)
