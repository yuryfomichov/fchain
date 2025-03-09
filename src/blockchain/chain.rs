use std::sync::{Arc, Mutex};
use thiserror::Error;

use super::{block::Block, transaction::Transaction, wallet::Address};

/// Errors that can occur in the blockchain
#[derive(Debug, Error)]
pub enum BlockchainError {
    #[error("Invalid block: {0}")]
    InvalidBlock(String),

    #[error("Invalid transaction: {0}")]
    InvalidTransaction(String),

    #[error("Chain validation failed: {0}")]
    ValidationFailed(String),
}

/// Represents the blockchain
#[derive(Debug, Clone)]
pub struct Blockchain {
    /// The chain of blocks
    pub chain: Vec<Block>,
    /// Pending transactions to be included in the next block
    pub pending_transactions: Vec<Transaction>,
    /// Mining difficulty (number of leading zeros required in block hash)
    pub difficulty: usize,
    /// Mining reward for adding a new block
    pub mining_reward: f64,
}

impl Blockchain {
    /// Creates a new blockchain with the genesis block
    pub fn new(difficulty: usize, mining_reward: f64) -> Self {
        let chain = vec![Block::genesis()];

        Self {
            chain,
            pending_transactions: Vec::new(),
            difficulty,
            mining_reward,
        }
    }

    /// Gets the latest block in the chain
    pub fn get_latest_block(&self) -> Option<&Block> {
        self.chain.last()
    }

    /// Adds a new transaction to the pending transactions
    pub fn create_transaction(&mut self, transaction: Transaction) -> Result<(), BlockchainError> {
        if !transaction.is_valid() {
            return Err(BlockchainError::InvalidTransaction(
                "Transaction is not valid".to_string(),
            ));
        }

        self.pending_transactions.push(transaction);
        Ok(())
    }

    /// Mines a new block with the pending transactions
    pub fn mine_pending_transactions(
        &mut self,
        miner_address: &str,
    ) -> Result<Block, BlockchainError> {
        // Create a mining reward transaction
        let reward_tx = Transaction::new(
            Address("system".to_string()),
            Address(miner_address.to_string()),
            self.mining_reward,
        );

        // Add the reward transaction to pending transactions
        self.pending_transactions.push(reward_tx);

        // Get the latest block
        let latest_block = self
            .get_latest_block()
            .ok_or_else(|| BlockchainError::ValidationFailed("Chain is empty".to_string()))?;

        // Create a new block with pending transactions
        let mut new_block = Block::new(
            latest_block.index + 1,
            self.pending_transactions.clone(),
            latest_block.hash.clone(),
        );

        // Mine the block
        new_block.mine(self.difficulty);

        // Add the block to the chain
        self.chain.push(new_block.clone());

        // Clear pending transactions
        self.pending_transactions = Vec::new();

        Ok(new_block)
    }

    /// Validates the entire blockchain
    pub fn is_chain_valid(&self) -> Result<bool, BlockchainError> {
        // Check if the chain has at least one block (genesis)
        if self.chain.is_empty() {
            return Err(BlockchainError::ValidationFailed(
                "Chain is empty".to_string(),
            ));
        }

        // Iterate through the chain and validate each block
        for i in 1..self.chain.len() {
            let current_block = &self.chain[i];
            let previous_block = &self.chain[i - 1];

            // Check if the block is valid
            if !current_block.is_valid() {
                return Err(BlockchainError::InvalidBlock(format!(
                    "Block {} has invalid hash",
                    current_block.index
                )));
            }

            // Check if the previous hash matches
            if current_block.previous_hash != previous_block.hash {
                return Err(BlockchainError::ValidationFailed(format!(
                    "Block {} has invalid previous hash reference",
                    current_block.index
                )));
            }

            // Check if the index is sequential
            if current_block.index != previous_block.index + 1 {
                return Err(BlockchainError::ValidationFailed(format!(
                    "Block {} has invalid index",
                    current_block.index
                )));
            }

            // Validate all transactions in the block
            for transaction in &current_block.transactions {
                if !transaction.is_valid() {
                    return Err(BlockchainError::InvalidTransaction(format!(
                        "Invalid transaction in block {}",
                        current_block.index
                    )));
                }
            }
        }

        Ok(true)
    }

