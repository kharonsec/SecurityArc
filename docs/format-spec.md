# SecureArc File Format Specification

## Overview

SecureArc is a self-destructing encrypted archive format designed for secure file storage with brute-force protection.

## File Structure

```
+------------------+
| Magic Number     | 8 bytes: "SECARC01"
+------------------+
| Security Header  | Variable size (serialized)
+------------------+
| Key Slots        | Variable size (one or more encrypted master keys)
+------------------+
| Central Directory| Encrypted, variable size
+------------------+
| Payload          | Encrypted compressed file data
+------------------+
```

## Magic Number

The first 8 bytes of every SecureArc file must be the ASCII string "SECARC01", identifying the file format and version.

## Security Header

The security header contains:

- **KDF Algorithm ID**: 1 byte (1 = Argon2id, 2 = PBKDF2-SHA256)
- **KDF Parameters**: Memory (u32), Iterations (u32), Parallelism (u32)
- **Salt**: 32 bytes, cryptographically random
- **Attempt Counter**: u32, current failed password attempts
- **Max Attempts**: u32, threshold for destruction (3-99)
- **Checksum**: 32 bytes, HMAC-SHA256 of header (excluding checksum field)
- **Destroyed Flag**: bool, indicates if archive has been destroyed

The header is serialized using bincode for compact binary representation.

## Key Slots

Key slots contain encrypted copies of the master key. Each slot is encrypted using a key derived from the user password.

- **Primary Slot**: Slot ID 0, encrypted with primary password
- **Recovery Slots**: Slot ID 1+, encrypted with recovery passwords (optional)

Each key slot contains:
- **Encrypted Key**: Variable size (encrypted 32-byte master key + IV + tag)
- **Slot ID**: u8
- **Active Flag**: bool

## Central Directory

The central directory contains metadata about all files in the archive:

- **File Entries**: Array of file metadata
  - Path (PathBuf)
  - Original size (u64)
  - Compressed size (u64)
  - Modified time (u64, Unix timestamp)
  - Attributes (u32)
  - Data offset (u64, offset in payload)

The directory is encrypted using the master key and stored before the payload.

## Payload

The payload contains encrypted and compressed file data. Each file is:
1. Compressed using the selected algorithm (LZMA2, Zstd, or Brotli)
2. Encrypted using the master key with the selected algorithm (AES-256-GCM or ChaCha20-Poly1305)

## Self-Destruct Mechanism

When the attempt counter reaches max_attempts:
1. All key slots are zeroized (overwritten with random data)
2. Security header KDF parameters are corrupted
3. Destruction flag is set
4. Header checksum is invalidated

This makes key derivation and decryption impossible.

