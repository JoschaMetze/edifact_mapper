//! gRPC TransformService implementation.

use std::pin::Pin;
use std::sync::Arc;

use tokio_stream::Stream;
use tonic::{Request, Response, Status, Streaming};

use crate::grpc::transform_proto::transform_service_server::TransformService;
use crate::grpc::transform_proto::{
    Bo4eToEdifactRequest, ConversionError, ConvertResponse as ProtoConvertResponse,
    EdifactToBo4eRequest, ErrorSeverity, MappingStep, MappingTrace,
};
use crate::state::CoordinatorRegistry;

/// gRPC implementation of TransformService.
pub struct TransformServiceImpl {
    registry: Arc<CoordinatorRegistry>,
}

impl TransformServiceImpl {
    pub fn new(registry: Arc<CoordinatorRegistry>) -> Self {
        Self { registry }
    }

    fn build_response(
        &self,
        result: Result<crate::contracts::convert::ConvertResponse, crate::error::ApiError>,
    ) -> Result<Response<ProtoConvertResponse>, Status> {
        match result {
            Ok(resp) => {
                let trace = if resp.trace.is_empty() {
                    None
                } else {
                    Some(MappingTrace {
                        coordinator_used: resp
                            .trace
                            .first()
                            .map(|t| t.mapper.clone())
                            .unwrap_or_default(),
                        steps: resp
                            .trace
                            .iter()
                            .map(|t| MappingStep {
                                mapper: t.mapper.clone(),
                                source_segment: t.source_segment.clone(),
                                target_path: t.target_path.clone(),
                                value: t.value.clone().unwrap_or_default(),
                                note: t.note.clone().unwrap_or_default(),
                            })
                            .collect(),
                        duration_ms: resp.duration_ms,
                    })
                };

                let errors: Vec<ConversionError> = resp
                    .errors
                    .iter()
                    .map(|e| ConversionError {
                        code: e.code.clone(),
                        message: e.message.clone(),
                        location: e.location.clone().unwrap_or_default(),
                        severity: match e.severity {
                            crate::contracts::error::ErrorSeverity::Warning => {
                                ErrorSeverity::Warning as i32
                            }
                            crate::contracts::error::ErrorSeverity::Error => {
                                ErrorSeverity::Error as i32
                            }
                            crate::contracts::error::ErrorSeverity::Critical => {
                                ErrorSeverity::Critical as i32
                            }
                        },
                    })
                    .collect();

                Ok(Response::new(ProtoConvertResponse {
                    success: resp.success,
                    result: resp.result.unwrap_or_default(),
                    trace,
                    errors,
                    duration_ms: resp.duration_ms,
                }))
            }
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }
}

#[tonic::async_trait]
impl TransformService for TransformServiceImpl {
    async fn convert_edifact_to_bo4e(
        &self,
        request: Request<EdifactToBo4eRequest>,
    ) -> Result<Response<ProtoConvertResponse>, Status> {
        let req = request.into_inner();
        tracing::info!(
            "gRPC: Converting EDIFACT to BO4E, format_version={}",
            if req.format_version.is_empty() {
                "auto"
            } else {
                &req.format_version
            }
        );

        let fv = if req.format_version.is_empty() {
            None
        } else {
            Some(req.format_version.as_str())
        };

        let result = self
            .registry
            .convert_edifact_to_bo4e(&req.edifact, fv, req.include_trace);

        self.build_response(result)
    }

    async fn convert_bo4e_to_edifact(
        &self,
        request: Request<Bo4eToEdifactRequest>,
    ) -> Result<Response<ProtoConvertResponse>, Status> {
        let req = request.into_inner();
        tracing::info!(
            "gRPC: Converting BO4E to EDIFACT, message_type={}",
            req.message_type
        );

        let fv = if req.format_version.is_empty() {
            None
        } else {
            Some(req.format_version.as_str())
        };

        let result = self.registry.convert_bo4e_to_edifact(&req.bo4e_json, fv);

        self.build_response(result)
    }

    type ConvertEdifactToBo4eStreamStream =
        Pin<Box<dyn Stream<Item = Result<ProtoConvertResponse, Status>> + Send>>;

    async fn convert_edifact_to_bo4e_stream(
        &self,
        request: Request<Streaming<EdifactToBo4eRequest>>,
    ) -> Result<Response<Self::ConvertEdifactToBo4eStreamStream>, Status> {
        let registry = self.registry.clone();
        let mut stream = request.into_inner();

        let output = async_stream::try_stream! {
            while let Some(req) = stream.message().await? {
                let fv = if req.format_version.is_empty() {
                    None
                } else {
                    Some(req.format_version.as_str())
                };

                let result = registry.convert_edifact_to_bo4e(
                    &req.edifact,
                    fv,
                    req.include_trace,
                );

                match result {
                    Ok(resp) => {
                        yield ProtoConvertResponse {
                            success: resp.success,
                            result: resp.result.unwrap_or_default(),
                            trace: None,
                            errors: vec![],
                            duration_ms: resp.duration_ms,
                        };
                    }
                    Err(e) => {
                        yield ProtoConvertResponse {
                            success: false,
                            result: String::new(),
                            trace: None,
                            errors: vec![ConversionError {
                                code: "CONVERSION_ERROR".to_string(),
                                message: e.to_string(),
                                location: String::new(),
                                severity: ErrorSeverity::Error as i32,
                            }],
                            duration_ms: 0.0,
                        };
                    }
                }
            }
        };

        Ok(Response::new(Box::pin(output)))
    }
}
