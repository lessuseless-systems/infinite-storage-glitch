//! Black/white pixel encoding
//!
//! This encoder converts binary data into black (1) and white (0) pixels,
//! then generates image frames that can be combined into a video.

use async_trait::async_trait;
use image::{ImageBuffer, Luma, Rgba, RgbaImage};
use isg_core::{EncodedData, Encoder, EncodingMetadata, EncodingStrategy, Error, Result};
use tracing::{debug, trace};

/// Pixel encoder configuration
#[derive(Clone, Debug)]
pub struct PixelEncoder {
    /// Block size (e.g., 4 means 4x4 pixels per bit)
    pub block_size: u32,

    /// Resolution (width, height)
    pub resolution: (u32, u32),

    /// Frames per second (for video generation)
    pub fps: u32,

    /// Threshold for reading pixels (0-255)
    pub threshold: u8,
}

impl PixelEncoder {
    /// Create a new pixel encoder with default settings
    pub fn new() -> Self {
        Self {
            block_size: 4,
            resolution: (1920, 1080),
            fps: 30,
            threshold: 128,
        }
    }

    /// Create with custom block size
    pub fn with_block_size(mut self, block_size: u32) -> Self {
        self.block_size = block_size;
        self
    }

    /// Create with custom resolution
    pub fn with_resolution(mut self, width: u32, height: u32) -> Self {
        self.resolution = (width, height);
        self
    }

    /// Create with custom FPS
    pub fn with_fps(mut self, fps: u32) -> Self {
        self.fps = fps;
        self
    }

    /// Calculate how many bits fit in one frame
    fn bits_per_frame(&self) -> usize {
        let (width, height) = self.resolution;
        let blocks_x = width / self.block_size;
        let blocks_y = height / self.block_size;
        (blocks_x * blocks_y) as usize
    }

    /// Encode binary data into image frames
    fn encode_to_frames(&self, data: &[u8]) -> Result<Vec<RgbaImage>> {
        let bits_per_frame = self.bits_per_frame();
        let total_bits = data.len() * 8;
        let num_frames = (total_bits + bits_per_frame - 1) / bits_per_frame;

        debug!(
            "Encoding {} bytes ({} bits) into {} frames",
            data.len(),
            total_bits,
            num_frames
        );

        let mut frames = Vec::new();
        let mut bit_index = 0;

        for frame_idx in 0..num_frames {
            let frame = self.create_frame(data, &mut bit_index)?;
            frames.push(frame);

            trace!("Created frame {}/{}", frame_idx + 1, num_frames);
        }

        Ok(frames)
    }

    /// Create a single frame from data
    fn create_frame(&self, data: &[u8], bit_index: &mut usize) -> Result<RgbaImage> {
        let (width, height) = self.resolution;
        let mut img = RgbaImage::new(width, height);

        let blocks_x = width / self.block_size;
        let blocks_y = height / self.block_size;

        for block_y in 0..blocks_y {
            for block_x in 0..blocks_x {
                // Get the bit value (0 or 1)
                let bit = self.get_bit(data, *bit_index);
                *bit_index += 1;

                // Color: 0 = white (255), 1 = black (0)
                let color = if bit { 0 } else { 255 };
                let pixel = Rgba([color, color, color, 255]);

                // Fill the block
                self.fill_block(&mut img, block_x, block_y, pixel);
            }
        }

        Ok(img)
    }

    /// Get a bit from data at given index
    fn get_bit(&self, data: &[u8], bit_index: usize) -> bool {
        let byte_index = bit_index / 8;
        let bit_offset = 7 - (bit_index % 8);

        if byte_index >= data.len() {
            return false; // Padding with zeros
        }

        (data[byte_index] >> bit_offset) & 1 == 1
    }

