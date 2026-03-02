//! Response message generation (APERAK / CONTRL) from validation results.
//!
//! Generates a response message based on validation outcome:
//! - **No errors** → positive APERAK (BGM+312) or positive CONTRL (UCI action=7)
//! - **Errors** → negative APERAK (BGM+313) or negative CONTRL (UCI action=4)

use mig_assembly::disassembler::Disassembler;
use mig_assembly::renderer::render_edifact;
use mig_assembly::tokenize::OwnedSegment;
use serde::{Deserialize, Serialize};

use crate::state::MigServiceRegistry;

/// Which response message type to generate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResponseType {
    Aperak,
    Contrl,
}

/// Desired output format for the generated response.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResponseFormat {
    Bo4e,
    Edifact,
}

/// Options controlling response generation.
#[derive(Debug, Clone)]
pub struct ResponseOptions {
    /// Explicit response type. `None` = auto-detect from variant.
    pub response_type: Option<ResponseType>,
    /// Output format.
    pub format: ResponseFormat,
}

/// Generated response message payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedResponse {
    /// "APERAK" or "CONTRL".
    pub message_type: String,
    /// BO4E JSON of the response message.
    pub bo4e: serde_json::Value,
    /// EDIFACT string (present only when format=Edifact).
    pub edifact: Option<String>,
}

/// Metadata extracted from the original message for populating the response.
#[derive(Debug, Clone, Default)]
pub struct OriginalMessageMeta {
    /// UNB d0020 — interchange control reference.
    pub interchange_ref: String,
    /// UNB S002 d0004 — sender identification.
    pub sender_id: String,
    /// UNB S002 d0007 — sender interchange qualifier.
    pub sender_qualifier: String,
    /// UNB S003 d0010 — receiver identification.
    pub receiver_id: String,
    /// UNB S003 d0007 — receiver interchange qualifier.
    pub receiver_qualifier: String,
    /// NAD+MS C082 d3055 — sender code list responsible agency (e.g., "293" for BDEW).
    /// Falls back to UNB qualifier if no NAD+MS found.
    pub sender_code_agency: String,
    /// NAD+MR C082 d3055 — receiver code list responsible agency.
    /// Falls back to UNB qualifier if no NAD+MR found.
    pub receiver_code_agency: String,
    /// UNH d0062 — message reference number.
    pub message_ref: String,
    /// BGM d1004 or IDE d7402 — document/transaction reference.
    pub transaction_ref: Option<String>,
}

/// Extract metadata from parsed EDIFACT segments (from `InterchangeChunks`).
pub fn extract_meta_from_edifact(
    envelope: &[OwnedSegment],
    msg_body: &[OwnedSegment],
    unh: &OwnedSegment,
) -> OriginalMessageMeta {
    let mut meta = OriginalMessageMeta::default();

    // UNH d0062 (element 0)
    if !unh.elements.is_empty() && !unh.elements[0].is_empty() {
        meta.message_ref = unh.elements[0][0].clone();
    }

    // UNB: element 0 = S001 (syntax), element 1 = S002 (sender), element 2 = S003 (receiver),
    //       element 3 = S004 (date/time), element 4 = d0020 (interchange ref)
    for seg in envelope {
        if seg.id == "UNB" {
            // S002 sender (element 1)
            if seg.elements.len() > 1 {
                if !seg.elements[1].is_empty() {
                    meta.sender_id = seg.elements[1][0].clone();
                }
                if seg.elements[1].len() > 1 {
                    meta.sender_qualifier = seg.elements[1][1].clone();
                }
            }
            // S003 receiver (element 2)
            if seg.elements.len() > 2 {
                if !seg.elements[2].is_empty() {
                    meta.receiver_id = seg.elements[2][0].clone();
                }
                if seg.elements[2].len() > 1 {
                    meta.receiver_qualifier = seg.elements[2][1].clone();
                }
            }
            // d0020 interchange ref (element 4)
            if seg.elements.len() > 4 && !seg.elements[4].is_empty() {
                meta.interchange_ref = seg.elements[4][0].clone();
            }
        }
    }

    // Scan body for BGM, IDE, and NAD segments
    for seg in msg_body {
        if seg.id == "BGM" && seg.elements.len() > 1 && !seg.elements[1].is_empty() {
            meta.transaction_ref = Some(seg.elements[1][0].clone());
        }
        if seg.id == "IDE" && seg.elements.len() > 1 && !seg.elements[1].is_empty() {
            meta.transaction_ref = Some(seg.elements[1][0].clone());
        }
        // NAD+MS/MR — extract C082 d3055 (code list responsible agency)
        // NAD structure: element 0 = d3035 qualifier, element 1 = C082 (d3039:d1131:d3055)
        if seg.id == "NAD" && !seg.elements.is_empty() && !seg.elements[0].is_empty() {
            let qualifier = &seg.elements[0][0];
            if seg.elements.len() > 1 && seg.elements[1].len() > 2 {
                let d3055 = &seg.elements[1][2];
                if !d3055.is_empty() {
                    match qualifier.as_str() {
                        "MS" => meta.sender_code_agency = d3055.clone(),
                        "MR" => meta.receiver_code_agency = d3055.clone(),
                        _ => {}
                    }
                }
            }
        }
    }

    // Fallback: if no NAD code agency found, derive from MP-ID prefix
    if meta.sender_code_agency.is_empty() {
        meta.sender_code_agency = code_agency_for_mp_id(&meta.sender_id).to_string();
    }
    if meta.receiver_code_agency.is_empty() {
        meta.receiver_code_agency = code_agency_for_mp_id(&meta.receiver_id).to_string();
    }

    meta
}

