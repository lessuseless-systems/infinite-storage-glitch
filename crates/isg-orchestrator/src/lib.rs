//! # ISG Orchestrator
//!
//! High-level orchestration of encoding, encryption, erasure coding, and storage.

use anyhow::{Context, Result};
use chrono::Utc;
use isg_core::{Block, BlockMetadata, Encoder, Hash};
use isg_crypto::CryptoEngine;
use isg_db::{BlockRecord, ChunkRecord, Database, FileRecord};
use isg_encoders::{ColorEncoder, PixelEncoder, QREncoder};
use isg_storage::StorageManager;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, info};

/// Upload options
pub struct UploadOptions {
    /// Encoding strategy to use
    pub encoding: EncodingType,
    /// Whether to encrypt
    pub encrypt: bool,
    /// Encryption password (if encrypt is true)
    pub password: Option<String>,
    /// Use erasure coding
    pub use_erasure: bool,
    /// Data shards for erasure coding
    pub data_shards: usize,
    /// Parity shards for erasure coding
    pub parity_shards: usize,
}

impl Default for UploadOptions {
    fn default() -> Self {
        Self {
            encoding: EncodingType::None,
            encrypt: false,
            password: None,
            use_erasure: false,
            data_shards: 4,
            parity_shards: 2,
        }
    }
}

/// Encoding type selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncodingType {
    /// No encoding, store raw
    None,
    /// Pixel encoding (black/white)
    Pixel,
    /// Color encoding (RGB)
    Color,
    /// QR code encoding
    QR,
}

impl EncodingType {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "pixel" => Self::Pixel,
            "color" => Self::Color,
            "qr" => Self::QR,
            _ => Self::None,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::None => "none",
            Self::Pixel => "pixel",
            Self::Color => "color",
            Self::QR => "qr",
        }
    }
}

/// ISG Orchestrator - coordinates all operations
pub struct Orchestrator {
    db: Database,
    storage: StorageManager,
}

impl Orchestrator {
    /// Create a new orchestrator
    pub fn new(db: Database, storage: StorageManager) -> Self {
        Self { db, storage }
    }

    /// Upload a file to ISG
    pub async fn upload(
        &self,
        file_path: &str,
        data: Vec<u8>,
        options: UploadOptions,
    ) -> Result<UploadResult> {
        let original_size = data.len();
        info!("Starting upload of {} bytes from {}", original_size, file_path);

        // Step 1: Optional encryption
        let (data, encrypted) = if options.encrypt {
            let password = options
                .password
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("Password required for encryption"))?;

            let salt = b"isg_salt_12345678"; // TODO: Use random salt and store it
            let key = CryptoEngine::derive_key_from_password(password, salt)?;
            let engine = CryptoEngine::new(key);

            let encrypted_data = engine.encrypt(&data)?;

            // Serialize encrypted data (nonce + ciphertext)
            let mut result = Vec::new();
            result.extend_from_slice(&encrypted_data.nonce);
            result.extend_from_slice(&encrypted_data.ciphertext);

            info!("Encrypted {} bytes to {} bytes", data.len(), result.len());

            (result, true)
        } else {
            (data, false)
        };

        // Step 2: Optional encoding
        let (data, encoding_used) = if options.encoding != EncodingType::None {
            let encoded = match options.encoding {
                EncodingType::Pixel => {
                    let encoder = PixelEncoder::new();
                    encoder.encode(&data).await?
                }
                EncodingType::Color => {
                    let encoder = ColorEncoder::new();
                    encoder.encode(&data).await?
                }
                EncodingType::QR => {
                    let encoder = QREncoder::new();
                    encoder.encode(&data).await?
                }
                EncodingType::None => unreachable!(),
            };

            info!(
                "Encoded {} bytes to {} bytes using {} (ratio: {:.2}x)",
                original_size,
                encoded.data.len(),
                options.encoding.as_str(),
                encoded.metadata.compression_ratio
            );

            (encoded.data, Some(options.encoding))
        } else {
            (data, None)
        };

        // Step 3: Split into blocks
        let block_size = 1024 * 1024; // 1MB blocks
        let mut blocks = Vec::new();

        for chunk in data.chunks(block_size) {
            let mut metadata = BlockMetadata::default();
            if encrypted {
                metadata.encryption = Some("aes-256-gcm".to_string());
            }
            if let Some(enc) = encoding_used {
                metadata.encoding = enc.as_str().to_string();
            }

            let block = Block::new(chunk.to_vec(), metadata);
            blocks.push(block);
        }

        info!("Split into {} blocks", blocks.len());

