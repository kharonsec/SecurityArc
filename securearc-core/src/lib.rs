//! SecureArc Core Library
//!
//! This library provides the core functionality for creating, reading, and managing
//! SecureArc self-destructing encrypted archive files.

pub mod archive;
pub mod compression;
pub mod crypto;
pub mod format;
pub mod self_destruct;

pub use archive::{ArchiveInfo, ArchiveReader, ArchiveWriter, SecureArcError};
pub use format::{CompressionAlgorithm, EncryptionAlgorithm, KdfAlgorithm};
