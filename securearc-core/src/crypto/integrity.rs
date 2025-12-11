//! HMAC-SHA256 integrity checking

use crate::SecureArcError;
use hmac::{Hmac, Mac};
use sha2::Sha256;

/// Size of HMAC-SHA256 output in bytes
pub const HMAC_SIZE: usize = 32;

/// Type alias for HMAC-SHA256
type HmacSha256 = Hmac<Sha256>;

/// Integrity key wrapper
#[derive(Clone)]
pub struct IntegrityKey {
    key: [u8; 32],
}

impl IntegrityKey {
    /// Create integrity key from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, SecureArcError> {
        if bytes.len() != 32 {
            return Err(SecureArcError::InvalidConfiguration(format!(
                "Integrity key must be 32 bytes, got {}",
                bytes.len()
            )));
        }
        let mut key = [0u8; 32];
        key.copy_from_slice(bytes);
        Ok(IntegrityKey { key })
    }

    /// Get key bytes
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.key
    }
}

/// Compute HMAC-SHA256 checksum
pub fn compute_checksum(data: &[u8], key: &IntegrityKey) -> [u8; HMAC_SIZE] {
    let mut mac = HmacSha256::new_from_slice(key.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(data);
    let result = mac.finalize();
    let mut checksum = [0u8; HMAC_SIZE];
    checksum.copy_from_slice(result.into_bytes().as_slice());
    checksum
}

/// Verify HMAC-SHA256 checksum
pub fn verify_checksum(
    data: &[u8],
    key: &IntegrityKey,
    expected_checksum: &[u8],
) -> Result<(), SecureArcError> {
    if expected_checksum.len() != HMAC_SIZE {
        return Err(SecureArcError::IntegrityCheckFailed(
            "Invalid checksum length".to_string(),
        ));
    }

    let computed = compute_checksum(data, key);
    if computed != expected_checksum {
        return Err(SecureArcError::IntegrityCheckFailed(
            "Checksum mismatch".to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hmac_computation() {
        let key = IntegrityKey::from_bytes(&[0u8; 32]).unwrap();
        let data = b"test data";

        let checksum1 = compute_checksum(data, &key);
        let checksum2 = compute_checksum(data, &key);

        // Same data and key should produce same checksum
        assert_eq!(checksum1, checksum2);

        // Different data should produce different checksum
        let checksum3 = compute_checksum(b"different data", &key);
        assert_ne!(checksum1, checksum3);
    }

    #[test]
    fn test_hmac_verification() {
        let key = IntegrityKey::from_bytes(&[0u8; 32]).unwrap();
        let data = b"test data";

        let checksum = compute_checksum(data, &key);
        assert!(verify_checksum(data, &key, &checksum).is_ok());

        // Wrong checksum should fail
        let wrong_checksum = [0u8; HMAC_SIZE];
        assert!(verify_checksum(data, &key, &wrong_checksum).is_err());
    }
}

