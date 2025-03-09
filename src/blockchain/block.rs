use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use utoipa::ToSchema;

use super::transaction::Transaction;

/// Represents a block in the blockchain
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Block {
    /// Index of the block in the chain
    pub index: u64,
    /// Timestamp when the block was created
    pub timestamp: DateTime<Utc>,
    /// Transactions included in this block
    pub transactions: Vec<Transaction>,
    /// Hash of the previous block
    pub previous_hash: String,
    /// Nonce used for mining (proof of work)
    pub nonce: u64,
    /// Hash of this block
    pub hash: String,
    /// Difficulty level used for mining this block
    pub difficulty: usize,
}

impl Block {
    /// Creates a new block
    pub fn new(index: u64, transactions: Vec<Transaction>, previous_hash: String) -> Self {
        let mut block = Self {
            index,
            timestamp: Utc::now(),
            transactions,
            previous_hash,
            nonce: 0,
            hash: String::new(),
            difficulty: 4,
        };

        block.hash = block.calculate_hash();
        block
    }

    /// Creates the genesis block (first block in the chain)
    pub fn genesis() -> Self {
        let mut block = Self {
            index: 0,
            timestamp: Utc::now(),
            transactions: vec![],
            previous_hash: "0".repeat(64),
            nonce: 0,
            hash: String::new(),
            difficulty: 4,
        };

        block.hash = block.calculate_hash();
        block
    }

    /// Calculates the hash of the block
    pub fn calculate_hash(&self) -> String {
        let mut hasher = Sha256::new();

        // Add block data to hasher in a more efficient way
        hasher.update(&self.index.to_be_bytes());
        hasher.update(&self.timestamp.timestamp().to_be_bytes());

        // Process transactions more efficiently
        for tx in &self.transactions {
            hasher.update(tx.hash.as_bytes());
        }

        hasher.update(self.previous_hash.as_bytes());
        hasher.update(&self.nonce.to_be_bytes());

        hex::encode(hasher.finalize())
    }

    /// Mines the block with a specific difficulty
    /// The difficulty determines how many leading zeros the hash must have
    pub fn mine(&mut self, difficulty: usize) {
        // Store the difficulty used for mining
        self.difficulty = difficulty;

        let target = "0".repeat(difficulty);

        while !self.hash.starts_with(&target) {
            self.nonce += 1;
            self.hash = self.calculate_hash();
        }
    }

    /// Verifies that the block meets the proof of work requirement
    pub fn verify_proof_of_work(&self, difficulty: usize) -> bool {
        if difficulty == 0 {
            return true;
        }

        let target = "0".repeat(difficulty);
        self.hash.starts_with(&target)
    }

    /// Validates the block
    pub fn is_valid(&self) -> bool {
        // Check if the hash is correct
        let calculated_hash = self.calculate_hash();
        if calculated_hash != self.hash {
            return false;
        }

        // Check if all transactions are valid
        for transaction in &self.transactions {
            if !transaction.is_valid() {
                return false;
            }
        }

        true
    }

