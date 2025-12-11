//! Attempt counter management with HMAC protection

use crate::crypto::integrity::{compute_checksum, IntegrityKey};
use crate::format::header::SecurityHeader;
use crate::SecureArcError;

/// Attempt counter manager with HMAC protection
pub struct AttemptCounter {
    /// Integrity key for HMAC computation
    integrity_key: IntegrityKey,
}

impl AttemptCounter {
    /// Create a new attempt counter manager
    pub fn new(integrity_key: IntegrityKey) -> Self {
        AttemptCounter { integrity_key }
    }

    /// Increment the attempt counter in the header
    /// This operation is cryptographically protected and cannot be rolled back
    pub fn increment(&self, header: &mut SecurityHeader) -> Result<(), SecureArcError> {
        // Check if already destroyed
        if header.should_destroy() {
            return Err(SecureArcError::ArchiveDestroyed);
        }

        // Increment counter
        header.attempt_counter += 1;

        // Recompute checksum with new counter value
        // This binds the counter to the header and prevents rollback
        self.update_checksum(header)?;

        Ok(())
    }

    /// Update the header checksum based on current header contents
    pub fn update_checksum(&self, header: &mut SecurityHeader) -> Result<(), SecureArcError> {
        // Serialize header without checksum for HMAC computation
        let header_data = self.serialize_header_for_hmac(header)?;
        let checksum = compute_checksum(&header_data, &self.integrity_key);
        header.checksum = checksum;
        Ok(())
    }

    /// Verify the header checksum
    pub fn verify_checksum(&self, header: &SecurityHeader) -> Result<(), SecureArcError> {
        let header_data = self.serialize_header_for_hmac(header)?;
        let computed_checksum = compute_checksum(&header_data, &self.integrity_key);

        if computed_checksum != header.checksum {
            return Err(SecureArcError::CounterTamperingDetected);
        }

        Ok(())
    }

    /// Serialize header for HMAC computation (excluding checksum field)
    fn serialize_header_for_hmac(&self, header: &SecurityHeader) -> Result<Vec<u8>, SecureArcError> {
        use bincode;
        // Create a temporary header without checksum for serialization
        let mut temp_header = header.clone();
        temp_header.checksum = [0u8; 32]; // Zero out checksum
        let result = bincode::serialize(&temp_header).map_err(|e| {
            SecureArcError::FormatError(format!("Failed to serialize header for HMAC: {}", e))
        })?;

        Ok(result)
    }

    /// Get current attempt count
    pub fn get_attempts(&self, header: &SecurityHeader) -> u32 {
        header.attempt_counter
    }

    /// Get remaining attempts before destruction
    pub fn get_remaining_attempts(&self, header: &SecurityHeader) -> u32 {
        if header.attempt_counter >= header.max_attempts {
            0
        } else {
            header.max_attempts - header.attempt_counter
        }
    }

    /// Check if archive should be destroyed
    pub fn should_destroy(&self, header: &SecurityHeader) -> bool {
        header.should_destroy()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::format::header::SecurityHeader;

    #[test]
    fn test_counter_increment() {
        let integrity_key = IntegrityKey::from_bytes(&[0u8; 32]).unwrap();
        let counter = AttemptCounter::new(integrity_key);
        let mut header = SecurityHeader::new(5).unwrap();

        assert_eq!(counter.get_attempts(&header), 0);
        assert_eq!(counter.get_remaining_attempts(&header), 5);

        counter.increment(&mut header).unwrap();
        assert_eq!(counter.get_attempts(&header), 1);
        assert_eq!(counter.get_remaining_attempts(&header), 4);

        // Verify checksum is valid
        assert!(counter.verify_checksum(&header).is_ok());
    }

    #[test]
    fn test_counter_tampering_detection() {
        let integrity_key = IntegrityKey::from_bytes(&[0u8; 32]).unwrap();
        let counter = AttemptCounter::new(integrity_key);
        let mut header = SecurityHeader::new(5).unwrap();

        // Increment counter first
        counter.increment(&mut header).unwrap();
        assert_eq!(header.attempt_counter, 1);
        assert!(counter.verify_checksum(&header).is_ok());

        // Tamper with counter (try to rollback)
        header.attempt_counter = 0;
        assert!(counter.verify_checksum(&header).is_err());
    }
}

