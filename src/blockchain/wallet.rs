// NOTE: This file contains only the types needed for the blockchain to verify transactions.
// The actual wallet functionality (key generation, signing) will be implemented in a separate application.

use ed25519_dalek::{Signature, VerifyingKey as PublicKey};
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;
use utoipa::ToSchema;

/// Errors that can occur when working with wallet-related functionality
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

/// Represents a digital signature for a transaction
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TransactionSignature(pub String);

impl TransactionSignature {
    /// Converts the signature to ed25519 signature
    pub fn to_ed25519_signature(&self) -> Result<Signature, WalletError> {
        let bytes =
            hex::decode(&self.0).map_err(|e| WalletError::InvalidKeyFormat(e.to_string()))?;

        let signature = Signature::try_from(bytes.as_slice())
            .map_err(|e| WalletError::InvalidKeyFormat(e.to_string()))?;

        Ok(signature)
    }
}

/// Verifies a signature against a message and public key
pub fn verify_signature(
    public_key_hex: &str,
    message: &[u8],
    signature: &TransactionSignature,
) -> Result<bool, WalletError> {
    let public_bytes =
        hex::decode(public_key_hex).map_err(|e| WalletError::InvalidKeyFormat(e.to_string()))?;

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
