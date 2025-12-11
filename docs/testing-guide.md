# SecureArc Testing Guide

## Quick Start

### 1. Build the Project

```bash
# Build all workspace members
cargo build

# Build in release mode (faster for testing)
cargo build --release
```

### 2. Run All Tests

```bash
# Run all tests (unit + integration)
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test suite
cargo test --test integration_tests
cargo test --test crypto_tests
cargo test --test format_tests

# Run tests for specific module
cargo test -p securearc-core
```

## Automated Test Suites

### Unit Tests

Unit tests are embedded in each module:

```bash
# Test cryptographic components
cargo test -p securearc-core crypto

# Test compression algorithms
cargo test -p securearc-core compression

# Test format parsing
cargo test -p securearc-core format

# Test self-destruct mechanism
cargo test -p securearc-core self_destruct
```

### Integration Tests

Integration tests verify end-to-end functionality:

```bash
# Run all integration tests
cargo test --test integration_tests

# Test archive creation and extraction
cargo test --test integration_tests test_create_and_extract_archive

# Test self-destruct mechanism
cargo test --test integration_tests test_self_destruct_mechanism

# Test multiple files
cargo test --test integration_tests test_multiple_files

# Test different encryption algorithms
cargo test --test integration_tests test_different_encryption_algorithms

# Test different compression algorithms
cargo test --test integration_tests test_different_compression_algorithms
```

## Manual CLI Testing

### 1. Create a Test Archive

```bash
# Create a simple archive
echo "Hello, SecureArc!" > test.txt
cargo run --bin securearc-cli -- create -o test.sarc test.txt

# Create archive with custom settings
cargo run --bin securearc-cli -- create -o test.sarc test.txt \
    --max-attempts 3 \
    --encryption chacha20 \
    --compression zstd
```

### 2. List Archive Contents

```bash
# List files (requires password)
cargo run --bin securearc-cli -- list test.sarc
```

### 3. Get Archive Information

```bash
# Get archive info (no password needed)
cargo run --bin securearc-cli -- info test.sarc
```

### 4. Extract Archive

```bash
# Extract to current directory
cargo run --bin securearc-cli -- extract test.sarc

# Extract to specific directory
cargo run --bin securearc-cli -- extract test.sarc -o output/
```

## Testing Self-Destruct Mechanism

### Test 1: Basic Self-Destruct

```bash
# Create archive with max_attempts = 3
echo "Secret data" > secret.txt
cargo run --bin securearc-cli -- create -o secret.sarc secret.txt --max-attempts 3

# Try wrong password 3 times (will trigger destruction)
cargo run --bin securearc-cli -- extract secret.sarc
# Enter wrong password 3 times

# Verify archive is destroyed (correct password should fail)
cargo run --bin securearc-cli -- extract secret.sarc
# Enter correct password - should fail with "Archive destroyed"
```

### Test 2: Self-Destruct with Info Command

```bash
# Create archive
cargo run --bin securearc-cli -- create -o test.sarc test.txt --max-attempts 5

# Check remaining attempts
cargo run --bin securearc-cli -- info test.sarc
# Should show: "Remaining attempts: 5"

# Try wrong password once
cargo run --bin securearc-cli -- extract test.sarc
# Enter wrong password

# Check remaining attempts again
cargo run --bin securearc-cli -- info test.sarc
# Should show: "Remaining attempts: 4"
```

## Testing Different Algorithms

### Encryption Algorithms

```bash
# Test AES-256-GCM
cargo run --bin securearc-cli -- create -o test_aes.sarc test.txt --encryption aes256

# Test ChaCha20-Poly1305
cargo run --bin securearc-cli -- create -o test_chacha.sarc test.txt --encryption chacha20

# Verify both work
cargo run --bin securearc-cli -- extract test_aes.sarc
cargo run --bin securearc-cli -- extract test_chacha.sarc
```

### Compression Algorithms

```bash
# Test LZMA2
cargo run --bin securearc-cli -- create -o test_lzma.sarc test.txt --compression lzma2

# Test Zstd
cargo run --bin securearc-cli -- create -o test_zstd.sarc test.txt --compression zstd

# Test Brotli
cargo run --bin securearc-cli -- create -o test_brotli.sarc test.txt --compression brotli

# Test no compression
cargo run --bin securearc-cli -- create -o test_none.sarc test.txt --compression none

# Verify all work
cargo run --bin securearc-cli -- extract test_lzma.sarc
cargo run --bin securearc-cli -- extract test_zstd.sarc
cargo run --bin securearc-cli -- extract test_brotli.sarc
cargo run --bin securearc-cli -- extract test_none.sarc
```

