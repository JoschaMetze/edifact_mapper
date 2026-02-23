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
