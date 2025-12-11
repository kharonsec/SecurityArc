//! Cryptographic component tests

use securearc_core::crypto::encryption::{decrypt_data, encrypt_data, generate_master_key, EncryptionKey};
use securearc_core::crypto::integrity::{compute_checksum, IntegrityKey, verify_checksum};
use securearc_core::crypto::kdf::{derive_key, KdfParams};
use securearc_core::format::{EncryptionAlgorithm, KdfAlgorithm};

#[test]
fn test_encryption_round_trip() {
    let key = generate_master_key();
    let encryption_key = EncryptionKey::from_bytes(&key).unwrap();
    let data = b"Test data for encryption";

    // Test AES-256-GCM
    let encrypted = encrypt_data(data, &encryption_key, EncryptionAlgorithm::Aes256Gcm).unwrap();
    let decrypted = decrypt_data(&encrypted, &encryption_key, EncryptionAlgorithm::Aes256Gcm).unwrap();
    assert_eq!(data, decrypted.as_slice());

    // Test ChaCha20-Poly1305
    let encrypted = encrypt_data(data, &encryption_key, EncryptionAlgorithm::ChaCha20Poly1305).unwrap();
    let decrypted = decrypt_data(&encrypted, &encryption_key, EncryptionAlgorithm::ChaCha20Poly1305).unwrap();
    assert_eq!(data, decrypted.as_slice());
}

#[test]
fn test_kdf_consistency() {
    let password = b"test_password";
    let salt = b"test_salt_12345678901234567890";
    let params = KdfParams::default();

    let key1 = derive_key(password, salt, &params).unwrap();
    let key2 = derive_key(password, salt, &params).unwrap();

    // Same password and salt should produce same key
    assert_eq!(key1, key2);

    // Different password should produce different key
    let key3 = derive_key(b"different", salt, &params).unwrap();
    assert_ne!(key1, key3);

    // Different salt should produce different key
    let key4 = derive_key(password, b"different_salt_1234567890123456", &params).unwrap();
    assert_ne!(key1, key4);
}

#[test]
fn test_hmac_integrity() {
    let key = IntegrityKey::from_bytes(&[0u8; 32]).unwrap();
    let data = b"Test data for HMAC";

    let checksum = compute_checksum(data, &key);
    assert!(verify_checksum(data, &key, &checksum).is_ok());

    // Tampered data should fail
    assert!(verify_checksum(b"Tampered data", &key, &checksum).is_err());

    // Wrong checksum should fail
    let wrong_checksum = [0u8; 32];
    assert!(verify_checksum(data, &key, &wrong_checksum).is_err());
}

#[test]
fn test_pbkdf2_kdf() {
    let password = b"test_password";
    let salt = b"test_salt_12345678901234567890";
    let params = KdfParams {
        algorithm: KdfAlgorithm::Pbkdf2Sha256,
        memory: 0,
        iterations: 10000,
        parallelism: 1,
    };

    let key1 = derive_key(password, salt, &params).unwrap();
    let key2 = derive_key(password, salt, &params).unwrap();
    assert_eq!(key1, key2);
}

