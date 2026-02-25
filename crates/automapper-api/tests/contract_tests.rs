//! Tests that verify JSON serialization of API contracts.
//!
//! These tests ensure the API contract is stable — any accidental field rename
//! or type change will break these tests.

use std::collections::HashMap;

use automapper_api::contracts::coordinators::CoordinatorInfo;
use automapper_api::contracts::error::ErrorSeverity;
use automapper_api::contracts::health::HealthResponse;
use automapper_api::contracts::inspect::{
    ComponentElement, DataElement, InspectRequest, InspectResponse, SegmentNode,
};
use automapper_api::contracts::validate_v2::{ValidateV2Request, ValidateV2Response};

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

// --- Validate V2 contract tests ---

#[test]
fn test_validate_request_deserialization() {
    let json = r#"{
        "input": "UNH+1+UTILMD'",
        "format_version": "FV2504"
    }"#;
    let req: ValidateV2Request = serde_json::from_str(json).unwrap();
    assert_eq!(req.input, "UNH+1+UTILMD'");
    assert_eq!(req.format_version, "FV2504");
    assert_eq!(req.level, "full"); // default
    assert!(req.external_conditions.is_none());
}

#[test]
fn test_validate_request_with_all_fields() {
    let json = r#"{
        "input": "UNH+1+UTILMD'",
        "format_version": "FV2504",
        "level": "structure",
        "external_conditions": {"DateKnown": true, "Splitting": false}
    }"#;
    let req: ValidateV2Request = serde_json::from_str(json).unwrap();
    assert_eq!(req.level, "structure");
    let ext = req.external_conditions.unwrap();
    assert_eq!(ext.get("DateKnown"), Some(&true));
    assert_eq!(ext.get("Splitting"), Some(&false));
}

#[test]
fn test_validate_response_serialization() {
    let resp = ValidateV2Response {
        report: serde_json::json!({
            "message_type": "UTILMD",
            "level": "Full",
            "issues": []
        }),
        duration_ms: 1.23,
    };

    let json = serde_json::to_value(&resp).unwrap();
    assert_eq!(json["report"]["message_type"], "UTILMD");
    assert_eq!(json["report"]["level"], "Full");
    assert_eq!(json["duration_ms"], 1.23);
}

#[test]
fn test_validate_request_level_defaults_to_full() {
    // Ensure the serde default works
    let json = r#"{"input": "x", "format_version": "FV2504"}"#;
    let req: ValidateV2Request = serde_json::from_str(json).unwrap();
    assert_eq!(req.level, "full");
}

#[test]
fn test_validate_request_external_conditions_hashmap() {
    let mut conditions = HashMap::new();
    conditions.insert("Cond1".to_string(), true);
    conditions.insert("Cond2".to_string(), false);

    let json = serde_json::json!({
        "input": "test",
        "format_version": "FV2504",
        "external_conditions": conditions
    });

    let req: ValidateV2Request = serde_json::from_str(&json.to_string()).unwrap();
    let ext = req.external_conditions.unwrap();
    assert_eq!(ext.len(), 2);
    assert_eq!(ext["Cond1"], true);
    assert_eq!(ext["Cond2"], false);
}
