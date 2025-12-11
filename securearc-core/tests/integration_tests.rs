//! Integration tests for SecureArc

use securearc_core::archive::{ArchiveConfig, ArchiveReader, ArchiveWriter};
use securearc_core::format::{CompressionAlgorithm, EncryptionAlgorithm};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_create_and_extract_archive() {
    let temp_dir = TempDir::new().unwrap();
    let archive_path = temp_dir.path().join("test.sarc");
    let test_file = temp_dir.path().join("test.txt");
    
    // Create test file
    fs::write(&test_file, b"Hello, SecureArc!").unwrap();
    
    // Create archive
    let config = ArchiveConfig::default();
    let mut writer = ArchiveWriter::new(config);
    writer.add_file(&test_file, PathBuf::from("test.txt")).unwrap();
    writer.write_to_file(&archive_path, b"password123").unwrap();
    
    // Extract archive
    let mut reader = ArchiveReader::open(&archive_path).unwrap();
    reader.unlock(b"password123").unwrap();
    
    let files = reader.list_files().unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(files[0], PathBuf::from("test.txt"));
    
    let output_file = temp_dir.path().join("extracted.txt");
    reader.extract_file(&files[0], &output_file).unwrap();
    
    let content = fs::read(&output_file).unwrap();
    assert_eq!(content, b"Hello, SecureArc!");
}

#[test]
fn test_self_destruct_mechanism() {
    let temp_dir = TempDir::new().unwrap();
    let archive_path = temp_dir.path().join("test.sarc");
    let test_file = temp_dir.path().join("test.txt");
    
    // Create test file
    fs::write(&test_file, b"Test data").unwrap();
    
    // Create archive with max_attempts = 3
    let mut config = ArchiveConfig::default();
    config.max_attempts = 3;
    let mut writer = ArchiveWriter::new(config);
    writer.add_file(&test_file, PathBuf::from("test.txt")).unwrap();
    writer.write_to_file(&archive_path, b"correct_password").unwrap();
    
    // Try wrong passwords
    let mut reader = ArchiveReader::open(&archive_path).unwrap();
    
    // First wrong attempt
    assert!(reader.unlock(b"wrong1").is_err());
    
    // Second wrong attempt
    let mut reader = ArchiveReader::open(&archive_path).unwrap();
    assert!(reader.unlock(b"wrong2").is_err());
    
    // Third wrong attempt should destroy archive
    let mut reader = ArchiveReader::open(&archive_path).unwrap();
    let result = reader.unlock(b"wrong3");
    assert!(result.is_err());
    
    // Archive should be destroyed - correct password should fail
    // Archive should be destroyed - opening should fail
    assert!(ArchiveReader::open(&archive_path).is_err());
}

#[test]
fn test_multiple_files() {
    let temp_dir = TempDir::new().unwrap();
    let archive_path = temp_dir.path().join("test.sarc");
    
    // Create multiple test files
    let file1 = temp_dir.path().join("file1.txt");
    let file2 = temp_dir.path().join("file2.txt");
    fs::write(&file1, b"File 1 content").unwrap();
    fs::write(&file2, b"File 2 content").unwrap();
    
    // Create archive
    let config = ArchiveConfig::default();
    let mut writer = ArchiveWriter::new(config);
    writer.add_file(&file1, PathBuf::from("file1.txt")).unwrap();
    writer.add_file(&file2, PathBuf::from("file2.txt")).unwrap();
    writer.write_to_file(&archive_path, b"password").unwrap();
    
    // List files
    let mut reader = ArchiveReader::open(&archive_path).unwrap();
    reader.unlock(b"password").unwrap();
    
    let files = reader.list_files().unwrap();
    assert_eq!(files.len(), 2);
}

#[test]
fn test_different_encryption_algorithms() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, b"Test data").unwrap();
    
    // Test AES-256-GCM
    let archive_path1 = temp_dir.path().join("test_aes.sarc");
    let mut config = ArchiveConfig::default();
    config.encryption_algorithm = EncryptionAlgorithm::Aes256Gcm;
    let mut writer = ArchiveWriter::new(config);
    writer.add_file(&test_file, PathBuf::from("test.txt")).unwrap();
    writer.write_to_file(&archive_path1, b"password").unwrap();
    
    let mut reader = ArchiveReader::open(&archive_path1).unwrap();
    assert!(reader.unlock(b"password").is_ok());
    
    // Test ChaCha20-Poly1305
    let archive_path2 = temp_dir.path().join("test_chacha.sarc");
    let mut config = ArchiveConfig::default();
    config.encryption_algorithm = EncryptionAlgorithm::ChaCha20Poly1305;
    let mut writer = ArchiveWriter::new(config);
    writer.add_file(&test_file, PathBuf::from("test.txt")).unwrap();
    writer.write_to_file(&archive_path2, b"password").unwrap();
    
    let mut reader = ArchiveReader::open(&archive_path2).unwrap();
    assert!(reader.unlock(b"password").is_ok());
}

#[test]
fn test_different_compression_algorithms() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, b"Test data for compression testing").unwrap();
    
    for &algo in &[
        CompressionAlgorithm::Lzma2,
        CompressionAlgorithm::Zstd,
        CompressionAlgorithm::Brotli,
    ] {
        let archive_path = temp_dir.path().join(format!("test_{:?}.sarc", algo));
        let mut config = ArchiveConfig::default();
        config.compression_algorithm = algo;
        let mut writer = ArchiveWriter::new(config);
        writer.add_file(&test_file, PathBuf::from("test.txt")).unwrap();
        writer.write_to_file(&archive_path, b"password").unwrap();
        
        let mut reader = ArchiveReader::open(&archive_path).unwrap();
        assert!(reader.unlock(b"password").is_ok());
    }
}

