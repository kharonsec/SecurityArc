//! File format tests

use securearc_core::format::directory::{CentralDirectory, FileEntry};
use securearc_core::format::header::SecurityHeader;
use securearc_core::format::keyslot::KeySlot;
use std::io::Cursor;
use std::path::PathBuf;

#[test]
fn test_header_creation() {
    let header = SecurityHeader::new(5).unwrap();
    assert_eq!(header.max_attempts, 5);
    assert_eq!(header.attempt_counter, 0);
    assert!(!header.destroyed);
}

#[test]
fn test_header_serialization() {
    let header = SecurityHeader::new(10).unwrap();
    let mut buffer = Vec::new();
    header.write(&mut buffer).unwrap();

    let mut reader = Cursor::new(buffer);
    let deserialized = SecurityHeader::read(&mut reader).unwrap();
    assert_eq!(header.max_attempts, deserialized.max_attempts);
}

#[test]
fn test_key_slot_zeroization() {
    let mut slot = KeySlot::new(0);
    slot.encrypted_key = vec![1, 2, 3, 4, 5];
    slot.active = true;

    assert!(!slot.is_zeroized());
    slot.zeroize();
    assert!(slot.is_zeroized());
    assert!(!slot.active);
}

#[test]
fn test_directory_operations() {
    let mut directory = CentralDirectory::new();

    let entry = FileEntry {
        path: PathBuf::from("test.txt"),
        original_size: 100,
        compressed_size: 50,
        encrypted_size: 60,
        modified_time: 1234567890,
        attributes: 0,
        data_offset: 0,
    };

    directory.add_entry(entry);
    assert_eq!(directory.entries().len(), 1);

    let found = directory.find_entry(&PathBuf::from("test.txt"));
    assert!(found.is_some());
    assert_eq!(found.unwrap().original_size, 100);
}
