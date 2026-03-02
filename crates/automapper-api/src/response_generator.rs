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
    /// UNB S002 d0007 — sender qualifier.
    pub sender_qualifier: String,
    /// UNB S003 d0010 — receiver identification.
    pub receiver_id: String,
    /// UNB S003 d0007 — receiver qualifier.
    pub receiver_qualifier: String,
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

    // Scan body for BGM d1004 (element 1, component 0) or IDE d7402 (element 1, component 0)
    for seg in msg_body {
        if seg.id == "BGM" && seg.elements.len() > 1 && !seg.elements[1].is_empty() {
            meta.transaction_ref = Some(seg.elements[1][0].clone());
        }
        if seg.id == "IDE" && seg.elements.len() > 1 && !seg.elements[1].is_empty() {
            meta.transaction_ref = Some(seg.elements[1][0].clone());
        }
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
    let marktteilnehmer = serde_json::json!([
        {
            "identifikation": &meta.receiver_id,
            "marktteilnehmerEdifact": {
                "marktrolle": "MS",
                "codepflegeCode": &meta.receiver_qualifier,
            }
        },
        {
            "identifikation": &meta.sender_id,
            "marktteilnehmerEdifact": {
                "marktrolle": "MR",
                "codepflegeCode": &meta.sender_qualifier,
            }
        }
    ]);

    let mut result = serde_json::json!({
        "nachricht": nachricht,
        "referenz": referenz_array,
        "marktteilnehmer": marktteilnehmer,
    });

    // SG4: ERC + FTX error groups → entity "Fehler"
    if !is_positive {
        let mut errors: Vec<serde_json::Value> = Vec::new();
        for issue in report.errors() {
            let error_code = map_validation_issue_to_aperak_code(issue);
            errors.push(serde_json::json!({
                "fehlerCode": error_code,
                "fehlerEdifact": {
                    "abweichungsInfo": &issue.message,
                }
            }));
        }
        if !errors.is_empty() {
            result["fehler"] = serde_json::Value::Array(errors);
        }

        // SG4.SG5: discriminated by RFF qualifier
        // fehler_01_nachricht_ref.toml (RFF+ACW) → FehlerNachrichtRef
        result["fehlerNachrichtRef"] = serde_json::json!({
            "nachrichtenReferenz": &meta.message_ref,
        });

        // fehler_02_nachricht_info.toml (RFF+AGO) → FehlerNachrichtInfo
        if let Some(ref tx_ref) = meta.transaction_ref {
            result["fehlerNachrichtInfo"] = serde_json::json!({
                "dokumentennummer": tx_ref,
            });

            // fehler_03_vorgang_info.toml (RFF+TN) → FehlerVorgangInfo
            result["fehlerVorgangInfo"] = serde_json::json!({
                "vorgangsnummer": tx_ref,
            });
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

    // Reverse map BO4E → AssembledTree
    let tree = engine.map_all_reverse(bo4e);

    // Disassemble and render
    let disassembler = Disassembler::new(mig);
    let dis_segments = disassembler.disassemble(&tree);
    let delimiters = edifact_types::EdifactDelimiters::default();
    let body_edifact = render_edifact(&dis_segments, &delimiters);

    // Wrap with UNB/UNH/UNT/UNZ envelope
    let unh_version = match msg_type {
        "APERAK" => "APERAK:D:07B:UN:2.1i",
        "CONTRL" => "CONTRL:D:07B:UN:2.0b",
        _ => msg_type,
    };
    let now_date = chrono_now_compact();
    let seg_count = dis_segments.len() + 2; // +2 for UNH+UNT

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
    // Body
    out.push_str(&body_edifact);
    // UNT
    out.push_str(&format!("UNT+{}+1'", seg_count));
    // UNZ
    out.push_str("UNZ+1+1'");

    Ok(out)
}

/// Map a validation issue to an APERAK error code.
fn map_validation_issue_to_aperak_code(
    issue: &automapper_validation::ValidationIssue,
) -> &'static str {
    // Best-effort mapping of error codes to APERAK Z-codes
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
        "Z31" // Generic: "Geschäftsvorfall wird vom Empfänger zurückgewiesen"
    }
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
        let meta = OriginalMessageMeta {
            interchange_ref: "INTREF001".into(),
            sender_id: "9900000000001".into(),
            sender_qualifier: "500".into(),
            receiver_id: "9900000000002".into(),
            receiver_qualifier: "500".into(),
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
        assert_eq!(
            bo4e["fehler"][0]["fehlerEdifact"]["abweichungsInfo"],
            "Required field missing"
        );
        // SG4.SG5 error references
        assert_eq!(bo4e["fehlerNachrichtRef"]["nachrichtenReferenz"], "MSG001");
        assert_eq!(bo4e["fehlerNachrichtInfo"]["dokumentennummer"], "TXN001");
        assert_eq!(bo4e["fehlerVorgangInfo"]["vorgangsnummer"], "TXN001");
    }

    #[test]
    fn test_positive_contrl_bo4e() {
        let meta = OriginalMessageMeta {
            interchange_ref: "INTREF001".into(),
            sender_id: "9900000000001".into(),
            sender_qualifier: "500".into(),
            receiver_id: "9900000000002".into(),
            receiver_qualifier: "500".into(),
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
}