    /// Fill a block with a color
    fn fill_block(&self, img: &mut RgbaImage, block_x: u32, block_y: u32, color: Rgba<u8>) {
        let start_x = block_x * self.block_size;
        let start_y = block_y * self.block_size;

        for dy in 0..self.block_size {
            for dx in 0..self.block_size {
                let x = start_x + dx;
                let y = start_y + dy;

                if x < img.width() && y < img.height() {
                    img.put_pixel(x, y, color);
                }
            }
        }
    }

    /// Decode frames back to binary data
    fn decode_from_frames(&self, frames: &[RgbaImage], expected_size: usize) -> Result<Vec<u8>> {
        let mut bits = Vec::new();

        for frame in frames {
            let frame_bits = self.read_frame(frame)?;
            bits.extend(frame_bits);
        }

        // Convert bits to bytes
        let mut data = Vec::new();
        for chunk in bits.chunks(8) {
            let mut byte = 0u8;
            for (i, &bit) in chunk.iter().enumerate() {
                if bit {
                    byte |= 1 << (7 - i);
                }
            }
            data.push(byte);

            if data.len() >= expected_size {
                break;
            }
        }

        data.truncate(expected_size);
        Ok(data)
    }

    /// Read bits from a frame
    fn read_frame(&self, frame: &RgbaImage) -> Result<Vec<bool>> {
        let (width, height) = (frame.width(), frame.height());
        let blocks_x = width / self.block_size;
        let blocks_y = height / self.block_size;

        let mut bits = Vec::new();

        for block_y in 0..blocks_y {
            for block_x in 0..blocks_x {
                let bit = self.read_block(frame, block_x, block_y);
                bits.push(bit);
            }
        }

        Ok(bits)
    }

    /// Read a bit from a block by averaging pixels
    fn read_block(&self, frame: &RgbaImage, block_x: u32, block_y: u32) -> bool {
        let start_x = block_x * self.block_size;
        let start_y = block_y * self.block_size;

        let mut sum = 0u32;
        let mut count = 0u32;

        for dy in 0..self.block_size {
            for dx in 0..self.block_size {
                let x = start_x + dx;
                let y = start_y + dy;

                if x < frame.width() && y < frame.height() {
                    let pixel = frame.get_pixel(x, y);
                    // Average RGB (they should all be the same for grayscale)
                    let brightness = (pixel[0] as u32 + pixel[1] as u32 + pixel[2] as u32) / 3;
                    sum += brightness;
                    count += 1;
                }
            }
        }

        let avg = if count > 0 { sum / count } else { 255 };

        // Below threshold = black (1), above = white (0)
        avg < self.threshold as u32
    }
}

impl Default for PixelEncoder {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Encoder for PixelEncoder {
    async fn encode(&self, data: &[u8]) -> Result<EncodedData> {
        debug!("Encoding {} bytes with pixel encoder", data.len());

        let frames = self.encode_to_frames(data)?;

        // Serialize frames to PNG sequence
        let mut encoded = Vec::new();
        let frame_count = frames.len() as u32;

        // Store metadata: frame count (4 bytes) + original size (8 bytes)
        encoded.extend_from_slice(&frame_count.to_le_bytes());
        encoded.extend_from_slice(&(data.len() as u64).to_le_bytes());

        // Encode each frame as PNG
        for frame in &frames {
            let mut png_data = Vec::new();
            frame
                .write_to(
                    &mut std::io::Cursor::new(&mut png_data),
                    image::ImageOutputFormat::Png,
                )
                .map_err(|e| Error::Encoding(format!("PNG encoding failed: {}", e)))?;

            // Store frame size and data
            let frame_size = png_data.len() as u32;
            encoded.extend_from_slice(&frame_size.to_le_bytes());
            encoded.extend_from_slice(&png_data);
        }

        let metadata = EncodingMetadata {
            original_size: data.len(),
            encoded_size: encoded.len(),
            compression_ratio: encoded.len() as f64 / data.len() as f64,
            strategy: "pixel".to_string(),
            parameters: serde_json::json!({
                "block_size": self.block_size,
                "resolution": self.resolution,
                "fps": self.fps,
                "frame_count": frame_count,
            }),
        };

        Ok(EncodedData {
            data: encoded,
            format: "png_sequence".to_string(),
            metadata,
        })
    }

