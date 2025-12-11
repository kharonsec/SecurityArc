//! Archive reader for reading and extracting SecureArc files

use crate::compression::decompress_data;
use crate::format::CompressionAlgorithm;
use crate::crypto::encryption::{decrypt_data, EncryptionKey};
use crate::crypto::integrity::IntegrityKey;
use crate::crypto::kdf::{derive_key, KdfParams};
use crate::format::directory::CentralDirectory;
use crate::format::header::SecurityHeader;
use crate::format::keyslot::KeySlot;
use crate::format::{CompressionAlgorithm as FormatCompression, EncryptionAlgorithm, MAGIC_NUMBER};
use crate::self_destruct::counter::AttemptCounter;
use crate::self_destruct::destruction::SelfDestruct;
use crate::SecureArcError;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

/// Archive reader for reading SecureArc files
pub struct ArchiveReader {
    file: BufReader<File>,
    header: SecurityHeader,
    key_slots: Vec<KeySlot>,
    directory: CentralDirectory,
    master_key: Option<[u8; 32]>,
    encryption_algorithm: EncryptionAlgorithm,
    compression_algorithm: FormatCompression,
    directory_offset: u64,
    payload_offset: u64,
}

impl ArchiveReader {
    /// Open an archive file for reading
    pub fn open<P: AsRef<Path>>(archive_path: P) -> Result<Self, SecureArcError> {
        let file = OpenOptions::new().read(true).write(true).open(archive_path)?;
        let mut reader = BufReader::new(file);

        // Read and verify magic number
        let mut magic = [0u8; 8];
        reader.read_exact(&mut magic)?;
        if magic != *MAGIC_NUMBER {
            return Err(SecureArcError::FormatError(
                "Invalid magic number".to_string(),
            ));
        }

        // Read security header (with size prefix)
        let mut header_size_bytes = [0u8; 4];
        reader.read_exact(&mut header_size_bytes)?;
        let header_size = u32::from_le_bytes(header_size_bytes) as usize;
        let mut header_data = vec![0u8; header_size];
        reader.read_exact(&mut header_data)?;
        let header: SecurityHeader = bincode::deserialize(&header_data).map_err(|e| {
            SecureArcError::HeaderCorrupted(format!("Failed to deserialize header: {}", e))
        })?;
        header.validate()?;

        // Check if destroyed
        if SelfDestruct::is_destroyed(&header) {
            return Err(SecureArcError::ArchiveDestroyed);
        }

        // Track offsets
        let header_offset = 8; // After magic number
        let header_total_size = 4 + header_data.len() as u64; // +4 for size prefix
        let key_slots_offset = header_offset + header_total_size;
        
        // Read key slots count and data
        let mut key_slots = Vec::new();
        let mut slot_count_bytes = [0u8; 4];
        reader.read_exact(&mut slot_count_bytes)?;
        let slot_count = u32::from_le_bytes(slot_count_bytes) as usize;
        
        let mut current_offset = key_slots_offset + 4; // +4 for slot count
        for _ in 0..slot_count {
            // Read slot size
            let mut slot_size_bytes = [0u8; 4];
            reader.read_exact(&mut slot_size_bytes)?;
            let slot_size = u32::from_le_bytes(slot_size_bytes) as usize;
            
            // Read slot data
            let mut slot_data = vec![0u8; slot_size];
            reader.read_exact(&mut slot_data)?;
            
            // Deserialize slot
            let slot = bincode::deserialize(&slot_data).map_err(|e| {
                SecureArcError::KeySlotError(format!("Failed to deserialize key slot: {}", e))
            })?;
            key_slots.push(slot);
            
            current_offset += 4 + slot_size as u64; // +4 for size prefix
        }
        
        let directory_offset = current_offset;
        reader.seek(SeekFrom::Start(directory_offset))?;
        
        // Read directory size and encrypted directory
        let mut dir_size_bytes = [0u8; 8];
        reader.read_exact(&mut dir_size_bytes)?;
        let dir_size = u64::from_le_bytes(dir_size_bytes) as usize;
        let mut encrypted_directory = vec![0u8; dir_size];
        reader.read_exact(&mut encrypted_directory)?;
        
        let payload_offset = directory_offset + 8 + dir_size as u64 + 8;
        
        // Extract algorithms before moving header
        let encryption_algorithm = header.encryption_algorithm;
        let compression_algorithm = header.compression_algorithm;

        Ok(ArchiveReader {
            file: reader,
            header,
            key_slots,
            directory: CentralDirectory::new(),
            master_key: None,
            encryption_algorithm,
            compression_algorithm,
            directory_offset,
            payload_offset,
        })
    }

