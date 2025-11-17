//! Error types for ISG

use thiserror::Error;

/// ISG error types
#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Encoding error: {0}")]
    Encoding(String),

    #[error("Decoding error: {0}")]
    Decoding(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Block not found: {0}")]
    BlockNotFound(String),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Invalid hash: {0}")]
    InvalidHash(String),

    #[error("Corruption detected: {0}")]
    Corruption(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Crypto error: {0}")]
    Crypto(String),

    #[error("Platform error: {0}")]
    Platform(String),

    #[error("Other error: {0}")]
    Other(String),
}

/// Result type alias
pub type Result<T> = std::result::Result<T, Error>;