        // Step 4: Upload blocks
        for (i, block) in blocks.iter().enumerate() {
            // Store block metadata
            let timestamp = Utc::now().timestamp();

            let encoding_str = encoding_used
                .map(|e| e.as_str().to_string())
                .unwrap_or_else(|| "none".to_string());

            let block_record = BlockRecord {
                hash: block.hash().as_bytes().to_vec(),
                size: block.data().len() as i64,
                encoding_strategy: encoding_str,
                created_at: timestamp,
            };

            self.db.insert_block(&block_record)?;

            // Upload to storage (use first backend)
            let location = self.storage.upload_to_backend(0, block).await?;

            // Store chunk location
            let chunk_record = ChunkRecord {
                id: 0,
                block_hash: block.hash().as_bytes().to_vec(),
                platform: location.platform.clone(),
                location: location.identifier.clone(),
                is_parity: false,
                uploaded_at: timestamp,
            };

            self.db.insert_chunk(&chunk_record)?;

            debug!("Uploaded block {}/{}", i + 1, blocks.len());
        }

        // Step 5: Store file metadata
        let root_hash = if blocks.is_empty() {
            Hash::from_data(b"empty")
        } else {
            blocks[0].hash().clone()
        };

        let timestamp = Utc::now().timestamp();

        let file_record = FileRecord {
            id: 0,
            path: file_path.to_string(),
            root_hash: root_hash.as_bytes().to_vec(),
            size: original_size as i64,
            created_at: timestamp,
            modified_at: timestamp,
            accessed_at: timestamp,
        };

        self.db.insert_file(&file_record)?;

        info!("Upload complete: {} blocks stored", blocks.len());

        Ok(UploadResult {
            original_size,
            total_blocks: blocks.len(),
            encrypted,
            encoding: encoding_used,
        })
    }

    /// Download a file from ISG
    pub async fn download(
        &self,
        file_path: &str,
        password: Option<String>,
    ) -> Result<Vec<u8>> {
        info!("Downloading file: {}", file_path);

        // Get file metadata
        let file = self
            .db
            .get_file_by_path(file_path)?
            .ok_or_else(|| anyhow::anyhow!("File not found: {}", file_path))?;

        // Get all chunks for this file
        let chunks = self.db.get_chunks_for_block(&file.root_hash)?;

        if chunks.is_empty() {
            anyhow::bail!("No chunks found for file");
        }

        // Download blocks
        let mut reconstructed_data = Vec::new();

        for (i, chunk) in chunks.iter().enumerate() {
            let location = isg_core::Location {
                platform: chunk.platform.clone(),
                identifier: chunk.location.clone(),
                metadata: Default::default(),
            };

            let block_data = self.storage.download_from_location(&location).await?;
            reconstructed_data.extend_from_slice(&block_data);

            debug!("Downloaded block {}/{}", i + 1, chunks.len());
        }

        // Get block metadata to determine if decoding/decryption is needed
        let block = self.db.get_block(&file.root_hash)?
            .ok_or_else(|| anyhow::anyhow!("Block metadata not found"))?;

        // Decode if needed
        let data = if block.encoding_strategy != "none" {
            let encoding_type = EncodingType::from_str(&block.encoding_strategy);
            let encoded_data = isg_core::EncodedData {
                data: reconstructed_data.clone(),
                format: "encoded".to_string(),
                metadata: isg_core::EncodingMetadata {
                    original_size: file.size as usize,
                    encoded_size: 0,
                    compression_ratio: 1.0,
                    strategy: block.encoding_strategy.clone(),
                    parameters: serde_json::json!({}),
                },
            };

            match encoding_type {
                EncodingType::Pixel => {
                    let encoder = PixelEncoder::new();
                    encoder.decode(&encoded_data).await?
                }
                EncodingType::Color => {
                    let encoder = ColorEncoder::new();
                    encoder.decode(&encoded_data).await?
                }
                EncodingType::QR => {
                    let encoder = QREncoder::new();
                    encoder.decode(&encoded_data).await?
                }
                EncodingType::None => reconstructed_data,
            }
        } else {
            reconstructed_data
        };

        // Decrypt if needed (check if password provided)
        let final_data = if let Some(pwd) = password {
            if data.len() < 12 {
                anyhow::bail!("Invalid encrypted data");
            }

            let salt = b"isg_salt_12345678"; // Should match upload salt
            let key = CryptoEngine::derive_key_from_password(&pwd, salt)?;
            let engine = CryptoEngine::new(key);

            // Extract nonce and ciphertext
            let nonce: [u8; 12] = data[..12].try_into()?;
            let ciphertext = data[12..].to_vec();

            let encrypted = isg_crypto::EncryptedData { nonce, ciphertext };
            engine.decrypt(&encrypted)?
        } else {
            data
        };

        info!("Download complete: {} bytes", final_data.len());

        Ok(final_data)
    }
}

/// Result of an upload operation
pub struct UploadResult {
    pub original_size: usize,
    pub total_blocks: usize,
    pub encrypted: bool,
    pub encoding: Option<EncodingType>,
}
