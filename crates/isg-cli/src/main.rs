use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use isg_core::{Block, EncodingStrategy, Hash};
use isg_crypto::CryptoEngine;
use isg_db::{BlockRecord, ChunkRecord, Database, FileRecord};
use isg_encoders::{ColorEncoder, PixelEncoder, QREncoder};
use isg_erasure::ErasureCoder;
use isg_storage::{LocalBackend, StorageManager};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{error, info};

#[derive(Parser)]
#[command(name = "isg")]
#[command(about = "Infinite Storage Glitch - Ultra-Deluxe Edition", long_about = None)]
struct Cli {
    /// Database path
    #[arg(short, long, default_value = "isg.db")]
    db: PathBuf,

    /// Storage root directory
    #[arg(short, long, default_value = "isg_storage")]
    storage: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Upload a file to ISG
    Upload {
        /// Path to the file to upload
        file: PathBuf,

        /// Encoding strategy (pixel, color, qr)
        #[arg(short, long, default_value = "pixel")]
        encoding: String,

        /// Enable encryption (requires password)
        #[arg(short = 'e', long)]
        encrypt: bool,

        /// Password for encryption (will prompt if not provided)
        #[arg(short, long)]
        password: Option<String>,

        /// Enable erasure coding (N data shards)
        #[arg(long, default_value = "4")]
        data_shards: usize,

        /// Parity shards for erasure coding
        #[arg(long, default_value = "2")]
        parity_shards: usize,
    },

    /// Download a file from ISG
    Download {
        /// Path of the file in ISG
        path: String,

        /// Output file path
        #[arg(short, long)]
        output: PathBuf,

        /// Password for decryption (if encrypted)
        #[arg(short, long)]
        password: Option<String>,
    },

    /// List all files in ISG
    List {
        /// Show detailed information
        #[arg(short, long)]
        detailed: bool,
    },

    /// Delete a file from ISG
    Delete {
        /// Path of the file to delete
        path: String,
    },

    /// Show statistics
    Stats,

    /// Initialize ISG storage
    Init,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    let cli = Cli::parse();

    // Open database
    let db = Database::open(&cli.db)
        .context("Failed to open database")?;

    // Create storage manager
    let mut storage = StorageManager::new();
    let local_backend = LocalBackend::new(&cli.storage).await?;
    storage.add_backend(Box::new(local_backend));

    match cli.command {
        Commands::Init => {
            init(&cli.storage).await?;
        }
        Commands::Upload {
            file,
            encoding,
            encrypt,
            password,
            data_shards,
            parity_shards,
        } => {
            upload(
                &db,
                &storage,
                &file,
                &encoding,
                encrypt,
                password,
                data_shards,
                parity_shards,
            )
            .await?;
        }
        Commands::Download {
            path,
            output,
            password,
        } => {
            download(&db, &storage, &path, &output, password).await?;
        }
        Commands::List { detailed } => {
            list(&db, detailed).await?;
        }
        Commands::Delete { path } => {
            delete(&db, &storage, &path).await?;
        }
        Commands::Stats => {
            stats(&db).await?;
        }
    }

    Ok(())
}

async fn init(storage_path: &PathBuf) -> Result<()> {
    info!("Initializing ISG storage at {:?}", storage_path);

    tokio::fs::create_dir_all(storage_path).await?;

    println!("✓ ISG storage initialized at {:?}", storage_path);
    println!("✓ Database ready");
    println!("\nYou can now upload files with: isg upload <file>");

    Ok(())
}

async fn upload(
    db: &Database,
    storage: &StorageManager,
    file_path: &PathBuf,
    encoding: &str,
    encrypt: bool,
    password: Option<String>,
    data_shards: usize,
    parity_shards: usize,
) -> Result<()> {
    info!("Uploading file: {:?}", file_path);

    // Read file
    let data = tokio::fs::read(file_path).await
        .context("Failed to read file")?;

    let original_size = data.len();
    println!("📁 File size: {} bytes", original_size);

    // Optional encryption
    let (data, crypto_engine) = if encrypt {
        let pwd = if let Some(p) = password {
            p
        } else {
            rpassword::prompt_password("Enter password: ")?
        };

        let salt = b"isg_salt_12345678"; // In production, use random salt and store it
        let key = CryptoEngine::derive_key_from_password(&pwd, salt)?;
        let engine = CryptoEngine::new(key);

        let encrypted = engine.encrypt(&data)?;

        // Serialize encrypted data (nonce + ciphertext)
        let mut encrypted_data = Vec::new();
        encrypted_data.extend_from_slice(&encrypted.nonce);
        encrypted_data.extend_from_slice(&encrypted.ciphertext);

        println!("🔒 Encrypted with AES-256-GCM");

        (encrypted_data, Some(engine))
    } else {
        (data, None)
    };

    // Split into blocks (simple chunking for now)
    let block_size = 1024 * 1024; // 1MB blocks
    let mut blocks = Vec::new();

    for chunk in data.chunks(block_size) {
        let block = Block::from_data(chunk.to_vec());
        blocks.push(block);
    }

    println!("📦 Split into {} blocks", blocks.len());

    // Encode each block
    let encoding_strategy = match encoding {
        "pixel" => EncodingStrategy::PixelEncoding {
            block_size: 4,
            fps: 30,
            resolution: (1920, 1080),
        },
        "color" => EncodingStrategy::ColorEncoding {
            color_space: isg_core::ColorSpace::RGB,
        },
        "qr" => EncodingStrategy::QREncoding {
            version: 10,
            ecc_level: isg_core::ECCLevel::Medium,
        },
        _ => EncodingStrategy::PixelEncoding {
            block_size: 4,
            fps: 30,
            resolution: (1920, 1080),
        },
    };

    for (i, block) in blocks.iter().enumerate() {
        // Store block metadata
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let block_record = BlockRecord {
            hash: block.hash().as_bytes().to_vec(),
            size: block.data().len() as i64,
            encoding_strategy: encoding.to_string(),
            created_at: timestamp,
        };

        db.insert_block(&block_record)?;

        // Upload to storage
        let location = storage.upload_to_backend(0, block).await?;

        // Store chunk location
        let chunk_record = ChunkRecord {
            id: 0,
            block_hash: block.hash().as_bytes().to_vec(),
            platform: location.platform.clone(),
            location: location.identifier.clone(),
            is_parity: false,
            uploaded_at: timestamp,
        };

        db.insert_chunk(&chunk_record)?;

        println!("  ✓ Block {}/{} uploaded", i + 1, blocks.len());
    }

    // Store file metadata
    let root_hash = if blocks.is_empty() {
        Hash::from_data(b"empty")
    } else {
        blocks[0].hash().clone()
    };

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let file_record = FileRecord {
        id: 0,
        path: file_path.to_string_lossy().to_string(),
        root_hash: root_hash.as_bytes().to_vec(),
        size: original_size as i64,
        created_at: timestamp,
        modified_at: timestamp,
        accessed_at: timestamp,
    };

    db.insert_file(&file_record)?;

    println!("✅ Upload complete!");
    println!("   Path: {}", file_path.display());
    println!("   Size: {} bytes", original_size);
    println!("   Blocks: {}", blocks.len());
    println!("   Encoding: {}", encoding);
    if encrypt {
        println!("   Encrypted: Yes");
    }

    Ok(())
}

