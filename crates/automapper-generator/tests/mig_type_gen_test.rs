use automapper_generator::codegen::mig_type_gen;
use automapper_generator::parsing::mig_parser::parse_mig;
use automapper_generator::schema::mig::MigSegmentGroup;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

fn load_utilmd_mig() -> automapper_generator::schema::mig::MigSchema {
    parse_mig(
        Path::new(
            "../../xml-migs-and-ahbs/FV2504/UTILMD_MIG_Strom_S2_1_Fehlerkorrektur_20250320.xml",
        ),
        "UTILMD",
        Some("Strom"),
        "FV2504",
    )
    .expect("Failed to parse UTILMD MIG XML")
}

#[test]
fn test_generate_enums_from_utilmd_mig() {
    let mig = load_utilmd_mig();

    let enums_source = mig_type_gen::generate_enums(&mig);

    // Should contain D3035Qualifier with MS, MR
    assert!(
        enums_source.contains("pub enum D3035Qualifier"),
        "Missing D3035 enum"
    );
    assert!(enums_source.contains("MS"), "Missing MS variant");
    assert!(enums_source.contains("MR"), "Missing MR variant");

    // Should derive standard traits
    assert!(enums_source
        .contains("#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]"));

    // Should have Display impl for roundtrip to string
    assert!(enums_source.contains("impl std::fmt::Display for D3035Qualifier"));

    // Should have FromStr impl for parsing
    assert!(enums_source.contains("impl std::str::FromStr for D3035Qualifier"));

    // Should compile as valid Rust (syntax check via string inspection)
    assert!(!enums_source.contains("TODO"));
}

#[test]
fn test_generate_composites_from_utilmd_mig() {
    let mig = load_utilmd_mig();

    let composites_source = mig_type_gen::generate_composites(&mig);

    // Should contain C082 (party identification)
    assert!(
        composites_source.contains("pub struct CompositeC082"),
        "Missing C082"
    );
    // Fields should use Option for conditional elements
    assert!(composites_source.contains("Option<String>"));
    // Fields with code lists should reference the enum type
    assert!(
        composites_source.contains("D3055Qualifier")
            || composites_source.contains("Option<D3055Qualifier>"),
        "Missing D3055Qualifier reference"
    );
    // Should derive Serialize, Deserialize
    assert!(
        composites_source.contains("#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]")
    );
}

#[test]
fn test_generate_segments_from_utilmd_mig() {
    let mig = load_utilmd_mig();

    let segments_source = mig_type_gen::generate_segments(&mig);

    // Should contain NAD segment
    assert!(
        segments_source.contains("pub struct SegNad"),
        "Missing SegNad"
    );
    // Should contain UNH segment
    assert!(
        segments_source.contains("pub struct SegUnh"),
        "Missing SegUnh"
    );
    // Should contain BGM segment
    assert!(
        segments_source.contains("pub struct SegBgm"),
        "Missing SegBgm"
    );
    // Segments should reference composites
    assert!(
        segments_source.contains("CompositeC082")
            || segments_source.contains("Option<CompositeC082>"),
        "Missing CompositeC082 reference"
    );
    // Segments should have direct data element fields too
    assert!(segments_source.contains("d3035"), "Missing d3035 field");
}

#[test]
fn test_generate_groups_from_utilmd_mig() {
    let mig = load_utilmd_mig();

    let groups_source = mig_type_gen::generate_groups(&mig);

    // Should contain SG2 (party group)
    assert!(
        groups_source.contains("pub struct Sg2"),
        "Missing SG2 group"
    );
    // Groups should reference segments
    assert!(
        groups_source.contains("SegNad"),
        "Missing SegNad reference in groups"
    );
}