/// Auto-detect response type from message variant string.
pub fn auto_detect_response_type(msg_variant: &str) -> ResponseType {
    if msg_variant.contains("Gas") {
        ResponseType::Contrl
    } else {
        ResponseType::Aperak
    }
}

/// Generate a response message from validation results.
pub fn generate_response(
    registry: &MigServiceRegistry,
    fv: &str,
    msg_variant: &str,
    report: &automapper_validation::ValidationReport,
    meta: &OriginalMessageMeta,
    opts: &ResponseOptions,
) -> Result<GeneratedResponse, crate::error::ApiError> {
    let response_type = opts
        .response_type
        .unwrap_or_else(|| auto_detect_response_type(msg_variant));

    let is_positive = report.is_valid();
    let msg_type_str = match response_type {
        ResponseType::Aperak => "APERAK",
        ResponseType::Contrl => "CONTRL",
    };

    let bo4e = match response_type {
        ResponseType::Aperak => build_aperak_bo4e(is_positive, meta, report),
        ResponseType::Contrl => build_contrl_bo4e(is_positive, meta, report),
    };

    let edifact = if opts.format == ResponseFormat::Edifact {
        Some(render_response_edifact(
            registry,
            fv,
            msg_type_str,
            &bo4e,
            meta,
        )?)
    } else {
        None
    };

    Ok(GeneratedResponse {
        message_type: msg_type_str.to_string(),
        bo4e,
        edifact,
    })
}

/// Build APERAK BO4E JSON from validation results.
///
/// The JSON structure must match what the MappingEngine's `map_all_forward` produces
/// so that `map_all_reverse` can reconstruct the EDIFACT segments correctly.
/// Key rules:
/// - Companion fields are keyed by camelCase companion_type (e.g., `marktteilnehmerEdifact`), not `_edifact`
/// - SG2 has 3 RFF qualifiers (ACE, AGO, TN) — each is a separate Referenz array element
/// - `codelisteCode` is a regular field (not companion), `codepflegeCode` is a companion field
fn build_aperak_bo4e(
    is_positive: bool,
    meta: &OriginalMessageMeta,
    report: &automapper_validation::ValidationReport,
) -> serde_json::Value {
    let bgm_code = if is_positive { "312" } else { "313" };
    let now = chrono_now_edifact();

    // Root: BGM + DTM[137] → entity "Nachricht"
    // nachricht.toml has no companion_fields with real targets (only defaults),
    // so no companion object is needed.
    let nachricht = serde_json::json!({
        "dokumentenCode": bgm_code,
        "nachrichtennummer": format!("{}BGM", meta.message_ref),
        "erstellungsdatum": &now,
    });

    // SG2: each RFF qualifier is a separate SG2 rep → array of Referenz objects
    // referenz.toml maps: rff[ACE] → datenaustauschreferenz, rff[AGO] → dokumentennummer,
    //   rff[TN] → vorgangsnummer, dtm[171] → referenzdatum
    let mut referenz_array: Vec<serde_json::Value> = Vec::new();

    // RFF+ACE (always present — datenaustauschreferenz) + DTM+171 (reference date)
    referenz_array.push(serde_json::json!({
        "datenaustauschreferenz": &meta.interchange_ref,
        "referenzdatum": &now,
    }));

    // RFF+AGO and RFF+TN (present when we have a transaction reference)
    if let Some(ref tx_ref) = meta.transaction_ref {
        referenz_array.push(serde_json::json!({
            "dokumentennummer": tx_ref,
        }));
        referenz_array.push(serde_json::json!({
            "vorgangsnummer": tx_ref,
        }));
    }

    // SG3: NAD parties → entity "Marktteilnehmer" (array)
    // marktteilnehmer.toml: fields={identifikation, codelisteCode},
    //   companion_fields under "marktteilnehmerEdifact"={marktrolle, codepflegeCode, ...}
    // Swap sender/receiver: responder (original receiver) becomes MS, original sender becomes MR
    // codepflegeCode = NAD C082 d3055 from the original message (e.g., "293" for BDEW)
    let marktteilnehmer = serde_json::json!([
        {
            "identifikation": &meta.receiver_id,
            "marktteilnehmerEdifact": {
                "marktrolle": "MS",
                "codepflegeCode": &meta.receiver_code_agency,
            }
        },
        {
            "identifikation": &meta.sender_id,
            "marktteilnehmerEdifact": {
                "marktrolle": "MR",
                "codepflegeCode": &meta.sender_code_agency,
            }
        }
    ]);

    let mut result = serde_json::json!({
        "nachricht": nachricht,
        "referenz": referenz_array,
        "marktteilnehmer": marktteilnehmer,
    });

    // SG4: ERC + FTX error groups → entity "Fehler"
    // Each SG4 has its own SG5 children (RFF+ACW, RFF+AGO, RFF+TN) nested inside.
    if !is_positive {
        let mut errors: Vec<serde_json::Value> = Vec::new();

        for issue in report.errors() {
            let error_code = map_validation_issue_to_aperak_code(issue);
            let german_text = sanitize_edifact_text(&format!(
                "{}: {}",
                aperak_code_german_label(error_code),
                issue.message,
            ));
            let mut referenzen = serde_json::json!({
                "nachrichtenReferenz": &meta.message_ref,
            });
            if let Some(ref tx_ref) = meta.transaction_ref {
                referenzen["dokumentennummer"] = serde_json::json!(tx_ref);
                referenzen["vorgangsnummer"] = serde_json::json!(tx_ref);
            }

            let error_obj = serde_json::json!({
                "fehlerCode": error_code,
                "fehlerEdifact": {
                    "abweichungsInfo": german_text,
                },
                "referenzen": referenzen,
            });

            errors.push(error_obj);
        }

        if !errors.is_empty() {
            result["fehler"] = serde_json::Value::Array(errors);
        }
    }

    result
}

