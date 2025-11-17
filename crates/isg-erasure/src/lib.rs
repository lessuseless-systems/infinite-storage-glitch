//! # ISG Erasure
//!
//! Reed-Solomon erasure coding for the Infinite Storage Glitch system.
//!
//! This crate provides erasure coding capabilities to survive block loss:
//! - N data shards + K parity shards
//! - Can reconstruct original data from any N shards
//! - Self-healing architecture

use isg_core::{Error, Result};
use reed_solomon_erasure::galois_8::ReedSolomon;

/// Erasure coder using Reed-Solomon
pub struct ErasureCoder {
    data_shards: usize,
    parity_shards: usize,
    encoder: ReedSolomon,
}

impl ErasureCoder {
    /// Create a new erasure coder
    ///
    /// # Arguments
    /// * `data_shards` - Number of data shards (N)
    /// * `parity_shards` - Number of parity shards (K)
    ///
    /// With these settings, the original data can be reconstructed from any N shards
    pub fn new(data_shards: usize, parity_shards: usize) -> Result<Self> {
        let encoder = ReedSolomon::new(data_shards, parity_shards)
            .map_err(|e| Error::Erasure(format!("Failed to create Reed-Solomon encoder: {:?}", e)))?;

        Ok(Self {
            data_shards,
            parity_shards,
            encoder,
        })
    }

    /// Encode data into shards
    ///
    /// Returns a vector of shards (data shards + parity shards)
    pub fn encode(&self, data: &[u8]) -> Result<Vec<Vec<u8>>> {
        // Calculate shard size (round up)
        let shard_size = (data.len() + self.data_shards - 1) / self.data_shards;

        // Create data shards
        let mut shards: Vec<Vec<u8>> = Vec::with_capacity(self.data_shards + self.parity_shards);

        // Split data into shards
        for i in 0..self.data_shards {
            let start = i * shard_size;
            let end = ((i + 1) * shard_size).min(data.len());

            let mut shard = vec![0u8; shard_size];
            if start < data.len() {
                let slice = &data[start..end];
                shard[..slice.len()].copy_from_slice(slice);
            }
            shards.push(shard);
        }

        // Add empty parity shards
        for _ in 0..self.parity_shards {
            shards.push(vec![0u8; shard_size]);
        }

        // Encode to generate parity shards
        self.encoder
            .encode(&mut shards)
            .map_err(|e| Error::Erasure(format!("Failed to encode: {:?}", e)))?;

        Ok(shards)
    }

    /// Reconstruct data from shards (some may be missing)
    ///
    /// # Arguments
    /// * `shards` - Vector of shards, where None represents a missing shard
    /// * `original_size` - The original data size (before encoding)
    pub fn reconstruct(&self, mut shards: Vec<Option<Vec<u8>>>, original_size: usize) -> Result<Vec<u8>> {
        // Verify we have enough shards
        let available = shards.iter().filter(|s| s.is_some()).count();
        if available < self.data_shards {
            return Err(Error::Erasure(format!(
                "Not enough shards to reconstruct: have {}, need {}",
                available, self.data_shards
            )));
        }

        // Reconstruct missing shards
        self.encoder
            .reconstruct(&mut shards)
            .map_err(|e| Error::Erasure(format!("Failed to reconstruct: {:?}", e)))?;

        // Combine data shards
        let mut result = Vec::with_capacity(original_size);
        for i in 0..self.data_shards {
            if let Some(ref shard) = shards[i] {
                result.extend_from_slice(shard);
            } else {
                return Err(Error::Erasure("Reconstruction failed".to_string()));
            }
        }

        // Truncate to original size
        result.truncate(original_size);

        Ok(result)
    }

    pub fn data_shards(&self) -> usize {
        self.data_shards
    }

    pub fn parity_shards(&self) -> usize {
        self.parity_shards
    }

    pub fn total_shards(&self) -> usize {
        self.data_shards + self.parity_shards
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode() {
        let coder = ErasureCoder::new(4, 2).unwrap();
        let data = b"Hello, world! This is a test of erasure coding.";

        let shards = coder.encode(data).unwrap();
        assert_eq!(shards.len(), 6); // 4 data + 2 parity

        // Convert to Option format
        let shard_opts: Vec<Option<Vec<u8>>> = shards.into_iter().map(Some).collect();

        // Reconstruct
        let reconstructed = coder.reconstruct(shard_opts, data.len()).unwrap();
        assert_eq!(&reconstructed, data);
    }

    #[test]
    fn test_reconstruct_with_missing_shards() {
        let coder = ErasureCoder::new(4, 2).unwrap();
        let data = b"Hello, world! This is a test of erasure coding.";

        let shards = coder.encode(data).unwrap();

        // Simulate losing 2 shards (indices 1 and 3)
        let mut shard_opts: Vec<Option<Vec<u8>>> = shards.into_iter().map(Some).collect();
        shard_opts[1] = None;
        shard_opts[3] = None;

        // Should still be able to reconstruct (we have 4 out of 6 shards, need 4)
        let reconstructed = coder.reconstruct(shard_opts, data.len()).unwrap();
        assert_eq!(&reconstructed, data);
    }
}
