use std::path::Path;

use quick_xml::events::Event;
use quick_xml::Reader;

use crate::error::GeneratorError;
use crate::schema::common::CodeDefinition;
use crate::schema::mig::*;

/// Parses a MIG XML file into a `MigSchema`.
///
/// The MIG XML uses element-name prefixes to distinguish types:
/// - `S_*` — segments (e.g., `S_UNH`, `S_BGM`)
/// - `G_*` — segment groups (e.g., `G_SG2`)
/// - `C_*` — composites (e.g., `C_S009`, `C_C002`)
/// - `D_*` — data elements (e.g., `D_0062`, `D_3035`)
/// - `M_*` — message containers (e.g., `M_UTILMD`)
/// - `Code` — code values within data elements
pub fn parse_mig(
    path: &Path,
    message_type: &str,
    variant: Option<&str>,
    format_version: &str,
) -> Result<MigSchema, GeneratorError> {
    if !path.exists() {
        return Err(GeneratorError::FileNotFound(path.to_path_buf()));
    }

    let xml_content = std::fs::read_to_string(path)?;
    let mut reader = Reader::from_str(&xml_content);
    reader.config_mut().trim_text(true);

    let mut schema = MigSchema {
        message_type: message_type.to_string(),
        variant: variant.map(|v| v.to_string()),
        version: String::new(),
        publication_date: String::new(),
        author: "BDEW".to_string(),
        format_version: format_version.to_string(),
        source_file: path.to_string_lossy().to_string(),
        segments: Vec::new(),
        segment_groups: Vec::new(),
    };

    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name = elem_name(e);

                if name.starts_with("M_") {
                    for attr in e.attributes().flatten() {
                        let key = std::str::from_utf8(attr.key.as_ref()).unwrap_or("");
                        let val = attr.unescape_value().unwrap_or_default().to_string();
                        match key {
                            "Versionsnummer" => schema.version = val,
                            "Veroeffentlichungsdatum" => schema.publication_date = val,
                            "Author" => schema.author = val,
                            _ => {}
                        }
                    }
                } else if name.starts_with("S_") {
                    let segment = parse_segment_from_xml(&name, e, &mut reader, path)?;
                    schema.segments.push(segment);
                } else if name.starts_with("G_") {
                    let group = parse_group_from_xml(&name, e, &mut reader, path)?;
                    schema.segment_groups.push(group);
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(GeneratorError::XmlParse {
                    path: path.to_path_buf(),
                    message: e.to_string(),
                    source: Some(e),
                })
            }
            _ => {}
        }
        buf.clear();
    }

    if schema.version.is_empty() {
        return Err(GeneratorError::MissingAttribute {
            path: path.to_path_buf(),
            element: format!("M_{}", message_type),
            attribute: "Versionsnummer".to_string(),
        });
    }

    Ok(schema)
}

/// Extract the element name as an owned String from a BytesStart event.
fn elem_name(e: &quick_xml::events::BytesStart) -> String {
    let qname = e.name();
    std::str::from_utf8(qname.as_ref())
        .unwrap_or("")
        .to_string()
}

/// Extract the element name as an owned String from a BytesEnd event.
fn end_name(e: &quick_xml::events::BytesEnd) -> String {
    let qname = e.name();
    std::str::from_utf8(qname.as_ref())
        .unwrap_or("")
        .to_string()
}

fn get_attr(e: &quick_xml::events::BytesStart, key: &str) -> Option<String> {
    e.attributes()
        .flatten()
        .find(|a| a.key.as_ref() == key.as_bytes())
        .and_then(|a| a.unescape_value().ok().map(|v| v.to_string()))
}