    async fn decode(&self, encoded: &EncodedData) -> Result<Vec<u8>> {
        debug!("Decoding pixel-encoded data");

        let data = &encoded.data;
        let mut cursor = 0;

        // Read metadata
        if data.len() < 12 {
            return Err(Error::Decoding("Data too short".to_string()));
        }

        let frame_count = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        let original_size = u64::from_le_bytes([
            data[4], data[5], data[6], data[7], data[8], data[9], data[10], data[11],
        ]) as usize;
        cursor = 12;

        debug!("Reading {} frames, expecting {} bytes", frame_count, original_size);

        // Decode frames
        let mut frames = Vec::new();
        for _ in 0..frame_count {
            if cursor + 4 > data.len() {
                return Err(Error::Decoding("Unexpected end of data".to_string()));
            }

            let frame_size = u32::from_le_bytes([
                data[cursor],
                data[cursor + 1],
                data[cursor + 2],
                data[cursor + 3],
            ]) as usize;
            cursor += 4;

            if cursor + frame_size > data.len() {
                return Err(Error::Decoding("Frame data truncated".to_string()));
            }

            let frame_data = &data[cursor..cursor + frame_size];
            let img = image::load_from_memory_with_format(frame_data, image::ImageFormat::Png)
                .map_err(|e| Error::Decoding(format!("PNG decoding failed: {}", e)))?
                .to_rgba8();

            frames.push(img);
            cursor += frame_size;
        }

        // Decode frames to data
        self.decode_from_frames(&frames, original_size)
    }

    fn strategy(&self) -> &EncodingStrategy {
        // Return a static reference - in real implementation, cache this
        static STRATEGY: once_cell::sync::Lazy<EncodingStrategy> =
            once_cell::sync::Lazy::new(|| {
                EncodingStrategy::PixelEncoding {
                    block_size: 4,
                    fps: 30,
                    resolution: (1920, 1080),
                }
            });
        &STRATEGY
    }

    fn estimate_size(&self, input_size: usize) -> usize {
        let total_bits = input_size * 8;
        let bits_per_frame = self.bits_per_frame();
        let num_frames = (total_bits + bits_per_frame - 1) / bits_per_frame;

        // Rough estimate: PNG compression ratio ~1.5x for pixel patterns
        let (width, height) = self.resolution;
        let bytes_per_frame = (width * height * 4) as usize; // RGBA
        let estimated = num_frames * bytes_per_frame * 3 / 2;

        estimated + 12 + (num_frames * 4) // metadata overhead
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pixel_encoder_roundtrip() {
        let encoder = PixelEncoder::new();
        let data = b"Hello, ISG World! This is a test of the pixel encoder.";

        let encoded = encoder.encode(data).await.unwrap();
        println!("Encoded {} bytes to {} bytes", data.len(), encoded.data.len());

        let decoded = encoder.decode(&encoded).await.unwrap();

        assert_eq!(data.as_slice(), decoded.as_slice());
    }

    #[tokio::test]
    async fn test_small_block_size() {
        let encoder = PixelEncoder::new().with_block_size(2);
        let data = b"Small blocks!";

        let encoded = encoder.encode(data).await.unwrap();
        let decoded = encoder.decode(&encoded).await.unwrap();

        assert_eq!(data.as_slice(), decoded.as_slice());
    }

    #[tokio::test]
    async fn test_large_block_size() {
        let encoder = PixelEncoder::new().with_block_size(10);
        let data = b"Large blocks make fewer pixels but less detail.";

        let encoded = encoder.encode(data).await.unwrap();
        let decoded = encoder.decode(&encoded).await.unwrap();

        assert_eq!(data.as_slice(), decoded.as_slice());
    }
}
