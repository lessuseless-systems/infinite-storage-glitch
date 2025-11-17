//! # ISG Database
//!
//! SQLite database layer for the Infinite Storage Glitch system.
//!
//! This crate provides:
//! - File metadata storage
//! - Block tracking
//! - Chunk location mapping
//! - Access pattern tracking

use isg_core::{Error, Hash, Result};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Database manager
pub struct Database {
    conn: Connection,
}

/// File record in the database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileRecord {
    pub id: i64,
    pub path: String,
    pub root_hash: Vec<u8>,
    pub size: i64,
    pub created_at: i64,
    pub modified_at: i64,
    pub accessed_at: i64,
}

/// Block record in the database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockRecord {
    pub hash: Vec<u8>,
    pub size: i64,
    pub encoding_strategy: String,
    pub created_at: i64,
}

/// Chunk record in the database (location of a block on a backend)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkRecord {
    pub id: i64,
    pub block_hash: Vec<u8>,
    pub platform: String,
    pub location: String,
    pub is_parity: bool,
    pub uploaded_at: i64,
}

impl Database {
    /// Open or create a database at the given path
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path)
            .map_err(|e| Error::Database(format!("Failed to open database: {}", e)))?;

        let db = Self { conn };
        db.initialize_schema()?;
        Ok(db)
    }

    /// Create an in-memory database (for testing)
    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()
            .map_err(|e| Error::Database(format!("Failed to create in-memory database: {}", e)))?;

        let db = Self { conn };
        db.initialize_schema()?;
        Ok(db)
    }

    /// Initialize database schema
    fn initialize_schema(&self) -> Result<()> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS files (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                path TEXT UNIQUE NOT NULL,
                root_hash BLOB NOT NULL,
                size INTEGER NOT NULL,
                created_at INTEGER NOT NULL,
                modified_at INTEGER NOT NULL,
                accessed_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS blocks (
                hash BLOB PRIMARY KEY,
                size INTEGER NOT NULL,
                encoding_strategy TEXT NOT NULL,
                created_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS chunks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                block_hash BLOB NOT NULL,
                platform TEXT NOT NULL,
                location TEXT NOT NULL,
                is_parity BOOLEAN NOT NULL DEFAULT 0,
                uploaded_at INTEGER NOT NULL,
                FOREIGN KEY (block_hash) REFERENCES blocks(hash)
            );

            CREATE INDEX IF NOT EXISTS idx_files_path ON files(path);
            CREATE INDEX IF NOT EXISTS idx_blocks_hash ON blocks(hash);
            CREATE INDEX IF NOT EXISTS idx_chunks_block_hash ON chunks(block_hash);
            CREATE INDEX IF NOT EXISTS idx_chunks_platform ON chunks(platform);
            "#,
        )
        .map_err(|e| Error::Database(format!("Failed to initialize schema: {}", e)))?;

        Ok(())
    }

    /// Insert a file record
    pub fn insert_file(&self, file: &FileRecord) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO files (path, root_hash, size, created_at, modified_at, accessed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                &file.path,
                &file.root_hash,
                file.size,
                file.created_at,
                file.modified_at,
                file.accessed_at
            ],
        )
        .map_err(|e| Error::Database(format!("Failed to insert file: {}", e)))?;

        Ok(self.conn.last_insert_rowid())
    }

    /// Get a file record by path
    pub fn get_file_by_path(&self, path: &str) -> Result<Option<FileRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, path, root_hash, size, created_at, modified_at, accessed_at
             FROM files WHERE path = ?1"
        )
        .map_err(|e| Error::Database(format!("Failed to prepare statement: {}", e)))?;

        let result = stmt.query_row(params![path], |row| {
            Ok(FileRecord {
                id: row.get(0)?,
                path: row.get(1)?,
                root_hash: row.get(2)?,
                size: row.get(3)?,
                created_at: row.get(4)?,
                modified_at: row.get(5)?,
                accessed_at: row.get(6)?,
            })
        });

        match result {
            Ok(file) => Ok(Some(file)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(Error::Database(format!("Failed to query file: {}", e))),
        }
    }

    /// List all files
    pub fn list_files(&self) -> Result<Vec<FileRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, path, root_hash, size, created_at, modified_at, accessed_at
             FROM files ORDER BY path"
        )
        .map_err(|e| Error::Database(format!("Failed to prepare statement: {}", e)))?;

        let files = stmt.query_map([], |row| {
            Ok(FileRecord {
                id: row.get(0)?,
                path: row.get(1)?,
                root_hash: row.get(2)?,
                size: row.get(3)?,
                created_at: row.get(4)?,
                modified_at: row.get(5)?,
                accessed_at: row.get(6)?,
            })
        })
        .map_err(|e| Error::Database(format!("Failed to query files: {}", e)))?;

        files.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| Error::Database(format!("Failed to collect files: {}", e)))
    }

    /// Insert a block record
    pub fn insert_block(&self, block: &BlockRecord) -> Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO blocks (hash, size, encoding_strategy, created_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                &block.hash,
                block.size,
                &block.encoding_strategy,
                block.created_at
            ],
        )
        .map_err(|e| Error::Database(format!("Failed to insert block: {}", e)))?;

        Ok(())
    }

    /// Get a block record by hash
    pub fn get_block(&self, hash: &[u8]) -> Result<Option<BlockRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT hash, size, encoding_strategy, created_at FROM blocks WHERE hash = ?1"
        )
        .map_err(|e| Error::Database(format!("Failed to prepare statement: {}", e)))?;

        let result = stmt.query_row(params![hash], |row| {
            Ok(BlockRecord {
                hash: row.get(0)?,
                size: row.get(1)?,
                encoding_strategy: row.get(2)?,
                created_at: row.get(3)?,
            })
        });

        match result {
            Ok(block) => Ok(Some(block)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(Error::Database(format!("Failed to query block: {}", e))),
        }
    }

    /// Insert a chunk record
    pub fn insert_chunk(&self, chunk: &ChunkRecord) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO chunks (block_hash, platform, location, is_parity, uploaded_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                &chunk.block_hash,
                &chunk.platform,
                &chunk.location,
                chunk.is_parity,
                chunk.uploaded_at
            ],
        )
        .map_err(|e| Error::Database(format!("Failed to insert chunk: {}", e)))?;

        Ok(self.conn.last_insert_rowid())
    }

    /// Get all chunks for a block
    pub fn get_chunks_for_block(&self, block_hash: &[u8]) -> Result<Vec<ChunkRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, block_hash, platform, location, is_parity, uploaded_at
             FROM chunks WHERE block_hash = ?1"
        )
        .map_err(|e| Error::Database(format!("Failed to prepare statement: {}", e)))?;

        let chunks = stmt.query_map(params![block_hash], |row| {
            Ok(ChunkRecord {
                id: row.get(0)?,
                block_hash: row.get(1)?,
                platform: row.get(2)?,
                location: row.get(3)?,
                is_parity: row.get(4)?,
                uploaded_at: row.get(5)?,
            })
        })
        .map_err(|e| Error::Database(format!("Failed to query chunks: {}", e)))?;

        chunks.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| Error::Database(format!("Failed to collect chunks: {}", e)))
    }

    /// Delete a file and its associated data
    pub fn delete_file(&self, path: &str) -> Result<()> {
        self.conn.execute("DELETE FROM files WHERE path = ?1", params![path])
            .map_err(|e| Error::Database(format!("Failed to delete file: {}", e)))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_operations() {
        let db = Database::in_memory().unwrap();

        // Insert a file
        let file = FileRecord {
            id: 0,
            path: "/test/file.txt".to_string(),
            root_hash: vec![1, 2, 3, 4],
            size: 1024,
            created_at: 1234567890,
            modified_at: 1234567890,
            accessed_at: 1234567890,
        };

        let file_id = db.insert_file(&file).unwrap();
        assert!(file_id > 0);

        // Retrieve the file
        let retrieved = db.get_file_by_path("/test/file.txt").unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.path, "/test/file.txt");

        // Insert a block
        let block = BlockRecord {
            hash: vec![5, 6, 7, 8],
            size: 256,
            encoding_strategy: "pixel".to_string(),
            created_at: 1234567890,
        };

        db.insert_block(&block).unwrap();

        // Retrieve the block
        let retrieved_block = db.get_block(&[5, 6, 7, 8]).unwrap();
        assert!(retrieved_block.is_some());
    }
}
