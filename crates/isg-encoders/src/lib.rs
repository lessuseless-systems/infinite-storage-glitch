//! # ISG Encoders
//!
//! Encoding and decoding implementations for the Infinite Storage Glitch system.
//!
//! This crate provides multiple encoding strategies:
//! - Pixel encoding (black/white)
//! - Color encoding (RGB-based)
//! - QR code encoding
//! - Raw compression
//! - And more!

pub mod pixel;
pub mod color;
pub mod qr;
pub mod compression;

pub use pixel::PixelEncoder;
pub use color::ColorEncoder;
pub use qr::QREncoder;
pub use compression::CompressionEncoder;
