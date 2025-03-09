use axum::{extract::State, Json};
use serde::Serialize;
use utoipa::ToSchema;

use crate::blockchain::{BlockchainError, SharedBlockchain};

/// Response for chain validation
#[derive(Debug, Serialize, ToSchema)]
pub struct ValidateChainResponse {
    /// Whether the chain is valid
    pub valid: bool,
    /// Additional information about the validation
    pub message: String,
}

/// Validate the blockchain
#[utoipa::path(
    get,
    path = "/chain/validate",
    tag = "Blockchain",
    responses(
        (status = 200, description = "Chain validation result", body = ValidateChainResponse),
        (status = 400, description = "Validation failed", body = ErrorResponse)
    )
)]
pub async fn validate_chain(
    State(blockchain): State<SharedBlockchain>,
) -> Result<Json<ValidateChainResponse>, BlockchainError> {
    let blockchain = blockchain.lock().unwrap();
    blockchain.is_chain_valid()?;

    Ok(Json(ValidateChainResponse {
        valid: true,
        message: "Blockchain is valid".to_string(),
    }))
}
