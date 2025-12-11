//! Archive writer for creating SecureArc files

use crate::compression::compress_data;
use crate::crypto::encryption::{encrypt_data, generate_master_key, EncryptionKey};
use crate::crypto::integrity::{compute_checksum, IntegrityKey};
use crate::crypto::kdf::{derive_key, KdfParams};
use crate::format::directory::{CentralDirectory, FileEntry};
use crate::format::header::SecurityHeader;
use crate::format::keyslot::{KeySlot, MASTER_KEY_SIZE};
use crate::format::CompressionAlgorithm;
use crate::format::{CompressionAlgorithm as FormatCompression, EncryptionAlgorithm, MAGIC_NUMBER};
use crate::SecureArcError;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

/// Configuration for creating a new archive
#[derive(Debug, Clone)]
pub struct ArchiveConfig {
    /// Maximum password attempts before destruction
    pub max_attempts: u32,
    /// Encryption algorithm to use
    pub encryption_algorithm: EncryptionAlgorithm,
    /// Compression algorithm to use
    pub compression_algorithm: FormatCompression,
    /// KDF parameters
    pub kdf_params: KdfParams,
}

impl Default for ArchiveConfig {
    fn default() -> Self {
        ArchiveConfig {
            max_attempts: 5,
            encryption_algorithm: EncryptionAlgorithm::Aes256Gcm,
            compression_algorithm: FormatCompression::Lzma2,
            kdf_params: KdfParams::default(),
        }
    }
}

/// Archive writer for creating SecureArc files
pub struct ArchiveWriter {
    config: ArchiveConfig,
    master_key: [u8; MASTER_KEY_SIZE],

    directory: CentralDirectory,
    payload: Vec<u8>,
}

impl ArchiveWriter {
    /// Create a new archive writer with the given configuration
    pub fn new(config: ArchiveConfig) -> Self {
        let master_key = generate_master_key();

        ArchiveWriter {
            config,
            master_key,
            directory: CentralDirectory::new(),
            payload: Vec::new(),
        }
    }

    /// Add a file to the archive
    pub fn add_file<P: AsRef<Path>>(
        &mut self,
        file_path: P,
        archive_path: PathBuf,
    ) -> Result<(), SecureArcError> {
        let path = file_path.as_ref();
        let file_data = std::fs::read(path)?;

        // Get file metadata
        let metadata = std::fs::metadata(path)?;
        let modified_time = metadata
            .modified()
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Compress data
        let compression_algo = match self.config.compression_algorithm {
            FormatCompression::Lzma2 => CompressionAlgorithm::Lzma2,
            FormatCompression::Zstd => CompressionAlgorithm::Zstd,
            FormatCompression::Brotli => CompressionAlgorithm::Brotli,
            FormatCompression::None => CompressionAlgorithm::None,
        };
        let compressed_data = compress_data(&file_data, compression_algo)?;

        // Encrypt data
        let encryption_key = EncryptionKey::from_bytes(&self.master_key)?;
        let encrypted_data = encrypt_data(
            &compressed_data,
            &encryption_key,
            self.config.encryption_algorithm,
        )?;

        // Record file entry
        let data_offset = self.payload.len() as u64;
        let encrypted_size = encrypted_data.len() as u64;
        let entry = FileEntry {
            path: archive_path,
            original_size: file_data.len() as u64,
            compressed_size: compressed_data.len() as u64,
            encrypted_size,
            modified_time,
            attributes: 0,
            data_offset,
        };

        // Append to payload
        self.payload.extend_from_slice(&encrypted_data);
        self.directory.add_entry(entry);

        Ok(())
    }

    /// Write the archive to a file
    pub fn write_to_file<P: AsRef<Path>>(
        &mut self,
        output_path: P,
        password: &[u8],
    ) -> Result<(), SecureArcError> {
        let file = File::create(output_path)?;
        let mut writer = BufWriter::new(file);

        // Write magic number
        writer.write_all(MAGIC_NUMBER)?;

        // Create security header
        let mut header = SecurityHeader::new(self.config.max_attempts)?;
        header.kdf_algorithm = self.config.kdf_params.algorithm;
        header.kdf_memory = self.config.kdf_params.memory;
        header.kdf_iterations = self.config.kdf_params.iterations;
        header.kdf_parallelism = self.config.kdf_params.parallelism;
        header.encryption_algorithm = self.config.encryption_algorithm;
        header.compression_algorithm = self.config.compression_algorithm;

        // Derive key from password
        let derived_key = derive_key(password, &header.salt, &self.config.kdf_params)?;

        // Derive integrity key from password (separate from encryption key)
        let mut integrity_salt = header.salt;
        integrity_salt[0] ^= 0xFF; // Modify salt for integrity key derivation
        let integrity_key = IntegrityKey::from_bytes(&derive_key(
            password,
            &integrity_salt,
            &self.config.kdf_params,
        )?)?;

        // Create key slots (encrypt master key with derived key)
        let mut key_slots = Vec::new();
        let encryption_key = EncryptionKey::from_bytes(&derived_key)?;

        // Primary key slot
        let master_key_encrypted = encrypt_data(
            &self.master_key,
            &encryption_key,
            self.config.encryption_algorithm,
        )?;
        let mut primary_slot = KeySlot::new(0);
        primary_slot.encrypted_key = master_key_encrypted;
        primary_slot.active = true;
        key_slots.push(primary_slot);

        // Update header checksum
        let header_data = self.serialize_header_for_hmac(&header)?;

        header.checksum = compute_checksum(&header_data, &integrity_key);

        // Write header (with size prefix)
        let header_data = bincode::serialize(&header).map_err(|e| {
            SecureArcError::FormatError(format!("Failed to serialize header: {}", e))
        })?;
        writer.write_all(&(header_data.len() as u32).to_le_bytes())?;
        writer.write_all(&header_data)?;

        // Write key slots (with size prefix)
        writer.write_all(&(key_slots.len() as u32).to_le_bytes())?;
        for slot in &key_slots {
            let slot_data = bincode::serialize(slot).map_err(|e| {
                SecureArcError::KeySlotError(format!("Failed to serialize key slot: {}", e))
            })?;
            writer.write_all(&(slot_data.len() as u32).to_le_bytes())?;
            writer.write_all(&slot_data)?;
        }

        // Encrypt and write central directory
        let directory_data = bincode::serialize(&self.directory).map_err(|e| {
            SecureArcError::FormatError(format!("Failed to serialize directory: {}", e))
        })?;
        let encrypted_directory = encrypt_data(
            &directory_data,
            &EncryptionKey::from_bytes(&self.master_key)?,
            self.config.encryption_algorithm,
        )?;
        writer.write_all(&(encrypted_directory.len() as u64).to_le_bytes())?;
        writer.write_all(&encrypted_directory)?;

        // Write payload
        writer.write_all(&(self.payload.len() as u64).to_le_bytes())?;
        writer.write_all(&self.payload)?;

        writer.flush()?;
        Ok(())
    }

    /// Serialize header for HMAC computation
    fn serialize_header_for_hmac(
        &self,
        header: &SecurityHeader,
    ) -> Result<Vec<u8>, SecureArcError> {
        let mut temp_header = header.clone();
        temp_header.checksum = [0u8; 32];
        let result = bincode::serialize(&temp_header).map_err(|e| {
            SecureArcError::FormatError(format!("Failed to serialize header: {}", e))
        })?;

        Ok(result)
    }
}
