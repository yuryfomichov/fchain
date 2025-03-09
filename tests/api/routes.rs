use fchain::blockchain::wallet::Wallet;
use fchain::blockchain::Transaction;
use http::StatusCode;
use serde_json::{json, Value};

use super::test_utils::create_test_server;

#[tokio::test]
async fn test_get_blocks() {
    // Arrange
    let server = create_test_server().await;

    // Act
    let response = server.get("/blocks").await;

    // Assert
    response.assert_status(StatusCode::OK);

    // Should have at least the genesis block
    let blocks: Vec<Value> = response.json();
    assert!(!blocks.is_empty());

    // Check genesis block properties
    let genesis = &blocks[0];
    assert_eq!(genesis["index"], 0);
    assert_eq!(genesis["previous_hash"], "0".repeat(64));
    assert!(genesis["transactions"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_create_transaction() {
    // Arrange
    let server = create_test_server().await;

    // Use a system transaction which doesn't require a signature
    let tx_data = json!({
        "sender": "system",
        "recipient": "recipient",
        "amount": 10.0,
        "signature": null
    });

    // Act
    let response = server.post("/transactions").json(&tx_data).await;

    // Assert
    response.assert_status(StatusCode::OK);

    let body: Value = response.json();
    assert_eq!(body["message"], "Transaction created successfully");

    // Check the transaction fields
    let tx = &body["transaction"];
    assert_eq!(tx["sender"], "system");
    assert_eq!(tx["recipient"], "recipient");
    assert_eq!(tx["amount"], 10.0);

    // Verify transaction was added to pending
    let pending_response = server.get("/transactions/pending").await;
    let pending: Vec<Value> = pending_response.json();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0]["sender"], "system");
}

#[tokio::test]
async fn test_create_signed_transaction() {
    // Arrange
    let server = create_test_server().await;
    let wallet = Wallet::new().unwrap();

    // Create a transaction with the exact same parameters as we'll send to the API
    let mut tx = Transaction::new(
        wallet.get_address().clone(),
        wallet.get_address().clone(), // Send to self for testing
        10.0,
    );

    // Sign the transaction
    tx.sign(&wallet).unwrap();

    // Create the API request with the SAME parameters
    let tx_data = json!({
        "sender": wallet.get_address().0,
        "recipient": wallet.get_address().0, // Must match what we signed
        "amount": 10.0,
        "signature": tx.signature.as_ref().map(|s| s.0.clone())
    });

    // Act
    let response = server.post("/transactions").json(&tx_data).await;

    // Assert
    response.assert_status(StatusCode::OK);

    let body: Value = response.json();
    assert_eq!(body["message"], "Transaction created successfully");

    // Check the transaction fields
    let tx_response = &body["transaction"];
    assert_eq!(tx_response["sender"], wallet.get_address().0);
    assert_eq!(tx_response["recipient"], wallet.get_address().0);
    assert_eq!(tx_response["amount"], 10.0);
    assert!(tx_response["signature"].is_object() || tx_response["signature"].is_string());
}

#[tokio::test]
async fn test_mine_block() {
    // Arrange
    let server = create_test_server().await;

    // Use a system transaction which doesn't require a signature
    let tx_data = json!({
        "sender": "system",
        "recipient": "recipient",
        "amount": 10.0,
        "signature": null
    });
    server.post("/transactions").json(&tx_data).await;

    // Act - Mine a block
    let mine_data = json!({
        "miner_address": "test_miner"
    });
    let response = server.post("/mine").json(&mine_data).await;

    // Assert
    response.assert_status(StatusCode::OK);

    let body: Value = response.json();
    assert_eq!(body["message"], "Block mined successfully");
    assert_eq!(body["block"]["index"], 1); // Genesis is 0, this should be 1

    // Verify block was added to chain
    let blocks_response = server.get("/blocks").await;
    let blocks: Vec<Value> = blocks_response.json();
    assert_eq!(blocks.len(), 2); // Genesis + new block

    // Verify pending transactions were cleared
    let pending_response = server.get("/transactions/pending").await;
    let pending: Vec<Value> = pending_response.json();
    assert!(pending.is_empty());
}

#[tokio::test]
async fn test_validate_chain() {
    // Arrange
    let server = create_test_server().await;

    // Act
    let response = server.get("/chain/validate").await;

    // Assert
    response.assert_status(StatusCode::OK);

    let body: Value = response.json();
    assert_eq!(body["valid"], true);
    assert_eq!(body["message"], "Blockchain is valid");
}

#[tokio::test]
async fn test_get_pending_transactions_empty() {
    // Arrange
    let server = create_test_server().await;

    // Act
    let response = server.get("/transactions/pending").await;

    // Assert
    response.assert_status(StatusCode::OK);

    let pending: Vec<Value> = response.json();
    assert!(pending.is_empty());
}

#[tokio::test]
async fn test_invalid_transaction() {
    // Arrange
    let server = create_test_server().await;
    let tx_data = json!({
        "sender": "",  // Empty sender should be invalid
        "recipient": "test_recipient",
        "amount": -10.0,  // Negative amount should be invalid
        "signature": null
    });

    // Act
    let response = server.post("/transactions").json(&tx_data).await;

    // Assert
    response.assert_status(StatusCode::BAD_REQUEST);

    let body: Value = response.json();
    assert!(body["error"].as_str().unwrap().contains("not valid"));
}
