//! Snapshot tests for assembled tree structure.
//!
//! Uses insta for snapshot testing to detect regressions in assembly output.

use automapper_generator::parsing::mig_parser::parse_mig;
use automapper_generator::schema::mig::MigSchema;
use mig_assembly::assembler::{AssembledGroup, AssembledTree, Assembler};
use mig_assembly::tokenize::parse_to_segments;
use std::path::Path;

const MIG_XML_PATH: &str =
    "../../xml-migs-and-ahbs/FV2504/UTILMD_MIG_Strom_S2_1_Fehlerkorrektur_20250320.xml";
const FIXTURE_DIR: &str =
    "../../example_market_communication_bo4e_transactions/UTILMD/FV2504";

fn load_real_mig() -> Option<MigSchema> {
    let path = Path::new(MIG_XML_PATH);
    if !path.exists() {
        eprintln!("MIG XML not found at {MIG_XML_PATH}, skipping");
        return None;
    }
    Some(parse_mig(path, "UTILMD", Some("Strom"), "FV2504").expect("Failed to parse MIG XML"))
}

#[test]
fn test_assembled_tree_snapshot() {
    let Some(mig) = load_real_mig() else { return };

    let fixture_path = Path::new(FIXTURE_DIR).join("55001_UTILMD_S2.1_ALEXANDE121980.edi");
    if !fixture_path.exists() {
        eprintln!("Fixture not found, skipping");
        return;
    }

    let content = std::fs::read(&fixture_path).unwrap();
    let segments = parse_to_segments(&content).unwrap();

    let assembler = Assembler::new(&mig);
    let tree = assembler.assemble_generic(&segments).unwrap();

    let summary = summarize_tree(&tree);
    insta::assert_snapshot!(summary);
}

fn summarize_tree(tree: &AssembledTree) -> String {
    let mut out = String::new();
    out.push_str(&format!("Top-level segments: {}\n", tree.segments.len()));
    out.push_str(&format!(
        "  Pre-group: {}\n",
        tree.post_group_start
    ));
    out.push_str(&format!(
        "  Post-group: {}\n",
        tree.segments.len() - tree.post_group_start
    ));
    for seg in &tree.segments {
        out.push_str(&format!("  {}: {} elements\n", seg.tag, seg.elements.len()));
    }
    out.push_str(&format!("Groups: {}\n", tree.groups.len()));
    for group in &tree.groups {
        summarize_group(group, &mut out, 1);
    }
    out
}

fn summarize_group(group: &AssembledGroup, out: &mut String, indent: usize) {
    let pad = "  ".repeat(indent);
    out.push_str(&format!(
        "{}{}: {} repetitions\n",
        pad,
        group.group_id,
        group.repetitions.len()
    ));
    for (i, rep) in group.repetitions.iter().enumerate() {
        out.push_str(&format!(
            "{}  rep[{i}]: {} segments, {} child groups\n",
            pad,
            rep.segments.len(),
            rep.child_groups.len()
        ));
        for seg in &rep.segments {
            out.push_str(&format!("{}    {}\n", pad, seg.tag));
        }
        for child in &rep.child_groups {
            summarize_group(child, out, indent + 2);
        }
    }
}