/// Build CONTRL BO4E JSON from validation results.
///
/// The JSON structure must match what the MappingEngine's `map_all_forward` produces.
/// Companion fields are keyed by camelCase companion_type, not `_edifact`.
fn build_contrl_bo4e(
    is_positive: bool,
    meta: &OriginalMessageMeta,
    _report: &automapper_validation::ValidationReport,
) -> serde_json::Value {
    let action = if is_positive { "7" } else { "4" };

    // Root: UCI → entity "Uebertragungspruefung"
    // uebertragungspruefung.toml: fields={datenaustauschreferenz, aktion},
    //   companion_fields under "uebertragungspruefungEdifact"={absenderMpId, ...}
    let mut result = serde_json::json!({
        "uebertragungspruefung": {
            "datenaustauschreferenz": &meta.interchange_ref,
            "aktion": action,
            "uebertragungspruefungEdifact": {
                "absenderMpId": &meta.sender_id,
                "absenderCodeQualifier": &meta.sender_qualifier,
                "empfaengerMpId": &meta.receiver_id,
                "empfaengerCodeQualifier": &meta.receiver_qualifier,
            }
        }
    });

    // SG1: UCM → entity "Nachrichtenpruefung"
    // nachrichtenpruefung.toml: fields={nachrichtenReferenznummer, aktion}
    if !is_positive {
        result["nachrichtenpruefung"] = serde_json::json!({
            "nachrichtenReferenznummer": &meta.message_ref,
            "aktion": "4",
        });
    }

    result
}

/// Render the response BO4E JSON as EDIFACT using the response engine + MIG.
///
/// Error segments (SG4/SG5) are rendered manually because the reverse mapper
/// cannot distribute SG5 children across multiple SG4 parent instances.
/// The header entities (nachricht, referenz, marktteilnehmer) are reverse-mapped normally.
fn render_response_edifact(
    registry: &MigServiceRegistry,
    fv: &str,
    msg_type: &str,
    bo4e: &serde_json::Value,
    meta: &OriginalMessageMeta,
) -> Result<String, crate::error::ApiError> {
    let engine =
        registry
            .response_engine(fv, msg_type)
            .ok_or_else(|| crate::error::ApiError::Internal {
                message: format!("No response mapping engine for {fv}/{msg_type}"),
            })?;

    let mig =
        registry
            .response_mig(fv, msg_type)
            .ok_or_else(|| crate::error::ApiError::Internal {
                message: format!("No response MIG for {fv}/{msg_type}"),
            })?;

    // Strip error entities from BO4E — reverse-map only header entities
    let header_bo4e = strip_error_entities(bo4e);

    // Reverse map header BO4E → AssembledTree
    let tree = engine.map_all_reverse(&header_bo4e);

    // Disassemble and render header
    let disassembler = Disassembler::new(mig);
    let dis_segments = disassembler.disassemble(&tree);
    let delimiters = edifact_types::EdifactDelimiters::default();
    let header_edifact = render_edifact(&dis_segments, &delimiters);

    // Render error segments manually from nested fehler array
    let error_edifact = render_error_segments(bo4e, &delimiters);
    let error_seg_count = count_error_segments(bo4e);

    // Wrap with UNB/UNH/UNT/UNZ envelope
    let unh_version = match msg_type {
        "APERAK" => "APERAK:D:07B:UN:2.1i",
        "CONTRL" => "CONTRL:D:07B:UN:2.0b",
        _ => msg_type,
    };
    let now_date = chrono_now_compact();
    let seg_count = dis_segments.len() + error_seg_count + 2; // +2 for UNH+UNT

    let mut out = String::new();
    // UNB — swap sender/receiver for response
    out.push_str(&format!(
        "UNB+UNOC:3+{}:{}+{}:{}+{}+{}'",
        meta.receiver_id,
        meta.receiver_qualifier,
        meta.sender_id,
        meta.sender_qualifier,
        now_date,
        format_args!("RESP{}", &meta.interchange_ref.get(..6).unwrap_or("000000"))
    ));
    // UNH
    out.push_str(&format!("UNH+1+{}'", unh_version));
    // Header body (BGM, DTM, RFF, NAD)
    out.push_str(&header_edifact);
    // Error segments (ERC+FTX, then SG5 RFFs per error)
    out.push_str(&error_edifact);
    // UNT
    out.push_str(&format!("UNT+{}+1'", seg_count));
    // UNZ
    out.push_str("UNZ+1+1'");

    Ok(out)
}

