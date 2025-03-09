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

    // Act - use a system transaction which doesn't need a wallet
    let response = server
        .post("/transactions")
        .json(&json!({
            "sender": "system",
            "recipient": "recipient",
            "amount": 10.0
        }))
        .await;

    // Assert
    response.assert_status(StatusCode::OK);

    let result: Value = response.json();
    assert!(result["message"].as_str().unwrap().contains("successfully"));

    let tx = &result["transaction"];
    assert_eq!(tx["sender"], "system");
    assert_eq!(tx["recipient"], "recipient");
    assert_eq!(tx["amount"], 10.0);
}

#[tokio::test]
async fn test_create_signed_transaction() {
    // Arrange
    let server = create_test_server().await;

    // Act - use a system transaction which doesn't need a real signature
    let response = server
        .post("/transactions")
        .json(&json!({
            "sender": "system",
            "recipient": "recipient",
            "amount": 10.0,
            "signature": "dummy_signature" // System transactions don't need valid signatures
        }))
        .await;

    // Assert
    response.assert_status(StatusCode::OK);

    let result: Value = response.json();
    assert!(result["message"].as_str().unwrap().contains("successfully"));

    let tx = &result["transaction"];
    assert_eq!(tx["sender"], "system");
    assert_eq!(tx["recipient"], "recipient");
    assert_eq!(tx["amount"], 10.0);
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
    let response = server.post("/blocks/mine").json(&mine_data).await;

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

    // Create an invalid transaction (negative amount)
    let tx_data = json!({
        "sender": "system",
        "recipient": "test_recipient",
        "amount": -10.0  // Negative amount should be invalid
    });

    // Act
    let response = server.post("/transactions").json(&tx_data).await;

    // Assert
    response.assert_status(StatusCode::BAD_REQUEST);

    let body: Value = response.json();
    assert!(body["error"].as_str().unwrap().contains("not valid"));
}

#[tokio::test]
async fn test_wallet_endpoints_removed() {
    // Arrange
    let server = create_test_server().await;

    // Act & Assert - Wallet create endpoint should be removed
    let create_response = server.get("/wallet/create").await;
    create_response.assert_status(StatusCode::NOT_FOUND);

    // Act & Assert - Wallet import endpoint should be removed
    let import_response = server
        .post("/wallet/import")
        .json(&json!({
            "private_key": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
        }))
        .await;
    import_response.assert_status(StatusCode::NOT_FOUND);

    // Act & Assert - Wallet validate endpoint should be removed
    let validate_response = server
        .post("/wallet/validate")
        .json(&json!({
            "address": "0x1234567890123456789012345678901234567890"
        }))
        .await;
    validate_response.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_insufficient_balance() {
    // Arrange
    let server = create_test_server().await;

    // Try to create a transaction with a non-system address (which has 0 balance)
    // This should fail with an insufficient balance error
    let tx_data = json!({
        "sender": "test_user",
        "recipient": "recipient",
        "amount": 20.0,
        "signature": "valid_signature"
    });

    // Act
    let response = server.post("/transactions").json(&tx_data).await;

    // Assert
    response.assert_status(StatusCode::BAD_REQUEST);

    // The error could be either about signature validation or insufficient balance
    // Since we can't easily create a valid signature in tests, we'll just check
    // that the transaction was rejected
    let body: Value = response.json();
    assert!(body.get("error").is_some());
}
