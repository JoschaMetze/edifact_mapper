//! Full roundtrip test: EDIFACT → forward → reverse → disassemble → render → compare.
//!
//! Validates that map_interchange() followed by map_interchange_reverse()
//! produces a tree that can be disassembled back to the original EDIFACT.

mod common;
use common::test_utils;

#[test]
fn test_forward_reverse_roundtrip_55001() {
    test_utils::run_full_roundtrip("55001");
}

#[test]
fn test_forward_reverse_roundtrip_55002() {
    test_utils::run_full_roundtrip("55002");
}