#[test]
fn test_generate_mig_types_writes_files() {
    let output_dir = tempfile::tempdir().unwrap();

    mig_type_gen::generate_mig_types(
        Path::new(
            "../../xml-migs-and-ahbs/FV2504/UTILMD_MIG_Strom_S2_1_Fehlerkorrektur_20250320.xml",
        ),
        "UTILMD",
        Some("Strom"),
        "FV2504",
        output_dir.path(),
    )
    .unwrap();

    // Should create version/message module structure
    let base = output_dir.path().join("fv2504").join("utilmd");
    assert!(base.join("enums.rs").exists(), "Missing enums.rs");
    assert!(base.join("composites.rs").exists(), "Missing composites.rs");
    assert!(base.join("segments.rs").exists(), "Missing segments.rs");
    assert!(base.join("groups.rs").exists(), "Missing groups.rs");
    assert!(base.join("mod.rs").exists(), "Missing mod.rs");

    // mod.rs should re-export all modules
    let mod_content = std::fs::read_to_string(base.join("mod.rs")).unwrap();
    assert!(mod_content.contains("pub mod enums;"));
    assert!(mod_content.contains("pub mod composites;"));
    assert!(mod_content.contains("pub mod segments;"));
    assert!(mod_content.contains("pub mod groups;"));
}

// ---------------------------------------------------------------------------
// Real-data validation tests
// ---------------------------------------------------------------------------

/// Recursively collect all group definitions from the MIG tree.
/// Returns a merged map: group_id -> (set of segment IDs, set of nested group IDs).
fn collect_mig_group_contents(
    groups: &[MigSegmentGroup],
) -> BTreeMap<String, (BTreeSet<String>, BTreeSet<String>)> {
    let mut result: BTreeMap<String, (BTreeSet<String>, BTreeSet<String>)> = BTreeMap::new();

    fn visit(
        group: &MigSegmentGroup,
        result: &mut BTreeMap<String, (BTreeSet<String>, BTreeSet<String>)>,
    ) {
        // Collect segment IDs and nested group IDs into local sets to avoid
        // borrowing `result` mutably while it's already borrowed.
        let seg_ids: Vec<String> = group.segments.iter().map(|s| s.id.clone()).collect();
        let nested_ids: Vec<String> = group.nested_groups.iter().map(|n| n.id.clone()).collect();

        let entry = result.entry(group.id.clone()).or_default();
        for seg_id in seg_ids {
            entry.0.insert(seg_id);
        }
        for nested_id in &nested_ids {
            entry.1.insert(nested_id.clone());
        }

        for nested in &group.nested_groups {
            visit(nested, result);
        }
    }

    for g in groups {
        visit(g, &mut result);
    }
    result
}

/// Parse the generated groups source code and extract struct fields.
/// Returns: struct_name -> (segment field types, nested group field types).
fn parse_generated_groups(source: &str) -> BTreeMap<String, (BTreeSet<String>, BTreeSet<String>)> {
    let mut result: BTreeMap<String, (BTreeSet<String>, BTreeSet<String>)> = BTreeMap::new();
    let mut current_struct: Option<String> = None;

    for line in source.lines() {
        if let Some(rest) = line.strip_prefix("pub struct ") {
            let name = rest.split_whitespace().next().unwrap_or("");
            current_struct = Some(name.to_string());
            result.entry(name.to_string()).or_default();
        } else if let Some(ref struct_name) = current_struct {
            if line.contains("pub ") && line.contains(':') {
                // Extract the type — look for Seg* or Sg* references
                if let Some(seg_match) = line.split("Seg").nth(1) {
                    let seg_type = seg_match
                        .split(|c: char| !c.is_alphanumeric())
                        .next()
                        .unwrap_or("");
                    if !seg_type.is_empty() {
                        let seg_id = seg_type.to_uppercase();
                        result
                            .entry(struct_name.clone())
                            .or_default()
                            .0
                            .insert(seg_id);
                    }
                }
                // Check for nested group references (Sg followed by digits)
                for word in line.split(|c: char| !c.is_alphanumeric()) {
                    if word.starts_with("Sg")
                        && word.len() > 2
                        && word[2..].chars().all(|c| c.is_ascii_digit())
                    {
                        let group_id = format!("SG{}", &word[2..]);
                        result
                            .entry(struct_name.clone())
                            .or_default()
                            .1
                            .insert(group_id);
                    }
                }
            }
            if line == "}" {
                current_struct = None;
            }
        }
    }
    result
}

