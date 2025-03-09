use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Represents a transaction in the blockchain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    /// Sender of the transaction
    pub sender: String,
    /// Recipient of the transaction
    pub recipient: String,
    /// Amount being transferred
    pub amount: f64,
    /// Timestamp when the transaction was created
    pub timestamp: DateTime<Utc>,
    /// Transaction hash
    pub hash: String,
}

impl Transaction {
    /// Creates a new transaction
    pub fn new(sender: String, recipient: String, amount: f64) -> Self {
        let timestamp = Utc::now();
        let mut transaction = Self {
            sender,
            recipient,
            amount,
            timestamp,
            hash: String::new(),
        };

        transaction.hash = transaction.calculate_hash();
        transaction
    }

    /// Calculates the hash of the transaction
    pub fn calculate_hash(&self) -> String {
        let mut hasher = Sha256::new();
        let data = format!(
            "{}{}{}{}",
            self.sender,
            self.recipient,
            self.amount,
            self.timestamp.timestamp()
        );

        hasher.update(data.as_bytes());
        let result = hasher.finalize();
        hex::encode(result)
    }

    /// Validates the transaction
    pub fn is_valid(&self) -> bool {
        let calculated_hash = self.calculate_hash();
        calculated_hash == self.hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_creation() {
        let tx = Transaction::new("Alice".to_string(), "Bob".to_string(), 10.0);

        assert_eq!(tx.sender, "Alice");
        assert_eq!(tx.recipient, "Bob");
        assert_eq!(tx.amount, 10.0);
        assert!(!tx.hash.is_empty());
        assert!(tx.is_valid());
    }

    #[test]
    fn test_transaction_validation() {
        let mut tx = Transaction::new("Alice".to_string(), "Bob".to_string(), 10.0);

        assert!(tx.is_valid());

        // Tamper with the transaction
        tx.amount = 100.0;

        // Hash should no longer be valid
        assert!(!tx.is_valid());
    }
}