/// Strip error-related entities from BO4E JSON, returning only header entities.
fn strip_error_entities(bo4e: &serde_json::Value) -> serde_json::Value {
    if let Some(obj) = bo4e.as_object() {
        let mut header = serde_json::Map::new();
        for (key, value) in obj {
            if key != "fehler" {
                header.insert(key.clone(), value.clone());
            }
        }
        serde_json::Value::Object(header)
    } else {
        bo4e.clone()
    }
}

/// Render SG4/SG5 error segments manually from the nested fehler array.
///
/// Each fehler object produces:
/// - `ERC+{code}'`
/// - `FTX+ABO+++{text}'`
/// - `RFF+ACW:{nachrichtenReferenz}'` (from referenzen)
/// - `RFF+AGO:{dokumentennummer}'` (from referenzen, if present)
/// - `RFF+TN:{vorgangsnummer}'` (from referenzen, if present)
fn render_error_segments(
    bo4e: &serde_json::Value,
    delimiters: &edifact_types::EdifactDelimiters,
) -> String {
    let fehler = match bo4e.get("fehler").and_then(|v| v.as_array()) {
        Some(arr) => arr,
        None => return String::new(),
    };

    let release = delimiters.release as char;
    let mut out = String::new();

    for error in fehler {
        // ERC segment
        let code = error
            .get("fehlerCode")
            .and_then(|v| v.as_str())
            .unwrap_or("Z31");
        out.push_str(&format!("ERC+{code}'"));

        // FTX+ABO segment
        let text = error
            .pointer("/fehlerEdifact/abweichungsInfo")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        out.push_str("FTX+ABO+++");
        escape_edifact_value(text, delimiters, release, &mut out);
        out.push('\'');

        let refs = error.get("referenzen");

        // SG5: RFF+ACW (message reference)
        if let Some(ref_val) = refs
            .and_then(|r| r.get("nachrichtenReferenz"))
            .and_then(|v| v.as_str())
        {
            out.push_str("RFF+ACW:");
            escape_edifact_value(ref_val, delimiters, release, &mut out);
            out.push('\'');
        }

        // SG5: RFF+AGO (document number)
        if let Some(doc_val) = refs
            .and_then(|r| r.get("dokumentennummer"))
            .and_then(|v| v.as_str())
        {
            out.push_str("RFF+AGO:");
            escape_edifact_value(doc_val, delimiters, release, &mut out);
            out.push('\'');
        }

        // SG5: RFF+TN (transaction reference)
        if let Some(txn_val) = refs
            .and_then(|r| r.get("vorgangsnummer"))
            .and_then(|v| v.as_str())
        {
            out.push_str("RFF+TN:");
            escape_edifact_value(txn_val, delimiters, release, &mut out);
            out.push('\'');
        }
    }

    out
}

/// Count the number of EDIFACT segments generated for errors.
fn count_error_segments(bo4e: &serde_json::Value) -> usize {
    let fehler = match bo4e.get("fehler").and_then(|v| v.as_array()) {
        Some(arr) => arr,
        None => return 0,
    };

    let mut count = 0;
    for error in fehler {
        count += 2; // ERC + FTX always present
        if let Some(refs) = error.get("referenzen") {
            if refs.get("nachrichtenReferenz").is_some() {
                count += 1;
            }
            if refs.get("dokumentennummer").is_some() {
                count += 1;
            }
            if refs.get("vorgangsnummer").is_some() {
                count += 1;
            }
        }
    }
    count
}

/// Escape a value for inline EDIFACT rendering (same rules as renderer::escape_component).
fn escape_edifact_value(
    value: &str,
    delimiters: &edifact_types::EdifactDelimiters,
    release: char,
    out: &mut String,
) {
    let special = [
        delimiters.element,
        delimiters.component,
        delimiters.segment,
        delimiters.release,
    ];
    for b in value.bytes() {
        if special.contains(&b) {
            out.push(release);
        }
        out.push(b as char);
    }
}

/// Derive the code list responsible agency (NAD C082 d3055) from an MP-ID prefix.
///
/// German energy market convention:
/// - `99xxxx` → "293" (BDEW — Bundesverband der Energie- und Wasserwirtschaft)
/// - `98xxxx` → "332" (DVGW — Deutscher Verein des Gas- und Wasserfaches)
/// - All others → "500" (GS1 — Global Standard 1, GLN numbers)
fn code_agency_for_mp_id(mp_id: &str) -> &'static str {
    if mp_id.starts_with("99") {
        "293"
    } else if mp_id.starts_with("98") {
        "332"
    } else {
        "500"
    }
}

