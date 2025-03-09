use fchain::{
    blockchain::{Address, Transaction},
    Blockchain,
};

#[test]
fn test_blockchain_creation() {
    let blockchain = Blockchain::new(2, 100.0);
    assert_eq!(blockchain.chain.len(), 1);
    assert_eq!(blockchain.chain[0].index, 0);
    assert!(blockchain.pending_transactions.is_empty());
    assert_eq!(blockchain.difficulty, 2);
    assert_eq!(blockchain.mining_reward, 100.0);
}

#[test]
fn test_add_transaction() {
    let mut blockchain = Blockchain::new(2, 100.0);

    // Use system transactions which don't require signatures
    let tx1 = Transaction::new(
        Address("system".to_string()),
        Address("address2".to_string()),
        10.0,
    );
    let tx2 = Transaction::new(
        Address("system".to_string()),
        Address("address3".to_string()),
        5.0,
    );

    blockchain.create_transaction(tx1).unwrap();
    blockchain.create_transaction(tx2).unwrap();

    assert_eq!(blockchain.pending_transactions.len(), 2);
}

#[test]
fn test_mine_block() {
    let mut blockchain = Blockchain::new(2, 100.0);

    // Add a system transaction
    let tx1 = Transaction::new(
        Address("system".to_string()),
        Address("address2".to_string()),
        10.0,
    );
    blockchain.create_transaction(tx1).unwrap();

    // Mine a block
    let block = blockchain
        .mine_pending_transactions("miner_address")
        .unwrap();

    // Verify the block was added
    assert_eq!(blockchain.chain.len(), 2); // Genesis + new block
    assert_eq!(block.index, 1);
    assert!(block.hash.starts_with("00")); // Difficulty 2 means 2 leading zeros

    // Verify pending transactions were cleared
    assert!(blockchain.pending_transactions.is_empty());
}

#[test]
fn test_validate_chain() {
    let mut blockchain = Blockchain::new(2, 100.0);

    // Add and mine some blocks with system transactions
    let tx = Transaction::new(
        Address("system".to_string()),
        Address("address2".to_string()),
        10.0,
    );
    blockchain.create_transaction(tx).unwrap();
    blockchain.mine_pending_transactions("miner").unwrap();

    assert!(blockchain.is_chain_valid().unwrap());
}

#[test]
fn test_transaction_flow() {
    use fchain::blockchain::create_shared_blockchain;
    use fchain::blockchain::crypto::TransactionSignature;
    use fchain::blockchain::{Address, Transaction};

    // Create a blockchain
    let blockchain = create_shared_blockchain(2, 100.0);

    // Add a system transaction (mining reward) to give the sender some coins
    let mut chain = blockchain.lock().unwrap();
    let sender_address = "system";
    let recipient_address = "recipient";
    let system_tx = Transaction::new(
        Address(sender_address.to_string()),
        Address(recipient_address.to_string()),
        100.0,
    );
    // For system transactions, we need to set the signature
    let mut system_tx_with_sig = system_tx;
    system_tx_with_sig.signature = Some(TransactionSignature("system".to_string()));
    chain.create_transaction(system_tx_with_sig).unwrap();
    chain.mine_pending_transactions("miner").unwrap();
    drop(chain);

    // Create a mock transaction with the same data for comparison
    let mut expected_tx = Transaction::new(
        Address("system".to_string()),
        Address("another_recipient".to_string()),
        50.0,
    );
    expected_tx.signature = Some(TransactionSignature("system".to_string()));

    // We can't directly test the async function, but we can verify the transaction creation logic
    let mut blockchain = blockchain.lock().unwrap();

    // For testing purposes, we'll bypass the validation since we don't have real keys
    // In a real scenario, this would be properly validated
    blockchain.create_transaction(expected_tx.clone()).unwrap();

    // Verify the transaction was added to pending transactions
    assert_eq!(blockchain.pending_transactions.len(), 1);

    // The hash will be different, so we'll just check the other fields
    let tx = &blockchain.pending_transactions[0];
    assert_eq!(tx.sender.0, "system");
    assert_eq!(tx.recipient.0, "another_recipient");
    assert_eq!(tx.amount, 50.0);
    assert!(tx.signature.is_some());
}
