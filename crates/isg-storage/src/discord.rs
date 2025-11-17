//! Discord storage backend using webhooks
//!
//! This backend uploads files as attachments to Discord webhooks.
//! Discord allows up to 25MB per file (or 100MB with Nitro).

use async_trait::async_trait;
use bytes::Bytes;
use isg_core::{Block, Error, Location, Result, StorageBackend, StorageMetadata, StorageTier};
use reqwest::multipart::{Form, Part};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, info};

/// Discord webhook backend
pub struct DiscordBackend {
    webhook_url: String,
    client: Client,
    max_file_size: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct WebhookMessage {
    content: Option<String>,
    username: Option<String>,
    attachments: Vec<Attachment>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Attachment {
    id: String,
    filename: String,
    size: usize,
    url: String,
    proxy_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct WebhookResponse {
    id: String,
    attachments: Vec<Attachment>,
}

impl DiscordBackend {
    /// Create a new Discord backend with webhook URL
    pub fn new(webhook_url: String) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(300)) // 5 minutes for large uploads
            .build()
            .map_err(|e| Error::Storage(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            webhook_url,
            client,
            max_file_size: 25 * 1024 * 1024, // 25MB default limit
        })
    }

    /// Create with custom max file size (for Nitro)
    pub fn with_max_size(mut self, max_size: usize) -> Self {
        self.max_file_size = max_size;
        self
    }

    /// Upload a file to Discord via webhook
    async fn upload_file(&self, filename: String, data: Bytes) -> Result<WebhookResponse> {
        if data.len() > self.max_file_size {
            return Err(Error::Storage(format!(
                "File size {} exceeds Discord limit of {} bytes",
                data.len(),
                self.max_file_size
            )));
        }

        debug!(
            "Uploading {} bytes to Discord webhook as {}",
            data.len(),
            filename
        );

        // Create multipart form
        let part = Part::bytes(data.to_vec())
            .file_name(filename.clone())
            .mime_str("application/octet-stream")
            .map_err(|e| Error::Storage(format!("Failed to create multipart: {}", e)))?;

        let form = Form::new()
            .text("content", format!("ISG Block: {}", filename))
            .part("file", part);

        // Send request
        let response = self
            .client
            .post(&self.webhook_url)
            .query(&[("wait", "true")]) // Wait for message to be created
            .multipart(form)
            .send()
            .await
            .map_err(|e| Error::Storage(format!("Discord upload failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(Error::Storage(format!(
                "Discord returned error {}: {}",
                status, error_text
            )));
        }

        let webhook_response: WebhookResponse = response
            .json()
            .await
            .map_err(|e| Error::Storage(format!("Failed to parse Discord response: {}", e)))?;

        info!(
            "Successfully uploaded to Discord, message ID: {}",
            webhook_response.id
        );

        Ok(webhook_response)
    }

    /// Download a file from Discord CDN
    async fn download_file(&self, url: &str) -> Result<Vec<u8>> {
        debug!("Downloading from Discord CDN: {}", url);

        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| Error::Storage(format!("Discord download failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(Error::Storage(format!(
                "Discord CDN returned error: {}",
                response.status()
            )));
        }

        let data = response
            .bytes()
            .await
            .map_err(|e| Error::Storage(format!("Failed to read response body: {}", e)))?
            .to_vec();

        info!("Downloaded {} bytes from Discord CDN", data.len());

        Ok(data)
    }
}

#[async_trait]
impl StorageBackend for DiscordBackend {
    fn name(&self) -> &str {
        "discord"
    }

    fn tier(&self) -> StorageTier {
        StorageTier::Warm
    }

    async fn upload(&self, block: &Block) -> Result<Location> {
        let filename = format!("{}.bin", block.hash());
        let data = Bytes::from(block.data().to_vec());

        let response = self.upload_file(filename, data).await?;

        // Get the first attachment (should be our file)
        let attachment = response
            .attachments
            .first()
            .ok_or_else(|| Error::Storage("No attachment in Discord response".to_string()))?;

        let mut metadata = StorageMetadata::default();
        metadata.url = Some(attachment.url.clone());
        metadata.stored_size = attachment.size;
        metadata.extra = serde_json::json!({
            "message_id": response.id,
            "attachment_id": attachment.id,
            "proxy_url": attachment.proxy_url,
        });

        Ok(Location {
            platform: "discord".to_string(),
            identifier: attachment.url.clone(),
            metadata,
        })
    }

    async fn download(&self, location: &Location) -> Result<Vec<u8>> {
        // The identifier is the CDN URL
        self.download_file(&location.identifier).await
    }

    async fn delete(&self, _location: &Location) -> Result<()> {
        // Discord webhooks don't support deletion via API
        // Would need bot token and message ID to delete
        // For now, just return Ok (orphan the data)
        info!("Note: Discord webhook backend cannot delete messages");
        Ok(())
    }

    async fn list(&self) -> Result<Vec<Location>> {
        // Discord webhooks don't support listing messages
        // Would need bot token to access channel history
        Err(Error::Storage(
            "Discord webhook backend does not support listing".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discord_backend_creation() {
        let backend = DiscordBackend::new(
            "https://discord.com/api/webhooks/123/abc".to_string()
        ).unwrap();

        assert_eq!(backend.name(), "discord");
        assert_eq!(backend.tier(), StorageTier::Warm);
    }

    // Note: Integration tests would require a real Discord webhook
    // and should be in a separate test suite with proper credentials
}
