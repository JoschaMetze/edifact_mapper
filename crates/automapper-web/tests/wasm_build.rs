//! Tests for frontend types and API client contracts.
//!
//! These tests run on the native target (not WASM) to verify
//! serialization and deserialization of API types.

use automapper_web::types::*;

#[test]
fn test_direction_labels() {
    let dir = Direction::EdifactToBo4e;
    assert_eq!(dir.input_label(), "EDIFACT");
    assert_eq!(dir.output_label(), "BO4E JSON");
    assert_eq!(dir.api_path(), "/api/v2/convert");
    assert_eq!(dir.label(), "EDIFACT -> BO4E");
}

#[test]
fn test_convert_v2_request_serialization() {
    let req = ConvertV2Request {
        input: "UNH+1+UTILMD'".to_string(),
        mode: "bo4e".to_string(),
        format_version: "FV2504".to_string(),
    };

    let json = serde_json::to_value(&req).unwrap();
    assert_eq!(json["input"], "UNH+1+UTILMD'");
    assert_eq!(json["mode"], "bo4e");
    assert_eq!(json["format_version"], "FV2504");
}

#[test]
fn test_convert_v2_response_deserialization() {
    let json = r#"{
        "mode": "bo4e",
        "result": {"stammdaten": {}, "transaktionsdaten": {}},
        "duration_ms": 12.3
    }"#;

    let resp: ConvertV2Response = serde_json::from_str(json).unwrap();
    assert_eq!(resp.mode, "bo4e");
    assert_eq!(resp.duration_ms, 12.3);
    assert!(resp.result.is_object());
}

#[test]
fn test_inspect_response_deserialization() {
    let json = r#"{
        "segments": [
            {
                "tag": "UNH",
                "line_number": 1,
                "raw_content": "UNH+1+UTILMD:D:11A:UN",
                "elements": [
                    {
                        "position": 1,
                        "value": "1",
                        "components": null
                    }
                ],
                "children": null
            }
        ],
        "segment_count": 1,
        "message_type": "UTILMD",
        "format_version": null
    }"#;

    let resp: InspectResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.segment_count, 1);
    assert_eq!(resp.segments[0].tag, "UNH");
    assert_eq!(resp.message_type, Some("UTILMD".to_string()));
}

#[test]
fn test_coordinator_info_deserialization() {
    let json = r#"{
        "message_type": "UTILMD",
        "description": "UTILMD coordinator",
        "supported_versions": ["FV2504", "FV2510"]
    }"#;

    let info: CoordinatorInfo = serde_json::from_str(json).unwrap();
    assert_eq!(info.message_type, "UTILMD");
    assert_eq!(info.supported_versions.len(), 2);
}

#[test]
fn test_health_response_deserialization() {
    let json = r#"{
        "healthy": true,
        "version": "0.1.0",
        "available_coordinators": ["UTILMD"],
        "uptime_seconds": 123.456
    }"#;

    let health: HealthResponse = serde_json::from_str(json).unwrap();
    assert!(health.healthy);
    assert_eq!(health.version, "0.1.0");
}

#[test]
fn test_error_entry_deserialization() {
    let json = r#"{
        "code": "PARSE_ERROR",
        "message": "unterminated segment",
        "location": "byte 42",
        "severity": "error"
    }"#;

    let err: ApiErrorEntry = serde_json::from_str(json).unwrap();
    assert_eq!(err.code, "PARSE_ERROR");
    assert_eq!(err.severity, "error");
    assert_eq!(err.location, Some("byte 42".to_string()));
}

#[test]
fn test_validate_v2_request_serialization() {
    let req = ValidateV2Request {
        input: "UNH+1+UTILMD'".to_string(),
        format_version: "FV2504".to_string(),
        ..Default::default()
    };

    let json = serde_json::to_value(&req).unwrap();
    assert_eq!(json["input"], "UNH+1+UTILMD'");
    assert_eq!(json["format_version"], "FV2504");
    assert_eq!(json["level"], "full");
}

#[test]
fn test_validate_v2_response_deserialization() {
    let json = r#"{
        "report": {
            "valid": true,
            "issues": [],
            "summary": {"total": 0}
        },
        "duration_ms": 5.67
    }"#;

    let resp: ValidateV2Response = serde_json::from_str(json).unwrap();
    assert_eq!(resp.duration_ms, 5.67);
    assert!(resp.report.is_object());
    assert_eq!(resp.report["valid"], true);
}

#[test]
fn test_convert_v2_response_with_validation() {
    let json = r#"{
        "mode": "bo4e",
        "result": {"stammdaten": {}},
        "duration_ms": 10.0,
        "validation": {
            "valid": false,
            "issues": [{"severity": "Warning", "code": "W001", "message": "minor issue"}]
        }
    }"#;

    let resp: ConvertV2Response = serde_json::from_str(json).unwrap();
    assert_eq!(resp.mode, "bo4e");
    assert!(resp.validation.is_some());
    let validation = resp.validation.unwrap();
    assert_eq!(validation["valid"], false);
    assert_eq!(validation["issues"].as_array().unwrap().len(), 1);
}

