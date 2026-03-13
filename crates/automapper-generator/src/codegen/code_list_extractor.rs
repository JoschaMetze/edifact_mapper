//! Extracts all code lists from a MIG XML schema into a flat JSON file.
//!
//! Output format: `{ "<data_element_id>": { "name": "...", "codes": [{"value": "...", "name": "..."}] } }`
//!
//! Run once per format version. Output is committed as generated artifact.

use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::schema::mig::{MigComposite, MigDataElement, MigSchema, MigSegment, MigSegmentGroup};

/// A single code list entry for a data element.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeListEntry {
    /// Human-readable name of the data element.
    pub name: String,
    /// All valid code values for this data element.
    pub codes: Vec<CodeValueEntry>,
}

/// A single code value with its description.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeValueEntry {
    /// The code value (e.g., "UTILMD", "Z16").
    pub value: String,
    /// Human-readable name/description.
    pub name: String,
}

/// Extract all code lists from a parsed MIG schema.
pub fn extract_code_lists(mig: &MigSchema) -> BTreeMap<String, CodeListEntry> {
    let mut result: BTreeMap<String, CodeListEntry> = BTreeMap::new();

    fn visit_de(de: &MigDataElement, result: &mut BTreeMap<String, CodeListEntry>) {
        if de.codes.is_empty() {
            return;
        }
        let entry = result
            .entry(de.id.clone())
            .or_insert_with(|| CodeListEntry {
                name: de.name.clone(),
                codes: Vec::new(),
            });
        for code in &de.codes {
            if !entry.codes.iter().any(|c| c.value == code.value) {
                entry.codes.push(CodeValueEntry {
                    value: code.value.clone(),
                    name: code.name.clone(),
                });
            }
        }
    }

    fn visit_composite(comp: &MigComposite, result: &mut BTreeMap<String, CodeListEntry>) {
        for de in &comp.data_elements {
            visit_de(de, result);
        }
    }

    fn visit_segment(seg: &MigSegment, result: &mut BTreeMap<String, CodeListEntry>) {
        for de in &seg.data_elements {
            visit_de(de, result);
        }
        for comp in &seg.composites {
            visit_composite(comp, result);
        }
    }

    fn visit_group(group: &MigSegmentGroup, result: &mut BTreeMap<String, CodeListEntry>) {
        for seg in &group.segments {
            visit_segment(seg, result);
        }
        for child in &group.nested_groups {
            visit_group(child, result);
        }
    }

    for seg in &mig.segments {
        visit_segment(seg, &mut result);
    }
    for group in &mig.segment_groups {
        visit_group(group, &mut result);
    }

    result
}

