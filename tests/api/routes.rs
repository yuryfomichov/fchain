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

    // First create a wallet to get a valid private key
    let create_response = server.get("/wallet/create").await;
    let wallet_data: Value = create_response.json();
    let private_key = wallet_data["private_key"].as_str().unwrap();
    let address = wallet_data["address"].as_str().unwrap();

    // Mine a block to get some coins
    let mine_data = json!({
        "miner_address": address
    });
    server.post("/mine").json(&mine_data).await;

    // Use the private key to create a transaction
    let tx_data = json!({
        "recipient": "recipient",
        "amount": 10.0,
        "private_key": private_key
    });

    // Act
    let response = server.post("/transactions").json(&tx_data).await;

    // Assert
    response.assert_status(StatusCode::OK);

    let body: Value = response.json();
    assert!(body["message"].as_str().unwrap().contains("successfully"));

    // Check the transaction fields
    let tx = &body["transaction"];
    assert_eq!(tx["recipient"], "recipient");
    assert_eq!(tx["amount"], 10.0);

    // Verify transaction was added to pending
    let pending_response = server.get("/transactions/pending").await;
    let pending: Vec<Value> = pending_response.json();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0]["recipient"], "recipient");
}

#[tokio::test]
async fn test_create_signed_transaction() {
    // Arrange
    let server = create_test_server().await;

    // Create a wallet
    let wallet_response = server.get("/wallet/create").await;
    let wallet_data: Value = wallet_response.json();
    let private_key = wallet_data["private_key"].as_str().unwrap();
    let address = wallet_data["address"].as_str().unwrap();
    let recipient = "test_recipient";

    // Mine a block to get some coins
    let mine_data = json!({
        "miner_address": address
    });
    server.post("/mine").json(&mine_data).await;

    // Create the API request
    let tx_data = json!({
        "recipient": recipient,
        "amount": 10.0,
        "private_key": private_key
    });

    // Act
    let response = server.post("/transactions").json(&tx_data).await;

    // Assert
    response.assert_status(StatusCode::OK);

    let body: Value = response.json();
    assert!(body["message"].as_str().unwrap().contains("successfully"));

    // Check the transaction fields
    let tx_response = &body["transaction"];
    assert_eq!(tx_response["recipient"], recipient);
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

    // Create a wallet
    let wallet_response = server.get("/wallet/create").await;
    let wallet_data: Value = wallet_response.json();
    let private_key = wallet_data["private_key"].as_str().unwrap();

    // Create an invalid transaction (negative amount)
    let tx_data = json!({
        "recipient": "test_recipient",
        "amount": -10.0,  // Negative amount should be invalid
        "private_key": private_key
    });

    // Act
    let response = server.post("/transactions").json(&tx_data).await;

    // Assert
    response.assert_status(StatusCode::BAD_REQUEST);

    let body: Value = response.json();
    assert!(body["error"].as_str().unwrap().contains("not valid"));
}

#[tokio::test]
async fn test_create_wallet() {
    // Arrange
    let server = create_test_server().await;

    // Act
    let response = server.get("/wallet/create").await;

    // Assert
    response.assert_status(StatusCode::OK);

    let wallet_response: Value = response.json();

    // Check that the wallet has the expected fields
    assert!(wallet_response["address"].is_string());
    assert!(wallet_response["public_key"].is_string());
    assert!(wallet_response["private_key"].is_string());

    // Verify the address is not empty
    let address = wallet_response["address"].as_str().unwrap();
    assert!(!address.is_empty());
    assert_eq!(address.len(), 64); // Hex-encoded public key

    // Verify the public key is not empty
    let public_key = wallet_response["public_key"].as_str().unwrap();
    assert!(!public_key.is_empty());
    assert_eq!(public_key.len(), 64); // Hex-encoded public key

    // Verify the private key is not empty
    let private_key = wallet_response["private_key"].as_str().unwrap();
    assert!(!private_key.is_empty());
    assert_eq!(private_key.len(), 64); // Hex-encoded private key
}

#[tokio::test]
async fn test_import_wallet() {
    // Arrange
    let server = create_test_server().await;

    // First create a wallet to get a valid private key
    let create_response = server.get("/wallet/create").await;
    let create_data: Value = create_response.json();
    let private_key = create_data["private_key"].as_str().unwrap();
    let expected_address = create_data["address"].as_str().unwrap();
    let expected_public_key = create_data["public_key"].as_str().unwrap();

    // Act - Import the wallet
    let import_data = json!({
        "private_key": private_key
    });
    let response = server.post("/wallet/import").json(&import_data).await;

    // Assert
    response.assert_status(StatusCode::OK);

    let import_response: Value = response.json();

    // Check that the imported wallet has the expected fields
    assert_eq!(import_response["address"], expected_address);
    assert_eq!(import_response["public_key"], expected_public_key);
    assert_eq!(import_response["message"], "Wallet imported successfully");
}

#[tokio::test]
async fn test_validate_address() {
    // Arrange
    let server = create_test_server().await;

    // First create a wallet to get a valid address
    let create_response = server.get("/wallet/create").await;
    let create_data: Value = create_response.json();
    let valid_address = create_data["address"].as_str().unwrap();

    // Act - Validate a valid address
    let validate_data = json!({
        "address": valid_address
    });
    let response = server.post("/wallet/validate").json(&validate_data).await;

    // Assert
    response.assert_status(StatusCode::OK);

    let validate_response: Value = response.json();
    assert_eq!(validate_response["is_valid"], true);
    assert_eq!(validate_response["message"], "Address is valid");

    // Act - Validate an invalid address
    let invalid_data = json!({
        "address": "invalid_address"
    });
    let response = server.post("/wallet/validate").json(&invalid_data).await;

    // Assert
    response.assert_status(StatusCode::OK);

    let validate_response: Value = response.json();
    assert_eq!(validate_response["is_valid"], false);
    assert_eq!(validate_response["message"], "Address is not valid");
}
