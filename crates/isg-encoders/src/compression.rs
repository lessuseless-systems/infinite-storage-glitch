//! Raw compression encoder
//!
//! For platforms that store data without re-encoding (like local storage or R2),
//! we can just use compression without converting to video.

use async_trait::async_trait;
use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use isg_core::{
    CompressionCodec, EncodedData, Encoder, EncodingMetadata, EncodingStrategy, Error, Result,
};
use std::io::{Read, Write};
use tracing::debug;

/// Compression-based encoder
#[derive(Clone, Debug)]
pub struct CompressionEncoder {
    codec: CompressionCodec,
}

impl CompressionEncoder {
    /// Create with zstd compression
    pub fn zstd(level: i32) -> Self {
        Self {
            codec: CompressionCodec::Zstd { level },
        }
    }

    /// Create with gzip compression
    pub fn gzip(level: u32) -> Self {
        Self {
            codec: CompressionCodec::Gzip { level },
        }
    }

    /// Create with no compression (passthrough)
    pub fn none() -> Self {
        Self {
            codec: CompressionCodec::None,
        }
    }

    /// Compress data with the configured codec
    fn compress(&self, data: &[u8]) -> Result<Vec<u8>> {
        match &self.codec {
            CompressionCodec::Zstd { level } => {
                zstd::encode_all(data, *level)
                    .map_err(|e| Error::Encoding(format!("Zstd compression failed: {}", e)))
            }

            CompressionCodec::Gzip { level } => {
                let mut encoder = GzEncoder::new(Vec::new(), Compression::new(*level));
                encoder
                    .write_all(data)
                    .map_err(|e| Error::Encoding(format!("Gzip compression failed: {}", e)))?;
                encoder
                    .finish()
                    .map_err(|e| Error::Encoding(format!("Gzip finish failed: {}", e)))
            }

            CompressionCodec::Brotli { level } => {
                let mut output = Vec::new();
                let mut writer = brotli::CompressorWriter::new(&mut output, 4096, *level, 22);
                writer
                    .write_all(data)
                    .map_err(|e| Error::Encoding(format!("Brotli compression failed: {}", e)))?;
                drop(writer);
                Ok(output)
            }

            CompressionCodec::None => Ok(data.to_vec()),
        }
    }

    /// Decompress data
    fn decompress(&self, data: &[u8]) -> Result<Vec<u8>> {
        match &self.codec {
            CompressionCodec::Zstd { .. } => zstd::decode_all(data)
                .map_err(|e| Error::Decoding(format!("Zstd decompression failed: {}", e))),

            CompressionCodec::Gzip { .. } => {
                let mut decoder = GzDecoder::new(data);
                let mut decompressed = Vec::new();
                decoder
                    .read_to_end(&mut decompressed)
                    .map_err(|e| Error::Decoding(format!("Gzip decompression failed: {}", e)))?;
                Ok(decompressed)
            }

            CompressionCodec::Brotli { .. } => {
                let mut output = Vec::new();
                let mut reader = brotli::Decompressor::new(data, 4096);
                reader
                    .read_to_end(&mut output)
                    .map_err(|e| Error::Decoding(format!("Brotli decompression failed: {}", e)))?;
                Ok(output)
            }

            CompressionCodec::None => Ok(data.to_vec()),
        }
    }
}

impl Default for CompressionEncoder {
    fn default() -> Self {
        // Zstd level 3 is a good balance of speed and compression
        Self::zstd(3)
    }
}

#[async_trait]
impl Encoder for CompressionEncoder {
    async fn encode(&self, data: &[u8]) -> Result<EncodedData> {
        debug!(
            "Compressing {} bytes with {:?}",
            data.len(),
            self.codec
        );

        let compressed = self.compress(data)?;

        let metadata = EncodingMetadata {
            original_size: data.len(),
            encoded_size: compressed.len(),
            compression_ratio: compressed.len() as f64 / data.len() as f64,
            strategy: "compression".to_string(),
            parameters: serde_json::json!({
                "codec": match &self.codec {
                    CompressionCodec::Zstd { level } => format!("zstd-{}", level),
                    CompressionCodec::Gzip { level } => format!("gzip-{}", level),
                    CompressionCodec::Brotli { level } => format!("brotli-{}", level),
                    CompressionCodec::None => "none".to_string(),
                }
            }),
        };

        debug!(
            "Compressed to {} bytes ({:.2}% of original)",
            compressed.len(),
            metadata.compression_ratio * 100.0
        );

        Ok(EncodedData {
            data: compressed,
            format: "compressed".to_string(),
            metadata,
        })
    }

    async fn decode(&self, encoded: &EncodedData) -> Result<Vec<u8>> {
        debug!("Decompressing {} bytes", encoded.data.len());
        self.decompress(&encoded.data)
    }

    fn strategy(&self) -> &EncodingStrategy {
        static STRATEGY: once_cell::sync::Lazy<EncodingStrategy> =
            once_cell::sync::Lazy::new(|| {
                EncodingStrategy::RawCompressed {
                    codec: CompressionCodec::Zstd { level: 3 },
                }
            });
        &STRATEGY
    }

    fn estimate_size(&self, input_size: usize) -> usize {
        // Rough estimates based on typical compression ratios
        match &self.codec {
            CompressionCodec::Zstd { .. } => input_size / 3,  // ~3:1 ratio
            CompressionCodec::Gzip { .. } => input_size / 2,  // ~2:1 ratio
            CompressionCodec::Brotli { .. } => input_size / 3, // ~3:1 ratio
            CompressionCodec::None => input_size,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_zstd_roundtrip() {
        let encoder = CompressionEncoder::zstd(3);
        let data = b"Hello, ISG! ".repeat(100); // Compressible data

        let encoded = encoder.encode(&data).await.unwrap();
        println!(
            "Zstd: {} bytes -> {} bytes ({:.2}%)",
            data.len(),
            encoded.data.len(),
            (encoded.data.len() as f64 / data.len() as f64) * 100.0
        );

        let decoded = encoder.decode(&encoded).await.unwrap();
        assert_eq!(data.as_slice(), decoded.as_slice());
    }

    #[tokio::test]
    async fn test_gzip_roundtrip() {
        let encoder = CompressionEncoder::gzip(6);
        let data = b"Test data for gzip compression!".repeat(50);

        let encoded = encoder.encode(&data).await.unwrap();
        let decoded = encoder.decode(&encoded).await.unwrap();

        assert_eq!(data.as_slice(), decoded.as_slice());
    }

    #[tokio::test]
    async fn test_no_compression() {
        let encoder = CompressionEncoder::none();
        let data = b"No compression applied";

        let encoded = encoder.encode(data).await.unwrap();
        assert_eq!(encoded.data.len(), data.len());

        let decoded = encoder.decode(&encoded).await.unwrap();
        assert_eq!(data.as_slice(), decoded.as_slice());
    }
}
