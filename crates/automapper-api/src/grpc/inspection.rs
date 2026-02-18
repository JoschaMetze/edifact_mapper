//! gRPC InspectionService implementation.

use std::sync::Arc;

use tonic::{Request, Response, Status};

use crate::grpc::inspection_proto::inspection_service_server::InspectionService;
use crate::grpc::inspection_proto::{
    ComponentElement as ProtoComponentElement, CoordinatorInfo as ProtoCoordinatorInfo,
    DataElement as ProtoDataElement, InspectEdifactRequest, InspectEdifactResponse,
    ListCoordinatorsRequest, ListCoordinatorsResponse, SegmentNode as ProtoSegmentNode,
};
use crate::state::CoordinatorRegistry;

/// gRPC implementation of InspectionService.
pub struct InspectionServiceImpl {
    registry: Arc<CoordinatorRegistry>,
}

impl InspectionServiceImpl {
    pub fn new(registry: Arc<CoordinatorRegistry>) -> Self {
        Self { registry }
    }
}

#[tonic::async_trait]
impl InspectionService for InspectionServiceImpl {
    async fn inspect_edifact(
        &self,
        request: Request<InspectEdifactRequest>,
    ) -> Result<Response<InspectEdifactResponse>, Status> {
        let req = request.into_inner();
        tracing::info!("gRPC: Inspecting EDIFACT, length={}", req.edifact.len());

        let result = self
            .registry
            .inspect_edifact(&req.edifact)
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        let segments: Vec<ProtoSegmentNode> =
            result.segments.iter().map(segment_node_to_proto).collect();

        Ok(Response::new(InspectEdifactResponse {
            segments,
            segment_count: result.segment_count as u32,
            message_type: result.message_type.unwrap_or_default(),
            format_version: result.format_version.unwrap_or_default(),
        }))
    }

    async fn list_coordinators(
        &self,
        _request: Request<ListCoordinatorsRequest>,
    ) -> Result<Response<ListCoordinatorsResponse>, Status> {
        tracing::info!("gRPC: Listing coordinators");

        let coordinators: Vec<ProtoCoordinatorInfo> = self
            .registry
            .list()
            .iter()
            .map(|c| ProtoCoordinatorInfo {
                message_type: c.message_type.clone(),
                description: c.description.clone(),
                supported_versions: c.supported_versions.clone(),
            })
            .collect();

        Ok(Response::new(ListCoordinatorsResponse { coordinators }))
    }
}

/// Convert a REST `SegmentNode` to a proto `SegmentNode`.
fn segment_node_to_proto(node: &crate::contracts::inspect::SegmentNode) -> ProtoSegmentNode {
    let elements: Vec<ProtoDataElement> = node
        .elements
        .iter()
        .map(|e| ProtoDataElement {
            position: e.position,
            value: e.value.clone().unwrap_or_default(),
            components: e
                .components
                .as_ref()
                .map(|comps| {
                    comps
                        .iter()
                        .map(|c| ProtoComponentElement {
                            position: c.position,
                            value: c.value.clone().unwrap_or_default(),
                        })
                        .collect()
                })
                .unwrap_or_default(),
        })
        .collect();

    let children: Vec<ProtoSegmentNode> = node
        .children
        .as_ref()
        .map(|ch| ch.iter().map(segment_node_to_proto).collect())
        .unwrap_or_default();

    ProtoSegmentNode {
        tag: node.tag.clone(),
        line_number: node.line_number,
        raw_content: node.raw_content.clone(),
        elements,
        children,
    }
}
