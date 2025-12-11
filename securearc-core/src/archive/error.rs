//! Error types for SecureArc operations

use thiserror::Error;

/// Errors that can occur when working with SecureArc archives
#[derive(Error, Debug)]
pub enum SecureArcError {
    /// Invalid password provided
    #[error("Invalid password")]
    InvalidPassword,

    /// Maximum password attempts exceeded, archive destroyed
    #[error("Maximum password attempts exceeded. Archive has been destroyed.")]
    MaxAttemptsExceeded,

    /// Archive has been destroyed and is no longer accessible
    #[error("Archive has been destroyed and is no longer accessible")]
    ArchiveDestroyed,

    /// Security header is corrupted or invalid
    #[error("Security header is corrupted or invalid: {0}")]
    HeaderCorrupted(String),

    /// Integrity check failed (HMAC verification failed)
    #[error("Integrity check failed: {0}")]
    IntegrityCheckFailed(String),

    /// File format error
    #[error("File format error: {0}")]
    FormatError(String),

    /// I/O error
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// Encryption/decryption error
    #[error("Encryption error: {0}")]
    EncryptionError(String),

    /// Compression/decompression error
    #[error("Compression error: {0}")]
    CompressionError(String),

    /// Key derivation error
    #[error("Key derivation error: {0}")]
    KeyDerivationError(String),

    /// Invalid configuration parameter
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),

    /// Attempt counter manipulation detected
    #[error("Attempt counter manipulation detected. Archive may be compromised.")]
    CounterTamperingDetected,

    /// Key slot error
    #[error("Key slot error: {0}")]
    KeySlotError(String),

    /// File not found in archive
    #[error("File not found in archive: {0}")]
    FileNotFound(String),

    /// Archive is empty
    #[error("Archive is empty")]
    EmptyArchive,
}