fn get_attr_i32(e: &quick_xml::events::BytesStart, key: &str, default: i32) -> i32 {
    get_attr(e, key)
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn parse_segment_from_xml(
    element_name: &str,
    start: &quick_xml::events::BytesStart,
    reader: &mut Reader<&[u8]>,
    path: &Path,
) -> Result<MigSegment, GeneratorError> {
    let id = element_name
        .strip_prefix("S_")
        .unwrap_or(element_name)
        .to_string();

    let mut segment = MigSegment {
        id,
        name: get_attr(start, "Name").unwrap_or_default(),
        description: get_attr(start, "Description"),
        counter: get_attr(start, "Counter"),
        level: get_attr_i32(start, "Level", 0),
        number: get_attr(start, "Number"),
        max_rep_std: get_attr_i32(start, "MaxRep_Std", 1),
        max_rep_spec: get_attr_i32(start, "MaxRep_Specification", 1),
        status_std: get_attr(start, "Status_Std"),
        status_spec: get_attr(start, "Status_Specification"),
        example: get_attr(start, "Example"),
        data_elements: Vec::new(),
        composites: Vec::new(),
    };

    let mut position: usize = 0;
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name = elem_name(e);

                if name.starts_with("D_") {
                    let de = parse_data_element_from_xml(&name, e, reader, path, position)?;
                    segment.data_elements.push(de);
                    position += 1;
                } else if name.starts_with("C_") {
                    let comp = parse_composite_from_xml(&name, e, reader, path, position)?;
                    segment.composites.push(comp);
                    position += 1;
                }
            }
            Ok(Event::Empty(ref e)) => {
                let name = elem_name(e);

                if name.starts_with("D_") {
                    let de = MigDataElement {
                        id: name.strip_prefix("D_").unwrap_or(&name).to_string(),
                        name: get_attr(e, "Name").unwrap_or_default(),
                        description: get_attr(e, "Description"),
                        status_std: get_attr(e, "Status_Std"),
                        status_spec: get_attr(e, "Status_Specification"),
                        format_std: get_attr(e, "Format_Std"),
                        format_spec: get_attr(e, "Format_Specification"),
                        codes: Vec::new(),
                        position,
                    };
                    segment.data_elements.push(de);
                    position += 1;
                }
            }
            Ok(Event::End(ref e)) => {
                let name = end_name(e);
                if name == element_name {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(GeneratorError::XmlParse {
                    path: path.to_path_buf(),
                    message: e.to_string(),
                    source: Some(e),
                })
            }
            _ => {}
        }
        buf.clear();
    }

    Ok(segment)
}

fn parse_group_from_xml(
    element_name: &str,
    start: &quick_xml::events::BytesStart,
    reader: &mut Reader<&[u8]>,
    path: &Path,
) -> Result<MigSegmentGroup, GeneratorError> {
    let id = element_name
        .strip_prefix("G_")
        .unwrap_or(element_name)
        .to_string();

    let mut group = MigSegmentGroup {
        id,
        name: get_attr(start, "Name").unwrap_or_default(),
        description: get_attr(start, "Description"),
        counter: get_attr(start, "Counter"),
        level: get_attr_i32(start, "Level", 0),
        max_rep_std: get_attr_i32(start, "MaxRep_Std", 1),
        max_rep_spec: get_attr_i32(start, "MaxRep_Specification", 1),
        status_std: get_attr(start, "Status_Std"),
        status_spec: get_attr(start, "Status_Specification"),
        segments: Vec::new(),
        nested_groups: Vec::new(),
    };

    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name = elem_name(e);

                if name.starts_with("S_") {
                    let seg = parse_segment_from_xml(&name, e, reader, path)?;
                    group.segments.push(seg);
                } else if name.starts_with("G_") {
                    let nested = parse_group_from_xml(&name, e, reader, path)?;
                    group.nested_groups.push(nested);
                }
            }
            Ok(Event::End(ref e)) => {
                let name = end_name(e);
                if name == element_name {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(GeneratorError::XmlParse {
                    path: path.to_path_buf(),
                    message: e.to_string(),
                    source: Some(e),
                })
            }
            _ => {}
        }
        buf.clear();
    }

    Ok(group)
}