#[test]
fn test_extract_validation_issues_from_report() {
    use automapper_web::types::extract_validation_issues;

    let report = serde_json::json!({
        "valid": false,
        "issues": [
            {
                "severity": "Error",
                "category": "structure",
                "code": "E001",
                "message": "missing mandatory segment UNH",
                "field_path": "UNH",
                "segment_position": 0
            },
            {
                "severity": "Warning",
                "category": "condition",
                "code": "W002",
                "message": "condition [1] not met",
                "field_path": "SG4.IDE.0",
                "segment_position": 5
            },
            {
                "severity": "Info",
                "category": "hint",
                "code": "I003",
                "message": "optional group SG12 omitted",
                "field_path": null,
                "segment_position": null
            }
        ]
    });

    let entries = extract_validation_issues(&report);
    assert_eq!(entries.len(), 3);

    assert_eq!(entries[0].severity, "error");
    assert_eq!(entries[0].code, "E001");
    assert_eq!(entries[0].message, "missing mandatory segment UNH");
    assert_eq!(entries[0].location, Some("UNH".to_string()));

    assert_eq!(entries[1].severity, "warning");
    assert_eq!(entries[1].code, "W002");
    assert_eq!(entries[1].message, "condition [1] not met");
    assert_eq!(entries[1].location, Some("SG4.IDE.0".to_string()));

    assert_eq!(entries[2].severity, "info");
    assert_eq!(entries[2].code, "I003");
    assert_eq!(entries[2].message, "optional group SG12 omitted");
    assert_eq!(entries[2].location, None);
}

#[test]
fn test_response_generation_options_with_explicit_type() {
    let opts = ResponseGenerationOptions {
        response_type: Some("aperak".to_string()),
        format: Some("edifact".to_string()),
    };

    let json = serde_json::to_value(&opts).unwrap();
    assert_eq!(json["response_type"], "aperak");
    assert_eq!(json["format"], "edifact");
}

#[test]
fn test_response_generation_options_auto_detect() {
    let opts = ResponseGenerationOptions {
        response_type: None,
        format: Some("bo4e".to_string()),
    };

    let json = serde_json::to_value(&opts).unwrap();
    assert!(json.get("response_type").is_none());
    assert_eq!(json["format"], "bo4e");
}

#[test]
fn test_validate_v2_request_with_generate_response() {
    let req = ValidateV2Request {
        input: "UNH+1+UTILMD'".to_string(),
        format_version: "FV2504".to_string(),
        level: "full".to_string(),
        generate_response: Some(ResponseGenerationOptions {
            response_type: Some("contrl".to_string()),
            format: Some("bo4e".to_string()),
        }),
    };

    let json = serde_json::to_value(&req).unwrap();
    assert!(json.get("generate_response").is_some());
    assert_eq!(json["generate_response"]["response_type"], "contrl");
    assert_eq!(json["generate_response"]["format"], "bo4e");
}

#[test]
fn test_validate_v2_request_without_generate_response() {
    let req = ValidateV2Request {
        input: "UNH+1+UTILMD'".to_string(),
        format_version: "FV2504".to_string(),
        ..Default::default()
    };

    let json = serde_json::to_value(&req).unwrap();
    assert!(json.get("generate_response").is_none());
}

#[test]
fn test_validate_v2_response_with_response_message() {
    let json = r#"{
        "report": {"valid": true, "issues": []},
        "duration_ms": 5.0,
        "response_message": {
            "message_type": "APERAK",
            "bo4e": {"typ": "Marktlokation"},
            "edifact": null
        }
    }"#;

    let resp: ValidateV2Response = serde_json::from_str(json).unwrap();
    assert!(resp.response_message.is_some());
    let msg = resp.response_message.unwrap();
    assert_eq!(msg.message_type, "APERAK");
    assert!(msg.bo4e.is_some());
    assert!(msg.edifact.is_none());
}

#[test]
fn test_validate_v2_response_without_response_message() {
    let json = r#"{
        "report": {"valid": true, "issues": []},
        "duration_ms": 3.0
    }"#;

    let resp: ValidateV2Response = serde_json::from_str(json).unwrap();
    assert!(resp.response_message.is_none());
}

#[test]
fn test_generated_response_payload_with_edifact() {
    let json = r#"{
        "message_type": "CONTRL",
        "bo4e": null,
        "edifact": "UNA:+.? '\nUNB+UNOC:3+recv+send+260302:1200+resp1'\nUNH+1+CONTRL:D:3:UN'\nUNT+2+1'\nUNZ+1+resp1'"
    }"#;

    let payload: GeneratedResponsePayload = serde_json::from_str(json).unwrap();
    assert_eq!(payload.message_type, "CONTRL");
    assert!(payload.bo4e.is_none());
    assert!(payload.edifact.is_some());
    assert!(payload.edifact.unwrap().contains("CONTRL"));
}

#[test]
fn test_extract_validation_issues_empty_report() {
    use automapper_web::types::extract_validation_issues;

    let report = serde_json::json!({
        "valid": true,
        "issues": []
    });

    let entries = extract_validation_issues(&report);
    assert!(entries.is_empty());
}
