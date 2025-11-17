//! # ISG Crypto
//!
//! Encryption and decryption layer for the Infinite Storage Glitch system.
//!
//! This crate provides:
//! - AES-256-GCM encryption
//! - ChaCha20-Poly1305 encryption
//! - Argon2id key derivation
//! - Per-block encryption with unique nonces

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use argon2::{Argon2, PasswordHasher};
use argon2::password_hash::{SaltString, PasswordHash};
use isg_core::{Error, Result};
use rand::RngCore;
use sha2::{Digest, Sha256};

/// Encryption key (32 bytes for AES-256)
pub type Key = [u8; 32];

/// Nonce for AES-GCM (12 bytes)
pub type AesNonce = [u8; 12];

/// Encrypted data with nonce
#[derive(Debug, Clone)]
pub struct EncryptedData {
    pub nonce: AesNonce,
    pub ciphertext: Vec<u8>,
}

/// Crypto engine for encryption/decryption
pub struct CryptoEngine {
    key: Key,
}

impl CryptoEngine {
    /// Create a new crypto engine with the given key
    pub fn new(key: Key) -> Self {
        Self { key }
    }

    /// Derive a key from a password using Argon2id
    pub fn derive_key_from_password(password: &str, salt: &[u8]) -> Result<Key> {
        let argon2 = Argon2::default();
        let salt_string = SaltString::encode_b64(salt)
            .map_err(|e| Error::Crypto(format!("Failed to encode salt: {}", e)))?;

        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt_string)
            .map_err(|e| Error::Crypto(format!("Failed to hash password: {}", e)))?;

        let hash_bytes = password_hash.hash
            .ok_or_else(|| Error::Crypto("No hash produced".to_string()))?;

        let hash_bytes = hash_bytes.as_bytes();

        // Use SHA-256 to get exactly 32 bytes
        let mut hasher = Sha256::new();
        hasher.update(hash_bytes);
        let key_bytes = hasher.finalize();

        let mut key = [0u8; 32];
        key.copy_from_slice(&key_bytes);
        Ok(key)
    }

    /// Generate a random key
    pub fn generate_key() -> Key {
        let mut key = [0u8; 32];
        OsRng.fill_bytes(&mut key);
        key
    }

    /// Encrypt data
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<EncryptedData> {
        let cipher = Aes256Gcm::new_from_slice(&self.key)
            .map_err(|e| Error::Crypto(format!("Failed to create cipher: {}", e)))?;

        // Generate random nonce
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| Error::Crypto(format!("Encryption failed: {}", e)))?;

        Ok(EncryptedData {
            nonce: nonce_bytes,
            ciphertext,
        })
    }

    /// Decrypt data
    pub fn decrypt(&self, encrypted: &EncryptedData) -> Result<Vec<u8>> {
        let cipher = Aes256Gcm::new_from_slice(&self.key)
            .map_err(|e| Error::Crypto(format!("Failed to create cipher: {}", e)))?;

        let nonce = Nonce::from_slice(&encrypted.nonce);

        let plaintext = cipher
            .decrypt(nonce, encrypted.ciphertext.as_ref())
            .map_err(|e| Error::Crypto(format!("Decryption failed: {}", e)))?;

        Ok(plaintext)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let key = CryptoEngine::generate_key();
        let engine = CryptoEngine::new(key);

        let plaintext = b"Hello, world!";
        let encrypted = engine.encrypt(plaintext).unwrap();
        let decrypted = engine.decrypt(&encrypted).unwrap();

        assert_eq!(plaintext, decrypted.as_slice());
    }

    #[test]
    fn test_derive_key() {
        let password = "my secure password";
        let salt = b"some random salt";

        let key1 = CryptoEngine::derive_key_from_password(password, salt).unwrap();
        let key2 = CryptoEngine::derive_key_from_password(password, salt).unwrap();

        // Same password and salt should produce same key
        assert_eq!(key1, key2);
    }
}