    /// Verify password and unlock the archive
    pub fn unlock(&mut self, password: &[u8]) -> Result<(), SecureArcError> {
        // Check if already destroyed
        if SelfDestruct::is_destroyed(&self.header) {
            return Err(SecureArcError::ArchiveDestroyed);
        }

        // Derive integrity key from password (separate from encryption key)
        // Use a different salt derivation for integrity key
        let mut integrity_salt = self.header.salt;
        integrity_salt[0] ^= 0xFF; // Modify salt for integrity key derivation

        let integrity_key = IntegrityKey::from_bytes(&derive_key(
            password,
            &integrity_salt,
            &KdfParams {
                algorithm: self.header.kdf_algorithm,
                memory: self.header.kdf_memory,
                iterations: self.header.kdf_iterations,
                parallelism: self.header.kdf_parallelism,
            },
        )?)?;


        // Verify header checksum
        let counter = AttemptCounter::new(integrity_key.clone());

        if counter.verify_checksum(&self.header).is_err() {
            // Checksum verification failed - increment counter
            counter.increment(&mut self.header)?;
            
            // Check if we should destroy
            if counter.should_destroy(&self.header) {
                SelfDestruct::execute_destruction(&mut self.header, &mut self.key_slots)?;
                self.update_file()?;
                return Err(SecureArcError::MaxAttemptsExceeded);
            }
            
            self.update_file()?;
            return Err(SecureArcError::InvalidPassword);
        }

        // Derive key from password
        let kdf_params = KdfParams {
            algorithm: self.header.kdf_algorithm,
            memory: self.header.kdf_memory,
            iterations: self.header.kdf_iterations,
            parallelism: self.header.kdf_parallelism,
        };
        let derived_key = derive_key(password, &self.header.salt, &kdf_params)?;

        // Try to decrypt master key from key slot
        let encryption_key = EncryptionKey::from_bytes(&derived_key)?;

        let master_key_data = decrypt_data(
            &self.key_slots[0].encrypted_key,
            &encryption_key,
            self.encryption_algorithm,
        )
        .map_err(|_e| {

            SecureArcError::InvalidPassword
        })?;


        if master_key_data.len() != 32 {
            // Wrong password - increment counter
            counter.increment(&mut self.header)?;
            
            if counter.should_destroy(&self.header) {
                SelfDestruct::execute_destruction(&mut self.header, &mut self.key_slots)?;
                self.update_file()?;
                return Err(SecureArcError::MaxAttemptsExceeded);
            }
            
            self.update_file()?;
            return Err(SecureArcError::InvalidPassword);
        }

        // Store master key
        let mut master_key = [0u8; 32];
        master_key.copy_from_slice(&master_key_data);
        self.master_key = Some(master_key);

        // Decrypt directory
        let master_encryption_key = EncryptionKey::from_bytes(&master_key)?;
        let current_pos = self.file.stream_position()?;
        self.file.seek(SeekFrom::Start(self.directory_offset))?;
        
        let mut dir_size_bytes = [0u8; 8];
        self.file.read_exact(&mut dir_size_bytes)?;
        let dir_size = u64::from_le_bytes(dir_size_bytes) as usize;
        let mut encrypted_directory = vec![0u8; dir_size];
        self.file.read_exact(&mut encrypted_directory)?;
        
        let directory_data = decrypt_data(
            &encrypted_directory,
            &master_encryption_key,
            self.encryption_algorithm,
        )?;
        
        self.directory = bincode::deserialize(&directory_data).map_err(|e| {
            SecureArcError::FormatError(format!("Failed to deserialize directory: {}", e))
        })?;

        self.file.seek(SeekFrom::Start(current_pos))?;
        Ok(())
    }

