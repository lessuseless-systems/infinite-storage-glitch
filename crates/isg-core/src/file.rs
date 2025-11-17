//! File representation with Merkle tree structure

use crate::Hash;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// A file stored in ISG
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct File {
    /// File path (logical path in ISG filesystem)
    pub path: PathBuf,

    /// Root hash of Merkle tree
    pub root_hash: Hash,

    /// Block hashes (leaf nodes of Merkle tree)
    pub block_hashes: Vec<Hash>,

    /// File metadata
    pub metadata: FileMetadata,
}

impl File {
    /// Create a new file
    pub fn new(path: PathBuf, block_hashes: Vec<Hash>, metadata: FileMetadata) -> Self {
        let root_hash = Self::compute_merkle_root(&block_hashes);

        Self {
            path,
            root_hash,
            block_hashes,
            metadata,
        }
    }

    /// Compute Merkle root from block hashes
    pub fn compute_merkle_root(hashes: &[Hash]) -> Hash {
        if hashes.is_empty() {
            return Hash::from_data(b"");
        }

        if hashes.len() == 1 {
            return hashes[0];
        }

        // Build Merkle tree bottom-up
        let mut current_level: Vec<Hash> = hashes.to_vec();

        while current_level.len() > 1 {
            let mut next_level = Vec::new();

            for chunk in current_level.chunks(2) {
                let combined = if chunk.len() == 2 {
                    // Combine two hashes
                    let mut data = Vec::new();
                    data.extend_from_slice(chunk[0].as_bytes());
                    data.extend_from_slice(chunk[1].as_bytes());
                    Hash::from_data(&data)
                } else {
                    // Odd one out, hash it with itself
                    let mut data = Vec::new();
                    data.extend_from_slice(chunk[0].as_bytes());
                    data.extend_from_slice(chunk[0].as_bytes());
                    Hash::from_data(&data)
                };

                next_level.push(combined);
            }

            current_level = next_level;
        }

        current_level[0]
    }

    /// Verify file integrity
    pub fn verify(&self) -> bool {
        Self::compute_merkle_root(&self.block_hashes) == self.root_hash
    }

    /// Get total size
    pub fn size(&self) -> u64 {
        self.metadata.size
    }

    /// Get number of blocks
    pub fn block_count(&self) -> usize {
        self.block_hashes.len()
    }
}

/// File metadata
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FileMetadata {
    /// Total file size in bytes
    pub size: u64,

    /// MIME type (if known)
    pub mime_type: Option<String>,

    /// Created timestamp
    pub created_at: DateTime<Utc>,

    /// Modified timestamp
    pub modified_at: DateTime<Utc>,

    /// Last accessed timestamp
    pub accessed_at: DateTime<Utc>,

    /// Unix permissions (if applicable)
    pub permissions: Option<u32>,

    /// Original file path (before adding to ISG)
    pub original_path: Option<PathBuf>,

    /// Custom user metadata
    pub user_metadata: serde_json::Value,
}

impl Default for FileMetadata {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            size: 0,
            mime_type: None,
            created_at: now,
            modified_at: now,
            accessed_at: now,
            permissions: None,
            original_path: None,
            user_metadata: serde_json::Value::Null,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merkle_root_single() {
        let hash = Hash::from_data(b"test");
        let hashes = vec![hash];
        let root = File::compute_merkle_root(&hashes);
        assert_eq!(root, hash);
    }

    #[test]
    fn test_merkle_root_multiple() {
        let hash1 = Hash::from_data(b"block1");
        let hash2 = Hash::from_data(b"block2");
        let hashes = vec![hash1, hash2];

        let root = File::compute_merkle_root(&hashes);

        // Root should be different from individual hashes
        assert_ne!(root, hash1);
        assert_ne!(root, hash2);

        // Should be deterministic
        let root2 = File::compute_merkle_root(&hashes);
        assert_eq!(root, root2);
    }

    #[test]
    fn test_file_verification() {
        let hash1 = Hash::from_data(b"block1");
        let hash2 = Hash::from_data(b"block2");
        let hashes = vec![hash1, hash2];

        let file = File::new(
            PathBuf::from("/test/file.txt"),
            hashes.clone(),
            FileMetadata::default(),
        );

        assert!(file.verify());

        // Modify root hash - should fail verification
        let mut file_corrupted = file.clone();
        file_corrupted.root_hash = Hash::from_data(b"wrong");
        assert!(!file_corrupted.verify());
    }
}
