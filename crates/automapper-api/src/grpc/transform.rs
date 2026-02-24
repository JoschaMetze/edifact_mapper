//! gRPC TransformService implementation.
//!
//! Legacy pipeline has been removed. All transform methods return UNIMPLEMENTED
//! status, directing callers to use the v2 REST API instead.

use std::pin::Pin;

use tokio_stream::Stream;
use tonic::{Request, Response, Status, Streaming};

use crate::grpc::transform_proto::transform_service_server::TransformService;
use crate::grpc::transform_proto::{
    Bo4eToEdifactRequest, ConvertResponse as ProtoConvertResponse, EdifactToBo4eRequest,
};

/// gRPC implementation of TransformService.
///
/// All methods currently return `UNIMPLEMENTED` — callers should use the v2 REST API.
pub struct TransformServiceImpl;

impl TransformServiceImpl {
    pub fn new(_registry: std::sync::Arc<crate::state::CoordinatorRegistry>) -> Self {
        Self
    }
}

#[tonic::async_trait]
impl TransformService for TransformServiceImpl {
    async fn convert_edifact_to_bo4e(
        &self,
        _request: Request<EdifactToBo4eRequest>,
    ) -> Result<Response<ProtoConvertResponse>, Status> {
        Err(Status::unimplemented(
            "Legacy pipeline removed, use v2 REST API",
        ))
    }

    async fn convert_bo4e_to_edifact(
        &self,
        _request: Request<Bo4eToEdifactRequest>,
    ) -> Result<Response<ProtoConvertResponse>, Status> {
        Err(Status::unimplemented(
            "Legacy pipeline removed, use v2 REST API",
        ))
    }

    type ConvertEdifactToBo4eStreamStream =
        Pin<Box<dyn Stream<Item = Result<ProtoConvertResponse, Status>> + Send>>;

    async fn convert_edifact_to_bo4e_stream(
        &self,
        _request: Request<Streaming<EdifactToBo4eRequest>>,
    ) -> Result<Response<Self::ConvertEdifactToBo4eStreamStream>, Status> {
        Err(Status::unimplemented(
            "Legacy pipeline removed, use v2 REST API",
        ))
    }
}
