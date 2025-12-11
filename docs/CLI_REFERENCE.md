# SecureArc CLI Reference

The Command Line Interface (CLI) provides advanced control over SecureArc operations and allows for automation. For general user documentation, see [User Guide](USER_GUIDE.md).

## Usage
```bash
securearc-cli [COMMAND] [OPTIONS]
```

## Commands

### `create`
Creates a new secure archive.

**Usage:**
```bash
securearc-cli create -o <OUTPUT_ARCHIVE> <INPUT_FILES>... [OPTIONS]
```

**Options:**
- `-o, --output <OUTPUT_ARCHIVE>`: Output archive file path (required)
- `-p, --password <PASSWORD>`: Set password directly (not recommended for history). If omitted, prompts securely.
- `-m, --max-attempts <N>`: Set max password attempts (Default: 5, range: 3-99).
- `-e, --encryption <ALG>`: Choose encryption algorithm (`aes256` [default], `chacha20`).
- `-c, --compression <ALG>`: Choose compression algorithm (`lzma2` [default], `zstd`, `brotli`, `none`).

**Examples:**
```bash
# Create archive with default settings
securearc-cli create -o secret.sarc ./data/report.pdf ./data/image.png

# Create archive with custom settings
securearc-cli create -o secret.sarc file1.txt file2.txt \
    --max-attempts 10 \
    --encryption chacha20 \
    --compression zstd

# Create archive with password from command line (not recommended)
securearc-cli create -o secret.sarc file.txt -p "mypassword"
```

### `extract`
Extracts files from an archive.

**Usage:**
```bash
securearc-cli extract <ARCHIVE> [OPTIONS]
```

**Options:**
- `-o, --output <OUTPUT_DIR>`: Output directory for extracted files (Default: current directory)
- `-p, --password <PASSWORD>`: Provide password. If omitted, prompts securely.

**Examples:**
```bash
# Extract to current directory
securearc-cli extract secret.sarc

# Extract to specific directory
securearc-cli extract secret.sarc -o ./extracted_data

# Extract with password from command line
securearc-cli extract secret.sarc -p "mypassword" -o ./output
```

### `list`
Lists the contents of an encrypted archive. Requires password.

**Usage:**
```bash
securearc-cli list <ARCHIVE> [OPTIONS]
```

**Options:**
- `-p, --password <PASSWORD>`: Provide password. If omitted, prompts securely.

**Examples:**
```bash
# List files (will prompt for password)
securearc-cli list secret.sarc

# List files with password from command line
securearc-cli list secret.sarc -p "mypassword"
```

### `info`
Displays public metadata (verification status, attempt count) without requiring the password.

**Usage:**
```bash
securearc-cli info <ARCHIVE>
```

**Example:**
```bash
securearc-cli info secret.sarc
```

**Output includes:**
- Maximum allowed attempts
- Current failed attempts
- Remaining attempts before destruction
- Whether archive has been destroyed
- Number of files in archive

## Exit Codes
- `0`: Success
- `1`: General Error (IO, Invalid Arguments)
- `2`: Authentication Failed (Wrong Password)
- `3`: Archive Destroyed (Self-destruct triggered or previously destroyed)

## Notes

- All commands support both Windows PowerShell and Unix shells (bash, zsh, etc.)
- When password is not provided via `-p`, the CLI will securely prompt for it
- The `info` command does not require a password and can be used to check archive status safely
- Archive file extension is typically `.sarc` but any extension can be used