/// Map a validation issue to an APERAK error code.
fn map_validation_issue_to_aperak_code(
    issue: &automapper_validation::ValidationIssue,
) -> &'static str {
    let code = &issue.code;
    if code.contains("UNEXPECTED_SEGMENT") || code.contains("REPETITION") {
        "Z40"
    } else if code.contains("MISSING") {
        "Z29"
    } else if code.contains("FORMAT") {
        "Z35"
    } else if code.contains("CODE") || code.contains("INVALID") {
        "Z39"
    } else {
        "Z31"
    }
}

/// German label for an APERAK Z-code (used in FTX+ABO error description).
fn aperak_code_german_label(code: &str) -> &'static str {
    match code {
        "Z29" => "Pflichtfeld fehlt",
        "Z31" => "Zurueckweisung",
        "Z35" => "Formatfehler",
        "Z39" => "Ungueltiger Code",
        "Z40" => "Strukturfehler",
        _ => "Fehler",
    }
}

/// Sanitize text for EDIFACT FTX segments.
///
/// Replaces single quotes (`'`) with double quotes (`"`) since `'` is the
/// EDIFACT segment terminator. Even when escaped (`?'`), it can cause issues.
fn sanitize_edifact_text(text: &str) -> String {
    text.replace('\'', "\"")
}

/// Current timestamp in EDIFACT DTM+137 format (CCYYMMDDHHMMZZZ).
fn chrono_now_edifact() -> String {
    // Use a simple UTC timestamp without chrono dependency
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    // Convert to approximate YYYYMMDDHHMMSS+00
    let secs_per_day = 86400u64;
    let days = now / secs_per_day;
    let time_of_day = now % secs_per_day;
    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;

    // Approximate date calculation (good enough for response generation)
    let mut year = 1970u64;
    let mut remaining_days = days;
    loop {
        let days_in_year = if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) {
            366
        } else {
            365
        };
        if remaining_days < days_in_year {
            break;
        }
        remaining_days -= days_in_year;
        year += 1;
    }
    let days_in_months: [u64; 12] = if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    let mut month = 1u64;
    for &dim in &days_in_months {
        if remaining_days < dim {
            break;
        }
        remaining_days -= dim;
        month += 1;
    }
    let day = remaining_days + 1;

    format!(
        "{:04}{:02}{:02}{:02}{:02}+00",
        year, month, day, hours, minutes
    )
}

/// Current date/time in UNB S004 format (YYMMDD:HHMM).
fn chrono_now_compact() -> String {
    let ts = chrono_now_edifact();
    // ts is "YYYYMMDDHHMMSS+00", extract to "YYMMDD:HHMM"
    if ts.len() >= 12 {
        format!("{}:{}", &ts[2..8], &ts[8..12])
    } else {
        "260101:0000".to_string()
    }
}

