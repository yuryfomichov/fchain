use axum::{extract::State, Json};
use log::{error, info};
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
    info!("GET /chain/validate - Validating blockchain");

    let blockchain = blockchain.lock().unwrap();
    match blockchain.is_chain_valid() {
        Ok(_) => {
            info!("GET /chain/validate - Blockchain is valid, returning status 200");
            Ok(Json(ValidateChainResponse {
                valid: true,
                message: "Blockchain is valid".to_string(),
            }))
        }
        Err(err) => {
            error!(
                "GET /chain/validate - Blockchain validation failed: {}",
                err
            );
            Err(err)
        }
    }
}
