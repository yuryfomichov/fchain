use fchain::blockchain::blockchain::Blockchain;
use fchain::blockchain::transaction::Transaction;

#[test]
fn test_blockchain_integration() {
    // Create a new blockchain with difficulty 2 and mining reward 50
    let mut blockchain = Blockchain::new(2, 50.0);

    // Add some transactions
    let tx1 = Transaction::new("address1".to_string(), "address2".to_string(), 10.0);
    let tx2 = Transaction::new("address2".to_string(), "address3".to_string(), 5.0);

    blockchain.create_transaction(tx1).unwrap();
    blockchain.create_transaction(tx2).unwrap();

    // Mine a block
    let block = blockchain
        .mine_pending_transactions("miner_address")
        .unwrap();

    // Verify the block was added
    assert_eq!(blockchain.chain.len(), 2); // Genesis block + new block
    assert_eq!(block.index, 1);
    assert!(block.hash.starts_with("00")); // Difficulty 2 means 2 leading zeros

    // Verify the chain is valid
    assert!(blockchain.is_chain_valid().unwrap());

    // Verify pending transactions were cleared
    assert!(blockchain.pending_transactions.is_empty());

    // Verify mining reward transaction was added
    let reward_tx = Transaction::new("system".to_string(), "miner_address".to_string(), 50.0);
    blockchain.create_transaction(reward_tx).unwrap();

    // Mine another block
    let block2 = blockchain
        .mine_pending_transactions("miner_address")
        .unwrap();

    // Verify the second block was added
    assert_eq!(blockchain.chain.len(), 3);
    assert_eq!(block2.index, 2);
    assert!(blockchain.is_chain_valid().unwrap());
}