/// Parse response options from request fields.
pub fn parse_response_options(
    response_type: Option<&str>,
    format: Option<&str>,
) -> Option<ResponseOptions> {
    // If neither field is provided, no response generation
    if response_type.is_none() && format.is_none() {
        return None;
    }

    let rt = response_type.and_then(|s| match s.to_lowercase().as_str() {
        "aperak" => Some(ResponseType::Aperak),
        "contrl" => Some(ResponseType::Contrl),
        _ => None,
    });

    let fmt = match format.map(|s| s.to_lowercase()).as_deref() {
        Some("edifact") => ResponseFormat::Edifact,
        _ => ResponseFormat::Bo4e,
    };

    Some(ResponseOptions {
        response_type: rt,
        format: fmt,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_detect_aperak_for_strom() {
        assert_eq!(
            auto_detect_response_type("UTILMD_Strom"),
            ResponseType::Aperak
        );
    }

    #[test]
    fn test_auto_detect_contrl_for_gas() {
        assert_eq!(
            auto_detect_response_type("UTILMD_Gas"),
            ResponseType::Contrl
        );
    }

    #[test]
    fn test_positive_aperak_bo4e() {
        // MP-IDs starting with 99 → code agency 293 (BDEW)
        let meta = OriginalMessageMeta {
            interchange_ref: "INTREF001".into(),
            sender_id: "9900000000001".into(),
            sender_qualifier: "500".into(),
            receiver_id: "9900000000002".into(),
            receiver_qualifier: "500".into(),
            sender_code_agency: "293".into(),
            receiver_code_agency: "293".into(),
            message_ref: "MSG001".into(),
            transaction_ref: Some("TXN001".into()),
        };

        let report = automapper_validation::ValidationReport::new(
            "UTILMD",
            automapper_validation::ValidationLevel::Full,
        );

        let bo4e = build_aperak_bo4e(true, &meta, &report);
        // Root entity: nachricht (camelCase)
        assert_eq!(bo4e["nachricht"]["dokumentenCode"], "312");
        assert_eq!(bo4e["nachricht"]["nachrichtennummer"], "MSG001BGM");
        // SG2 referenz is an array (one per RFF qualifier)
        assert!(bo4e["referenz"].is_array());
        assert_eq!(bo4e["referenz"].as_array().unwrap().len(), 3);
        assert_eq!(bo4e["referenz"][0]["datenaustauschreferenz"], "INTREF001");
        assert_eq!(bo4e["referenz"][1]["dokumentennummer"], "TXN001");
        assert_eq!(bo4e["referenz"][2]["vorgangsnummer"], "TXN001");
        // SG3 marktteilnehmer with companion under marktteilnehmerEdifact
        assert!(bo4e["marktteilnehmer"].is_array());
        assert_eq!(
            bo4e["marktteilnehmer"][0]["identifikation"],
            "9900000000002"
        );
        assert_eq!(
            bo4e["marktteilnehmer"][0]["marktteilnehmerEdifact"]["marktrolle"],
            "MS"
        );
        assert_eq!(
            bo4e["marktteilnehmer"][1]["marktteilnehmerEdifact"]["marktrolle"],
            "MR"
        );
        // codepflegeCode should come from NAD d3055 (293), not UNB d0007 (500)
        assert_eq!(
            bo4e["marktteilnehmer"][0]["marktteilnehmerEdifact"]["codepflegeCode"],
            "293"
        );
        assert_eq!(
            bo4e["marktteilnehmer"][1]["marktteilnehmerEdifact"]["codepflegeCode"],
            "293"
        );
        // No errors
        assert!(bo4e.get("fehler").is_none());
    }

    #[test]
    fn test_negative_aperak_bo4e() {
        let meta = OriginalMessageMeta {
            interchange_ref: "INTREF001".into(),
            sender_id: "9900000000001".into(),
            sender_qualifier: "500".into(),
            receiver_id: "9900000000002".into(),
            receiver_qualifier: "500".into(),
            sender_code_agency: "293".into(),
            receiver_code_agency: "293".into(),
            message_ref: "MSG001".into(),
            transaction_ref: Some("TXN001".into()),
        };

        let mut report = automapper_validation::ValidationReport::new(
            "UTILMD",
            automapper_validation::ValidationLevel::Full,
        );
        report.add_issue(automapper_validation::ValidationIssue::new(
            automapper_validation::Severity::Error,
            automapper_validation::ValidationCategory::Ahb,
            "MISSING_VALUE",
            "Required field missing",
        ));

        let bo4e = build_aperak_bo4e(false, &meta, &report);
        assert_eq!(bo4e["nachricht"]["dokumentenCode"], "313");
        // SG4 fehler with companion under fehlerEdifact
        assert!(bo4e["fehler"].is_array());
        assert_eq!(bo4e["fehler"].as_array().unwrap().len(), 1);
        assert_eq!(bo4e["fehler"][0]["fehlerCode"], "Z29");
        // FTX+ABO text should be German
        let abo_text = bo4e["fehler"][0]["fehlerEdifact"]["abweichungsInfo"]
            .as_str()
            .unwrap();
        assert!(
            abo_text.starts_with("Pflichtfeld fehlt"),
            "Error text should be German, got: {abo_text}"
        );
        assert!(
            abo_text.contains("Required field missing"),
            "Should include original detail"
        );
        // SG5 references nested inside each fehler object as single "referenzen"
        let refs = &bo4e["fehler"][0]["referenzen"];
        assert_eq!(refs["nachrichtenReferenz"], "MSG001");
        assert_eq!(refs["dokumentennummer"], "TXN001");
        assert_eq!(refs["vorgangsnummer"], "TXN001");
        // No top-level SG5 entities
        assert!(bo4e.get("fehlerNachrichtRef").is_none());
        assert!(bo4e.get("fehlerNachrichtInfo").is_none());
        assert!(bo4e.get("fehlerVorgangInfo").is_none());
    }

    #[test]
    fn test_negative_aperak_multiple_errors() {
        let meta = OriginalMessageMeta {
            interchange_ref: "INTREF001".into(),
            sender_id: "9900000000001".into(),
            sender_qualifier: "500".into(),
            receiver_id: "9900000000002".into(),
            receiver_qualifier: "500".into(),
            sender_code_agency: "293".into(),
            receiver_code_agency: "293".into(),
            message_ref: "MSG001".into(),
            transaction_ref: Some("TXN001".into()),
        };

        let mut report = automapper_validation::ValidationReport::new(
            "UTILMD",
            automapper_validation::ValidationLevel::Full,
        );
        report.add_issue(automapper_validation::ValidationIssue::new(
            automapper_validation::Severity::Error,
            automapper_validation::ValidationCategory::Ahb,
            "MISSING_VALUE",
            "First error",
        ));
        report.add_issue(automapper_validation::ValidationIssue::new(
            automapper_validation::Severity::Error,
            automapper_validation::ValidationCategory::Ahb,
            "INVALID_CODE",
            "Second error",
        ));

        let bo4e = build_aperak_bo4e(false, &meta, &report);
        let errors = bo4e["fehler"].as_array().unwrap();
        assert_eq!(errors.len(), 2);

        // Each error has its own nested referenzen
        assert_eq!(errors[0]["fehlerCode"], "Z29");
        assert_eq!(errors[0]["referenzen"]["nachrichtenReferenz"], "MSG001");
        assert_eq!(errors[0]["referenzen"]["dokumentennummer"], "TXN001");

        assert_eq!(errors[1]["fehlerCode"], "Z39");
        assert_eq!(errors[1]["referenzen"]["nachrichtenReferenz"], "MSG001");
        assert_eq!(errors[1]["referenzen"]["vorgangsnummer"], "TXN001");
    }

    #[test]
    fn test_render_error_segments() {
        let bo4e = serde_json::json!({
            "fehler": [
                {
                    "fehlerCode": "Z31",
                    "fehlerEdifact": { "abweichungsInfo": "Zurueckweisung: test" },
                    "referenzen": {
                        "nachrichtenReferenz": "MSG001",
                        "dokumentennummer": "DOC001",
                        "vorgangsnummer": "DOC001",
                    },
                },
                {
                    "fehlerCode": "Z29",
                    "fehlerEdifact": { "abweichungsInfo": "Pflichtfeld fehlt: field X" },
                    "referenzen": {
                        "nachrichtenReferenz": "MSG001",
                        "dokumentennummer": "DOC001",
                        "vorgangsnummer": "DOC001",
                    },
                }
            ]
        });

        let delimiters = edifact_types::EdifactDelimiters::default();
        let rendered = render_error_segments(&bo4e, &delimiters);

        // Should have: ERC+FTX+RFF(ACW)+RFF(AGO)+RFF(TN) per error = 5 segments each
        let segments: Vec<&str> = rendered.split('\'').filter(|s| !s.is_empty()).collect();
        assert_eq!(segments.len(), 10);

        // First error
        assert_eq!(segments[0], "ERC+Z31");
        assert!(segments[1].starts_with("FTX+ABO+++Zurueckweisung"));
        assert_eq!(segments[2], "RFF+ACW:MSG001");
        assert_eq!(segments[3], "RFF+AGO:DOC001");
        assert_eq!(segments[4], "RFF+TN:DOC001");

        // Second error
        assert_eq!(segments[5], "ERC+Z29");
        assert!(segments[6].starts_with("FTX+ABO+++Pflichtfeld"));
        assert_eq!(segments[7], "RFF+ACW:MSG001");
        assert_eq!(segments[8], "RFF+AGO:DOC001");
        assert_eq!(segments[9], "RFF+TN:DOC001");
    }

    #[test]
    fn test_count_error_segments() {
        // With all SG5 references
        let bo4e = serde_json::json!({
            "fehler": [
                {
                    "fehlerCode": "Z31",
                    "fehlerEdifact": {},
                    "referenzen": {
                        "nachrichtenReferenz": "MSG001",
                        "dokumentennummer": "DOC001",
                        "vorgangsnummer": "DOC001",
                    },
                }
            ]
        });
        assert_eq!(count_error_segments(&bo4e), 5); // ERC + FTX + 3 RFFs

        // Without optional SG5 references
        let bo4e_minimal = serde_json::json!({
            "fehler": [
                {
                    "fehlerCode": "Z31",
                    "fehlerEdifact": {},
                    "referenzen": { "nachrichtenReferenz": "MSG001" },
                }
            ]
        });
        assert_eq!(count_error_segments(&bo4e_minimal), 3); // ERC + FTX + 1 RFF

        // No errors
        let bo4e_empty = serde_json::json!({});
        assert_eq!(count_error_segments(&bo4e_empty), 0);
    }

    #[test]
    fn test_positive_contrl_bo4e() {
        let meta = OriginalMessageMeta {
            interchange_ref: "INTREF001".into(),
            sender_id: "9900000000001".into(),
            sender_qualifier: "500".into(),
            receiver_id: "9900000000002".into(),
            receiver_qualifier: "500".into(),
            sender_code_agency: "293".into(),
            receiver_code_agency: "293".into(),
            message_ref: "MSG001".into(),
            transaction_ref: None,
        };

        let report = automapper_validation::ValidationReport::new(
            "UTILMD",
            automapper_validation::ValidationLevel::Full,
        );

        let bo4e = build_contrl_bo4e(true, &meta, &report);
        assert_eq!(bo4e["uebertragungspruefung"]["aktion"], "7");
        assert_eq!(
            bo4e["uebertragungspruefung"]["datenaustauschreferenz"],
            "INTREF001"
        );
        // Companion fields under uebertragungspruefungEdifact
        assert_eq!(
            bo4e["uebertragungspruefung"]["uebertragungspruefungEdifact"]["absenderMpId"],
            "9900000000001"
        );
        assert!(bo4e.get("nachrichtenpruefung").is_none());
    }

    #[test]
    fn test_negative_contrl_bo4e() {
        let meta = OriginalMessageMeta {
            interchange_ref: "INTREF001".into(),
            sender_id: "9900000000001".into(),
            sender_qualifier: "500".into(),
            receiver_id: "9900000000002".into(),
            receiver_qualifier: "500".into(),
            sender_code_agency: "293".into(),
            receiver_code_agency: "293".into(),
            message_ref: "MSG001".into(),
            transaction_ref: None,
        };

        let mut report = automapper_validation::ValidationReport::new(
            "UTILMD",
            automapper_validation::ValidationLevel::Full,
        );
        report.add_issue(automapper_validation::ValidationIssue::new(
            automapper_validation::Severity::Error,
            automapper_validation::ValidationCategory::Structure,
            "UNEXPECTED_SEGMENT",
            "Unexpected segment",
        ));

        let bo4e = build_contrl_bo4e(false, &meta, &report);
        assert_eq!(bo4e["uebertragungspruefung"]["aktion"], "4");
        assert_eq!(bo4e["nachrichtenpruefung"]["aktion"], "4");
        assert_eq!(
            bo4e["nachrichtenpruefung"]["nachrichtenReferenznummer"],
            "MSG001"
        );
    }

    #[test]
    fn test_parse_response_options_none() {
        assert!(parse_response_options(None, None).is_none());
    }

    #[test]
    fn test_parse_response_options_bo4e() {
        let opts = parse_response_options(Some("aperak"), Some("bo4e")).unwrap();
        assert_eq!(opts.response_type, Some(ResponseType::Aperak));
        assert_eq!(opts.format, ResponseFormat::Bo4e);
    }

    #[test]
    fn test_parse_response_options_edifact() {
        let opts = parse_response_options(Some("contrl"), Some("edifact")).unwrap();
        assert_eq!(opts.response_type, Some(ResponseType::Contrl));
        assert_eq!(opts.format, ResponseFormat::Edifact);
    }

    #[test]
    fn test_parse_response_options_auto_detect() {
        let opts = parse_response_options(None, Some("bo4e")).unwrap();
        assert_eq!(opts.response_type, None);
        assert_eq!(opts.format, ResponseFormat::Bo4e);
    }

    #[test]
    fn test_code_agency_for_mp_id() {
        // 99-prefix → BDEW (293)
        assert_eq!(code_agency_for_mp_id("9900000000001"), "293");
        assert_eq!(code_agency_for_mp_id("9978842000002"), "293");
        // 98-prefix → DVGW (332)
        assert_eq!(code_agency_for_mp_id("9800000000001"), "332");
        // Other → GS1 (500)
        assert_eq!(code_agency_for_mp_id("1234567890128"), "500");
        assert_eq!(code_agency_for_mp_id("4012345000009"), "500");
    }

    #[test]
    fn test_extract_meta_nad_code_agency() {
        use mig_assembly::tokenize::OwnedSegment;

        // UNB with d0007=500 (interchange qualifier)
        let envelope = vec![OwnedSegment {
            id: "UNB".into(),
            elements: vec![
                vec!["UNOC".into(), "3".into()],            // S001
                vec!["9978842000002".into(), "500".into()], // S002 sender
                vec!["9900269000000".into(), "500".into()], // S003 receiver
                vec!["250331".into(), "1329".into()],       // S004
                vec!["ALEXANDE121980".into()],              // d0020
            ],
            segment_number: 0,
        }];

        let unh = OwnedSegment {
            id: "UNH".into(),
            elements: vec![vec!["MSG001".into()]],
            segment_number: 1,
        };

        // Body with NAD+MS/MR that have d3055=293 (different from UNB d0007=500)
        let body = vec![
            OwnedSegment {
                id: "BGM".into(),
                elements: vec![vec!["E01".into()], vec!["DOC001".into()]],
                segment_number: 2,
            },
            OwnedSegment {
                id: "NAD".into(),
                elements: vec![
                    vec!["MS".into()],
                    vec!["9978842000002".into(), String::new(), "293".into()],
                ],
                segment_number: 3,
            },
            OwnedSegment {
                id: "NAD".into(),
                elements: vec![
                    vec!["MR".into()],
                    vec!["9900269000000".into(), String::new(), "293".into()],
                ],
                segment_number: 4,
            },
        ];

        let meta = extract_meta_from_edifact(&envelope, &body, &unh);

        // UNB qualifiers are "500"
        assert_eq!(meta.sender_qualifier, "500");
        assert_eq!(meta.receiver_qualifier, "500");
        // NAD code agencies should be "293" (from NAD d3055, not UNB d0007)
        assert_eq!(meta.sender_code_agency, "293");
        assert_eq!(meta.receiver_code_agency, "293");
    }

    #[test]
    fn test_extract_meta_fallback_derives_from_mp_id() {
        use mig_assembly::tokenize::OwnedSegment;

        // UNB without NAD segments in body — fallback to MP-ID derivation
        let envelope = vec![OwnedSegment {
            id: "UNB".into(),
            elements: vec![
                vec!["UNOC".into(), "3".into()],
                vec!["9900123456789".into(), "500".into()],
                vec!["9800987654321".into(), "500".into()],
                vec!["260101".into(), "1200".into()],
                vec!["REF001".into()],
            ],
            segment_number: 0,
        }];
        let unh = OwnedSegment {
            id: "UNH".into(),
            elements: vec![vec!["MSG001".into()]],
            segment_number: 1,
        };
        let body = vec![OwnedSegment {
            id: "BGM".into(),
            elements: vec![vec!["E01".into()]],
            segment_number: 2,
        }];

        let meta = extract_meta_from_edifact(&envelope, &body, &unh);

        // No NAD segments → fallback derives from MP-ID prefix
        assert_eq!(meta.sender_code_agency, "293"); // 99-prefix → BDEW
        assert_eq!(meta.receiver_code_agency, "332"); // 98-prefix → DVGW
    }
}
