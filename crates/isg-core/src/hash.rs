//! Content-addressable hashing using SHA-256

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt;

/// A SHA-256 hash used for content addressing
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Hash([u8; 32]);

impl Hash {
    /// Compute hash from data
    pub fn from_data(data: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        Self(hash)
    }

    /// Create hash from raw bytes
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Get raw bytes
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Convert to hex string
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    /// Parse from hex string
    pub fn from_hex(s: &str) -> crate::Result<Self> {
        let bytes = hex::decode(s)
            .map_err(|e| crate::Error::InvalidHash(format!("Invalid hex: {}", e)))?;

        if bytes.len() != 32 {
            return Err(crate::Error::InvalidHash(format!(
                "Expected 32 bytes, got {}",
                bytes.len()
            )));
        }

        let mut hash = [0u8; 32];
        hash.copy_from_slice(&bytes);
        Ok(Self(hash))
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl fmt::Debug for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Hash({})", &self.to_hex()[..16])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_from_data() {
        let data = b"hello world";
        let hash1 = Hash::from_data(data);
        let hash2 = Hash::from_data(data);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_hex() {
        let data = b"test";
        let hash = Hash::from_data(data);
        let hex = hash.to_hex();
        let parsed = Hash::from_hex(&hex).unwrap();
        assert_eq!(hash, parsed);
    }
}
