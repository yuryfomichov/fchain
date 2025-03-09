use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use utoipa::ToSchema;

use super::wallet::{Address, TransactionSignature, Wallet};

/// Represents a transaction in the blockchain
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Transaction {
    /// Sender's address
    pub sender: Address,
    /// Recipient's address
    pub recipient: Address,
    /// Amount being transferred
    pub amount: f64,
    /// Timestamp when the transaction was created
    pub timestamp: DateTime<Utc>,
    /// Transaction hash
    pub hash: String,
    /// Digital signature of the transaction
    pub signature: Option<TransactionSignature>,
}

impl Transaction {
    /// Creates a new transaction
    pub fn new(sender: Address, recipient: Address, amount: f64) -> Self {
        let timestamp = Utc::now();
        let mut transaction = Self {
            sender,
            recipient,
            amount,
            timestamp,
            hash: String::new(),
            signature: None,
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
        // Check if the amount is valid
        if self.amount <= 0.0 {
            println!("Transaction invalid: amount <= 0");
            return false;
        }

        // Check if the addresses are valid
        if self.sender.0.is_empty() || self.recipient.0.is_empty() {
            println!("Transaction invalid: empty sender or recipient");
            return false;
        }

        // Check if the hash is correct
        let calculated_hash = self.calculate_hash();
        if calculated_hash != self.hash {
            println!(
                "Transaction invalid: hash mismatch. Expected: {}, Got: {}",
                self.hash, calculated_hash
            );
            return false;
        }

        // System transactions (mining rewards) don't need signatures
        if self.sender.0 == "system" {
            return true;
        }

        // Check if the transaction is signed
        let signature = match &self.signature {
            Some(sig) => sig,
            None => {
                println!("Transaction invalid: missing signature");
                return false;
            }
        };

        // Verify the signature
        let result = Wallet::verify(&self.sender.0, self.hash.as_bytes(), signature);
        match result {
            Ok(valid) => {
                if !valid {
                    println!("Transaction invalid: signature verification failed");
                }
                valid
            }
            Err(e) => {
                println!("Transaction invalid: signature verification error: {}", e);
                false
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_creation() {
        let wallet = Wallet::new().unwrap();
        let recipient = Address("recipient".to_string());
        let tx = Transaction::new(wallet.get_address().clone(), recipient, 10.0);

        assert_eq!(&tx.sender, wallet.get_address());
        assert_eq!(tx.recipient.0, "recipient");
        assert_eq!(tx.amount, 10.0);
        assert!(!tx.hash.is_empty());
        assert!(!tx.is_valid()); // Not valid until signed
    }

    #[test]
    fn test_transaction_signing() {
        let wallet = Wallet::new().unwrap();
        let recipient = Address("recipient".to_string());
        let mut tx = Transaction::new(wallet.get_address().clone(), recipient, 10.0);

        // Sign the transaction using wallet directly
        let signature = wallet.sign(tx.hash.as_bytes()).unwrap();
        tx.signature = Some(signature);
        assert!(tx.is_valid());

        // Try to sign with wrong wallet (should still work but validation will fail)
        let wrong_wallet = Wallet::new().unwrap();
        let wrong_signature = wrong_wallet.sign(tx.hash.as_bytes()).unwrap();
        tx.signature = Some(wrong_signature);
        assert!(!tx.is_valid());
    }

    #[test]
    fn test_system_transaction() {
        let system_addr = Address("system".to_string());
        let recipient = Address("miner".to_string());
        let tx = Transaction::new(system_addr, recipient, 50.0);

        // System transactions are valid without signatures
        assert!(tx.is_valid());
    }

    #[test]
    fn test_transaction_tampering() {
        let wallet = Wallet::new().unwrap();
        let recipient = Address("recipient".to_string());
        let mut tx = Transaction::new(wallet.get_address().clone(), recipient, 10.0);

        // Sign the transaction using wallet directly
        let signature = wallet.sign(tx.hash.as_bytes()).unwrap();
        tx.signature = Some(signature);
        assert!(tx.is_valid());

        // Tamper with the amount
        tx.amount = 100.0;
        assert!(!tx.is_valid());
    }
}
