use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::blockchain::wallet::Wallet;
use crate::blockchain::{Address, Block, BlockchainError, SharedBlockchain, Transaction};

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

/// Request to create a transaction (signed with private key)
#[derive(Debug, Deserialize, ToSchema)]
pub struct SignAndCreateTransactionRequest {
    /// The recipient address
    pub recipient: String,
    /// The amount to transfer
    pub amount: f64,
    /// The private key to sign with
    pub private_key: String,
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

/// Wallet-related request and response structures
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateWalletResponse {
    /// The wallet's address
    pub address: String,
    /// The wallet's public key
    pub public_key: String,
    /// The wallet's private key (keep this secret!)
    pub private_key: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ImportWalletRequest {
    /// The private key to import
    pub private_key: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ImportWalletResponse {
    /// The wallet's address
    pub address: String,
    /// The wallet's public key
    pub public_key: String,
    /// Success message
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ValidateAddressRequest {
    /// The address to validate
    pub address: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ValidateAddressResponse {
    /// Whether the address is valid
    pub is_valid: bool,
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
    path = "/mine",
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

/// Create a new wallet
#[utoipa::path(
    get,
    path = "/wallet/create",
    tag = "Wallet",
    responses(
        (status = 200, description = "Wallet created successfully", body = CreateWalletResponse),
        (status = 400, description = "Failed to create wallet", body = ErrorResponse)
    )
)]
pub async fn create_wallet() -> Result<Json<CreateWalletResponse>, BlockchainError> {
    let wallet = Wallet::new().map_err(|e| {
        BlockchainError::ValidationFailed(format!("Failed to create wallet: {}", e))
    })?;

    Ok(Json(CreateWalletResponse {
        address: wallet.get_address().0.clone(),
        public_key: wallet.get_public_key_hex(),
        private_key: wallet.get_secret_key_hex(),
    }))
}

/// Import a wallet from a private key
#[utoipa::path(
    post,
    path = "/wallet/import",
    tag = "Wallet",
    request_body = ImportWalletRequest,
    responses(
        (status = 200, description = "Wallet imported successfully", body = ImportWalletResponse),
        (status = 400, description = "Failed to import wallet", body = ErrorResponse)
    )
)]
pub async fn import_wallet(
    Json(request): Json<ImportWalletRequest>,
) -> Result<Json<ImportWalletResponse>, BlockchainError> {
    let wallet = Wallet::from_secret_key(&request.private_key).map_err(|e| {
        BlockchainError::ValidationFailed(format!("Failed to import wallet: {}", e))
    })?;

    Ok(Json(ImportWalletResponse {
        address: wallet.get_address().0.clone(),
        public_key: wallet.get_public_key_hex(),
        message: "Wallet imported successfully".to_string(),
    }))
}

/// Validate an address
#[utoipa::path(
    post,
    path = "/wallet/validate",
    tag = "Wallet",
    request_body = ValidateAddressRequest,
    responses(
        (status = 200, description = "Address validation result", body = ValidateAddressResponse)
    )
)]
pub async fn validate_address(
    Json(request): Json<ValidateAddressRequest>,
) -> Result<Json<ValidateAddressResponse>, BlockchainError> {
    let address = Address(request.address);
    let is_valid = address.is_valid();

    Ok(Json(ValidateAddressResponse {
        is_valid,
        message: if is_valid {
            "Address is valid".to_string()
        } else {
            "Address is not valid".to_string()
        },
    }))
}

