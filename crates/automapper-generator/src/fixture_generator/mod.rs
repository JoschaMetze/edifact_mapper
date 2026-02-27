pub mod enhancer;
pub mod id_generators;
mod placeholders;
pub mod seed_data;
mod validate;

use serde_json::Value;

use mig_assembly::assembler::{AssembledSegment, Assembler};
use mig_assembly::disassembler::Disassembler;
use mig_assembly::renderer::render_edifact;
use mig_assembly::tokenize::{parse_to_segments, split_messages};
use mig_bo4e::engine::MappingEngine;
use mig_types::schema::mig::MigSchema;

pub use enhancer::{build_code_map, enhance_mapped_message, EnhancerConfig};
use placeholders::{placeholder_datetime, placeholder_for_data_element};
pub use validate::validate_fixture;

/// Generate a complete EDIFACT fixture from a PID schema JSON.
///
/// Produces a structurally valid .edi file with:
/// - UNB/UNZ interchange envelope
/// - UNH/UNT message envelope (version from schema)
/// - All root segments (BGM, DTM) and group content
/// - Type-aware placeholder data values
/// - Valid code values (first AHB-filtered code from schema)
pub fn generate_fixture(schema: &Value) -> String {
    let mut segments: Vec<String> = Vec::new();

    let root_segments = schema["root_segments"]
        .as_array()
        .expect("schema must have root_segments array");

    // 1. UNB — synthetic interchange header
    segments
        .push("UNB+UNOC:3+1234567890128:500+9876543210987:500+250401:1200+GENERATED00001".into());

    // 2. UNH — from root_segments, with placeholder message reference
    if let Some(unh) = root_segments.iter().find(|s| s["id"] == "UNH") {
        segments.push(generate_segment_with_placeholders(unh));
    }

    // 3. Non-envelope root segments (BGM, DTM — skip UNH/UNT)
    for seg in root_segments {
        let id = seg["id"].as_str().unwrap_or("");
        if id != "UNH" && id != "UNT" {
            segments.push(generate_segment_with_placeholders(seg));
        }
    }

    // 4. Walk the fields tree depth-first
    if let Some(fields) = schema["fields"].as_object() {
        let mut keys: Vec<&String> = fields.keys().collect();
        keys.sort_by_key(|k| group_sort_key(k));

        for key in keys {
            emit_group(&fields[key], &mut segments);
        }
    }

    // 5. UNT — segment count includes UNH through UNT inclusive
    let count = segments.len(); // currently includes UNB + UNH + everything, subtract UNB and add UNT
    let message_seg_count = count - 1 + 1; // -1 for UNB, +1 for UNT itself
    segments.push(format!("UNT+{message_seg_count}+GENERATED00001"));

    // 6. UNZ
    segments.push("UNZ+1+GENERATED00001".into());

    // Join with segment terminator
    segments
        .iter()
        .map(|s| format!("{s}'"))
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}

