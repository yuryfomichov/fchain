use ed25519_dalek::SigningKey as SecretKey;
use ed25519_dalek::VerifyingKey as PublicKey;
use ed25519_dalek::{Signature, Signer};
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;
use utoipa::ToSchema;

/// Errors that can occur when working with wallets
#[derive(Debug, Error)]
pub enum WalletError {
    #[error("Invalid key format: {0}")]
    InvalidKeyFormat(String),
}

/// Represents a blockchain address
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct Address(pub String);

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Address {
    /// Creates a new address from a public key
    pub fn from_public_key(public_key: &PublicKey) -> Self {
        let public_key_bytes = public_key.to_bytes();
        Self(hex::encode(public_key_bytes))
    }

    /// Checks if the address is valid
    pub fn is_valid(&self) -> bool {
        // For now, just check if it's not empty and has the right length for a hex-encoded public key
        !self.0.is_empty() && (self.0 == "system" || self.0.len() == 64)
    }
}

/// Represents a digital signature for a transaction
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TransactionSignature(pub String);

impl TransactionSignature {
    /// Creates a new signature from ed25519 signature
    pub fn from_ed25519_signature(signature: &Signature) -> Self {
        Self(hex::encode(signature.to_bytes()))
    }

    /// Converts the signature to ed25519 signature
    pub fn to_ed25519_signature(&self) -> Result<Signature, WalletError> {
        let bytes =
            hex::decode(&self.0).map_err(|e| WalletError::InvalidKeyFormat(e.to_string()))?;

        let signature = Signature::try_from(bytes.as_slice())
            .map_err(|e| WalletError::InvalidKeyFormat(e.to_string()))?;

        Ok(signature)
    }
}

/// Represents a wallet for the blockchain
#[derive(Debug)]
pub struct Wallet {
    secret_key: SecretKey,
    public_key: PublicKey,
    address: Address,
}

impl Wallet {
    /// Creates a new wallet with a randomly generated keypair
    pub fn new() -> Result<Self, WalletError> {
        // Create a random secret key using OsRng
        let mut csprng = OsRng;
        let mut secret_key_bytes = [0u8; 32];
        csprng.fill_bytes(&mut secret_key_bytes);

        let secret_key = SecretKey::from_bytes(&secret_key_bytes);
        let public_key = secret_key.verifying_key();
        let address = Address::from_public_key(&public_key);

        Ok(Self {
            secret_key,
            public_key,
            address,
        })
    }

    /// Creates a wallet from an existing secret key
    pub fn from_secret_key(secret_key_hex: &str) -> Result<Self, WalletError> {
        let secret_bytes = hex::decode(secret_key_hex)
            .map_err(|e| WalletError::InvalidKeyFormat(e.to_string()))?;

        let secret_key_bytes: [u8; 32] = secret_bytes
            .try_into()
            .map_err(|_| WalletError::InvalidKeyFormat("Invalid secret key length".to_string()))?;

        let secret_key = SecretKey::from_bytes(&secret_key_bytes);
        let public_key = secret_key.verifying_key();
        let address = Address::from_public_key(&public_key);

        Ok(Self {
            secret_key,
            public_key,
            address,
        })
    }

    /// Gets the wallet's address
    pub fn get_address(&self) -> &Address {
        &self.address
    }

    /// Gets the wallet's public key as a hex string
    pub fn get_public_key_hex(&self) -> String {
        hex::encode(self.public_key.to_bytes())
    }

    /// Gets the wallet's secret key as a hex string
    pub fn get_secret_key_hex(&self) -> String {
        hex::encode(self.secret_key.to_bytes())
    }

    /// Signs a message with the wallet's private key
    pub fn sign(&self, message: &[u8]) -> Result<TransactionSignature, WalletError> {
        let signature = self.secret_key.sign(message);
        Ok(TransactionSignature::from_ed25519_signature(&signature))
    }

    /// Verifies a signature with the wallet's public key
    pub fn verify(
        public_key_hex: &str,
        message: &[u8],
        signature: &TransactionSignature,
    ) -> Result<bool, WalletError> {
        let public_bytes = hex::decode(public_key_hex)
            .map_err(|e| WalletError::InvalidKeyFormat(e.to_string()))?;

        let public_key_bytes: [u8; 32] = public_bytes
            .try_into()
            .map_err(|_| WalletError::InvalidKeyFormat("Invalid public key length".to_string()))?;

        let public_key = PublicKey::from_bytes(&public_key_bytes)
            .map_err(|e| WalletError::InvalidKeyFormat(e.to_string()))?;

        let ed25519_signature = signature.to_ed25519_signature()?;

        match public_key.verify_strict(message, &ed25519_signature) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallet_creation() {
        let wallet = Wallet::new().unwrap();
        assert!(wallet.get_address().is_valid());
    }

    #[test]
    fn test_wallet_from_secret_key() {
        let wallet1 = Wallet::new().unwrap();
        let secret_key_hex = wallet1.get_secret_key_hex();

        let wallet2 = Wallet::from_secret_key(&secret_key_hex).unwrap();
        assert_eq!(wallet1.get_address(), wallet2.get_address());
    }

    #[test]
    fn test_signing_and_verification() {
        let wallet = Wallet::new().unwrap();
        let message = b"Hello, blockchain!";

        let signature = wallet.sign(message).unwrap();
        let public_key_hex = wallet.get_public_key_hex();

        let is_valid = Wallet::verify(&public_key_hex, message, &signature).unwrap();
        assert!(is_valid);

        // Test with wrong message
        let wrong_message = b"Wrong message";
        let is_valid = Wallet::verify(&public_key_hex, wrong_message, &signature).unwrap();
        assert!(!is_valid);
    }
}
