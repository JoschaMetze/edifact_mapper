//! gRPC service implementations.
//!
//! Generated protobuf types are included via `tonic::include_proto!`.

pub mod inspection;
pub mod transform;

/// Generated protobuf types for the transform service.
pub mod transform_proto {
    tonic::include_proto!("automapper.transform");
}

/// Generated protobuf types for the inspection service.
pub mod inspection_proto {
    tonic::include_proto!("automapper.inspection");
}
