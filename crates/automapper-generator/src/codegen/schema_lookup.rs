//! Schema lookup: structured output of PID schema for TOML mapping authoring.
//!
//! Reads a PID schema JSON and produces:
//! - A group listing (all groups with entity hints, data quality, discriminators)
//! - Detail for one group (all segments with element paths)
//! - A pre-filled TOML template

use serde_json::Value;

/// Summary of a group in the schema.
struct GroupSummary {
    path: String,
    source_group: String,
    entity_hint: Option<String>,
    data_quality_hint: Option<String>,
    discriminator: Option<String>,
}

/// Info about a single element within a segment.
struct ElementInfo {
    edifact_path: String,
    element_type: String,
    name: String,
    codes: Vec<(String, String)>,
}

/// Print a list of all groups in the PID schema.
pub fn print_group_list(schema: &Value) -> String {
    let beschreibung = schema
        .get("beschreibung")
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown");

    let mut groups = Vec::new();
    if let Some(fields) = schema.get("fields").and_then(|f| f.as_object()) {
        for (key, val) in fields {
            collect_groups(val, key, &mut groups);
        }
    }

    // Sort by path for deterministic output
    groups.sort_by(|a, b| a.path.cmp(&b.path));

    let mut out = String::new();
    out.push_str(&format!("PID — {}\n\n", beschreibung));
    out.push_str("Groups:\n");

    for g in &groups {
        let mut line = format!("  {:<30} {:<14}", g.path, g.source_group);
        if let Some(ref e) = g.entity_hint {
            line.push_str(&format!("  entity={:<24}", e));
        }
        if let Some(ref q) = g.data_quality_hint {
            line.push_str(&format!("  quality={:<12}", q));
        }
        if let Some(ref d) = g.discriminator {
            line.push_str(&format!("  disc={}", d));
        }
        out.push_str(line.trim_end());
        out.push('\n');
    }

    out
}

