//! Demonstration of ISG encoding workflow
//!
//! This example shows:
//! - Pixel encoding of data
//! - Encryption with AES-256-GCM
//! - Complete upload/download roundtrip

use isg_core::Encoder;
use isg_crypto::CryptoEngine;
use isg_encoders::PixelEncoder;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🚀 ISG Encoding Demo\n");

    let test_data = b"Hello, ISG! This demonstrates the full encoding workflow with encryption and pixel encoding.";

    println!("📝 Original data: {} bytes", test_data.len());
    println!("   Content: {:?}\n", String::from_utf8_lossy(test_data));

    // Step 1: Encryption
    println!("🔒 Step 1: Encrypting with AES-256-GCM...");
    let password = "super_secret_password";
    let salt = b"demo_salt_123456";

    let key = CryptoEngine::derive_key_from_password(password, salt)?;
    let engine = CryptoEngine::new(key);

    let encrypted = engine.encrypt(test_data)?;

    let mut encrypted_bytes = Vec::new();
    encrypted_bytes.extend_from_slice(&encrypted.nonce);
    encrypted_bytes.extend_from_slice(&encrypted.ciphertext);

    println!("   ✓ Encrypted to {} bytes\n", encrypted_bytes.len());

    // Step 2: Pixel Encoding
    println!("🎨 Step 2: Encoding with Pixel Encoder...");
    let encoder = PixelEncoder::new()
        .with_block_size(4)
        .with_resolution(1920, 1080);

    let encoded = encoder.encode(&encrypted_bytes).await?;

    println!("   ✓ Encoded to {} bytes", encoded.data.len());
    println!("   ✓ Format: {}", encoded.format);
    println!("   ✓ Compression ratio: {:.2}x", encoded.metadata.compression_ratio);
    println!("   ✓ Frame count: {}\n", encoded.metadata.parameters["frame_count"]);

    // Step 3: Decoding
    println!("🔄 Step 3: Decoding pixel-encoded data...");
    let decoded_encrypted = encoder.decode(&encoded).await?;

    println!("   ✓ Decoded to {} bytes\n", decoded_encrypted.len());

    // Step 4: Decryption
    println!("🔓 Step 4: Decrypting...");
    let nonce: [u8; 12] = decoded_encrypted[..12].try_into()?;
    let ciphertext = decoded_encrypted[12..].to_vec();

    let encrypted_data = isg_crypto::EncryptedData { nonce, ciphertext };
    let decrypted = engine.decrypt(&encrypted_data)?;

    println!("   ✓ Decrypted to {} bytes\n", decrypted.len());

    // Verify
    println!("✅ Verification:");
    if test_data == decrypted.as_slice() {
        println!("   ✓ Data integrity verified!");
        println!("   ✓ Original: {:?}", String::from_utf8_lossy(test_data));
        println!("   ✓ Roundtrip successful!\n");
    } else {
        println!("   ✗ Data mismatch!\n");
        anyhow::bail!("Verification failed");
    }

    println!("📊 Summary:");
    println!("   Original size:     {} bytes", test_data.len());
    println!("   After encryption:  {} bytes", encrypted_bytes.len());
    println!("   After encoding:    {} bytes", encoded.data.len());
    println!("   Storage overhead:  {:.1}x", encoded.data.len() as f64 / test_data.len() as f64);
    println!("\n🎉 Complete workflow validated!");

    Ok(())
}
