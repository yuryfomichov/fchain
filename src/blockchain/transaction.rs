use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::wallet::{Address, TransactionSignature, Wallet, WalletError};

/// Represents a transaction in the blockchain
#[derive(Debug, Clone, Serialize, Deserialize)]
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

    /// Signs the transaction with a wallet
    pub fn sign(&mut self, wallet: &Wallet) -> Result<(), WalletError> {
        // Verify that the signer is the sender
        if wallet.get_address() != &self.sender {
            return Err(WalletError::SigningError(
                "Wallet address does not match sender".to_string(),
            ));
        }

        // Sign the transaction hash
        let signature = wallet.sign(self.hash.as_bytes())?;
        self.signature = Some(signature);
        Ok(())
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
            return false;
        }

        // Check if the addresses are valid
        if self.sender.0.is_empty() || self.recipient.0.is_empty() {
            return false;
        }

        // Check if the hash is correct
        if self.calculate_hash() != self.hash {
            return false;
        }

        // System transactions (mining rewards) don't need signatures
        if self.sender.0 == "system" {
            return true;
        }

        // Check if the transaction is signed
        let signature = match &self.signature {
            Some(sig) => sig,
            None => return false,
        };

        // Verify the signature
        Wallet::verify(&self.sender.0, self.hash.as_bytes(), signature).unwrap_or(false)
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

        // Sign the transaction
        tx.sign(&wallet).unwrap();
        assert!(tx.is_valid());

        // Try to sign with wrong wallet
        let wrong_wallet = Wallet::new().unwrap();
        assert!(tx.sign(&wrong_wallet).is_err());
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

        // Sign the transaction
        tx.sign(&wallet).unwrap();
        assert!(tx.is_valid());

        // Tamper with the amount
        tx.amount = 100.0;
        assert!(!tx.is_valid());
    }
}
