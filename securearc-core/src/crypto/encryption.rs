//! Encryption and decryption operations

use crate::format::EncryptionAlgorithm;
use crate::SecureArcError;
use chacha20poly1305::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    ChaCha20Poly1305, Key, Nonce,
};
use rand::RngCore;

/// Size of encryption key in bytes (256 bits)
pub const ENCRYPTION_KEY_SIZE: usize = 32;

/// Size of nonce/IV for ChaCha20-Poly1305
pub const CHACHA20_NONCE_SIZE: usize = 12;

/// Size of nonce/IV for AES-256-GCM
pub const AES_NONCE_SIZE: usize = 12;

/// Encryption key wrapper
#[derive(Clone)]
pub struct EncryptionKey {
    key: [u8; ENCRYPTION_KEY_SIZE],
}

impl EncryptionKey {
    /// Create encryption key from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, SecureArcError> {
        if bytes.len() != ENCRYPTION_KEY_SIZE {
            return Err(SecureArcError::InvalidConfiguration(format!(
                "Key must be {} bytes, got {}",
                ENCRYPTION_KEY_SIZE,
                bytes.len()
            )));
        }
        let mut key = [0u8; ENCRYPTION_KEY_SIZE];
        key.copy_from_slice(bytes);
        Ok(EncryptionKey { key })
    }

    /// Get key bytes
    pub fn as_bytes(&self) -> &[u8; ENCRYPTION_KEY_SIZE] {
        &self.key
    }
}

/// Generate a cryptographically random master key
pub fn generate_master_key() -> [u8; ENCRYPTION_KEY_SIZE] {
    let mut key = [0u8; ENCRYPTION_KEY_SIZE];
    rand::thread_rng().fill_bytes(&mut key);
    key
}

/// Encrypt data using the specified algorithm
pub fn encrypt_data(
    data: &[u8],
    key: &EncryptionKey,
    algorithm: EncryptionAlgorithm,
) -> Result<Vec<u8>, SecureArcError> {
    match algorithm {
        EncryptionAlgorithm::Aes256Gcm => {
            // Use ring for AES-256-GCM
            use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM};
            use ring::rand::{SecureRandom, SystemRandom};

            let unbound_key = UnboundKey::new(&AES_256_GCM, key.as_bytes())
                .map_err(|e| SecureArcError::EncryptionError(format!("Invalid key: {}", e)))?;

            let rng = SystemRandom::new();
            let mut nonce_bytes = [0u8; AES_NONCE_SIZE];
            rng.fill(&mut nonce_bytes)
                .map_err(|e| SecureArcError::EncryptionError(format!("Failed to generate nonce: {}", e)))?;

            let nonce = Nonce::assume_unique_for_key(nonce_bytes);
            let key = LessSafeKey::new(unbound_key);
            let mut in_out = data.to_vec();
            key.seal_in_place_append_tag(nonce, Aad::empty(), &mut in_out)
                .map_err(|e| SecureArcError::EncryptionError(format!("Encryption failed: {}", e)))?;

            // Prepend nonce
            let mut result = nonce_bytes.to_vec();
            result.extend_from_slice(&in_out);
            Ok(result)
        }
        EncryptionAlgorithm::ChaCha20Poly1305 => {
            let cipher = ChaCha20Poly1305::new(Key::from_slice(key.as_bytes()).into());
            let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
            let ciphertext = cipher
                .encrypt(&nonce, data)
                .map_err(|e| SecureArcError::EncryptionError(format!("Encryption failed: {}", e)))?;

            // Prepend nonce
            let mut result = nonce.to_vec();
            result.extend_from_slice(&ciphertext);
            Ok(result)
        }
    }
}

/// Decrypt data using the specified algorithm
pub fn decrypt_data(
    encrypted_data: &[u8],
    key: &EncryptionKey,
    algorithm: EncryptionAlgorithm,
) -> Result<Vec<u8>, SecureArcError> {
    match algorithm {
        EncryptionAlgorithm::Aes256Gcm => {
            // Use ring for AES-256-GCM
            use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM};

            if encrypted_data.len() < AES_NONCE_SIZE + 16 {
                return Err(SecureArcError::EncryptionError(
                    "Encrypted data too short".to_string(),
                ));
            }

            let nonce_bytes: [u8; AES_NONCE_SIZE] = encrypted_data[..AES_NONCE_SIZE]
                .try_into()
                .map_err(|_| SecureArcError::EncryptionError("Invalid nonce size".to_string()))?;

            let unbound_key = UnboundKey::new(&AES_256_GCM, key.as_bytes())
                .map_err(|e| SecureArcError::EncryptionError(format!("Invalid key: {}", e)))?;

            let nonce = Nonce::assume_unique_for_key(nonce_bytes);
            let key = LessSafeKey::new(unbound_key);
            let mut in_out = encrypted_data[AES_NONCE_SIZE..].to_vec();

            let plaintext = key.open_in_place(nonce, Aad::empty(), &mut in_out)
                .map_err(|e| SecureArcError::EncryptionError(format!("Decryption failed: {}", e)))?;

            // open_in_place returns a slice excluding the tag, convert to Vec
            Ok(plaintext.to_vec())
        }
        EncryptionAlgorithm::ChaCha20Poly1305 => {
            if encrypted_data.len() < CHACHA20_NONCE_SIZE {
                return Err(SecureArcError::EncryptionError(
                    "Encrypted data too short".to_string(),
                ));
            }

            let nonce = Nonce::from_slice(&encrypted_data[..CHACHA20_NONCE_SIZE]);
            let ciphertext = &encrypted_data[CHACHA20_NONCE_SIZE..];

            let cipher = ChaCha20Poly1305::new(Key::from_slice(key.as_bytes()).into());
            cipher
                .decrypt(nonce, ciphertext)
                .map_err(|e| SecureArcError::EncryptionError(format!("Decryption failed: {}", e)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aes256_gcm_encryption() {
        let key = generate_master_key();
        let encryption_key = EncryptionKey::from_bytes(&key).unwrap();
        let data = b"Hello, SecureArc!";

        let encrypted = encrypt_data(data, &encryption_key, EncryptionAlgorithm::Aes256Gcm).unwrap();
        let decrypted = decrypt_data(&encrypted, &encryption_key, EncryptionAlgorithm::Aes256Gcm).unwrap();

        assert_eq!(data, decrypted.as_slice());
    }

    #[test]
    fn test_chacha20_encryption() {
        let key = generate_master_key();
        let encryption_key = EncryptionKey::from_bytes(&key).unwrap();
        let data = b"Hello, SecureArc!";

        let encrypted = encrypt_data(data, &encryption_key, EncryptionAlgorithm::ChaCha20Poly1305).unwrap();
        let decrypted = decrypt_data(&encrypted, &encryption_key, EncryptionAlgorithm::ChaCha20Poly1305).unwrap();

        assert_eq!(data, decrypted.as_slice());
    }
}

