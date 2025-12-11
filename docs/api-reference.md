# SecureArc API Reference

## Core Library (securearc-core)

### ArchiveWriter

Creates new SecureArc archives.

```rust
use securearc_core::archive::{ArchiveConfig, ArchiveWriter};
use securearc_core::format::{CompressionAlgorithm, EncryptionAlgorithm};

let config = ArchiveConfig {
    max_attempts: 5,
    encryption_algorithm: EncryptionAlgorithm::Aes256Gcm,
    compression_algorithm: CompressionAlgorithm::Lzma2,
    ..Default::default()
};

let mut writer = ArchiveWriter::new(config);
writer.add_file("file.txt", PathBuf::from("file.txt"))?;
writer.write_to_file("archive.sarc", b"password")?;
```

### ArchiveReader

Reads and extracts SecureArc archives.

```rust
use securearc_core::archive::ArchiveReader;

let mut reader = ArchiveReader::open("archive.sarc")?;
reader.unlock(b"password")?;

let files = reader.list_files()?;
for file in &files {
    reader.extract_file(file, &output_path)?;
}
```

### ArchiveConfig

Configuration for archive creation.

- `max_attempts`: Maximum password attempts before destruction (3-99)
- `encryption_algorithm`: AES-256-GCM or ChaCha20-Poly1305
- `compression_algorithm`: LZMA2, Zstd, Brotli, or None
- `kdf_params`: Key derivation function parameters

### ArchiveInfo

Information about an archive.

- `max_attempts`: Maximum allowed attempts
- `current_attempts`: Current failed attempts
- `remaining_attempts`: Remaining attempts before destruction
- `destroyed`: Whether archive has been destroyed
- `file_count`: Number of files in archive

## Error Handling

All operations return `Result<T, SecureArcError>` where errors include:

- `InvalidPassword`: Wrong password provided
- `MaxAttemptsExceeded`: Archive destroyed due to too many attempts
- `ArchiveDestroyed`: Archive has been destroyed
- `HeaderCorrupted`: Security header is invalid
- `IntegrityCheckFailed`: HMAC verification failed
- `FormatError`: File format error
- `IoError`: I/O operation failed

## CLI Tool (securearc-cli)

### Create Archive

```bash
securearc-cli create -o archive.sarc file1.txt file2.txt --max-attempts 5
```

### Extract Archive

```bash
securearc-cli extract archive.sarc -o output/
```

### List Files

```bash
securearc-cli list archive.sarc
```

### Archive Info

```bash
securearc-cli info archive.sarc
```

## GUI Application (securearc-gui)

### Tauri Commands

- `create_archive(request)`: Create a new archive
- `extract_archive(request)`: Extract files from archive
- `list_archive(request)`: List files in archive
- `get_archive_info(archive_path)`: Get archive information without password

All commands are async and return `Result<T, String>`.

