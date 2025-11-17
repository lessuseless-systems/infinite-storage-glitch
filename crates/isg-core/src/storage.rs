//! Storage backend trait and types

use crate::{Block, Hash, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A location where a block is stored
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Location {
    /// Platform name (e.g., "youtube", "discord", "local")
    pub platform: String,

    /// Platform-specific identifier (e.g., video ID, webhook URL, file path)
    pub identifier: String,

    /// Additional metadata
    pub metadata: StorageMetadata,
}

/// Metadata about storage
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorageMetadata {
    /// When it was uploaded
    pub uploaded_at: DateTime<Utc>,

    /// URL or path to access
    pub url: Option<String>,

    /// Size as stored on platform
    pub stored_size: usize,

    /// Additional platform-specific data
    pub extra: serde_json::Value,
}

impl Default for StorageMetadata {
    fn default() -> Self {
        Self {
            uploaded_at: Utc::now(),
            url: None,
            stored_size: 0,
            extra: serde_json::Value::Null,
        }
    }
}

/// Storage tier for intelligent tiering
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum StorageTier {
    /// Hot: Fast local storage (SSD, RAM cache)
    Hot,
    /// Warm: Medium-speed cloud storage (Discord, Telegram, R2)
    Warm,
    /// Cold: Slow but cheap storage (YouTube, IPFS, Archive.org)
    Cold,
}

/// Upload progress callback
pub type ProgressCallback = Box<dyn Fn(usize, usize) + Send + Sync>;

/// Trait for storage backends
#[async_trait]
pub trait StorageBackend: Send + Sync {
    /// Platform name
    fn name(&self) -> &str;

    /// Storage tier
    fn tier(&self) -> StorageTier;

    /// Upload a block
    async fn upload(&self, block: &Block) -> Result<Location>;

    /// Upload with progress callback
    async fn upload_with_progress(
        &self,
        block: &Block,
        progress: ProgressCallback,
    ) -> Result<Location> {
        // Default implementation ignores progress
        let _ = progress;
        self.upload(block).await
    }

    /// Download a block
    async fn download(&self, location: &Location) -> Result<Vec<u8>>;

    /// Download with progress callback
    async fn download_with_progress(
        &self,
        location: &Location,
        progress: ProgressCallback,
    ) -> Result<Vec<u8>> {
        // Default implementation ignores progress
        let _ = progress;
        self.download(location).await
    }

    /// Delete a block
    async fn delete(&self, location: &Location) -> Result<()>;

    /// List all stored blocks
    async fn list(&self) -> Result<Vec<Location>>;

    /// Check if a block exists
    async fn exists(&self, location: &Location) -> Result<bool> {
        // Default implementation tries to download metadata
        match self.download(location).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Get storage statistics
    async fn stats(&self) -> Result<StorageStats> {
        // Default implementation
        Ok(StorageStats::default())
    }

    /// Verify block integrity at location
    async fn verify(&self, location: &Location, expected_hash: &Hash) -> Result<bool> {
        let data = self.download(location).await?;
        Ok(Hash::from_data(&data) == *expected_hash)
    }
}

/// Storage statistics
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct StorageStats {
    /// Total blocks stored
    pub total_blocks: usize,

    /// Total bytes stored
    pub total_bytes: usize,

    /// Available space (if applicable)
    pub available_bytes: Option<usize>,

    /// Platform-specific stats
    pub extra: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_location_serialization() {
        let location = Location {
            platform: "youtube".to_string(),
            identifier: "abc123".to_string(),
            metadata: StorageMetadata::default(),
        };

        let json = serde_json::to_string(&location).unwrap();
        let deserialized: Location = serde_json::from_str(&json).unwrap();

        assert_eq!(location, deserialized);
    }

    #[test]
    fn test_storage_tier() {
        assert_ne!(StorageTier::Hot, StorageTier::Cold);
    }
}