#[test]
fn test_generated_groups_contain_all_mig_segments() {
    let mig = load_utilmd_mig();
    let groups_source = mig_type_gen::generate_groups(&mig);

    // Get the ground truth from MIG XML
    let mig_contents = collect_mig_group_contents(&mig.segment_groups);
    // Parse the generated code
    let gen_contents = parse_generated_groups(&groups_source);

    for (group_id, (expected_segs, expected_nested)) in &mig_contents {
        let group_num = group_id.trim_start_matches("SG");
        let struct_name = format!("Sg{group_num}");

        let gen = gen_contents.get(&struct_name).unwrap_or_else(|| {
            panic!(
                "Generated code missing group struct {struct_name} (for MIG {})",
                group_id
            )
        });

        for seg_id in expected_segs {
            assert!(
                gen.0.contains(seg_id),
                "Group {group_id} ({struct_name}) missing segment {seg_id}.\n\
                 Expected: {expected_segs:?}\n\
                 Got: {:?}",
                gen.0
            );
        }

        for nested_id in expected_nested {
            assert!(
                gen.1.contains(nested_id),
                "Group {group_id} ({struct_name}) missing nested group {nested_id}.\n\
                 Expected: {expected_nested:?}\n\
                 Got: {:?}",
                gen.1
            );
        }
    }
}

#[test]
fn test_all_mig_groups_are_generated() {
    let mig = load_utilmd_mig();
    let groups_source = mig_type_gen::generate_groups(&mig);

    let mig_contents = collect_mig_group_contents(&mig.segment_groups);
    let gen_contents = parse_generated_groups(&groups_source);

    for group_id in mig_contents.keys() {
        let group_num = group_id.trim_start_matches("SG");
        let struct_name = format!("Sg{group_num}");
        assert!(
            gen_contents.contains_key(&struct_name),
            "MIG group {group_id} has no generated struct {struct_name}. \
             Generated groups: {:?}",
            gen_contents.keys().collect::<Vec<_>>()
        );
    }
}

#[test]
fn test_sg4_has_dtm_ftx_agr_segments() {
    let mig = load_utilmd_mig();
    let groups_source = mig_type_gen::generate_groups(&mig);

    // SG4 is the main transaction group — must have DTM, FTX, AGR, STS, IDE
    assert!(
        groups_source.contains("pub struct Sg4"),
        "Missing Sg4 struct"
    );

    // Extract the Sg4 struct body
    let sg4_start = groups_source.find("pub struct Sg4").unwrap();
    let sg4_body = &groups_source[sg4_start..];
    let sg4_end = sg4_body.find("\n}\n").unwrap() + 3;
    let sg4_def = &sg4_body[..sg4_end];

    assert!(sg4_def.contains("SegIde"), "Sg4 missing IDE segment");
    assert!(sg4_def.contains("SegSts"), "Sg4 missing STS segment");
    assert!(sg4_def.contains("SegDtm"), "Sg4 missing DTM segment");
    assert!(sg4_def.contains("SegFtx"), "Sg4 missing FTX segment");
    assert!(sg4_def.contains("SegAgr"), "Sg4 missing AGR segment");
    assert!(sg4_def.contains("Sg5"), "Sg4 missing nested SG5");
    assert!(sg4_def.contains("Sg6"), "Sg4 missing nested SG6");
    assert!(sg4_def.contains("Sg8"), "Sg4 missing nested SG8");
    assert!(sg4_def.contains("Sg12"), "Sg4 missing nested SG12");
}

