//! Central directory for file listing and metadata

use crate::SecureArcError;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::path::PathBuf;

/// File entry in the central directory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    /// File path within archive
    pub path: PathBuf,
    /// Original file size (uncompressed)
    pub original_size: u64,
    /// Compressed size
    pub compressed_size: u64,
    /// Encrypted size (compressed + encryption overhead)
    pub encrypted_size: u64,
    /// File modification time (Unix timestamp)
    pub modified_time: u64,
    /// File attributes/permissions
    pub attributes: u32,
    /// Offset to encrypted file data in payload
    pub data_offset: u64,
}

/// Central directory containing encrypted file listing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CentralDirectory {
    /// File entries
    pub entries: Vec<FileEntry>,
    /// Encrypted directory data (when stored encrypted)
    pub encrypted_data: Vec<u8>,
}

impl CentralDirectory {
    /// Create a new empty central directory
    pub fn new() -> Self {
        CentralDirectory {
            entries: Vec::new(),
            encrypted_data: Vec::new(),
        }
    }

    /// Add a file entry
    pub fn add_entry(&mut self, entry: FileEntry) {
        self.entries.push(entry);
    }

    /// Find a file entry by path
    pub fn find_entry(&self, path: &PathBuf) -> Option<&FileEntry> {
        self.entries.iter().find(|e| &e.path == path)
    }

    /// Get all file entries
    pub fn entries(&self) -> &[FileEntry] {
        &self.entries
    }

    /// Read central directory from reader
    pub fn read<R: Read>(reader: &mut R) -> Result<Self, SecureArcError> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        bincode::deserialize(&buf).map_err(|e| {
            SecureArcError::FormatError(format!("Failed to deserialize directory: {}", e))
        })
    }

    /// Write central directory to writer
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<(), SecureArcError> {
        let serialized = bincode::serialize(self).map_err(|e| {
            SecureArcError::FormatError(format!("Failed to serialize directory: {}", e))
        })?;
        writer.write_all(&serialized)?;
        Ok(())
    }
}

impl Default for CentralDirectory {
    fn default() -> Self {
        Self::new()
    }
}

