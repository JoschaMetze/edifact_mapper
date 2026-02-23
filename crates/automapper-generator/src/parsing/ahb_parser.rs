use std::path::Path;

use quick_xml::events::Event;
use quick_xml::Reader;

use crate::error::GeneratorError;
use crate::schema::ahb::*;

/// Parses an AHB XML file into an `AhbSchema`.
///
/// AHB XML structure:
/// - Root element: `AHB_UTILMD` (or similar) with `Versionsnummer`
/// - `AWF` elements with `Pruefidentifikator`, `Beschreibung`, `Kommunikation_von`
///   - `Uebertragungsdatei` -> `M_UTILMD` -> nested segments/groups with `AHB_Status`
/// - `Bedingungen` -> `Bedingung` elements with `Nummer` attribute and text content
pub fn parse_ahb(
    path: &Path,
    message_type: &str,
    variant: Option<&str>,
    format_version: &str,
) -> Result<AhbSchema, GeneratorError> {
    if !path.exists() {
        return Err(GeneratorError::FileNotFound(path.to_path_buf()));
    }

    let xml_content = std::fs::read_to_string(path)?;
    let mut reader = Reader::from_str(&xml_content);
    reader.config_mut().trim_text(true);

    let mut schema = AhbSchema {
        message_type: message_type.to_string(),
        variant: variant.map(|v| v.to_string()),
        version: String::new(),
        format_version: format_version.to_string(),
        source_file: path.to_string_lossy().to_string(),
        workflows: Vec::new(),
        bedingungen: Vec::new(),
    };

    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name = elem_name(e);

                if name == "AHB" || name.starts_with("AHB_") {
                    if let Some(v) = get_attr(e, "Versionsnummer") {
                        schema.version = v;
                    }
                } else if name == "AWF" {
                    let workflow = parse_workflow(e, &mut reader, path)?;
                    schema.workflows.push(workflow);
                } else if name == "Bedingungen" {
                    schema.bedingungen = parse_bedingungen(&mut reader, path)?;
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
            element: format!("AHB or AHB_{}", message_type),
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

fn parse_workflow(
    start: &quick_xml::events::BytesStart,
    reader: &mut Reader<&[u8]>,
    path: &Path,
) -> Result<Pruefidentifikator, GeneratorError> {
    let pid = get_attr(start, "Pruefidentifikator").unwrap_or_default();
    let beschreibung = get_attr(start, "Beschreibung").unwrap_or_default();
    let kommunikation_von = get_attr(start, "Kommunikation_von");

    let mut fields = Vec::new();
    let mut segment_numbers = Vec::new();
    let mut path_stack: Vec<String> = Vec::new();
    let mut current_segment_number: Option<String> = None;
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name = elem_name(e);

                let prefix_stripped = name.strip_prefix("S_").or_else(|| name.strip_prefix("G_"));

                if let Some(stripped) = prefix_stripped {
                    path_stack.push(stripped.to_string());

                    // Capture MIG segment Number for S_* elements
                    if name.starts_with("S_") {
                        let number = get_attr(e, "Number");
                        if let Some(ref num) = number {
                            segment_numbers.push(num.clone());
                        }
                        current_segment_number = number;
                    } else {
                        current_segment_number = None;
                    }

                    // Capture group-level conditional AHB_Status
                    if let Some(ahb_status) = get_attr(e, "AHB_Status") {
                        if ahb_status.contains('[') {
                            let seg_path = path_stack.join("/");
                            let field_name =
                                get_attr(e, "Name").unwrap_or_else(|| stripped.to_string());
                            fields.push(AhbFieldDefinition {
                                segment_path: seg_path,
                                name: field_name,
                                ahb_status,
                                description: None,
                                codes: Vec::new(),
                                mig_number: current_segment_number.clone(),
                            });
                        }
                    }
                } else if let Some(stripped) = name.strip_prefix("C_") {
                    path_stack.push(stripped.to_string());
                } else if let Some(stripped) = name.strip_prefix("D_") {
                    let data_element_id = stripped.to_string();
                    let ahb_status = get_attr(e, "AHB_Status").unwrap_or_default();
                    let field_name = get_attr(e, "Name").unwrap_or_else(|| data_element_id.clone());

                    // Parse codes within this data element
                    let codes = parse_ahb_codes(reader, &name, path)?;

                    let has_ahb_status = !ahb_status.is_empty();
                    let has_codes_with_status = codes.iter().any(|c| c.ahb_status.is_some());

                    if has_ahb_status || has_codes_with_status {
                        let seg_path = format!("{}/{}", path_stack.join("/"), data_element_id);

                        let effective_status = if !ahb_status.is_empty() {
                            ahb_status
                        } else {
                            codes
                                .iter()
                                .find_map(|c| c.ahb_status.clone())
                                .unwrap_or_default()
                        };

                        fields.push(AhbFieldDefinition {
                            segment_path: seg_path,
                            name: field_name,
                            ahb_status: effective_status,
                            description: None,
                            codes,
                            mig_number: current_segment_number.clone(),
                        });
                    }
                    // Note: we already consumed the end tag in parse_ahb_codes
                    buf.clear();
                    continue;
                } else if name.starts_with("M_") || name == "Uebertragungsdatei" {
                    // Message containers — just continue parsing children
                }
            }
            Ok(Event::Empty(ref e)) => {
                let name = elem_name(e);

                if let Some(stripped) = name.strip_prefix("D_") {
                    let data_element_id = stripped.to_string();
                    let ahb_status = get_attr(e, "AHB_Status").unwrap_or_default();

                    if !ahb_status.is_empty() {
                        let seg_path = format!("{}/{}", path_stack.join("/"), data_element_id);
                        let field_name =
                            get_attr(e, "Name").unwrap_or_else(|| data_element_id.clone());

                        fields.push(AhbFieldDefinition {
                            segment_path: seg_path,
                            name: field_name,
                            ahb_status,
                            description: None,
                            codes: Vec::new(),
                            mig_number: current_segment_number.clone(),
                        });
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                let name = end_name(e);

                if name == "AWF" {
                    break;
                } else if name.starts_with("S_") || name.starts_with("G_") || name.starts_with("C_")
                {
                    if name.starts_with("S_") {
                        current_segment_number = None;
                    }
                    path_stack.pop();
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

    Ok(Pruefidentifikator {
        id: pid,
        beschreibung,
        kommunikation_von,
        fields,
        segment_numbers,
    })
}

/// Parse Code elements within a data element, consuming up to the closing D_ tag.
fn parse_ahb_codes(
    reader: &mut Reader<&[u8]>,
    end_element: &str,
    path: &Path,
) -> Result<Vec<AhbCodeValue>, GeneratorError> {
    let mut codes = Vec::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name = elem_name(e);
                if name == "Code" {
                    let code_name = get_attr(e, "Name").unwrap_or_default();
                    let description = get_attr(e, "Description");
                    let ahb_status = get_attr(e, "AHB_Status");

                    // Read text content
                    let mut value = String::new();
                    let mut inner_buf = Vec::new();
                    loop {
                        match reader.read_event_into(&mut inner_buf) {
                            Ok(Event::Text(ref t)) => {
                                value = t.unescape().unwrap_or_default().trim().to_string();
                            }
                            Ok(Event::End(ref end)) => {
                                let end_tag = end_name(end);
                                if end_tag == "Code" {
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
                        inner_buf.clear();
                    }

                    codes.push(AhbCodeValue {
                        value,
                        name: code_name,
                        description,
                        ahb_status,
                    });
                }
            }
            Ok(Event::End(ref e)) => {
                let name = end_name(e);
                if name == end_element {
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

    Ok(codes)
}

fn parse_bedingungen(
    reader: &mut Reader<&[u8]>,
    path: &Path,
) -> Result<Vec<BedingungDefinition>, GeneratorError> {
    let mut bedingungen = Vec::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name = elem_name(e);
                if name == "Bedingung" {
                    let nummer = get_attr(e, "Nummer").unwrap_or_default();
                    let id = nummer.trim_matches(|c| c == '[' || c == ']').to_string();

                    let mut description = String::new();
                    let mut inner_buf = Vec::new();
                    loop {
                        match reader.read_event_into(&mut inner_buf) {
                            Ok(Event::Text(ref t)) => {
                                description = t.unescape().unwrap_or_default().trim().to_string();
                            }
                            Ok(Event::End(ref end)) => {
                                let end_tag = end_name(end);
                                if end_tag == "Bedingung" {
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
                        inner_buf.clear();
                    }

                    if !id.is_empty() {
                        bedingungen.push(BedingungDefinition { id, description });
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                let name = end_name(e);
                if name == "Bedingungen" {
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

    Ok(bedingungen)
}
