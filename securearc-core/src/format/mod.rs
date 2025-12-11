//! SecureArc file format specification and structures

pub mod directory;
pub mod header;
pub mod keyslot;

use serde::{Deserialize, Serialize};

/// Magic number identifying SecureArc files
pub const MAGIC_NUMBER: &[u8; 8] = b"SECARC01";

/// File format version
pub const FORMAT_VERSION: u16 = 1;

/// Maximum number of key slots supported
pub const MAX_KEY_SLOTS: usize = 8;

/// Minimum allowed max attempts
pub const MIN_MAX_ATTEMPTS: u32 = 3;

/// Maximum allowed max attempts
pub const MAX_MAX_ATTEMPTS: u32 = 99;

/// Key derivation function algorithms
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum KdfAlgorithm {
    /// Argon2id (recommended)
    Argon2id = 1,
    /// PBKDF2-SHA256 (legacy support)
    Pbkdf2Sha256 = 2,
}

/// Encryption algorithms
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum EncryptionAlgorithm {
    /// AES-256-GCM
    Aes256Gcm = 1,
    /// ChaCha20-Poly1305
    ChaCha20Poly1305 = 2,
}

/// Compression algorithms
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum CompressionAlgorithm {
    /// LZMA2
    Lzma2 = 1,
    /// Zstd
    Zstd = 2,
    /// Brotli
    Brotli = 3,
    /// No compression
    None = 0,
}

/// Complete SecureArc file structure
#[derive(Debug)]
pub struct SecureArcFile {
    /// Security header
    pub header: header::SecurityHeader,
    /// Key slots
    pub key_slots: Vec<keyslot::KeySlot>,
    /// Encrypted payload
    pub payload: Vec<u8>,
    /// Central directory
    pub directory: directory::CentralDirectory,
}

