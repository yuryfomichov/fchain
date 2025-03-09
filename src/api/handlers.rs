use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::blockchain::{
    Address, Block, BlockchainError, SharedBlockchain, Transaction, TransactionSignature,
};

/// Error response for the API
#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
}

/// Convert BlockchainError to an API response
impl IntoResponse for BlockchainError {
    fn into_response(self) -> Response {
        let status = match self {
            BlockchainError::InvalidTransaction(_) => StatusCode::BAD_REQUEST,
            BlockchainError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let body = Json(ErrorResponse {
            error: self.to_string(),
        });

        (status, body).into_response()
    }
}

/// Request to create a transaction
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateTransactionRequest {
    /// The sender's address
    pub sender: String,
    /// The recipient address
    pub recipient: String,
    /// The amount to transfer
    pub amount: f64,
    /// The transaction signature (optional, for signed transactions)
    pub signature: Option<String>,
}

/// Response for a successful transaction creation
#[derive(Debug, Serialize, ToSchema)]
pub struct CreateTransactionResponse {
    /// Success message
    pub message: String,
    /// The created transaction
    pub transaction: Transaction,
}

/// Request to mine a new block
#[derive(Debug, Deserialize, ToSchema)]
pub struct MineBlockRequest {
    /// The address where mining rewards should be sent
    pub miner_address: String,
}

/// Response for a successful block mining
#[derive(Debug, Serialize, ToSchema)]
pub struct MineBlockResponse {
    /// Success message
    pub message: String,
    /// The mined block
    pub block: Block,
}

/// Response for chain validation
#[derive(Debug, Serialize, ToSchema)]
pub struct ValidateChainResponse {
    /// Whether the chain is valid
    pub valid: bool,
    /// Additional information about the validation
    pub message: String,
}

/// Get all blocks in the chain
#[utoipa::path(
    get,
    path = "/blocks",
    tag = "Blockchain",
    responses(
        (status = 200, description = "List of all blocks in the chain", body = Vec<Block>)
    )
)]
pub async fn get_blocks(State(blockchain): State<SharedBlockchain>) -> Json<Vec<Block>> {
    let blockchain = blockchain.lock().unwrap();
    Json(blockchain.chain.clone())
}

/// Mine a new block
#[utoipa::path(
    post,
    path = "/blocks/mine",
    tag = "Blockchain",
    request_body = MineBlockRequest,
    responses(
        (status = 200, description = "Block mined successfully", body = MineBlockResponse),
        (status = 400, description = "Mining failed", body = ErrorResponse)
    )
)]
pub async fn mine_block(
    State(blockchain): State<SharedBlockchain>,
    Json(request): Json<MineBlockRequest>,
) -> Result<Json<MineBlockResponse>, BlockchainError> {
    let mut blockchain = blockchain.lock().unwrap();
    let block = blockchain.mine_pending_transactions(&request.miner_address)?;

    Ok(Json(MineBlockResponse {
        message: "Block mined successfully".to_string(),
        block,
    }))
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

/// Get pending transactions
#[utoipa::path(
    get,
    path = "/transactions/pending",
    tag = "Blockchain",
    responses(
        (status = 200, description = "List of pending transactions", body = Vec<Transaction>)
    )
)]
pub async fn get_pending_transactions(
    State(blockchain): State<SharedBlockchain>,
) -> Json<Vec<Transaction>> {
    let blockchain = blockchain.lock().unwrap();
    Json(blockchain.pending_transactions.clone())
}

/// Creates a transaction
#[utoipa::path(
    post,
    path = "/transactions",
    request_body = CreateTransactionRequest,
    responses(
        (status = 200, description = "Transaction created successfully", body = CreateTransactionResponse),
        (status = 400, description = "Invalid transaction", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
pub async fn create_transaction(
    State(blockchain): State<SharedBlockchain>,
    Json(request): Json<CreateTransactionRequest>,
) -> Result<Json<CreateTransactionResponse>, BlockchainError> {
    // Create transaction with the provided data
    let sender = Address(request.sender.clone());
    let recipient = Address(request.recipient);

    // Create the transaction
    let mut transaction = Transaction::new(sender, recipient, request.amount);

    // Special handling for system transactions
    if request.sender == "system" {
        // System transactions don't need signature validation or balance checks
        transaction.signature = Some(TransactionSignature("system".to_string()));
    } else if let Some(signature) = request.signature {
        // Add the signature from the external wallet
        transaction.signature = Some(TransactionSignature(signature));

        // Validate the transaction
        if !transaction.is_valid() {
            return Err(BlockchainError::InvalidTransaction(
                "Transaction signature verification failed".to_string(),
            ));
        }

        // Check if sender has sufficient balance
        let chain = blockchain.lock().unwrap();
        let balance = chain.get_balance(&request.sender);
        if balance < request.amount {
            return Err(BlockchainError::InvalidTransaction(format!(
                "Insufficient balance: {} has only {} coins",
                request.sender, balance
            )));
        }
        // Release the lock before proceeding
        drop(chain);
    } else {
        return Err(BlockchainError::InvalidTransaction(
            "Non-system transactions require a signature".to_string(),
        ));
    }

    // Add the transaction to the blockchain
    let mut chain = blockchain.lock().unwrap();
    chain.create_transaction(transaction.clone())?;

    Ok(Json(CreateTransactionResponse {
        message: "Transaction created successfully".to_string(),
        transaction,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blockchain::chain::Blockchain;
    use axum::extract::State;
    use axum::Json;
    use std::sync::{Arc, Mutex};
    use tokio_test::block_on;

    #[test]
    fn test_transaction_flow() {
        // Create a blockchain
        let blockchain = Arc::new(Mutex::new(Blockchain::new(2, 100.0)));
        let state = State(blockchain.clone());

        // Create a transaction
        let tx_request = CreateTransactionRequest {
            sender: "system".to_string(),
            recipient: "recipient".to_string(),
            amount: 10.0,
            signature: None,
        };

        let result = block_on(create_transaction(state.clone(), Json(tx_request))).unwrap();
        let tx_response = result.0;
        assert!(tx_response.message.contains("created successfully"));

        // Mine a block
        let mine_request = MineBlockRequest {
            miner_address: "miner".to_string(),
        };
        let result = block_on(mine_block(state.clone(), Json(mine_request))).unwrap();
        let mine_response = result.0;
        assert!(mine_response.message.contains("mined successfully"));

        // Validate the chain
        let result = block_on(validate_chain(state.clone())).unwrap();
        let validate_response = result.0;
        assert!(validate_response.valid);
    }
}
