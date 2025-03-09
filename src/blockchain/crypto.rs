// NOTE: This file contains only the types needed for the blockchain to verify transactions.
// The actual wallet functionality (key generation, signing) will be implemented in a separate application.

use ed25519_dalek::{Signature, VerifyingKey as PublicKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt;
use thiserror::Error;
use utoipa::ToSchema;

/// Errors that can occur when working with cryptographic functionality
#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("Invalid key format: {0}")]
    InvalidKeyFormat(String),
}

/// Represents a blockchain address (hash of a public key)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct Address(pub String);

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Represents a public key in the blockchain
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PublicKeyHex(pub String);

impl PublicKeyHex {
    /// Converts the hex string to ed25519 public key
    pub fn to_ed25519_public_key(&self) -> Result<PublicKey, CryptoError> {
        let public_bytes =
            hex::decode(&self.0).map_err(|e| CryptoError::InvalidKeyFormat(e.to_string()))?;

        let public_key_bytes: [u8; 32] = public_bytes
            .try_into()
            .map_err(|_| CryptoError::InvalidKeyFormat("Invalid public key length".to_string()))?;

        PublicKey::from_bytes(&public_key_bytes)
            .map_err(|e| CryptoError::InvalidKeyFormat(e.to_string()))
    }

    /// Derives an address from this public key (hash of the public key)
    pub fn to_address(&self) -> Result<Address, CryptoError> {
        let public_bytes =
            hex::decode(&self.0).map_err(|e| CryptoError::InvalidKeyFormat(e.to_string()))?;

        // Hash the public key using SHA-256
        let mut hasher = Sha256::new();
        hasher.update(&public_bytes);
        let hash = hasher.finalize();

        // Take the first 20 bytes of the hash (similar to Bitcoin's RIPEMD160 after SHA256)
        // In a real implementation, you might want to use RIPEMD160 after SHA256
        let address = hex::encode(&hash[..20]);

        Ok(Address(address))
    }
}

/// Represents a digital signature for a transaction
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TransactionSignature(pub String);

impl TransactionSignature {
    /// Converts the signature to ed25519 signature
    pub fn to_ed25519_signature(&self) -> Result<Signature, CryptoError> {
        let bytes =
            hex::decode(&self.0).map_err(|e| CryptoError::InvalidKeyFormat(e.to_string()))?;

        let signature = Signature::try_from(bytes.as_slice())
            .map_err(|e| CryptoError::InvalidKeyFormat(e.to_string()))?;

        Ok(signature)
    }
}

/// Verifies a signature against a message using the provided public key
pub fn verify_signature(
    public_key: &PublicKeyHex,
    message: &[u8],
    signature: &TransactionSignature,
) -> Result<bool, CryptoError> {
    let ed25519_public_key = public_key.to_ed25519_public_key()?;
    let ed25519_signature = signature.to_ed25519_signature()?;

    match ed25519_public_key.verify_strict(message, &ed25519_signature) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// Verifies that an address was derived from the given public key
pub fn verify_address(public_key: &PublicKeyHex, address: &Address) -> Result<bool, CryptoError> {
    let derived_address = public_key.to_address()?;
    Ok(derived_address.0 == address.0)
}