/// Generate an enhanced EDIFACT fixture with realistic data values.
///
/// Pipeline:
/// 1. Generate basic EDIFACT from schema (placeholder values)
/// 2. Tokenize and split into message chunks
/// 3. Assemble using PID-filtered MIG
/// 4. Forward map to BO4E (MappedMessage)
/// 5. Enhance with realistic values (names, IDs, dates, addresses)
/// 6. Reverse map back to AssembledTree
/// 7. Disassemble and render to EDIFACT string
/// 8. Wrap with UNB/UNZ interchange envelope
pub fn generate_enhanced_fixture(
    schema: &Value,
    filtered_mig: &MigSchema,
    msg_engine: &MappingEngine,
    tx_engine: &MappingEngine,
    seed: u64,
    variant: usize,
) -> Result<String, crate::error::GeneratorError> {
    // Step 1: Generate basic fixture with placeholder values
    let edi = generate_fixture(schema);

    // Step 2: Tokenize
    let segments = parse_to_segments(edi.as_bytes()).map_err(|e| {
        crate::error::GeneratorError::Validation {
            message: format!("tokenization of generated fixture failed: {e}"),
        }
    })?;

    // Step 3: Split into message chunks
    let chunks =
        split_messages(segments).map_err(|e| crate::error::GeneratorError::Validation {
            message: format!("message splitting failed: {e}"),
        })?;

    if chunks.messages.is_empty() {
        return Err(crate::error::GeneratorError::Validation {
            message: "no UNH/UNT message pairs found in generated fixture".to_string(),
        });
    }

    let msg_chunk = &chunks.messages[0];

    // Step 4: Build message segments (UNH + body + UNT)
    let mut msg_segs = vec![msg_chunk.unh.clone()];
    msg_segs.extend(msg_chunk.body.iter().cloned());
    msg_segs.push(msg_chunk.unt.clone());

    // Step 5: Assemble with PID-filtered MIG
    let assembler = Assembler::new(filtered_mig);
    let original_tree = assembler.assemble_generic(&msg_segs)?;

    // Step 6: Forward map to MappedMessage
    let mut mapped =
        MappingEngine::map_interchange(msg_engine, tx_engine, &original_tree, "SG4", true);

    // Step 7: Enhance with realistic values
    // Collect all definitions from both engines for code map building
    let mut all_defs: Vec<_> = msg_engine.definitions().to_vec();
    all_defs.extend(tx_engine.definitions().iter().cloned());
    let code_map = build_code_map(schema, &all_defs);
    let config = EnhancerConfig::new(seed, variant);
    enhance_mapped_message(&mut mapped, &Some(code_map), &config);

    // Step 8: Reverse map back to AssembledTree
    let mut reverse_tree =
        MappingEngine::map_interchange_reverse(msg_engine, tx_engine, &mapped, "SG4");

    // Add UNH envelope (mapping engine handles content only)
    let unh_assembled = owned_to_assembled(&msg_chunk.unh);
    reverse_tree.segments.insert(0, unh_assembled);
    reverse_tree.post_group_start += 1;

    // Add UNT if the original tree had it
    let original_has_unt = original_tree.segments.last().map(|s| s.tag.as_str()) == Some("UNT");
    if original_has_unt {
        let unt_assembled = owned_to_assembled(&msg_chunk.unt);
        reverse_tree.segments.push(unt_assembled);
    }

    // Step 9: Disassemble and render to EDIFACT
    let disassembler = Disassembler::new(filtered_mig);
    let delimiters = edifact_types::EdifactDelimiters::default();

    let dis_segments = disassembler.disassemble(&reverse_tree);
    let message_edi = render_edifact(&dis_segments, &delimiters);

    // Step 10: Wrap with UNB/UNZ envelope from the original split result
    let unb_seg = chunks.envelope.iter().find(|s| s.id == "UNB");
    let envelope_edi = if let Some(unb) = unb_seg {
        // Reconstruct UNB from the parsed segment data
        let unb_rendered = render_owned_segment(unb, &delimiters);
        let unz_rendered = if let Some(unz) = &chunks.unz {
            render_owned_segment(unz, &delimiters)
        } else {
            // Fallback UNZ
            format!("UNZ+1+ENHANCED{seed:05}'")
        };
        format!("{unb_rendered}{message_edi}{unz_rendered}")
    } else {
        // No UNB found — return message content only
        message_edi
    };

    Ok(envelope_edi)
}

/// Convert an OwnedSegment to an AssembledSegment.
fn owned_to_assembled(seg: &mig_assembly::tokenize::OwnedSegment) -> AssembledSegment {
    AssembledSegment {
        tag: seg.id.clone(),
        elements: seg
            .elements
            .iter()
            .map(|el| el.iter().map(|c| c.to_string()).collect())
            .collect(),
    }
}

/// Render an OwnedSegment to an EDIFACT string with segment terminator.
fn render_owned_segment(
    seg: &mig_assembly::tokenize::OwnedSegment,
    delimiters: &edifact_types::EdifactDelimiters,
) -> String {
    use mig_assembly::disassembler::DisassembledSegment;

    let dis_seg = DisassembledSegment {
        tag: seg.id.clone(),
        elements: seg
            .elements
            .iter()
            .map(|el| el.iter().map(|c| c.to_string()).collect())
            .collect(),
    };
    render_edifact(&[dis_seg], delimiters)
}

/// Recursively emit segments for a group and its children.
fn emit_group(group: &Value, segments: &mut Vec<String>) {
    // Emit this group's own segments
    if let Some(segs) = group["segments"].as_array() {
        for seg in segs {
            segments.push(generate_segment_with_placeholders(seg));
        }
    }

    // Recurse into children, sorted by source_group number then key name
    if let Some(children) = group["children"].as_object() {
        let mut keys: Vec<&String> = children.keys().collect();
        keys.sort_by_key(|k| group_sort_key(k));

        for key in keys {
            emit_group(&children[key], segments);
        }
    }
}

