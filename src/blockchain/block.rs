use super::transaction::Transaction;
use super::wallet::Address;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use utoipa::ToSchema;

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
        };

        block.hash = block.calculate_hash();
        block
    }

    /// Calculates the hash of the block
    pub fn calculate_hash(&self) -> String {
        let mut hasher = Sha256::new();
        let data = format!(
            "{}{}{}{}{}",
            self.index,
            self.timestamp.timestamp(),
            serde_json::to_string(&self.transactions).unwrap_or_default(),
            self.previous_hash,
            self.nonce
        );

        hasher.update(data.as_bytes());
        let result = hasher.finalize();
        hex::encode(result)
    }

    /// Mines the block with a specific difficulty
    /// The difficulty determines how many leading zeros the hash must have
    pub fn mine(&mut self, difficulty: usize) {
        let target = "0".repeat(difficulty);

        while self.hash[0..difficulty] != target {
            self.nonce += 1;
            self.hash = self.calculate_hash();
        }
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_genesis_block() {
        let genesis = Block::genesis();

        assert_eq!(genesis.index, 0);
        assert_eq!(genesis.previous_hash, "0".repeat(64));
        assert!(genesis.transactions.is_empty());
        assert!(!genesis.hash.is_empty());
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
            "previous_hash".to_string(),
        );

        block.mine(2);
        assert!(block.hash.starts_with("00"));
    }
}