#[test]
fn test_sg8_has_pia_and_nested_sg9() {
    let mig = load_utilmd_mig();
    let groups_source = mig_type_gen::generate_groups(&mig);

    let sg8_start = groups_source.find("pub struct Sg8").unwrap();
    let sg8_body = &groups_source[sg8_start..];
    let sg8_end = sg8_body.find("\n}\n").unwrap() + 3;
    let sg8_def = &sg8_body[..sg8_end];

    assert!(sg8_def.contains("SegSeq"), "Sg8 missing SEQ segment");
    assert!(sg8_def.contains("SegRff"), "Sg8 missing RFF segment");
    assert!(sg8_def.contains("SegPia"), "Sg8 missing PIA segment");
    assert!(sg8_def.contains("Sg9"), "Sg8 missing nested SG9");
    assert!(sg8_def.contains("Sg10"), "Sg8 missing nested SG10");
}

#[test]
fn test_sg9_exists_with_qty_dtm() {
    let mig = load_utilmd_mig();
    let groups_source = mig_type_gen::generate_groups(&mig);

    assert!(
        groups_source.contains("pub struct Sg9"),
        "Missing Sg9 struct"
    );

    let sg9_start = groups_source.find("pub struct Sg9").unwrap();
    let sg9_body = &groups_source[sg9_start..];
    let sg9_end = sg9_body.find("\n}\n").unwrap() + 3;
    let sg9_def = &sg9_body[..sg9_end];

    assert!(sg9_def.contains("SegQty"), "Sg9 missing QTY segment");
    assert!(sg9_def.contains("SegDtm"), "Sg9 missing DTM segment");
}

#[test]
fn test_sg10_has_cav_segment() {
    let mig = load_utilmd_mig();
    let groups_source = mig_type_gen::generate_groups(&mig);

    let sg10_start = groups_source.find("pub struct Sg10").unwrap();
    let sg10_body = &groups_source[sg10_start..];
    let sg10_end = sg10_body.find("\n}\n").unwrap() + 3;
    let sg10_def = &sg10_body[..sg10_end];

    assert!(sg10_def.contains("SegCci"), "Sg10 missing CCI segment");
    assert!(sg10_def.contains("SegCav"), "Sg10 missing CAV segment");
}

#[test]
fn test_sg12_exists_with_nad_rff() {
    let mig = load_utilmd_mig();
    let groups_source = mig_type_gen::generate_groups(&mig);

    assert!(
        groups_source.contains("pub struct Sg12"),
        "Missing Sg12 struct"
    );

    let sg12_start = groups_source.find("pub struct Sg12").unwrap();
    let sg12_body = &groups_source[sg12_start..];
    let sg12_end = sg12_body.find("\n}\n").unwrap() + 3;
    let sg12_def = &sg12_body[..sg12_end];

    assert!(sg12_def.contains("SegNad"), "Sg12 missing NAD segment");
    assert!(sg12_def.contains("SegRff"), "Sg12 missing RFF segment");
}

#[test]
fn test_sg6_has_dtm_segment() {
    let mig = load_utilmd_mig();
    let groups_source = mig_type_gen::generate_groups(&mig);

    let sg6_start = groups_source.find("pub struct Sg6").unwrap();
    let sg6_body = &groups_source[sg6_start..];
    let sg6_end = sg6_body.find("\n}\n").unwrap() + 3;
    let sg6_def = &sg6_body[..sg6_end];

    assert!(sg6_def.contains("SegRff"), "Sg6 missing RFF segment");
    assert!(sg6_def.contains("SegDtm"), "Sg6 missing DTM segment");
}

