//! Encoding strategies and traits

use crate::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Encoded data representation
#[derive(Clone, Debug)]
pub struct EncodedData {
    /// The encoded data (could be video frames, images, etc.)
    pub data: Vec<u8>,

    /// Format/container type (e.g., "mp4", "png", "qr")
    pub format: String,

    /// Metadata about the encoding
    pub metadata: EncodingMetadata,
}

/// Metadata about encoded data
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EncodingMetadata {
    /// Original size before encoding
    pub original_size: usize,

    /// Encoded size
    pub encoded_size: usize,

    /// Compression ratio
    pub compression_ratio: f64,

    /// Strategy used
    pub strategy: String,

    /// Additional parameters
    pub parameters: serde_json::Value,
}

/// Encoding strategy enum
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum EncodingStrategy {
    /// Traditional black/white pixel encoding
    PixelEncoding {
        /// Block size (e.g., 2x2, 4x4, 10x10)
        block_size: u32,
        /// Frames per second
        fps: u32,
        /// Resolution
        resolution: (u32, u32),
    },

    /// RGB color-based encoding
    ColorEncoding {
        /// Color space to use
        color_space: ColorSpace,
    },

    /// QR code grid encoding
    QREncoding {
        /// QR code version
        version: u8,
        /// Error correction level
        ecc_level: ECCLevel,
    },

    /// Steganography - hide in existing media
    Steganography {
        /// Cover media path
        cover_media: String,
        /// Method to use
        method: StegoMethod,
    },

    /// Raw compression (for platforms that don't re-encode)
    RawCompressed {
        /// Compression codec
        codec: CompressionCodec,
    },

    /// DNA sequence encoding (experimental)
    DNAEncoding,

    /// Hybrid - use multiple strategies
    Hybrid {
        /// Strategies to use
        strategies: Vec<Box<EncodingStrategy>>,
    },
}

/// Color space options
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ColorSpace {
    RGB,
    YUV,
    HSV,
}

/// Error correction level for QR codes
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ECCLevel {
    Low,
    Medium,
    Quartile,
    High,
}

/// Steganography methods
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum StegoMethod {
    LSB,        // Least Significant Bit
    DCT,        // Discrete Cosine Transform
    DWT,        // Discrete Wavelet Transform
}

/// Compression codecs
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CompressionCodec {
    Zstd { level: i32 },
    Gzip { level: u32 },
    Brotli { level: u32 },
    None,
}

/// Trait for encoding/decoding data
#[async_trait]
pub trait Encoder: Send + Sync {
    /// Encode data
    async fn encode(&self, data: &[u8]) -> Result<EncodedData>;

    /// Decode data
    async fn decode(&self, encoded: &EncodedData) -> Result<Vec<u8>>;

    /// Get encoding strategy info
    fn strategy(&self) -> &EncodingStrategy;

    /// Estimate encoded size
    fn estimate_size(&self, input_size: usize) -> usize {
        // Default: assume no compression
        input_size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encoding_strategy_serialization() {
        let strategy = EncodingStrategy::PixelEncoding {
            block_size: 4,
            fps: 30,
            resolution: (1920, 1080),
        };

        let json = serde_json::to_string(&strategy).unwrap();
        let deserialized: EncodingStrategy = serde_json::from_str(&json).unwrap();

        match deserialized {
            EncodingStrategy::PixelEncoding { block_size, fps, resolution } => {
                assert_eq!(block_size, 4);
                assert_eq!(fps, 30);
                assert_eq!(resolution, (1920, 1080));
            }
            _ => panic!("Wrong variant"),
        }
    }
}