/// Sign and create a transaction in one step
#[utoipa::path(
    post,
    path = "/transactions",
    tag = "Blockchain",
    request_body = SignAndCreateTransactionRequest,
    responses(
        (status = 200, description = "Transaction signed and created successfully", body = CreateTransactionResponse),
        (status = 400, description = "Failed to create transaction", body = ErrorResponse)
    )
)]
pub async fn sign_and_create_transaction(
    State(blockchain): State<SharedBlockchain>,
    Json(request): Json<SignAndCreateTransactionRequest>,
) -> Result<Json<CreateTransactionResponse>, BlockchainError> {
    // Import wallet from private key
    let wallet = Wallet::from_secret_key(&request.private_key).map_err(|e| {
        BlockchainError::ValidationFailed(format!("Failed to import wallet: {}", e))
    })?;

    let sender_address = wallet.get_address().0.clone();
    println!("Sender address: {}", sender_address);

    // Check if sender has enough balance (except for system transactions)
    if sender_address != "system" {
        let bc = blockchain.lock().unwrap();
        let balance = bc.get_balance(&sender_address);
        println!(
            "Sender balance: {}, Transaction amount: {}",
            balance, request.amount
        );

        if balance < request.amount {
            return Err(BlockchainError::InvalidTransaction(format!(
                "Insufficient balance: {} < {}",
                balance, request.amount
            )));
        }
        // Release the lock before continuing
        drop(bc);
    }

    // Create the transaction
    let mut transaction = Transaction::new(
        Address(sender_address.clone()),
        Address(request.recipient.clone()),
        request.amount,
    );

    // Calculate the hash and sign it
    let hash = transaction.hash.clone();
    let signature = wallet.sign(hash.as_bytes()).map_err(|e| {
        BlockchainError::ValidationFailed(format!("Failed to sign transaction: {}", e))
    })?;
    println!("Signature: {}", signature.0);

    // Set the signature
    transaction.signature = Some(signature);
    println!("Transaction created: {:?}", transaction);

    // Validate the transaction
    if !transaction.is_valid() {
        println!("Transaction validation failed!");
        return Err(BlockchainError::InvalidTransaction(
            "Transaction is not valid".to_string(),
        ));
    }

    // Add to blockchain
    let mut blockchain = blockchain.lock().unwrap();
    blockchain.create_transaction(transaction.clone())?;

    Ok(Json(CreateTransactionResponse {
        message: "Transaction signed and created successfully".to_string(),
        transaction,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blockchain::chain::Blockchain;
    use crate::blockchain::wallet::Wallet;
    use axum::extract::State;
    use axum::Json;
    use std::sync::{Arc, Mutex};
    use tokio::runtime::Runtime;

    #[test]
    fn test_create_wallet_handler() {
        let result = tokio_test::block_on(create_wallet());
        assert!(result.is_ok());

        let response = result.unwrap();
        let wallet_response = response.0;

        assert!(!wallet_response.address.is_empty());
        assert!(!wallet_response.public_key.is_empty());
        assert!(!wallet_response.private_key.is_empty());

        // Verify that the wallet is valid by importing it
        let wallet = Wallet::from_secret_key(&wallet_response.private_key).unwrap();
        assert_eq!(wallet.get_address().0, wallet_response.address);
        assert_eq!(wallet.get_public_key_hex(), wallet_response.public_key);
    }

    #[test]
    fn test_import_wallet_handler() {
        // Create a wallet first
        let wallet = Wallet::new().unwrap();
        let private_key = wallet.get_secret_key_hex();
        let expected_address = wallet.get_address().0.clone();
        let expected_public_key = wallet.get_public_key_hex();

        let request = ImportWalletRequest {
            private_key: private_key.clone(),
        };

        let result = tokio_test::block_on(import_wallet(Json(request)));
        assert!(result.is_ok());

        let response = result.unwrap();
        let wallet_response = response.0;

        assert_eq!(wallet_response.address, expected_address);
        assert_eq!(wallet_response.public_key, expected_public_key);
        assert_eq!(wallet_response.message, "Wallet imported successfully");
    }

    #[test]
    fn test_validate_address_handler() {
        // Test with valid address
        let wallet = Wallet::new().unwrap();
        let valid_address = wallet.get_address().0.clone();

        let request = ValidateAddressRequest {
            address: valid_address,
        };

        let result = tokio_test::block_on(validate_address(Json(request)));
        assert!(result.is_ok());

        let response = result.unwrap();
        let validate_response = response.0;

        assert!(validate_response.is_valid);
        assert_eq!(validate_response.message, "Address is valid");

        // Test with invalid address
        let request = ValidateAddressRequest {
            address: "invalid_address".to_string(),
        };

        let result = tokio_test::block_on(validate_address(Json(request)));
        assert!(result.is_ok());

        let response = result.unwrap();
        let validate_response = response.0;

        assert!(!validate_response.is_valid);
        assert_eq!(validate_response.message, "Address is not valid");
    }

    #[test]
    fn test_transaction_flow() {
        // Create a runtime for running async functions in tests
        let rt = Runtime::new().unwrap();

        // 1. Create a shared blockchain
        let blockchain = Arc::new(Mutex::new(Blockchain::new(2, 100.0))); // Set difficulty and mining reward

        // 2. Create a wallet for the recipient
        let recipient_wallet_response = rt.block_on(create_wallet()).unwrap().0;
        let recipient_address = recipient_wallet_response.address.clone();

        // 3. Create a system transaction directly (bypassing the API)
        {
            let mut bc = blockchain.lock().unwrap();
            let system_tx = Transaction::new(
                Address("system".to_string()),
                Address(recipient_address.clone()),
                50.0,
            );
            bc.create_transaction(system_tx).unwrap();
        }

        // 4. Mine a block to include the transaction
        let mine_request = MineBlockRequest {
            miner_address: recipient_address.clone(),
        };

        let _mine_response = rt
            .block_on(mine_block(State(blockchain.clone()), Json(mine_request)))
            .unwrap()
            .0;

        // 5. Verify the transaction is in the blockchain
        let blocks = rt.block_on(get_blocks(State(blockchain.clone()))).0;
        assert!(!blocks.is_empty());

        // The transaction should be in the latest block
        let latest_block = &blocks[blocks.len() - 1];
        let found_transaction = latest_block
            .transactions
            .iter()
            .any(|tx| tx.amount == 50.0 && tx.recipient.0 == recipient_address);

        assert!(found_transaction, "Transaction not found in the blockchain");

        // 6. Verify there are no more pending transactions
        let pending = rt
            .block_on(get_pending_transactions(State(blockchain.clone())))
            .0;
        assert!(
            pending.is_empty(),
            "There should be no pending transactions"
        );

        // 7. Verify the balances are correct
        {
            let bc = blockchain.lock().unwrap();

            // Recipient should have 50 (from system) + 100 (mining reward) = 150
            assert_eq!(bc.get_balance(&recipient_address), 150.0);
        }
    }
}
