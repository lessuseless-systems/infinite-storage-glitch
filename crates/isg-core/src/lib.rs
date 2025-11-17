//! # ISG Core
//!
//! Core types and traits for the Infinite Storage Glitch system.
//!
//! This crate provides the fundamental abstractions used throughout the ISG ecosystem:
//! - Content-addressable blocks
//! - Encoding strategies
//! - Storage backends
//! - File and chunk representations

pub mod block;
pub mod encoding;
pub mod error;
pub mod file;
pub mod hash;
pub mod storage;

pub use block::{Block, BlockMetadata};
pub use encoding::{EncodedData, Encoder, EncodingStrategy};
pub use error::{Error, Result};
pub use file::{File, FileMetadata};
pub use hash::Hash;
pub use storage::{Location, StorageBackend, StorageMetadata};
