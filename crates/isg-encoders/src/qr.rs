//! QR code encoding
//!
//! Encode data as a grid of QR codes with built-in error correction.

use async_trait::async_trait;
use image::RgbaImage;
use isg_core::{EncodedData, Encoder, EncodingMetadata, EncodingStrategy, Error, Result};
use qrcode::QrCode;
use tracing::debug;

/// QR code encoder configuration
#[derive(Clone, Debug)]
pub struct QREncoder {
    /// Maximum bytes per QR code
    pub max_bytes_per_qr: usize,
}

impl QREncoder {
    /// Create a new QR encoder
    pub fn new() -> Self {
        Self {
            // QR code can hold up to ~2953 bytes in binary mode
            max_bytes_per_qr: 2000, // Conservative limit
        }
    }

    /// Create QR code images from data
    fn encode_to_qr_codes(&self, data: &[u8]) -> Result<Vec<RgbaImage>> {
        let chunks = data.chunks(self.max_bytes_per_qr);
        let mut qr_images = Vec::new();

        for (idx, chunk) in chunks.enumerate() {
            let qr = QrCode::new(chunk)
                .map_err(|e| Error::Encoding(format!("QR code generation failed: {}", e)))?;

            // Render as image with scaling
            let image = qr.render::<image::Luma<u8>>()
                .min_dimensions(512, 512)
                .build();

            // Convert to RGBA
            let rgba = RgbaImage::from_fn(image.width(), image.height(), |x, y| {
                let luma = image.get_pixel(x, y)[0];
                image::Rgba([luma, luma, luma, 255])
            });

            qr_images.push(rgba);
            debug!("Created QR code {}/{}", idx + 1, chunks.len());
        }

        Ok(qr_images)
    }
}

impl Default for QREncoder {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Encoder for QREncoder {
    async fn encode(&self, data: &[u8]) -> Result<EncodedData> {
        debug!("Encoding {} bytes as QR codes", data.len());

        let qr_images = self.encode_to_qr_codes(data)?;

        // Serialize QR images to PNG sequence (similar to pixel encoder)
        let mut encoded = Vec::new();
        let qr_count = qr_images.len() as u32;

        // Store metadata: QR count (4 bytes) + original size (8 bytes)
        encoded.extend_from_slice(&qr_count.to_le_bytes());
        encoded.extend_from_slice(&(data.len() as u64).to_le_bytes());

        // Encode each QR as PNG
        for qr_img in &qr_images {
            let mut png_data = Vec::new();
            qr_img
                .write_to(
                    &mut std::io::Cursor::new(&mut png_data),
                    image::ImageOutputFormat::Png,
                )
                .map_err(|e| Error::Encoding(format!("PNG encoding failed: {}", e)))?;

            let qr_size = png_data.len() as u32;
            encoded.extend_from_slice(&qr_size.to_le_bytes());
            encoded.extend_from_slice(&png_data);
        }

        let metadata = EncodingMetadata {
            original_size: data.len(),
            encoded_size: encoded.len(),
            compression_ratio: encoded.len() as f64 / data.len() as f64,
            strategy: "qr".to_string(),
            parameters: serde_json::json!({
                "qr_count": qr_count,
                "max_bytes_per_qr": self.max_bytes_per_qr,
            }),
        };

        Ok(EncodedData {
            data: encoded,
            format: "qr_sequence".to_string(),
            metadata,
        })
    }

    async fn decode(&self, encoded: &EncodedData) -> Result<Vec<u8>> {
        debug!("Decoding QR-encoded data");

        // TODO: Implement QR decoding using rqrr or similar
        // For now, this is a placeholder
        Err(Error::Decoding(
            "QR decoding not yet implemented - requires QR scanner library".to_string(),
        ))
    }

    fn strategy(&self) -> &EncodingStrategy {
        static STRATEGY: once_cell::sync::Lazy<EncodingStrategy> =
            once_cell::sync::Lazy::new(|| {
                EncodingStrategy::QREncoding {
                    version: 40, // Max version
                    ecc_level: isg_core::ECCLevel::Medium,
                }
            });
        &STRATEGY
    }

    fn estimate_size(&self, input_size: usize) -> usize {
        let num_qr = (input_size + self.max_bytes_per_qr - 1) / self.max_bytes_per_qr;
        // Each QR ~512x512 pixels RGBA = ~1MB
        num_qr * 1024 * 1024
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_qr_encoding() {
        let encoder = QREncoder::new();
        let data = b"Hello, QR Code World!";

        let encoded = encoder.encode(data).await.unwrap();
        assert!(encoded.data.len() > 0);

        // Decoding not implemented yet
        // let decoded = encoder.decode(&encoded).await.unwrap();
        // assert_eq!(data.as_slice(), decoded.as_slice());
    }
}