#[test]
fn test_all_segment_tags_in_fixtures_have_generated_types() {
    let mig = load_utilmd_mig();
    let segments_source = mig_type_gen::generate_segments(&mig);

    // Collect all segment tags from real EDIFACT fixture files
    let fixtures_dir =
        Path::new("../../example_market_communication_bo4e_transactions/UTILMD/FV2504");
    if !fixtures_dir.exists() {
        eprintln!("Skipping fixture test — submodule not checked out");
        return;
    }

    let mut all_segment_tags: BTreeSet<String> = BTreeSet::new();
    for entry in std::fs::read_dir(fixtures_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "edi") {
            let content = std::fs::read_to_string(&path).unwrap_or_default();
            for segment_str in content.split('\'') {
                let tag = segment_str.split(['+', ':']).next().unwrap_or("").trim();
                if !tag.is_empty()
                    && tag.chars().all(|c| c.is_ascii_alphanumeric())
                    && tag.len() <= 4
                {
                    all_segment_tags.insert(tag.to_string());
                }
            }
        }
    }

    // Envelope segments (UNA, UNB, UNZ) are not in the MIG — filter them out
    let envelope_tags: BTreeSet<String> = ["UNA", "UNB", "UNZ"]
        .iter()
        .map(|s| s.to_string())
        .collect();

    let missing: Vec<_> = all_segment_tags
        .difference(&envelope_tags)
        .filter(|tag| {
            let struct_name = format!("pub struct Seg{}", capitalize_segment_id(tag));
            !segments_source.contains(&struct_name)
        })
        .collect();

    assert!(
        missing.is_empty(),
        "Segment tags found in EDIFACT fixtures but missing generated types: {:?}",
        missing
    );
}

/// Helper to capitalize segment IDs the same way the generator does.
fn capitalize_segment_id(id: &str) -> String {
    let mut chars = id.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => {
            let rest: String = chars.map(|c| c.to_ascii_lowercase()).collect();
            format!("{}{}", first.to_ascii_uppercase(), rest)
        }
    }
}

#[test]
fn test_enum_coverage_against_fixtures() {
    let mig = load_utilmd_mig();
    let enums_source = mig_type_gen::generate_enums(&mig);

    let fixtures_dir =
        Path::new("../../example_market_communication_bo4e_transactions/UTILMD/FV2504");
    if !fixtures_dir.exists() {
        eprintln!("Skipping fixture test — submodule not checked out");
        return;
    }

    // Collect NAD qualifier values from real files (first data element after NAD+)
    let mut nad_qualifiers: BTreeSet<String> = BTreeSet::new();
    // Collect IDE qualifier values
    let mut ide_qualifiers: BTreeSet<String> = BTreeSet::new();
    // Collect LOC qualifier values
    let mut loc_qualifiers: BTreeSet<String> = BTreeSet::new();

    for entry in std::fs::read_dir(fixtures_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "edi") {
            let content = std::fs::read_to_string(&path).unwrap_or_default();
            for segment_str in content.split('\'') {
                let parts: Vec<&str> = segment_str.split('+').collect();
                let tag = parts.first().map(|s| s.trim()).unwrap_or("");
                match tag {
                    "NAD" => {
                        if let Some(qual) = parts.get(1) {
                            nad_qualifiers.insert(qual.to_string());
                        }
                    }
                    "IDE" => {
                        if let Some(qual) = parts.get(1) {
                            ide_qualifiers.insert(qual.to_string());
                        }
                    }
                    "LOC" => {
                        if let Some(qual) = parts.get(1) {
                            loc_qualifiers.insert(qual.to_string());
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    // D3035 = NAD qualifier
    for qual in &nad_qualifiers {
        assert!(
            enums_source.contains(&format!("\"{}\" =>", qual))
                || enums_source.contains(&format!("=> write!(f, \"{}\")", qual)),
            "NAD qualifier '{}' from fixtures not found in D3035Qualifier enum",
            qual
        );
    }

    // D7495 = IDE qualifier
    for qual in &ide_qualifiers {
        assert!(
            enums_source.contains(&format!("\"{}\" =>", qual))
                || enums_source.contains(&format!("=> write!(f, \"{}\")", qual)),
            "IDE qualifier '{}' from fixtures not found in D7495Qualifier enum",
            qual
        );
    }

    // D3227 = LOC qualifier
    for qual in &loc_qualifiers {
        assert!(
            enums_source.contains(&format!("\"{}\" =>", qual))
                || enums_source.contains(&format!("=> write!(f, \"{}\")", qual)),
            "LOC qualifier '{}' from fixtures not found in D3227Qualifier enum",
            qual
        );
    }
}
