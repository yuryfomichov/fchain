use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
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
        let body = Json(ErrorResponse {
            error: self.to_string(),
        });
        (status, body).into_response()
    }
}