/// Print detail for one group: all segments with element paths.
pub fn print_group_detail(schema: &Value, group_path: &str) -> Option<String> {
    let group = resolve_group(schema, group_path)?;
    let mut out = String::new();

    let entity_hint = group
        .get("entity_hint")
        .and_then(|v| v.as_str())
        .unwrap_or("?");
    let data_quality = group
        .get("data_quality_hint")
        .and_then(|v| v.as_str())
        .unwrap_or("?");
    let source_group = group
        .get("source_group")
        .and_then(|v| v.as_str())
        .unwrap_or("?");

    out.push_str(&format!("{}\n", group_path));
    out.push_str(&format!("entity_hint: {}\n", entity_hint));
    out.push_str(&format!("data_quality_hint: {}\n", data_quality));
    out.push_str(&format!("source_group: {}\n", source_group));
    out.push_str(&format!("source_path: {}\n", group_path));

    if let Some(disc) = group.get("discriminator") {
        let disc_str = format_discriminator(disc);
        out.push_str(&format!("discriminator: {}\n", disc_str));
    }

    out.push('\n');

    // Segments
    if let Some(segments) = group.get("segments").and_then(|s| s.as_array()) {
        for seg in segments {
            let seg_id = seg.get("id").and_then(|v| v.as_str()).unwrap_or("?");
            let seg_name = seg.get("name").and_then(|v| v.as_str()).unwrap_or("");
            // Truncate long segment descriptions
            let seg_desc = if seg_name.len() > 80 {
                format!("{}...", &seg_name[..77])
            } else {
                seg_name.to_string()
            };
            out.push_str(&format!("Segment: {} — {}\n", seg_id, seg_desc));

            let elements = collect_element_paths(seg_id, seg);
            for elem in &elements {
                let mut line = format!(
                    "  {:<24} {:<6} {}",
                    elem.edifact_path, elem.element_type, elem.name
                );
                if !elem.codes.is_empty() {
                    let codes_str: Vec<String> = elem
                        .codes
                        .iter()
                        .map(|(v, n)| format!("{}={}", v, n))
                        .collect();
                    line.push_str(&format!("    codes: {}", codes_str.join(", ")));
                }
                out.push_str(line.trim_end());
                out.push('\n');
            }
            out.push('\n');
        }
    }

    // Children
    if let Some(children) = group.get("children").and_then(|c| c.as_object()) {
        let mut child_keys: Vec<&String> = children.keys().collect();
        child_keys.sort();
        if !child_keys.is_empty() {
            out.push_str(&format!(
                "Children: {}\n",
                child_keys
                    .iter()
                    .map(|k| k.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
    }

    Some(out)
}

/// Print a pre-filled TOML template for a group.
pub fn print_toml_template(schema: &Value, group_path: &str) -> Option<String> {
    let group = resolve_group(schema, group_path)?;
    let mut out = String::new();

    let entity_hint = group
        .get("entity_hint")
        .and_then(|v| v.as_str())
        .unwrap_or("TODO");
    let source_group = group
        .get("source_group")
        .and_then(|v| v.as_str())
        .unwrap_or("TODO");

    out.push_str(&format!(
        "# Template for {} ({}, {})\n",
        group_path,
        entity_hint,
        group
            .get("data_quality_hint")
            .and_then(|v| v.as_str())
            .unwrap_or("?"),
    ));

    // [meta]
    out.push_str("[meta]\n");
    out.push_str(&format!("entity = \"{}\"\n", entity_hint));
    out.push_str(&format!("bo4e_type = \"{}\"\n", entity_hint));
    out.push_str(&format!("source_group = \"{}\"\n", source_group));
    out.push_str(&format!("source_path = \"{}\"\n", group_path));

    if let Some(disc) = group.get("discriminator") {
        let disc_str = format_discriminator(disc);
        out.push_str(&format!("discriminator = \"{}\"\n", disc_str));
    }

    out.push('\n');

    // [fields]
    out.push_str("[fields]\n");

    if let Some(segments) = group.get("segments").and_then(|s| s.as_array()) {
        for seg in segments {
            let seg_id = seg.get("id").and_then(|v| v.as_str()).unwrap_or("?");
            let seg_name = seg.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let seg_desc = if seg_name.len() > 60 {
                format!("{}...", &seg_name[..57])
            } else {
                seg_name.to_string()
            };
            out.push_str(&format!("# {} — {}\n", seg_id, seg_desc));

            let elements = collect_element_paths(seg_id, seg);
            for elem in &elements {
                let codes_comment = if !elem.codes.is_empty() {
                    let codes_str: Vec<String> = elem
                        .codes
                        .iter()
                        .map(|(v, n)| format!("{}={}", v, n))
                        .collect();
                    format!("  [{}]", codes_str.join(", "))
                } else {
                    String::new()
                };

                out.push_str(&format!(
                    "# {}  {}  {}{}\n",
                    elem.edifact_path, elem.element_type, elem.name, codes_comment
                ));

                // Generate the mapping line
                if elem.element_type == "code" && elem.codes.len() == 1 {
                    // Single code → use default
                    out.push_str(&format!(
                        "\"{}\" = {{ target = \"\", default = \"{}\" }}\n",
                        elem.edifact_path, elem.codes[0].0
                    ));
                } else {
                    out.push_str(&format!(
                        "\"{}\" = {{ target = \"\" }}\n",
                        elem.edifact_path
                    ));
                }
            }
        }
    }

    Some(out)
}

/// Resolve a group by dotted path (e.g., "sg4.sg8_zf0").
fn resolve_group<'a>(schema: &'a Value, path: &str) -> Option<&'a Value> {
    let parts: Vec<&str> = path.split('.').collect();
    let fields = schema.get("fields")?.as_object()?;

    // First part is a top-level group key
    let mut current = fields.get(parts[0])?;

    // Remaining parts navigate into children
    for &part in &parts[1..] {
        current = current.get("children")?.as_object()?.get(part)?;
    }

    Some(current)
}

/// Recursively collect group summaries.
fn collect_groups(group: &Value, prefix: &str, out: &mut Vec<GroupSummary>) {
    let source_group = group
        .get("source_group")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let entity_hint = group
        .get("entity_hint")
        .and_then(|v| v.as_str())
        .map(String::from);
    let data_quality_hint = group
        .get("data_quality_hint")
        .and_then(|v| v.as_str())
        .map(String::from);
    let discriminator = group.get("discriminator").map(format_discriminator);

    out.push(GroupSummary {
        path: prefix.to_string(),
        source_group,
        entity_hint,
        data_quality_hint,
        discriminator,
    });

    // Recurse into children
    if let Some(children) = group.get("children").and_then(|c| c.as_object()) {
        for (key, val) in children {
            let child_path = format!("{}.{}", prefix, key);
            collect_groups(val, &child_path, out);
        }
    }
}

/// Format a discriminator object from the schema to an EDIFACT ID string.
///
/// Input: `{"segment": "SEQ", "element": "1229", "values": ["ZF0"]}`
/// Output: `"SEQ.d1229=ZF0"`
fn format_discriminator(disc: &Value) -> String {
    let segment = disc.get("segment").and_then(|v| v.as_str()).unwrap_or("?");
    let element = disc.get("element").and_then(|v| v.as_str()).unwrap_or("?");

    let values = disc
        .get("values")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .collect::<Vec<_>>()
                .join(",")
        })
        .unwrap_or_default();

    if values.is_empty() {
        format!("{}.d{}", segment, element)
    } else {
        format!("{}.d{}={}", segment, element, values)
    }
}

