# SecureArc

**Self-Destructing Encrypted Archive Format**

SecureArc is a next-generation archive format designed to compete with established formats like RAR, 7z, and ZIP while introducing a critical security feature: automatic content destruction after a configurable number of failed password attempts.

## Features

- **Self-Destruct Mechanism**: Automatically destroys archive contents after N failed password attempts (configurable: 3-99)
- **Modern Cryptography**: AES-256-GCM and ChaCha20-Poly1305 encryption
- **Advanced Compression**: LZMA2, Zstd, and Brotli support
- **Memory-Hard KDF**: Argon2id key derivation for resistance to GPU and side-channel attacks
- **Cross-Platform**: Windows, macOS, and Linux support
- **Multiple Interfaces**: Core library, CLI tool, and GUI applications

## Project Structure

- `securearc-core/` - Core library implementing the SecureArc format
- `securearc-cli/` - Command-line interface for archive management
- `securearc-gui/` - Desktop GUI application (Tauri-based)

## Quick Start

### Building

```bash
cargo build --release
```

### Using the CLI

```bash
# Create an archive
cargo run --bin securearc-cli -- create -o archive.sarc files/ --max-attempts 5

# Extract an archive
cargo run --bin securearc-cli -- extract archive.sarc

# List archive contents
cargo run --bin securearc-cli -- list archive.sarc

# Get archive information
cargo run --bin securearc-cli -- info archive.sarc
```

For detailed CLI usage, see [CLI Reference](docs/CLI_REFERENCE.md).

## Security Considerations

The self-destruct mechanism provides protection against casual attackers and automated brute-force attempts. However, sophisticated attackers may create backup copies before attempting decryption. Strong encryption (AES-256, ChaCha20) ensures that even with unlimited attempts against a file copy, brute-force remains computationally infeasible with a strong password.

## License

MIT OR Apache-2.0

## Documentation

Comprehensive documentation is available in the [`docs/`](docs/) directory. See the [Documentation Index](docs/README.md) for a complete overview.

**Quick Links:**
- **[User Guide](docs/USER_GUIDE.md)** - Complete end-user documentation
- **[Installation Guide](docs/INSTALLATION.md)** - Setup instructions for all platforms
- **[CLI Reference](docs/CLI_REFERENCE.md)** - Command-line interface documentation
- **[API Reference](docs/api-reference.md)** - Library API for developers
- **[Format Specification](docs/format-spec.md)** - Technical file format specification
- **[Security Model](docs/security-model.md)** - Security analysis and threat model
- **[Testing Guide](docs/testing-guide.md)** - Testing instructions and examples

