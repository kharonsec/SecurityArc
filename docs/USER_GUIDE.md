# SecureArc User Guide

SecureArc is a secure, self-destructing encrypted archive format designed for sensitive data storage with brute-force protection.

## Table of Contents

- [Installation](#installation)
- [Quick Start](#quick-start)
- [Using the GUI](#using-the-gui)
- [Using the CLI](#using-the-cli)
- [Self-Destruct Mechanism](#self-destruct-mechanism)
- [Best Practices](#best-practices)
- [Troubleshooting](#troubleshooting)

## Installation

### Windows

**Option 1: Installer (Recommended)**
1. Download `SecureArc_Installer.exe` from the [Releases](https://github.com/securearc/securearc/releases) page.
2. Run the installer and follow the on-screen instructions.
3. SecureArc will be added to your Start Menu and Desktop.

**Option 2: Build from Source**
See the [Installation Guide](INSTALLATION.md) for detailed instructions.

### Linux / macOS
(Coming soon - currently available via source build)

## Quick Start

### Create Your First Archive

**Using GUI:**
1. Open **SecureArc** from the Start Menu (or run `securearc-gui`).
2. Go to the **Create Archive** tab.
3. Click **+ Add Files** to select files you want to secure.
4. Choose an **Output Archive** location (e.g., `C:\MyDocs\secure.sarc`).
5. Enter a strong **Password** (you'll be asked to confirm it).
6. Optionally adjust **Max Attempts** (default: 5).
7. Click **Create Archive**.

**Using CLI:**
```bash
securearc-cli create -o archive.sarc file1.txt file2.txt
```

You'll be prompted to enter and confirm a password.

### Extract an Archive

**Using GUI:**
1. Open SecureArc and go to the **Extract Archive** tab.
2. Select your `.sarc` archive file.
3. Choose an **Output Directory**.
4. Enter the password when prompted.
5. Click **Extract Files**.

**Using CLI:**
```bash
securearc-cli extract archive.sarc -o output/
```

### View Archive Information

**Using GUI:**
1. Go to the **View / Info** tab.
2. Select your archive file.
3. View information including remaining attempts (no password required).

**Using CLI:**
```bash
securearc-cli info archive.sarc
```

## Using the GUI

The SecureArc GUI provides a user-friendly interface for all operations:

### Create Archive Tab
- **Add Files**: Select one or more files to include in the archive
- **Output Archive**: Choose where to save the `.sarc` file
- **Password**: Enter a strong password (shown as dots for security)
- **Max Attempts**: Set how many failed password attempts are allowed (3-99)
- **Encryption**: Choose between AES-256-GCM (default) or ChaCha20-Poly1305
- **Compression**: Choose compression algorithm (LZMA2, Zstd, Brotli, or None)

### Extract Archive Tab
- **Select Archive**: Choose the `.sarc` file to extract
- **Output Directory**: Where extracted files will be saved
- **Extract Files**: Button to start extraction (requires password)

### View / Info Tab
- **Select Archive**: Choose the `.sarc` file to inspect
- **Archive Information**: Displays:
  - Maximum allowed attempts
  - Current failed attempts
  - Remaining attempts
  - Whether archive is destroyed
  - Number of files in archive

## Using the CLI

The Command Line Interface provides advanced control and automation capabilities.

### Basic Commands

```bash
# Create archive
securearc-cli create -o archive.sarc file1.txt file2.txt

# Extract archive
securearc-cli extract archive.sarc -o output/

# List files in archive
securearc-cli list archive.sarc

# Get archive information
securearc-cli info archive.sarc
```

### Advanced Options

```bash
# Create with custom settings
securearc-cli create -o archive.sarc files/ \
    --max-attempts 10 \
    --encryption chacha20 \
    --compression zstd

# Extract to specific directory
securearc-cli extract archive.sarc -o /path/to/output
```

For complete CLI documentation, see [CLI Reference](CLI_REFERENCE.md).

## Self-Destruct Mechanism

SecureArc's unique feature is the self-destruct mechanism that protects against brute-force attacks.

### How It Works

1. **Attempt Tracking**: Each failed password attempt increments a counter stored in the archive header.
2. **Automatic Destruction**: When the counter reaches the maximum allowed attempts, the archive is automatically destroyed.
3. **Permanent Loss**: Once destroyed, the archive cannot be recovered, even with the correct password.

### Key Features

- **Configurable Threshold**: Set max attempts between 3-99 (default: 5)
- **Persistent Counter**: The attempt counter is stored in the archive itself
- **Cryptographic Protection**: The counter is protected by HMAC to prevent tampering
- **No Recovery**: Destruction is permanent and irreversible

### Example Scenario

```
Archive created with max_attempts = 5
Attempt 1: Wrong password → Remaining: 4
Attempt 2: Wrong password → Remaining: 3
Attempt 3: Wrong password → Remaining: 2
Attempt 4: Wrong password → Remaining: 1
Attempt 5: Wrong password → ARCHIVE DESTROYED
```

Even if you enter the correct password after destruction, the archive cannot be recovered.

## Best Practices

### Password Security
- Use strong, unique passwords (at least 16 characters recommended)
- Consider using a password manager
- Never share passwords via insecure channels
- Remember: If you forget the password and exceed max attempts, data is lost forever

### Max Attempts Configuration
- **High Security**: Use 3-5 attempts for highly sensitive data
- **Balanced**: Use 5-10 attempts for general use
- **Convenience**: Use 10+ attempts if you're concerned about accidental lockouts

### Archive Management
- **Backup Important Archives**: Make copies before distribution
- **Test Extraction**: Verify you can extract before deleting originals
- **Check Remaining Attempts**: Use `info` command regularly to monitor attempt count
- **Store Passwords Securely**: Use a password manager or secure notes

### File Selection
- Archive related files together for easier management
- Consider file sizes (very large files may take time to process)
- Be aware that compression works best on text and similar files

## Troubleshooting

### "Archive Destroyed" Error
**Cause**: Maximum number of password attempts exceeded.

**Solution**: The archive cannot be recovered. If you have a backup, restore it. If not, the data is permanently lost.

**Prevention**: 
- Use `info` command to check remaining attempts
- Be careful when entering passwords
- Consider using a password manager

### "Invalid Password" Error
**Cause**: Incorrect password entered.

**Solution**: 
- Double-check your password (case-sensitive)
- Check remaining attempts with `info` command
- If attempts are low, be very careful

### "File Not Found" Error
**Cause**: Specified file or archive doesn't exist.

**Solution**: 
- Verify file paths are correct
- Use absolute paths if relative paths don't work
- Check file permissions

### Performance Issues
**Cause**: Large files or many files take time to process.

**Solution**: 
- Be patient - encryption and compression are CPU-intensive
- Consider using faster compression (zstd) for large files
- Process files in smaller batches if needed

### GUI Not Responding
**Cause**: Processing large files or many files.

**Solution**: 
- Wait for the operation to complete
- Check task manager to verify the process is running
- Close and reopen the application if it appears frozen

## Additional Resources

- [Installation Guide](INSTALLATION.md) - Detailed setup instructions
- [CLI Reference](CLI_REFERENCE.md) - Complete command-line documentation
- [Security Model](security-model.md) - Security analysis and threat model
- [Format Specification](format-spec.md) - Technical format details
