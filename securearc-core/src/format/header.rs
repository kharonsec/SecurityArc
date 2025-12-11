//! Security header structure and operations

use crate::format::{CompressionAlgorithm, EncryptionAlgorithm, KdfAlgorithm, MAX_MAX_ATTEMPTS, MIN_MAX_ATTEMPTS};
use crate::SecureArcError;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

/// Size of salt in bytes
pub const SALT_SIZE: usize = 32;

/// Size of HMAC-SHA256 checksum in bytes
pub const CHECKSUM_SIZE: usize = 32;

/// Security header containing KDF parameters, attempt counter, and integrity checks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityHeader {
    /// KDF algorithm identifier
    pub kdf_algorithm: KdfAlgorithm,
    /// KDF memory cost (for Argon2, in KB)
    pub kdf_memory: u32,
    /// KDF iterations
    pub kdf_iterations: u32,
    /// KDF parallelism (threads)
    pub kdf_parallelism: u32,
    /// Encryption algorithm
    pub encryption_algorithm: EncryptionAlgorithm,
    /// Compression algorithm
    pub compression_algorithm: CompressionAlgorithm,
    /// Salt for key derivation (32 bytes)
    pub salt: [u8; SALT_SIZE],
    /// Current failed attempt counter
    pub attempt_counter: u32,
    /// Maximum allowed attempts before destruction
    pub max_attempts: u32,
    /// Header checksum (HMAC-SHA256, 32 bytes)
    pub checksum: [u8; CHECKSUM_SIZE],
    /// Destruction flag (set when archive is destroyed)
    pub destroyed: bool,
}

impl SecurityHeader {
    /// Create a new security header with default Argon2id parameters
    pub fn new(max_attempts: u32) -> Result<Self, SecureArcError> {
        if max_attempts < MIN_MAX_ATTEMPTS || max_attempts > MAX_MAX_ATTEMPTS {
            return Err(SecureArcError::InvalidConfiguration(format!(
                "max_attempts must be between {} and {}",
                MIN_MAX_ATTEMPTS, MAX_MAX_ATTEMPTS
            )));
        }

        let mut salt = [0u8; SALT_SIZE];
        rand::thread_rng().fill_bytes(&mut salt);

        Ok(SecurityHeader {
            kdf_algorithm: KdfAlgorithm::Argon2id,
            kdf_memory: 64 * 1024, // 64 MB in KB
            kdf_iterations: 3,
            kdf_parallelism: 4,
            encryption_algorithm: EncryptionAlgorithm::Aes256Gcm,
            compression_algorithm: CompressionAlgorithm::Lzma2,
            salt,
            attempt_counter: 0,
            max_attempts,
            checksum: [0u8; CHECKSUM_SIZE], // Will be computed after creation
            destroyed: false,
        })
    }

    /// Read security header from a reader
    pub fn read<R: Read>(reader: &mut R) -> Result<Self, SecureArcError> {
        // Header is serialized using serde
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        bincode::deserialize(&buf).map_err(|e| {
            SecureArcError::HeaderCorrupted(format!("Failed to deserialize header: {}", e))
        })
    }

    /// Write security header to a writer
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<(), SecureArcError> {
        let serialized = bincode::serialize(self).map_err(|e| {
            SecureArcError::FormatError(format!("Failed to serialize header: {}", e))
        })?;
        writer.write_all(&serialized)?;
        Ok(())
    }

    /// Validate header integrity and parameters
    pub fn validate(&self) -> Result<(), SecureArcError> {
        if self.max_attempts < MIN_MAX_ATTEMPTS || self.max_attempts > MAX_MAX_ATTEMPTS {
            return Err(SecureArcError::InvalidConfiguration(format!(
                "Invalid max_attempts: {} (must be between {} and {})",
                self.max_attempts, MIN_MAX_ATTEMPTS, MAX_MAX_ATTEMPTS
            )));
        }

        if self.attempt_counter > self.max_attempts {
            return Err(SecureArcError::ArchiveDestroyed);
        }

        if self.destroyed {
            return Err(SecureArcError::ArchiveDestroyed);
        }

        Ok(())
    }

    /// Check if archive should be destroyed
    pub fn should_destroy(&self) -> bool {
        self.attempt_counter >= self.max_attempts || self.destroyed
    }
}