/// Collect all element paths from a segment definition.
fn collect_element_paths(seg_tag: &str, segment: &Value) -> Vec<ElementInfo> {
    let mut elements = Vec::new();
    let seg_lower = seg_tag.to_ascii_lowercase();

    if let Some(elems) = segment.get("elements").and_then(|e| e.as_array()) {
        for elem in elems {
            if let Some(composite_id) = elem.get("composite").and_then(|v| v.as_str()) {
                // Composite element: emit one entry per component
                let composite_lower = format!("c{}", &composite_id[1..]).to_ascii_lowercase();

                if let Some(components) = elem.get("components").and_then(|c| c.as_array()) {
                    for comp in components {
                        let comp_id = comp.get("id").and_then(|v| v.as_str()).unwrap_or("?");
                        let comp_name = comp.get("name").and_then(|v| v.as_str()).unwrap_or("");
                        let comp_type = comp.get("type").and_then(|v| v.as_str()).unwrap_or("data");
                        let codes = extract_codes(comp);

                        elements.push(ElementInfo {
                            edifact_path: format!("{}.{}.d{}", seg_lower, composite_lower, comp_id),
                            element_type: comp_type.to_string(),
                            name: comp_name.to_string(),
                            codes,
                        });
                    }
                }
            } else {
                // Simple data element
                let elem_id = elem.get("id").and_then(|v| v.as_str()).unwrap_or("?");
                let elem_name = elem.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let elem_type = elem.get("type").and_then(|v| v.as_str()).unwrap_or("data");
                let codes = extract_codes(elem);

                elements.push(ElementInfo {
                    edifact_path: format!("{}.d{}", seg_lower, elem_id),
                    element_type: elem_type.to_string(),
                    name: elem_name.to_string(),
                    codes,
                });
            }
        }
    }

    elements
}

