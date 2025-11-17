//! # ISG Storage
//!
//! Storage backend implementations for the Infinite Storage Glitch system.
//!
//! This crate provides various storage backends:
//! - Local filesystem
//! - YouTube (OAuth2)
//! - Discord (webhooks)
//! - Telegram (bot API)
//! - Cloudflare R2 (S3-compatible)
//! - IPFS
//! - And more!

pub mod local;
pub mod backends;
pub mod discord;

pub use backends::BackendType;
pub use local::LocalBackend;
pub use discord::DiscordBackend;

use async_trait::async_trait;
use isg_core::{Block, Error, Location, Result, StorageBackend};
use std::path::PathBuf;

/// Storage manager that coordinates multiple backends
pub struct StorageManager {
    backends: Vec<Box<dyn StorageBackend>>,
}

impl StorageManager {
    pub fn new() -> Self {
        Self {
            backends: Vec::new(),
        }
    }

    pub fn add_backend(&mut self, backend: Box<dyn StorageBackend>) {
        self.backends.push(backend);
    }

    pub async fn upload_to_backend(
        &self,
        backend_idx: usize,
        block: &Block,
    ) -> Result<Location> {
        if let Some(backend) = self.backends.get(backend_idx) {
            backend.upload(block).await
        } else {
            Err(Error::Storage("Backend not found".to_string()))
        }
    }

    pub async fn download_from_location(&self, location: &Location) -> Result<Vec<u8>> {
        for backend in &self.backends {
            if backend.name() == location.platform {
                return backend.download(location).await;
            }
        }
        Err(Error::Storage("No backend can handle this location".to_string()))
    }
}

impl Default for StorageManager {
    fn default() -> Self {
        Self::new()
    }
}