/// Generate an EDIFACT segment string with type-aware placeholder data values.
///
/// Extends the skeleton approach: code elements get the first valid code,
/// data elements get placeholders based on their EDIFACT data element ID.
fn generate_segment_with_placeholders(segment_schema: &Value) -> String {
    let tag = segment_schema["id"].as_str().unwrap_or("???");

    let elements = segment_schema["elements"]
        .as_array()
        .cloned()
        .unwrap_or_default();

    if elements.is_empty() {
        return tag.to_string();
    }

    let max_index = elements
        .iter()
        .filter_map(|el| el["index"].as_u64())
        .max()
        .unwrap_or(0) as usize;

    let mut element_values: Vec<String> = vec![String::new(); max_index + 1];

    // Track DTM format code for date placeholder generation
    let mut dtm_format_code: Option<String> = None;

    for el in &elements {
        let Some(idx) = el["index"].as_u64() else {
            continue;
        };
        let idx = idx as usize;

        let el_type = el["type"].as_str().unwrap_or("data");
        let el_id = el["id"].as_str().unwrap_or("");
        let components = el["components"].as_array();

        if let Some(components) = components {
            if !components.is_empty() {
                let comp_str = build_composite_with_placeholders(components, tag);
                element_values[idx] = comp_str;

                // Extract DTM format code from C507 composite (sub_index 2 = format qualifier)
                if tag == "DTM" {
                    for comp in components {
                        if comp["id"].as_str() == Some("2379") {
                            if let Some(code) = first_code_value(comp) {
                                dtm_format_code = Some(code);
                            }
                        }
                    }
                }
                continue;
            }
        }

        // Simple element
        if el_type == "code" {
            if let Some(code) = first_code_value(el) {
                element_values[idx] = code;
            }
        } else {
            element_values[idx] = placeholder_for_data_element(el_id).to_string();
        }
    }

    // Post-process DTM: fix date value in C507 composite based on format code
    if tag == "DTM" {
        if let Some(ref fmt) = dtm_format_code {
            // The C507 composite is typically at index 0, with date at sub_index 0
            // and format code at sub_index 2. Replace the date placeholder.
            if let Some(c507) = element_values.first_mut() {
                let parts: Vec<&str> = c507.split(':').collect();
                if parts.len() >= 3 {
                    let date_val = escape_edifact(placeholder_datetime(Some(fmt)));
                    *c507 = format!("{}:{}:{}", parts[0], date_val, parts[2]);
                }
            }
        }
    }

    // Trim trailing empty elements
    while element_values.last().is_some_and(|v| v.is_empty()) {
        element_values.pop();
    }

    if element_values.is_empty() {
        tag.to_string()
    } else {
        format!("{}+{}", tag, element_values.join("+"))
    }
}

/// Build a composite element string with type-aware placeholders.
fn build_composite_with_placeholders(components: &[Value], _parent_tag: &str) -> String {
    let max_sub = components
        .iter()
        .filter_map(|c| c["sub_index"].as_u64())
        .max()
        .unwrap_or(0) as usize;

    let mut comp_values: Vec<String> = vec![String::new(); max_sub + 1];

    for comp in components {
        let Some(si) = comp["sub_index"].as_u64() else {
            continue;
        };
        let si = si as usize;
        let comp_type = comp["type"].as_str().unwrap_or("data");
        let comp_id = comp["id"].as_str().unwrap_or("");

        if comp_type == "code" {
            if let Some(code) = first_code_value(comp) {
                comp_values[si] = code;
            }
        } else {
            comp_values[si] = placeholder_for_data_element(comp_id).to_string();
        }
    }

    // Trim trailing empty components
    while comp_values.last().is_some_and(|v| v.is_empty()) {
        comp_values.pop();
    }

    comp_values.join(":")
}

/// Extract the first code value from a code element/component.
fn first_code_value(element: &Value) -> Option<String> {
    element["codes"]
        .as_array()?
        .first()?
        .get("value")?
        .as_str()
        .map(String::from)
}

/// Sort key for group names: (source_group_number, qualifier_suffix).
///
/// Examples: "sg5_z16" → (5, "z16"), "sg12_z04" → (12, "z04"), "sg10" → (10, "")
fn group_sort_key(key: &str) -> (u32, String) {
    // Strip "sg" prefix
    let rest = key.strip_prefix("sg").unwrap_or(key);

    // Split on first underscore: "5_z16" → ("5", "z16"), "12_z04" → ("12", "z04")
    if let Some((num_str, suffix)) = rest.split_once('_') {
        let num = num_str.parse::<u32>().unwrap_or(999);
        (num, suffix.to_string())
    } else {
        let num = rest.parse::<u32>().unwrap_or(999);
        (num, String::new())
    }
}

/// Escape special characters in EDIFACT data values.
fn escape_edifact(value: &str) -> String {
    value
        .replace('?', "??")
        .replace('+', "?+")
        .replace(':', "?:")
        .replace('\'', "?'")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_group_sort_key() {
        assert_eq!(group_sort_key("sg2"), (2, String::new()));
        assert_eq!(group_sort_key("sg5_z16"), (5, "z16".to_string()));
        assert_eq!(group_sort_key("sg12_z04"), (12, "z04".to_string()));
        assert_eq!(group_sort_key("sg8_z01"), (8, "z01".to_string()));

        // Verify ordering: sg5 < sg6 < sg8 < sg10 < sg12
        let mut keys = vec!["sg12_z04", "sg5_z16", "sg8_z01", "sg6", "sg10"];
        keys.sort_by_key(|k| group_sort_key(k));
        assert_eq!(keys, vec!["sg5_z16", "sg6", "sg8_z01", "sg10", "sg12_z04"]);
    }

    #[test]
    fn test_escape_edifact() {
        assert_eq!(escape_edifact("hello+world"), "hello?+world");
        assert_eq!(escape_edifact("a:b"), "a?:b");
        assert_eq!(escape_edifact("20250401120000+00"), "20250401120000?+00");
    }
}
