//! Key slot management for encrypted master keys

use crate::SecureArcError;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

/// Size of master key in bytes (256 bits)
pub const MASTER_KEY_SIZE: usize = 32;

/// Size of encrypted key slot in bytes
pub const KEY_SLOT_SIZE: usize = MASTER_KEY_SIZE + 16; // Key + IV/nonce + tag

/// Key slot containing an encrypted copy of the master key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeySlot {
    /// Encrypted master key
    pub encrypted_key: Vec<u8>,
    /// Key slot identifier (0 = primary, 1+ = recovery slots)
    pub slot_id: u8,
    /// Whether this slot is active
    pub active: bool,
}

impl KeySlot {
    /// Create a new key slot
    pub fn new(slot_id: u8) -> Self {
        KeySlot {
            encrypted_key: Vec::new(),
            slot_id,
            active: false,
        }
    }

    /// Zeroize the key slot (overwrite with random data for self-destruct)
    pub fn zeroize(&mut self) {
        if !self.encrypted_key.is_empty() {
            // Overwrite with cryptographically random data
            rand::thread_rng().fill_bytes(&mut self.encrypted_key);
        }
        self.active = false;
    }

    /// Check if key slot is zeroized (all zeros or random data)
    pub fn is_zeroized(&self) -> bool {
        !self.active || self.encrypted_key.is_empty()
    }

    /// Read key slot from reader
    pub fn read<R: Read>(reader: &mut R) -> Result<Self, SecureArcError> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        bincode::deserialize(&buf).map_err(|e| {
            SecureArcError::KeySlotError(format!("Failed to deserialize key slot: {}", e))
        })
    }

    /// Write key slot to writer
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<(), SecureArcError> {
        let serialized = bincode::serialize(self).map_err(|e| {
            SecureArcError::KeySlotError(format!("Failed to serialize key slot: {}", e))
        })?;
        writer.write_all(&serialized)?;
        Ok(())
    }
}