/// Extract codes from an element or component value.
fn extract_codes(value: &Value) -> Vec<(String, String)> {
    value
        .get("codes")
        .and_then(|c| c.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|c| {
                    let v = c.get("value").and_then(|v| v.as_str())?;
                    let n = c.get("name").and_then(|v| v.as_str()).unwrap_or("");
                    // Truncate long names
                    let n_short = if n.len() > 40 {
                        format!("{}...", &n[..37])
                    } else {
                        n.to_string()
                    };
                    Some((v.to_string(), n_short))
                })
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_schema() -> Value {
        serde_json::json!({
            "beschreibung": "Test PID",
            "fields": {
                "sg2": {
                    "source_group": "SG2",
                    "segments": [
                        {
                            "id": "NAD",
                            "name": "Marktteilnehmer",
                            "elements": [
                                {
                                    "id": "3035",
                                    "index": 0,
                                    "name": "Beteiligter, Qualifier",
                                    "type": "code",
                                    "codes": [{"value": "MR", "name": "Nachrichtenempfänger"}]
                                },
                                {
                                    "composite": "C082",
                                    "index": 1,
                                    "name": "Identifikation des Beteiligten",
                                    "components": [
                                        {
                                            "id": "3039",
                                            "sub_index": 0,
                                            "name": "MP-ID",
                                            "type": "data"
                                        }
                                    ]
                                }
                            ]
                        }
                    ],
                    "discriminator": {
                        "segment": "NAD",
                        "element": "3035"
                    }
                },
                "sg4": {
                    "source_group": "SG4",
                    "entity_hint": "Prozessdaten",
                    "data_quality_hint": "base",
                    "segments": [],
                    "children": {
                        "sg5_z16": {
                            "source_group": "SG5",
                            "entity_hint": "Marktlokation",
                            "data_quality_hint": "base",
                            "discriminator": {
                                "segment": "LOC",
                                "element": "3227",
                                "values": ["Z16"]
                            },
                            "segments": [
                                {
                                    "id": "LOC",
                                    "name": "Marktlokation",
                                    "elements": [
                                        {
                                            "id": "3227",
                                            "index": 0,
                                            "name": "Lokation, Qualifier",
                                            "type": "code",
                                            "codes": [{"value": "Z16", "name": "Marktlokation"}]
                                        },
                                        {
                                            "composite": "C517",
                                            "index": 1,
                                            "name": "Lokationsidentifikation",
                                            "components": [
                                                {
                                                    "id": "3225",
                                                    "sub_index": 0,
                                                    "name": "MaLo-ID",
                                                    "type": "data"
                                                }
                                            ]
                                        }
                                    ]
                                }
                            ]
                        },
                        "sg8_zf0": {
                            "source_group": "SG8",
                            "entity_hint": "TechnischeRessource",
                            "data_quality_hint": "informativ",
                            "discriminator": {
                                "segment": "SEQ",
                                "element": "1229",
                                "values": ["ZF0"]
                            },
                            "segments": [
                                {
                                    "id": "SEQ",
                                    "name": "Daten der Technischen Ressource",
                                    "elements": [
                                        {
                                            "id": "1229",
                                            "index": 0,
                                            "name": "Handlung, Code",
                                            "type": "code",
                                            "codes": [{"value": "ZF0", "name": "Informative Daten der Technischen Ressource"}]
                                        },
                                        {
                                            "composite": "C286",
                                            "index": 1,
                                            "name": "Information über eine Folge",
                                            "components": [
                                                {
                                                    "id": "1050",
                                                    "sub_index": 0,
                                                    "name": "Referenz auf Zeitraum-ID",
                                                    "type": "data"
                                                }
                                            ]
                                        }
                                    ]
                                }
                            ],
                            "children": {
                                "sg10": {
                                    "source_group": "SG10",
                                    "entity_hint": "TechnischeRessource",
                                    "data_quality_hint": "informativ",
                                    "segments": [
                                        {
                                            "id": "CCI",
                                            "name": "Info",
                                            "elements": [
                                                {
                                                    "id": "7059",
                                                    "index": 0,
                                                    "name": "Klassentyp, Code",
                                                    "type": "code",
                                                    "codes": [{"value": "Z63", "name": "Info"}]
                                                }
                                            ]
                                        }
                                    ]
                                }
                            }
                        }
                    }
                }
            }
        })
    }

    #[test]
    fn group_list_output() {
        let schema = test_schema();
        let output = print_group_list(&schema);
        assert!(output.contains("Test PID"));
        assert!(output.contains("sg2"));
        assert!(output.contains("sg4"));
        assert!(output.contains("sg4.sg5_z16"));
        assert!(output.contains("sg4.sg8_zf0"));
        assert!(output.contains("sg4.sg8_zf0.sg10"));
        assert!(output.contains("entity=Marktlokation"));
        assert!(output.contains("entity=TechnischeRessource"));
        assert!(output.contains("disc=SEQ.d1229=ZF0"));
    }

    #[test]
    fn group_detail_output() {
        let schema = test_schema();
        let output = print_group_detail(&schema, "sg4.sg8_zf0").unwrap();
        assert!(output.contains("entity_hint: TechnischeRessource"));
        assert!(output.contains("source_group: SG8"));
        assert!(output.contains("discriminator: SEQ.d1229=ZF0"));
        assert!(output.contains("Segment: SEQ"));
        assert!(output.contains("seq.d1229"));
        assert!(output.contains("seq.c286.d1050"));
        assert!(output.contains("Children: sg10"));
    }

    #[test]
    fn group_detail_unknown_group() {
        let schema = test_schema();
        assert!(print_group_detail(&schema, "sg99").is_none());
    }

    #[test]
    fn toml_template_output() {
        let schema = test_schema();
        let output = print_toml_template(&schema, "sg4.sg5_z16").unwrap();
        assert!(output.contains("[meta]"));
        assert!(output.contains("entity = \"Marktlokation\""));
        assert!(output.contains("source_group = \"SG5\""));
        assert!(output.contains("source_path = \"sg4.sg5_z16\""));
        assert!(output.contains("discriminator = \"LOC.d3227=Z16\""));
        assert!(output.contains("[fields]"));
        assert!(output.contains("\"loc.d3227\""));
        assert!(output.contains("default = \"Z16\""));
        assert!(output.contains("\"loc.c517.d3225\""));
    }

    #[test]
    fn format_discriminator_with_values() {
        let disc = serde_json::json!({
            "segment": "SEQ",
            "element": "1229",
            "values": ["ZF0"]
        });
        assert_eq!(format_discriminator(&disc), "SEQ.d1229=ZF0");
    }

    #[test]
    fn format_discriminator_no_values() {
        let disc = serde_json::json!({
            "segment": "NAD",
            "element": "3035"
        });
        assert_eq!(format_discriminator(&disc), "NAD.d3035");
    }

    #[test]
    fn resolve_nested_group() {
        let schema = test_schema();
        let group = resolve_group(&schema, "sg4.sg8_zf0.sg10");
        assert!(group.is_some());
        let g = group.unwrap();
        assert_eq!(g.get("source_group").and_then(|v| v.as_str()), Some("SG10"));
    }
}
