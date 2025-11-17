//! Backend type definitions

use serde::{Deserialize, Serialize};

/// Types of storage backends supported
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BackendType {
    /// Local filesystem storage
    Local,
    /// YouTube storage
    YouTube,
    /// Discord webhook storage
    Discord,
    /// Telegram bot storage
    Telegram,
    /// Cloudflare R2 (S3-compatible)
    R2,
    /// IPFS decentralized storage
    IPFS,
}

impl BackendType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::YouTube => "youtube",
            Self::Discord => "discord",
            Self::Telegram => "telegram",
            Self::R2 => "r2",
            Self::IPFS => "ipfs",
        }
    }
}