    /// List files in the archive
    pub fn list_files(&self) -> Result<Vec<PathBuf>, SecureArcError> {
        if self.master_key.is_none() {
            return Err(SecureArcError::InvalidConfiguration(
                "Archive must be unlocked first".to_string(),
            ));
        }
        Ok(self.directory.entries().iter().map(|e| e.path.clone()).collect())
    }

    /// Extract a file from the archive
    pub fn extract_file<P: AsRef<Path>>(
        &mut self,
        archive_path: &PathBuf,
        output_path: P,
    ) -> Result<(), SecureArcError> {
        let master_key = self.master_key.ok_or_else(|| {
            SecureArcError::InvalidConfiguration("Archive must be unlocked first".to_string())
        })?;

        // Find file entry
        let entry = self
            .directory
            .find_entry(archive_path)
            .ok_or_else(|| SecureArcError::FileNotFound(archive_path.display().to_string()))?;

        // Read encrypted file data
        self.file.seek(SeekFrom::Start(self.payload_offset + entry.data_offset))?;
        
        // Read encrypted data using stored encrypted_size
        let mut encrypted_data = vec![0u8; entry.encrypted_size as usize];
        self.file.read_exact(&mut encrypted_data)?;

        // Decrypt data
        let encryption_key = EncryptionKey::from_bytes(&master_key)?;

        let compressed_data = decrypt_data(&encrypted_data, &encryption_key, self.encryption_algorithm)
            .map_err(|e| {

                e
            })?;

        // Decompress data
        let compression_algo = match self.compression_algorithm {
            FormatCompression::Lzma2 => CompressionAlgorithm::Lzma2,
            FormatCompression::Zstd => CompressionAlgorithm::Zstd,
            FormatCompression::Brotli => CompressionAlgorithm::Brotli,
            FormatCompression::None => CompressionAlgorithm::None,
        };
        let file_data = decompress_data(&compressed_data, compression_algo)?;

        // Write to output file
        std::fs::write(output_path, file_data)?;
        Ok(())
    }

    /// Get archive information
    pub fn get_info(&self) -> ArchiveInfo {
        ArchiveInfo {
            max_attempts: self.header.max_attempts,
            current_attempts: self.header.attempt_counter,
            remaining_attempts: self.header.max_attempts.saturating_sub(self.header.attempt_counter),
            destroyed: self.header.destroyed,
            file_count: self.directory.entries().len(),
        }
    }

    /// Update the archive file with current header and key slots
    fn update_file(&mut self) -> Result<(), SecureArcError> {
        let current_pos = self.file.stream_position()?;
        
        // Write header
        self.file.seek(SeekFrom::Start(8))?; // Skip magic number
        
        let header_data = bincode::serialize(&self.header).map_err(|e| {
            SecureArcError::FormatError(format!("Failed to serialize header: {}", e))
        })?;
        
        // Write header size and data
        self.file.get_mut().write_all(&(header_data.len() as u32).to_le_bytes())?;
        self.file.get_mut().write_all(&header_data)?;
        
        // Write key slots
        // Note: we assume key slots count hasn't changed, only content (e.g. zeroization)
        let key_slots_count_offset = 8 + 4 + header_data.len() as u64; // magic + header_size + header
        self.file.seek(SeekFrom::Start(key_slots_count_offset))?;
        
        self.file.get_mut().write_all(&(self.key_slots.len() as u32).to_le_bytes())?;
        
        for slot in &self.key_slots {
            let slot_data = bincode::serialize(slot).map_err(|e| {
                SecureArcError::KeySlotError(format!("Failed to serialize key slot: {}", e))
            })?;
            self.file.get_mut().write_all(&(slot_data.len() as u32).to_le_bytes())?;
            self.file.get_mut().write_all(&slot_data)?;
        }
        
        self.file.get_mut().flush()?;
        self.file.seek(SeekFrom::Start(current_pos))?;
        Ok(())
    }

}

/// Archive information
#[derive(Debug, Clone)]
pub struct ArchiveInfo {
    pub max_attempts: u32,
    pub current_attempts: u32,
    pub remaining_attempts: u32,
    pub destroyed: bool,
    pub file_count: usize,
}