/// Write extracted code lists to a JSON file.
pub fn write_code_lists(
    code_lists: &BTreeMap<String, CodeListEntry>,
    output_path: &Path,
) -> Result<(), std::io::Error> {
    let mut json = serde_json::to_string_pretty(code_lists).map_err(std::io::Error::other)?;
    json.push('\n');
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(output_path, json)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::common::CodeDefinition;
    use crate::schema::mig::*;

    fn make_test_schema(segments: Vec<MigSegment>, groups: Vec<MigSegmentGroup>) -> MigSchema {
        MigSchema {
            message_type: "TEST".to_string(),
            variant: None,
            version: "1.0".to_string(),
            publication_date: "2025-01-01".to_string(),
            author: "Test".to_string(),
            format_version: "FV2504".to_string(),
            source_file: "test.xml".to_string(),
            segments,
            segment_groups: groups,
        }
    }

    fn make_data_element(id: &str, name: &str, codes: Vec<CodeDefinition>) -> MigDataElement {
        MigDataElement {
            id: id.to_string(),
            name: name.to_string(),
            description: None,
            status_std: None,
            status_spec: None,
            format_std: None,
            format_spec: None,
            codes,
            position: 0,
        }
    }

    fn make_segment(id: &str, data_elements: Vec<MigDataElement>) -> MigSegment {
        MigSegment {
            id: id.to_string(),
            name: id.to_string(),
            description: None,
            counter: None,
            level: 0,
            number: None,
            max_rep_std: 1,
            max_rep_spec: 1,
            status_std: None,
            status_spec: None,
            example: None,
            data_elements,
            composites: vec![],
        }
    }

    #[test]
    fn test_extract_code_lists_from_simple_schema() {
        let mig = make_test_schema(
            vec![make_segment(
                "BGM",
                vec![make_data_element(
                    "1001",
                    "Document name code",
                    vec![
                        CodeDefinition {
                            value: "E01".to_string(),
                            name: "Anmeldung".to_string(),
                            description: None,
                        },
                        CodeDefinition {
                            value: "E02".to_string(),
                            name: "Abmeldung".to_string(),
                            description: None,
                        },
                    ],
                )],
            )],
            vec![],
        );

        let result = extract_code_lists(&mig);
        assert_eq!(result.len(), 1);
        assert!(result.contains_key("1001"));
        let entry = &result["1001"];
        assert_eq!(entry.name, "Document name code");
        assert_eq!(entry.codes.len(), 2);
        assert_eq!(entry.codes[0].value, "E01");
        assert_eq!(entry.codes[1].value, "E02");
    }

    #[test]
    fn test_extract_deduplicates_codes() {
        let mig = make_test_schema(
            vec![
                make_segment(
                    "BGM",
                    vec![make_data_element(
                        "1001",
                        "Code",
                        vec![CodeDefinition {
                            value: "E01".to_string(),
                            name: "Anmeldung".to_string(),
                            description: None,
                        }],
                    )],
                ),
                make_segment(
                    "DTM",
                    vec![make_data_element(
                        "1001",
                        "Code",
                        vec![
                            CodeDefinition {
                                value: "E01".to_string(),
                                name: "Anmeldung".to_string(),
                                description: None,
                            },
                            CodeDefinition {
                                value: "E03".to_string(),
                                name: "Kündigung".to_string(),
                                description: None,
                            },
                        ],
                    )],
                ),
            ],
            vec![],
        );

        let result = extract_code_lists(&mig);
        assert_eq!(result.len(), 1);
        let entry = &result["1001"];
        // E01 should appear only once despite being in both segments
        assert_eq!(entry.codes.len(), 2);
        assert_eq!(entry.codes[0].value, "E01");
        assert_eq!(entry.codes[1].value, "E03");
    }

    #[test]
    fn test_extract_from_composites() {
        let mig = make_test_schema(
            vec![MigSegment {
                id: "NAD".to_string(),
                name: "Name and address".to_string(),
                description: None,
                counter: None,
                level: 0,
                number: None,
                max_rep_std: 1,
                max_rep_spec: 1,
                status_std: None,
                status_spec: None,
                example: None,
                data_elements: vec![],
                composites: vec![MigComposite {
                    id: "C082".to_string(),
                    name: "Party identification".to_string(),
                    description: None,
                    status_std: None,
                    status_spec: None,
                    position: 0,
                    data_elements: vec![make_data_element(
                        "3055",
                        "Code list responsible agency code",
                        vec![CodeDefinition {
                            value: "9".to_string(),
                            name: "GS1".to_string(),
                            description: None,
                        }],
                    )],
                }],
            }],
            vec![],
        );

        let result = extract_code_lists(&mig);
        assert_eq!(result.len(), 1);
        assert!(result.contains_key("3055"));
        assert_eq!(result["3055"].codes[0].value, "9");
    }

    #[test]
    fn test_extract_from_nested_groups() {
        let mig = make_test_schema(
            vec![],
            vec![MigSegmentGroup {
                id: "SG1".to_string(),
                name: "Group 1".to_string(),
                description: None,
                counter: None,
                level: 1,
                max_rep_std: 1,
                max_rep_spec: 1,
                status_std: None,
                status_spec: None,
                segments: vec![make_segment(
                    "RFF",
                    vec![make_data_element(
                        "1153",
                        "Reference code qualifier",
                        vec![CodeDefinition {
                            value: "TN".to_string(),
                            name: "Transaction number".to_string(),
                            description: None,
                        }],
                    )],
                )],
                nested_groups: vec![MigSegmentGroup {
                    id: "SG2".to_string(),
                    name: "Group 2".to_string(),
                    description: None,
                    counter: None,
                    level: 2,
                    max_rep_std: 1,
                    max_rep_spec: 1,
                    status_std: None,
                    status_spec: None,
                    segments: vec![make_segment(
                        "DTM",
                        vec![make_data_element(
                            "2005",
                            "Date or time or period function code qualifier",
                            vec![CodeDefinition {
                                value: "137".to_string(),
                                name: "Document date".to_string(),
                                description: None,
                            }],
                        )],
                    )],
                    nested_groups: vec![],
                }],
            }],
        );

        let result = extract_code_lists(&mig);
        assert_eq!(result.len(), 2);
        assert!(result.contains_key("1153"));
        assert!(result.contains_key("2005"));
        assert_eq!(result["1153"].codes[0].value, "TN");
        assert_eq!(result["2005"].codes[0].value, "137");
    }

    #[test]
    fn test_empty_schema_returns_empty() {
        let mig = make_test_schema(vec![], vec![]);
        let result = extract_code_lists(&mig);
        assert!(result.is_empty());
    }
}