    // Add a method to validate the block against a previous block
    pub fn is_valid_next_block(&self, previous_block: &Block) -> bool {
        // Check block sequence
        if self.index != previous_block.index + 1 {
            return false;
        }

        // Check previous hash reference
        if self.previous_hash != previous_block.hash {
            return false;
        }

        // Check hash integrity
        if self.hash != self.calculate_hash() {
            return false;
        }

        // Check proof of work
        if !self.verify_proof_of_work(self.difficulty) {
            return false;
        }

        // Validate timestamp (block must be after previous block)
        if self.timestamp <= previous_block.timestamp {
            return false;
        }

        // Prevent timestamps too far in the future (e.g., 2 hours)
        let future_limit = Utc::now() + chrono::Duration::hours(2);
        if self.timestamp > future_limit {
            return false;
        }

        // Validate all transactions in the block
        for transaction in &self.transactions {
            if !transaction.is_valid() {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blockchain::Address;

    #[test]
    fn test_genesis_block() {
        let genesis = Block::genesis();

        assert_eq!(genesis.index, 0);
        assert_eq!(genesis.previous_hash, "0".repeat(64));
        assert!(genesis.transactions.is_empty());
        assert!(!genesis.hash.is_empty());
        assert_eq!(genesis.difficulty, 4); // Check default difficulty

        // Additional check for timestamp being reasonable
        let now = Utc::now().timestamp();
        let block_time = genesis.timestamp.timestamp();
        assert!(block_time <= now);
        assert!(block_time > now - 10); // Genesis block should be very recent
    }

    #[test]
    fn test_mining() {
        let mut block = Block::new(
            1,
            vec![Transaction::new(
                Address("system".to_string()),
                Address("recipient".to_string()),
                50.0,
            )],
            "0".repeat(64),
        );

        // Check default difficulty is set
        assert_eq!(block.difficulty, 4);

        // Override with test difficulty
        block.mine(2);
        assert!(block.hash.starts_with("00"));
        assert!(block.verify_proof_of_work(2));

        // Test with higher difficulty
        let mut block2 = Block::new(
            2,
            vec![Transaction::new(
                Address("system".to_string()),
                Address("recipient".to_string()),
                50.0,
            )],
            block.hash.clone(),
        );
        block2.mine(4);
        assert!(block2.hash.starts_with("0000"));
        assert!(block2.verify_proof_of_work(4));
    }

    #[test]
    fn test_block_validation() {
        let genesis = Block::genesis();

        // Create a valid next block
        let mut block = Block::new(
            1,
            vec![Transaction::new(
                Address("system".to_string()),
                Address("user".to_string()),
                10.0,
            )],
            genesis.hash.clone(),
        );
        block.mine(2);

        // Should be valid
        assert!(block.is_valid());
        assert!(block.is_valid_next_block(&genesis));

        // Test with invalid index
        let mut invalid_block = block.clone();
        invalid_block.index = 5;
        assert!(!invalid_block.is_valid_next_block(&genesis));

        // Test with invalid previous hash
        let mut invalid_block = block.clone();
        invalid_block.previous_hash = "invalid_hash".to_string();
        assert!(!invalid_block.is_valid_next_block(&genesis));

        // Test with tampered hash
        let mut invalid_block = block.clone();
        invalid_block.hash = "tampered_hash".to_string();
        assert!(!invalid_block.is_valid());
        assert!(!invalid_block.is_valid_next_block(&genesis));

        // Test with tampered nonce
        let mut invalid_block = block.clone();
        invalid_block.nonce += 1;
        invalid_block.hash = invalid_block.calculate_hash();
        assert!(invalid_block.is_valid()); // The hash is still valid for the block itself
        assert!(!invalid_block.verify_proof_of_work(2)); // But it no longer meets the proof of work requirement

        // Test with invalid timestamp (before previous block)
        let mut invalid_block = block.clone();
        invalid_block.timestamp = genesis.timestamp - chrono::Duration::seconds(1);
        invalid_block.hash = invalid_block.calculate_hash();
        assert!(!invalid_block.is_valid_next_block(&genesis));
    }

    #[test]
    fn test_hash_consistency() {
        let block = Block::new(
            1,
            vec![Transaction::new(
                Address("system".to_string()),
                Address("recipient".to_string()),
                50.0,
            )],
            "0".repeat(64),
        );

        let hash1 = block.calculate_hash();
        let hash2 = block.calculate_hash();

        assert_eq!(hash1, hash2, "Hash calculation should be deterministic");
        assert_eq!(
            block.hash, hash1,
            "Stored hash should match calculated hash"
        );
    }

    #[test]
    fn test_future_timestamp_validation() {
        let genesis = Block::genesis();

        // Create a block with a timestamp too far in the future
        let mut invalid_block = Block::new(
            1,
            vec![Transaction::new(
                Address("system".to_string()),
                Address("user".to_string()),
                10.0,
            )],
            genesis.hash.clone(),
        );

        // Set timestamp to 3 hours in the future
        invalid_block.timestamp = Utc::now() + chrono::Duration::hours(3);
        invalid_block.hash = invalid_block.calculate_hash();

        assert!(!invalid_block.is_valid_next_block(&genesis));
    }
}
