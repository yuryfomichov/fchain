use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use log::error;
use serde::Serialize;
use utoipa::ToSchema;

use crate::blockchain::BlockchainError;

/// Error response for the API
#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
}

/// Convert BlockchainError to an HTTP response
impl IntoResponse for BlockchainError {
    fn into_response(self) -> Response {
        let status = StatusCode::BAD_REQUEST;
        let error_message = self.to_string();

        error!(
            "Error response with status {}: {}",
            status.as_u16(),
            error_message
        );

        let body = Json(ErrorResponse {
            error: error_message,
        });
        (status, body).into_response()
    }
}