## Testing Multiple Files

```bash
# Create multiple test files
echo "File 1" > file1.txt
echo "File 2" > file2.txt
echo "File 3" > file3.txt

# Create archive with multiple files
cargo run --bin securearc-cli -- create -o multi.sarc file1.txt file2.txt file3.txt

# List files
cargo run --bin securearc-cli -- list multi.sarc

# Extract all files
cargo run --bin securearc-cli -- extract multi.sarc -o extracted/
```

## Testing Error Cases

### Invalid Password

```bash
# Create archive
cargo run --bin securearc-cli -- create -o test.sarc test.txt

# Try wrong password
cargo run --bin securearc-cli -- extract test.sarc
# Enter wrong password - should show "Invalid password"
```

### Destroyed Archive

```bash
# Create archive with max_attempts = 1
cargo run --bin securearc-cli -- create -o test.sarc test.txt --max-attempts 1

# Trigger destruction
cargo run --bin securearc-cli -- extract test.sarc
# Enter wrong password

# Try to extract destroyed archive
cargo run --bin securearc-cli -- extract test.sarc
# Should show "Archive has been destroyed"
```

### Invalid File Format

```bash
# Try to open a non-SecureArc file
echo "not an archive" > fake.sarc
cargo run --bin securearc-cli -- info fake.sarc
# Should show "Invalid magic number" or "Format error"
```

## Performance Testing

### Large File Test

```bash
# Create a large test file (10MB)
dd if=/dev/urandom of=large.bin bs=1M count=10  # Linux/Mac
# On Windows, use: fsutil file createnew large.bin 10485760

# Create archive
cargo run --bin securearc-cli -- create -o large.sarc large.bin

# Extract and verify
cargo run --bin securearc-cli -- extract large.sarc -o extracted/
```

### Compression Ratio Test

```bash
# Create test file with repetitive data (compresses well)
yes "SecureArc test data" | head -n 10000 > repetitive.txt

# Create archive with compression
cargo run --bin securearc-cli -- create -o compressed.sarc repetitive.txt --compression lzma2

# Compare sizes
ls -lh repetitive.txt compressed.sarc  # Linux/Mac
dir repetitive.txt compressed.sarc   # Windows
```

## Library API Testing

### Using the Core Library Directly

Create a test file `test_lib.rs`:

```rust
use securearc_core::archive::{ArchiveConfig, ArchiveReader, ArchiveWriter};
use securearc_core::format::{CompressionAlgorithm, EncryptionAlgorithm};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create archive
    let config = ArchiveConfig {
        max_attempts: 5,
        encryption_algorithm: EncryptionAlgorithm::Aes256Gcm,
        compression_algorithm: CompressionAlgorithm::Lzma2,
        ..Default::default()
    };
    
    let mut writer = ArchiveWriter::new(config);
    writer.add_file("test.txt", std::path::PathBuf::from("test.txt"))?;
    writer.write_to_file("test.sarc", b"password123")?;
    
    // Read archive
    let mut reader = ArchiveReader::open("test.sarc")?;
    reader.unlock(b"password123")?;
    
    let files = reader.list_files()?;
    println!("Files in archive: {:?}", files);
    
    Ok(())
}
```

Run with:
```bash
cargo run --example test_lib
```

## Continuous Testing

### Run Tests in Watch Mode

```bash
# Install cargo-watch if not installed
cargo install cargo-watch

# Watch for changes and run tests
cargo watch -x test
```

### Test Coverage

```bash
# Install cargo-tarpaulin for coverage
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html
```

## Troubleshooting

### Common Issues

1. **"Failed to deserialize header"**
   - Archive file may be corrupted
   - Try creating a new archive

2. **"Invalid password" after correct password**
   - Check if max_attempts was exceeded
   - Use `info` command to check remaining attempts

3. **"Archive destroyed"**
   - Archive was destroyed due to too many failed attempts
   - Cannot be recovered (by design)

4. **Compilation errors**
   - Ensure all dependencies are installed: `cargo build`
   - Check Rust version: `rustc --version` (should be 1.70+)

### Debug Mode

Run with debug output:
```bash
RUST_LOG=debug cargo test
RUST_LOG=debug cargo run --bin securearc-cli -- create -o test.sarc test.txt
```

## Next Steps

After basic testing:
1. Test with real-world files
2. Test cross-platform compatibility
3. Test with very large archives
4. Test edge cases (empty files, very long paths, etc.)
5. Performance benchmarking

