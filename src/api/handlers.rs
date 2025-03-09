use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};

use crate::blockchain::{
    Address, Block, BlockchainError, SharedBlockchain, Transaction, TransactionSignature,
};

/// Error response for the API
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

/// Convert BlockchainError to an API response
impl IntoResponse for BlockchainError {
    fn into_response(self) -> Response {
        let status = StatusCode::BAD_REQUEST;
        let body = Json(ErrorResponse {
            error: self.to_string(),
        });

        (status, body).into_response()
    }
}

/// Request to create a new transaction
#[derive(Debug, Deserialize)]
pub struct CreateTransactionRequest {
    pub sender: String,
    pub recipient: String,
    pub amount: f64,
    pub signature: Option<String>,
}

/// Response for a successful transaction creation
#[derive(Debug, Serialize)]
pub struct CreateTransactionResponse {
    pub message: String,
    pub transaction: Transaction,
}

/// Request to mine a new block
#[derive(Debug, Deserialize)]
pub struct MineBlockRequest {
    pub miner_address: String,
}

/// Response for a successful block mining
#[derive(Debug, Serialize)]
pub struct MineBlockResponse {
    pub message: String,
    pub block: Block,
}

/// Response for chain validation
#[derive(Debug, Serialize)]
pub struct ValidateChainResponse {
    pub valid: bool,
    pub message: String,
}

/// Get all blocks in the chain
pub async fn get_blocks(State(blockchain): State<SharedBlockchain>) -> Json<Vec<Block>> {
    let blockchain = blockchain.lock().unwrap();
    Json(blockchain.chain.clone())
}

/// Create a new transaction
pub async fn create_transaction(
    State(blockchain): State<SharedBlockchain>,
    Json(request): Json<CreateTransactionRequest>,
) -> Result<Json<CreateTransactionResponse>, BlockchainError> {
    let mut transaction = Transaction::new(
        Address(request.sender.clone()),
        Address(request.recipient),
        request.amount,
    );

    // If it's a system transaction (mining reward), it doesn't need a signature
    if request.sender != "system" {
        // For non-system transactions, a signature is required for validation
        if let Some(signature_str) = request.signature {
            transaction.signature = Some(TransactionSignature(signature_str));
        }
    }

    // Validate the transaction
    if !transaction.is_valid() {
        return Err(BlockchainError::InvalidTransaction(
            "Transaction is not valid".to_string(),
        ));
    }

    let mut blockchain = blockchain.lock().unwrap();
    blockchain.create_transaction(transaction.clone())?;

    Ok(Json(CreateTransactionResponse {
        message: "Transaction created successfully".to_string(),
        transaction,
    }))
}

/// Mine a new block
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
pub async fn get_pending_transactions(
    State(blockchain): State<SharedBlockchain>,
) -> Json<Vec<Transaction>> {
    let blockchain = blockchain.lock().unwrap();
    Json(blockchain.pending_transactions.clone())
}
