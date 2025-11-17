//! Local filesystem storage backend

use async_trait::async_trait;
use isg_core::{Block, Error, Location, Result, StorageBackend, StorageMetadata, StorageTier};
use std::path::{Path, PathBuf};
use tokio::fs;
use uuid::Uuid;

/// Local filesystem storage backend
pub struct LocalBackend {
    root_path: PathBuf,
}

impl LocalBackend {
    /// Create a new local backend with the given root path
    pub async fn new<P: AsRef<Path>>(root_path: P) -> Result<Self> {
        let root_path = root_path.as_ref().to_path_buf();

        // Create root directory if it doesn't exist
        if !root_path.exists() {
            fs::create_dir_all(&root_path).await
                .map_err(|e| Error::Storage(format!("Failed to create directory: {}", e)))?;
        }

        Ok(Self { root_path })
    }

    fn get_file_path(&self, id: &str) -> PathBuf {
        // Use a simple two-level directory structure: first 2 chars / next 2 chars / rest
        let prefix1 = &id[..2.min(id.len())];
        let prefix2 = if id.len() > 2 { &id[2..4.min(id.len())] } else { "" };

        self.root_path.join(prefix1).join(prefix2).join(id)
    }
}

#[async_trait]
impl StorageBackend for LocalBackend {
    fn name(&self) -> &str {
        "local"
    }

    fn tier(&self) -> StorageTier {
        StorageTier::Hot
    }

    async fn upload(&self, block: &Block) -> Result<Location> {
        let id = Uuid::new_v4().to_string();
        let file_path = self.get_file_path(&id);

        // Create parent directories
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).await
                .map_err(|e| Error::Storage(format!("Failed to create directory: {}", e)))?;
        }

        // Write the file
        fs::write(&file_path, block.data()).await
            .map_err(|e| Error::Storage(format!("Failed to write file: {}", e)))?;

        Ok(Location {
            platform: "local".to_string(),
            identifier: id,
            metadata: StorageMetadata::default(),
        })
    }

    async fn download(&self, location: &Location) -> Result<Vec<u8>> {
        let file_path = self.get_file_path(&location.identifier);

        fs::read(&file_path).await
            .map_err(|e| Error::Storage(format!("Failed to read file: {}", e)))
    }

    async fn delete(&self, location: &Location) -> Result<()> {
        let file_path = self.get_file_path(&location.identifier);

        fs::remove_file(&file_path).await
            .map_err(|e| Error::Storage(format!("Failed to delete file: {}", e)))?;

        Ok(())
    }

    async fn list(&self) -> Result<Vec<Location>> {
        let mut locations = Vec::new();

        // Walk the directory tree
        let mut stack = vec![self.root_path.clone()];

        while let Some(dir) = stack.pop() {
            let mut entries = fs::read_dir(&dir).await
                .map_err(|e| Error::Storage(format!("Failed to read directory: {}", e)))?;

            while let Some(entry) = entries.next_entry().await
                .map_err(|e| Error::Storage(format!("Failed to read entry: {}", e)))? {

                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                } else if path.is_file() {
                    if let Some(filename) = path.file_name() {
                        if let Some(id) = filename.to_str() {
                            locations.push(Location {
                                platform: "local".to_string(),
                                identifier: id.to_string(),
                                metadata: StorageMetadata::default(),
                            });
                        }
                    }
                }
            }
        }

        Ok(locations)
    }
}
