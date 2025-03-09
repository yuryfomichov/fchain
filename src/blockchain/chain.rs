use std::sync::{Arc, Mutex};
use thiserror::Error;

use super::{block::Block, transaction::Transaction};

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
                "Transaction hash is invalid".to_string(),
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
            "System".to_string(),
            miner_address.to_string(),
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
        }

        Ok(true)
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

        // Add a transaction
        let tx = Transaction::new("Alice".to_string(), "Bob".to_string(), 10.0);
        blockchain.create_transaction(tx).unwrap();

        // Mine a block
        let block = blockchain.mine_pending_transactions("Miner").unwrap();

        // Check if the block was added to the chain
        assert_eq!(blockchain.chain.len(), 2);
        assert_eq!(block.index, 1);
        assert!(block.hash.starts_with("00"));

        // Check if pending transactions were cleared
        assert!(blockchain.pending_transactions.is_empty());

        // Validate the chain
        assert!(blockchain.is_chain_valid().unwrap());
    }
}