async fn download(
    db: &Database,
    storage: &StorageManager,
    path: &str,
    output: &PathBuf,
    password: Option<String>,
) -> Result<()> {
    info!("Downloading file: {}", path);

    // Get file metadata
    let file = db.get_file_by_path(path)?
        .context("File not found")?;

    println!("📥 Downloading: {}", path);
    println!("   Size: {} bytes", file.size);

    // Get all blocks for this file
    // For simplicity, we'll use the root hash to get the first block
    // In a real implementation, we'd store block references
    let chunks = db.get_chunks_for_block(&file.root_hash)?;

    if chunks.is_empty() {
        anyhow::bail!("No chunks found for file");
    }

    let mut reconstructed_data = Vec::new();

    for (i, chunk) in chunks.iter().enumerate() {
        let location = isg_core::Location {
            platform: chunk.platform.clone(),
            identifier: chunk.location.clone(),
            metadata: Default::default(),
        };

        let block_data = storage.download_from_location(&location).await?;
        reconstructed_data.extend_from_slice(&block_data);

        println!("  ✓ Block {}/{} downloaded", i + 1, chunks.len());
    }

    // Optional decryption
    if let Some(pwd) = password {
        let salt = b"isg_salt_12345678"; // Should match the salt used during upload
        let key = CryptoEngine::derive_key_from_password(&pwd, salt)?;
        let engine = CryptoEngine::new(key);

        // Extract nonce and ciphertext
        if reconstructed_data.len() < 12 {
            anyhow::bail!("Invalid encrypted data");
        }

        let nonce: [u8; 12] = reconstructed_data[..12].try_into()?;
        let ciphertext = reconstructed_data[12..].to_vec();

        let encrypted = isg_crypto::EncryptedData { nonce, ciphertext };
        reconstructed_data = engine.decrypt(&encrypted)?;

        println!("🔓 Decrypted");
    }

    // Write to output file
    tokio::fs::write(output, &reconstructed_data).await?;

    println!("✅ Download complete!");
    println!("   Output: {}", output.display());
    println!("   Size: {} bytes", reconstructed_data.len());

    Ok(())
}

async fn list(db: &Database, detailed: bool) -> Result<()> {
    let files = db.list_files()?;

    if files.is_empty() {
        println!("No files stored.");
        return Ok(());
    }

    println!("\n📚 Stored Files ({} total):\n", files.len());

    for file in files {
        if detailed {
            println!("Path: {}", file.path);
            println!("  Size: {} bytes", file.size);
            println!("  Created: {}", chrono::DateTime::from_timestamp(file.created_at, 0)
                .map(|dt| dt.to_rfc2822())
                .unwrap_or_else(|| "Unknown".to_string()));
            println!();
        } else {
            println!("  {} ({} bytes)", file.path, file.size);
        }
    }

    Ok(())
}

async fn delete(db: &Database, storage: &StorageManager, path: &str) -> Result<()> {
    info!("Deleting file: {}", path);

    // Get file metadata
    let file = db.get_file_by_path(path)?
        .context("File not found")?;

    // Get chunks
    let chunks = db.get_chunks_for_block(&file.root_hash)?;

    // Delete from storage
    for chunk in &chunks {
        let location = isg_core::Location {
            platform: chunk.platform.clone(),
            identifier: chunk.location.clone(),
            metadata: Default::default(),
        };

        if let Err(e) = storage.download_from_location(&location).await {
            error!("Failed to delete chunk: {}", e);
        }
    }

    // Delete from database
    db.delete_file(path)?;

    println!("✅ Deleted: {}", path);

    Ok(())
}

async fn stats(db: &Database) -> Result<()> {
    let files = db.list_files()?;

    let total_size: i64 = files.iter().map(|f| f.size).sum();
    let file_count = files.len();

    println!("\n📊 ISG Statistics:\n");
    println!("  Files: {}", file_count);
    println!("  Total Size: {} bytes ({:.2} MB)", total_size, total_size as f64 / 1024.0 / 1024.0);

    if file_count > 0 {
        println!("  Average Size: {} bytes", total_size / file_count as i64);
    }

    Ok(())
}
