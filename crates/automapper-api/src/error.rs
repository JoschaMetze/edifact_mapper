//! API error types and Axum error response mapping.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

/// API-level error that converts to an Axum response.
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("bad request: {message}")]
    BadRequest { message: String },

    #[error("not found: {message}")]
    NotFound { message: String },

    #[error("conversion error: {message}")]
    ConversionError { message: String },

    #[error("internal error: {message}")]
    Internal { message: String },
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_code, message) = match &self {
            ApiError::BadRequest { message } => {
                (StatusCode::BAD_REQUEST, "BAD_REQUEST", message.clone())
            }
            ApiError::NotFound { message } => (StatusCode::NOT_FOUND, "NOT_FOUND", message.clone()),
            ApiError::ConversionError { message } => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "CONVERSION_ERROR",
                message.clone(),
            ),
            ApiError::Internal { message } => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                message.clone(),
            ),
        };

        let body = json!({
            "error": {
                "code": error_code,
                "message": message,
            }
        });

        (status, Json(body)).into_response()
    }
}
