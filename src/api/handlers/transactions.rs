use axum::{extract::State, Json};
use log::{error, info};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::blockchain::crypto::{PublicKeyHex, TransactionSignature};
use crate::blockchain::{Address, BlockchainError, SharedBlockchain, Transaction};

/// Request to create a new transaction
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateTransactionRequest {
    /// The sender's address
    pub sender: String,
    /// The recipient address
    pub recipient: String,
    /// The amount to transfer
    pub amount: f64,
    /// The transaction signature (required)
    pub signature: String,
    /// The full public key of the sender (required for non-system transactions)
    pub public_key: Option<String>,
}

/// Response for a successful transaction creation
#[derive(Debug, Serialize, ToSchema)]
pub struct CreateTransactionResponse {
    /// Success message
    pub message: String,
    /// The created transaction
    pub transaction: Transaction,
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
    info!("GET /transactions/pending - Retrieving pending transactions");

    let blockchain = blockchain.lock().unwrap();
    let transactions = blockchain.pending_transactions.clone();

    info!(
        "GET /transactions/pending - Returning {} pending transactions with status 200",
        transactions.len()
    );
    Json(transactions)
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
    info!(
        "POST /transactions - Creating transaction from {} to {} for amount {}",
        request.sender, request.recipient, request.amount
    );

    // Create transaction with the provided data
    let sender = Address(request.sender.clone());
    let recipient = Address(request.recipient.clone());

    // Create the transaction
    let mut transaction = Transaction::new(sender, recipient, request.amount);

    // Special handling for system transactions
    if request.sender == "system" {
        info!("POST /transactions - Processing system transaction");
        // System transactions don't need signature validation or balance checks
        transaction.signature = Some(TransactionSignature("system".to_string()));
    } else {
        info!(
            "POST /transactions - Processing regular transaction, validating signature and balance"
        );
        // For regular transactions, we need both signature and public key
        let signature = request.signature;

        let public_key = match request.public_key {
            Some(pk) => pk,
            None => {
                let err_msg = "Non-system transactions require a public key";
                error!("POST /transactions - Validation failed: {}", err_msg);
                return Err(BlockchainError::InvalidTransaction(err_msg.to_string()));
            }
        };

        // Add the signature and public key from the external wallet
        transaction.signature = Some(TransactionSignature(signature));
        transaction.public_key = Some(PublicKeyHex(public_key));

        // Validate the transaction
        if !transaction.is_valid() {
            let err_msg = "Transaction validation failed";
            error!("POST /transactions - Validation failed: {}", err_msg);
            return Err(BlockchainError::InvalidTransaction(err_msg.to_string()));
        }

        // Check if sender has sufficient balance
        let chain = blockchain.lock().unwrap();
        let balance = chain.get_balance(&request.sender);
        if balance < request.amount {
            let err_msg = format!(
                "Insufficient balance: {} has only {} coins",
                request.sender, balance
            );
            error!("POST /transactions - {}", err_msg);
            return Err(BlockchainError::InvalidTransaction(err_msg));
        }
        // Release the lock before proceeding
        drop(chain);
    }

    // Add the transaction to the blockchain
    let mut chain = blockchain.lock().unwrap();
    match chain.create_transaction(transaction.clone()) {
        Ok(_) => {
            info!("POST /transactions - Transaction created successfully with status 200");
            Ok(Json(CreateTransactionResponse {
                message: "Transaction created successfully".to_string(),
                transaction,
            }))
        }
        Err(err) => {
            error!("POST /transactions - Failed to create transaction: {}", err);
            Err(err)
        }
    }
}
