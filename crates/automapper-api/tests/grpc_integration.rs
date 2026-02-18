//! Integration tests for gRPC services.
//!
//! Spins up a real TCP server and connects a tonic client.

use std::net::SocketAddr;

use tokio::net::TcpListener;

use automapper_api::grpc::inspection_proto::inspection_service_client::InspectionServiceClient;
use automapper_api::grpc::inspection_proto::{InspectEdifactRequest, ListCoordinatorsRequest};
use automapper_api::grpc::transform_proto::transform_service_client::TransformServiceClient;
use automapper_api::grpc::transform_proto::EdifactToBo4eRequest;
use automapper_api::state::AppState;

/// Start a test server and return the address.
async fn start_test_server() -> SocketAddr {
    let state = AppState::new();
    let app = automapper_api::build_router(state);

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    // Give the server a moment to start
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    addr
}

#[tokio::test]
async fn test_grpc_list_coordinators() {
    let addr = start_test_server().await;
    let url = format!("http://{addr}");

    let mut client = InspectionServiceClient::connect(url).await.unwrap();

    let response = client
        .list_coordinators(ListCoordinatorsRequest {})
        .await
        .unwrap();

    let coordinators = response.into_inner().coordinators;
    assert!(!coordinators.is_empty());
    assert!(coordinators.iter().any(|c| c.message_type == "UTILMD"));
}

#[tokio::test]
async fn test_grpc_inspect_edifact() {
    let addr = start_test_server().await;
    let url = format!("http://{addr}");

    let mut client = InspectionServiceClient::connect(url).await.unwrap();

    let response = client
        .inspect_edifact(InspectEdifactRequest {
            edifact: "UNH+1+UTILMD:D:11A:UN:5.2e'BGM+E01+DOC001'UNT+3+1'".to_string(),
        })
        .await
        .unwrap();

    let inner = response.into_inner();
    assert_eq!(inner.segment_count, 3);
    assert_eq!(inner.message_type, "UTILMD");
    assert_eq!(inner.segments[0].tag, "UNH");
    assert_eq!(inner.segments[1].tag, "BGM");
    assert_eq!(inner.segments[2].tag, "UNT");
}

#[tokio::test]
async fn test_grpc_convert_edifact_to_bo4e() {
    let addr = start_test_server().await;
    let url = format!("http://{addr}");

    let mut client = TransformServiceClient::connect(url).await.unwrap();

    let edifact = concat!(
        "UNB+UNOC:3+sender+receiver+231215:1200+ref001'",
        "UNH+1+UTILMD:D:11A:UN:5.2e'",
        "BGM+E01+DOC001'",
        "UNT+3+1'",
        "UNZ+1+ref001'"
    );

    let response = client
        .convert_edifact_to_bo4e(EdifactToBo4eRequest {
            edifact: edifact.to_string(),
            format_version: "FV2504".to_string(),
            include_trace: false,
        })
        .await
        .unwrap();

    let inner = response.into_inner();
    // The response should be well-formed (success or conversion error, not gRPC error)
    assert!(inner.duration_ms >= 0.0);
}

#[tokio::test]
async fn test_grpc_inspect_empty_edifact_returns_error() {
    let addr = start_test_server().await;
    let url = format!("http://{addr}");

    let mut client = InspectionServiceClient::connect(url).await.unwrap();

    let result = client
        .inspect_edifact(InspectEdifactRequest {
            edifact: String::new(),
        })
        .await;

    // Empty EDIFACT should return a gRPC error (INVALID_ARGUMENT)
    assert!(result.is_err());
    let status = result.unwrap_err();
    assert_eq!(status.code(), tonic::Code::InvalidArgument);
}
