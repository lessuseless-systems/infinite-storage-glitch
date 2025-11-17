//! Content-addressable blocks

use crate::{Hash, Location};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A content-addressable block of data
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Block {
    /// Content hash (SHA-256) - used as unique identifier
    pub hash: Hash,

    /// Raw data (may be encrypted/compressed)
    #[serde(skip)]
    pub data: Vec<u8>,

    /// Size in bytes
    pub size: usize,

    /// Metadata about the block
    pub metadata: BlockMetadata,

    /// Storage locations where this block is stored
    pub locations: Vec<Location>,
}

impl Block {
    /// Create a new block from data with metadata
    pub fn new(data: Vec<u8>, metadata: BlockMetadata) -> Self {
        let hash = Hash::from_data(&data);
        let size = data.len();

        Self {
            hash,
            data,
            size,
            metadata,
            locations: Vec::new(),
        }
    }

    /// Create a new block from data with default metadata
    pub fn from_data(data: Vec<u8>) -> Self {
        Self::new(data, BlockMetadata::default())
    }

    /// Get the block's hash
    pub fn hash(&self) -> &Hash {
        &self.hash
    }

    /// Get the block's data
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Add a storage location
    pub fn add_location(&mut self, location: Location) {
        if !self.locations.contains(&location) {
            self.locations.push(location);
        }
    }

    /// Verify data integrity
    pub fn verify(&self) -> bool {
        Hash::from_data(&self.data) == self.hash
    }
}

/// Metadata about a block
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlockMetadata {
    /// Encoding strategy used
    pub encoding: String,

    /// Whether this is a parity block (for erasure coding)
    pub is_parity: bool,

    /// Compression applied (if any)
    pub compression: Option<String>,

    /// Encryption applied (if any)
    pub encryption: Option<String>,

    /// When the block was created
    pub created_at: DateTime<Utc>,
}

impl Default for BlockMetadata {
    fn default() -> Self {
        Self {
            encoding: "raw".to_string(),
            is_parity: false,
            compression: None,
            encryption: None,
            created_at: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_creation() {
        let data = b"test data".to_vec();
        let metadata = BlockMetadata::default();
        let block = Block::new(data.clone(), metadata);

        assert_eq!(block.data, data);
        assert_eq!(block.size, data.len());
        assert!(block.verify());
    }

    #[test]
    fn test_block_verification() {
        let data = b"test data".to_vec();
        let metadata = BlockMetadata::default();
        let mut block = Block::new(data, metadata);

        assert!(block.verify());

        // Corrupt the data
        block.data[0] ^= 1;
        assert!(!block.verify());
    }
}