    /// Gets the balance of an address by examining all transactions in the blockchain
    pub fn get_balance(&self, address: &str) -> f64 {
        let mut balance = 0.0;

        // Check all blocks in the chain
        for block in &self.chain {
            for transaction in &block.transactions {
                // If this address is the recipient, add the amount
                if transaction.recipient.0 == address {
                    balance += transaction.amount;
                }

                // If this address is the sender, subtract the amount
                if transaction.sender.0 == address {
                    balance -= transaction.amount;
                }
            }
        }

        // Also check pending transactions
        for transaction in &self.pending_transactions {
            // If this address is the recipient, add the amount
            if transaction.recipient.0 == address {
                balance += transaction.amount;
            }

            // If this address is the sender, subtract the amount
            if transaction.sender.0 == address {
                balance -= transaction.amount;
            }
        }

        balance
    }
}

/// Thread-safe blockchain that can be shared between threads
pub type SharedBlockchain = Arc<Mutex<Blockchain>>;

/// Creates a new shared blockchain
pub fn create_shared_blockchain(difficulty: usize, mining_reward: f64) -> SharedBlockchain {
    Arc::new(Mutex::new(Blockchain::new(difficulty, mining_reward)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blockchain::wallet::Address;

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
    fn test_mining_block() {
        let mut blockchain = Blockchain::new(2, 100.0);

        // Create a system transaction (doesn't need signing)
        let tx = Transaction::new(
            Address("system".to_string()),
            Address("recipient".to_string()),
            10.0,
        );

        // Add transaction and mine block
        blockchain.create_transaction(tx).unwrap();
        let block = blockchain.mine_pending_transactions("miner").unwrap();

        // Check if the block was added to the chain
        assert_eq!(blockchain.chain.len(), 2);
        assert_eq!(block.index, 1);
        assert!(block.hash.starts_with("00"));

        // Check if pending transactions were cleared
        assert!(blockchain.pending_transactions.is_empty());

        // Validate the chain
        assert!(blockchain.is_chain_valid().unwrap());
    }

    #[test]
    fn test_get_balance() {
        let mut blockchain = Blockchain::new(2, 100.0);

        // Create some test addresses
        let address1 = "address1";
        let address2 = "address2";

        // Initially, balances should be zero
        assert_eq!(blockchain.get_balance(address1), 0.0);
        assert_eq!(blockchain.get_balance(address2), 0.0);

        // Add a transaction from system to address1 (system transactions don't need signatures)
        let tx1 = Transaction::new(
            Address("system".to_string()),
            Address(address1.to_string()),
            100.0,
        );
        blockchain.create_transaction(tx1).unwrap();

        // Mine the block to include the transaction
        blockchain.mine_pending_transactions(address2).unwrap();

        // Check balances after mining
        assert_eq!(blockchain.get_balance(address1), 100.0);
        assert_eq!(blockchain.get_balance(address2), 100.0); // Mining reward

        // For non-system transactions, we need to create wallets and sign properly
        // But for this test, we'll just use another system transaction
        let tx2 = Transaction::new(
            Address("system".to_string()),
            Address(address2.to_string()),
            50.0,
        );
        blockchain.create_transaction(tx2).unwrap();

        // Check balances with pending transaction
        assert_eq!(blockchain.get_balance(address1), 100.0);
        assert_eq!(blockchain.get_balance(address2), 150.0); // 100 + 50

        // Mine another block
        blockchain.mine_pending_transactions(address1).unwrap();

        // Check final balances
        assert_eq!(blockchain.get_balance(address1), 200.0); // 100 + 100 (mining reward)
        assert_eq!(blockchain.get_balance(address2), 150.0); // 150 (unchanged)
    }
}
