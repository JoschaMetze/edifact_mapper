//! Tests that verify JSON serialization of API contracts.
//!
//! These tests ensure the API contract is stable — any accidental field rename
//! or type change will break these tests.

use automapper_api::contracts::convert::{ConvertRequest, ConvertResponse};
use automapper_api::contracts::coordinators::CoordinatorInfo;
use automapper_api::contracts::error::{ApiErrorEntry, ErrorSeverity};
use automapper_api::contracts::health::HealthResponse;
use automapper_api::contracts::inspect::{
    ComponentElement, DataElement, InspectRequest, InspectResponse, SegmentNode,
};
use automapper_api::contracts::trace::TraceEntry;

#[test]
fn test_convert_request_deserialization() {
    let json = r#"{
        "content": "UNH+1+UTILMD:D:11A:UN:5.2e'",
        "format_version": "FV2504",
        "include_trace": true
    }"#;

    let req: ConvertRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.content, "UNH+1+UTILMD:D:11A:UN:5.2e'");
    assert_eq!(req.format_version, Some("FV2504".to_string()));
    assert!(req.include_trace);
}

#[test]
fn test_convert_request_defaults() {
    let json = r#"{ "content": "hello" }"#;

    let req: ConvertRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.content, "hello");
    assert_eq!(req.format_version, None);
    assert!(!req.include_trace);
}

#[test]
fn test_convert_response_serialization() {
    let resp = ConvertResponse {
        success: true,
        result: Some("{}".to_string()),
        trace: vec![TraceEntry {
            mapper: "UtilmdCoordinator".to_string(),
            source_segment: "UNH".to_string(),
            target_path: "transactions".to_string(),
            value: Some("1".to_string()),
            note: None,
        }],
        errors: vec![],
        duration_ms: 42.5,
    };

    let json = serde_json::to_value(&resp).unwrap();
    assert_eq!(json["success"], true);
    assert_eq!(json["result"], "{}");
    assert_eq!(json["duration_ms"], 42.5);
    assert_eq!(json["trace"][0]["mapper"], "UtilmdCoordinator");
    assert!(json["errors"].as_array().unwrap().is_empty());
}

#[test]
fn test_convert_response_with_errors() {
    let resp = ConvertResponse {
        success: false,
        result: None,
        trace: vec![],
        errors: vec![ApiErrorEntry {
            code: "PARSE_ERROR".to_string(),
            message: "unterminated segment at byte 42".to_string(),
            location: Some("byte 42".to_string()),
            severity: ErrorSeverity::Error,
        }],
        duration_ms: 1.2,
    };

    let json = serde_json::to_value(&resp).unwrap();
    assert_eq!(json["success"], false);
    assert!(json["result"].is_null());
    assert_eq!(json["errors"][0]["code"], "PARSE_ERROR");
    assert_eq!(json["errors"][0]["severity"], "error");
}

#[test]
fn test_inspect_request_deserialization() {
    let json = r#"{ "edifact": "UNH+1+UTILMD'" }"#;
    let req: InspectRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.edifact, "UNH+1+UTILMD'");
}

#[test]
fn test_inspect_response_serialization() {
    let resp = InspectResponse {
        segments: vec![SegmentNode {
            tag: "UNH".to_string(),
            line_number: 1,
            raw_content: "UNH+1+UTILMD:D:11A:UN".to_string(),
            elements: vec![
                DataElement {
                    position: 1,
                    value: Some("1".to_string()),
                    components: None,
                },
                DataElement {
                    position: 2,
                    value: None,
                    components: Some(vec![
                        ComponentElement {
                            position: 1,
                            value: Some("UTILMD".to_string()),
                        },
                        ComponentElement {
                            position: 2,
                            value: Some("D".to_string()),
                        },
                        ComponentElement {
                            position: 3,
                            value: Some("11A".to_string()),
                        },
                        ComponentElement {
                            position: 4,
                            value: Some("UN".to_string()),
                        },
                    ]),
                },
            ],
            children: None,
        }],
        segment_count: 1,
        message_type: Some("UTILMD".to_string()),
        format_version: None,
    };

    let json = serde_json::to_value(&resp).unwrap();
    assert_eq!(json["segment_count"], 1);
    assert_eq!(json["segments"][0]["tag"], "UNH");
    assert_eq!(
        json["segments"][0]["elements"][1]["components"][0]["value"],
        "UTILMD"
    );
}

#[test]
fn test_coordinator_info_serialization() {
    let info = CoordinatorInfo {
        message_type: "UTILMD".to_string(),
        description: "UTILMD coordinator".to_string(),
        supported_versions: vec!["FV2504".to_string(), "FV2510".to_string()],
    };

    let json = serde_json::to_value(&info).unwrap();
    assert_eq!(json["message_type"], "UTILMD");
    assert_eq!(json["supported_versions"][0], "FV2504");
    assert_eq!(json["supported_versions"][1], "FV2510");
}

#[test]
fn test_health_response_serialization() {
    let resp = HealthResponse {
        healthy: true,
        version: "0.1.0".to_string(),
        available_coordinators: vec!["UTILMD".to_string()],
        uptime_seconds: 123.456,
    };

    let json = serde_json::to_value(&resp).unwrap();
    assert_eq!(json["healthy"], true);
    assert_eq!(json["version"], "0.1.0");
    assert_eq!(json["uptime_seconds"], 123.456);
}

#[test]
fn test_error_severity_serialization() {
    assert_eq!(
        serde_json::to_string(&ErrorSeverity::Warning).unwrap(),
        r#""warning""#
    );
    assert_eq!(
        serde_json::to_string(&ErrorSeverity::Error).unwrap(),
        r#""error""#
    );
    assert_eq!(
        serde_json::to_string(&ErrorSeverity::Critical).unwrap(),
        r#""critical""#
    );
}

#[test]
fn test_error_severity_deserialization() {
    let w: ErrorSeverity = serde_json::from_str(r#""warning""#).unwrap();
    assert_eq!(w, ErrorSeverity::Warning);

    let e: ErrorSeverity = serde_json::from_str(r#""error""#).unwrap();
    assert_eq!(e, ErrorSeverity::Error);

    let c: ErrorSeverity = serde_json::from_str(r#""critical""#).unwrap();
    assert_eq!(c, ErrorSeverity::Critical);
}
