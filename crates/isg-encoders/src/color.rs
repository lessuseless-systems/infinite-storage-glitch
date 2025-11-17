//! RGB color encoding
//!
//! Maps data to unique RGB color values for higher density encoding.

use async_trait::async_trait;
use isg_core::{EncodedData, Encoder, EncodingMetadata, EncodingStrategy, Error, Result};

/// Color encoder (placeholder for future implementation)
#[derive(Clone, Debug)]
pub struct ColorEncoder {
    // TODO: Implement color-based encoding
}

impl ColorEncoder {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for ColorEncoder {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Encoder for ColorEncoder {
    async fn encode(&self, _data: &[u8]) -> Result<EncodedData> {
        Err(Error::Encoding(
            "Color encoding not yet implemented".to_string(),
        ))
    }

    async fn decode(&self, _encoded: &EncodedData) -> Result<Vec<u8>> {
        Err(Error::Decoding(
            "Color decoding not yet implemented".to_string(),
        ))
    }

    fn strategy(&self) -> &EncodingStrategy {
        static STRATEGY: once_cell::sync::Lazy<EncodingStrategy> =
            once_cell::sync::Lazy::new(|| {
                EncodingStrategy::ColorEncoding {
                    color_space: isg_core::ColorSpace::RGB,
                }
            });
        &STRATEGY
    }
}