fn parse_composite_from_xml(
    element_name: &str,
    start: &quick_xml::events::BytesStart,
    reader: &mut Reader<&[u8]>,
    path: &Path,
    position: usize,
) -> Result<MigComposite, GeneratorError> {
    let id = element_name
        .strip_prefix("C_")
        .unwrap_or(element_name)
        .to_string();

    let mut composite = MigComposite {
        id,
        name: get_attr(start, "Name").unwrap_or_default(),
        description: get_attr(start, "Description"),
        status_std: get_attr(start, "Status_Std"),
        status_spec: get_attr(start, "Status_Specification"),
        data_elements: Vec::new(),
        position,
    };

    let mut component_position: usize = 0;
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name = elem_name(e);

                if name.starts_with("D_") {
                    let de =
                        parse_data_element_from_xml(&name, e, reader, path, component_position)?;
                    composite.data_elements.push(de);
                    component_position += 1;
                }
            }
            Ok(Event::Empty(ref e)) => {
                let name = elem_name(e);

                if name.starts_with("D_") {
                    let de = MigDataElement {
                        id: name.strip_prefix("D_").unwrap_or(&name).to_string(),
                        name: get_attr(e, "Name").unwrap_or_default(),
                        description: get_attr(e, "Description"),
                        status_std: get_attr(e, "Status_Std"),
                        status_spec: get_attr(e, "Status_Specification"),
                        format_std: get_attr(e, "Format_Std"),
                        format_spec: get_attr(e, "Format_Specification"),
                        codes: Vec::new(),
                        position: component_position,
                    };
                    composite.data_elements.push(de);
                    component_position += 1;
                }
            }
            Ok(Event::End(ref e)) => {
                let name = end_name(e);
                if name == element_name {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(GeneratorError::XmlParse {
                    path: path.to_path_buf(),
                    message: e.to_string(),
                    source: Some(e),
                })
            }
            _ => {}
        }
        buf.clear();
    }

    Ok(composite)
}

fn parse_data_element_from_xml(
    element_name: &str,
    start: &quick_xml::events::BytesStart,
    reader: &mut Reader<&[u8]>,
    path: &Path,
    position: usize,
) -> Result<MigDataElement, GeneratorError> {
    let id = element_name
        .strip_prefix("D_")
        .unwrap_or(element_name)
        .to_string();

    let mut de = MigDataElement {
        id,
        name: get_attr(start, "Name").unwrap_or_default(),
        description: get_attr(start, "Description"),
        status_std: get_attr(start, "Status_Std"),
        status_spec: get_attr(start, "Status_Specification"),
        format_std: get_attr(start, "Format_Std"),
        format_spec: get_attr(start, "Format_Specification"),
        codes: Vec::new(),
        position,
    };

    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name = elem_name(e);
                if name == "Code" {
                    let code = parse_code_from_xml(e, reader, path)?;
                    de.codes.push(code);
                }
            }
            Ok(Event::End(ref e)) => {
                let name = end_name(e);
                if name == element_name {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(GeneratorError::XmlParse {
                    path: path.to_path_buf(),
                    message: e.to_string(),
                    source: Some(e),
                })
            }
            _ => {}
        }
        buf.clear();
    }

    Ok(de)
}

fn parse_code_from_xml(
    start: &quick_xml::events::BytesStart,
    reader: &mut Reader<&[u8]>,
    path: &Path,
) -> Result<CodeDefinition, GeneratorError> {
    let name = get_attr(start, "Name").unwrap_or_default();
    let description = get_attr(start, "Description");

    let mut value = String::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Text(ref t)) => {
                value = t.unescape().unwrap_or_default().trim().to_string();
            }
            Ok(Event::End(ref e)) => {
                let tag = end_name(e);
                if tag == "Code" {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(GeneratorError::XmlParse {
                    path: path.to_path_buf(),
                    message: e.to_string(),
                    source: Some(e),
                })
            }
            _ => {}
        }
        buf.clear();
    }

    Ok(CodeDefinition {
        value,
        name,
        description,
    })
}
