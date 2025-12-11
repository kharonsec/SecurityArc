//! Key derivation function implementations

use crate::format::KdfAlgorithm;
use crate::SecureArcError;
use argon2::Argon2;
use argon2::Params;
use pbkdf2::pbkdf2_hmac;
use sha2::Sha256;

/// Key derivation parameters
#[derive(Debug, Clone)]
pub struct KdfParams {
    /// Algorithm to use
    pub algorithm: KdfAlgorithm,
    /// Memory cost (for Argon2, in KB)
    pub memory: u32,
    /// Iterations
    pub iterations: u32,
    /// Parallelism (threads)
    pub parallelism: u32,
}

impl Default for KdfParams {
    fn default() -> Self {
        KdfParams {
            algorithm: KdfAlgorithm::Argon2id,
            memory: 64 * 1024, // 64 MB in KB
            iterations: 3,
            parallelism: 4,
        }
    }
}

/// Derive a 32-byte (256-bit) key from a password and salt
pub fn derive_key(
    password: &[u8],
    salt: &[u8],
    params: &KdfParams,
) -> Result<[u8; 32], SecureArcError> {
    let mut key = [0u8; 32];

    match params.algorithm {
        KdfAlgorithm::Argon2id => {
            // Validate parameters
            if params.memory < 8 * 1024 {
                return Err(SecureArcError::InvalidConfiguration(
                    "Argon2 memory must be at least 8 MB".to_string(),
                ));
            }
            if params.iterations < 1 {
                return Err(SecureArcError::InvalidConfiguration(
                    "Argon2 iterations must be at least 1".to_string(),
                ));
            }
            if params.parallelism < 1 || params.parallelism > 16 {
                return Err(SecureArcError::InvalidConfiguration(
                    "Argon2 parallelism must be between 1 and 16".to_string(),
                ));
            }

            // Create Argon2id instance
            let argon2_params = Params::new(
                params.memory,
                params.iterations,
                params.parallelism,
                Some(32), // Output length in bytes
            )
            .map_err(|e| {
                SecureArcError::KeyDerivationError(format!("Invalid Argon2 parameters: {}", e))
            })?;

            let argon2 = Argon2::new(
                argon2::Algorithm::Argon2id,
                argon2::Version::V0x13,
                argon2_params,
            );

            // Derive key
            argon2
                .hash_password_into(password, salt, &mut key)
                .map_err(|e| {
                    SecureArcError::KeyDerivationError(format!(
                        "Argon2 key derivation failed: {}",
                        e
                    ))
                })?;
        }
        KdfAlgorithm::Pbkdf2Sha256 => {
            // Validate parameters
            if params.iterations < 1000 {
                return Err(SecureArcError::InvalidConfiguration(
                    "PBKDF2 iterations must be at least 1000".to_string(),
                ));
            }

            // Derive key using PBKDF2-SHA256
            pbkdf2_hmac::<Sha256>(password, salt, params.iterations, &mut key);
        }
    }

    Ok(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_argon2id_key_derivation() {
        let password = b"test_password";
        let salt = b"test_salt_12345678901234567890";
        let params = KdfParams::default();

        let key1 = derive_key(password, salt, &params).unwrap();
        let key2 = derive_key(password, salt, &params).unwrap();

        // Same password and salt should produce same key
        assert_eq!(key1, key2);

        // Different password should produce different key
        let key3 = derive_key(b"different_password", salt, &params).unwrap();
        assert_ne!(key1, key3);

        // Different salt should produce different key
        let key4 = derive_key(password, b"different_salt_1234567890123456", &params).unwrap();
        assert_ne!(key1, key4);
    }

    #[test]
    fn test_pbkdf2_key_derivation() {
        let password = b"test_password";
        let salt = b"test_salt_12345678901234567890";
        let params = KdfParams {
            algorithm: KdfAlgorithm::Pbkdf2Sha256,
            memory: 0, // Not used for PBKDF2
            iterations: 10000,
            parallelism: 1, // Not used for PBKDF2
        };

        let key1 = derive_key(password, salt, &params).unwrap();
        let key2 = derive_key(password, salt, &params).unwrap();

        // Same password and salt should produce same key
        assert_eq!(key1, key2);
    }
}
