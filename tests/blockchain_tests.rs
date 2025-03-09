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
