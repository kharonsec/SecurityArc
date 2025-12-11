# SecureArc CLI Reference

The Command Line Interface (CLI) provides advanced control over SecureArc operations and allows for automation.

## Usage
`securearc-cli.exe [COMMAND] [OPTIONS]`

## Commands

### `create`
Creates a new secure archive.

**Usage:**
`securearc-cli create <OUTPUT_ARCHIVE> <INPUT_FILES>...`

**Options:**
- `-p, --password <PASSWORD>`: Set password directly (not recommended for history). If omitted, prompts securely.
- `--max-attempts <N>`: Set max password attempts (Default: 5).
- `--encryption <ALG>`: Choose algorithm (`aes256` [default], `chacha20`).
- `--compression <ALG>`: Choose compression (`lzma2` [default], `zstd`, `brotli`, `none`).

**Example:**
```powershell
securearc-cli create secret.sarc ./data/report.pdf ./data/image.png
```

### `extract`
Extracts files from an archive.

**Usage:**
`securearc-cli extract <ARCHIVE> [OUTPUT_DIR]`

**Options:**
- `-p, --password <PASSWORD>`: Provide password. If omitted, prompts securely.

**Example:**
```powershell
securearc-cli extract secret.sarc ./extracted_data
```

### `info`
Displays public metadata (verification status, attempt count) without requiring the password (unless listing protected metadata).

**Usage:**
`securearc-cli info <ARCHIVE>`

### `list`
Lists the contents of an encrypted archive. Requires password.

**Usage:**
`securearc-cli list <ARCHIVE>`

## Exit Codes
- `0`: Success
- `1`: General Error (IO, Invalid Arguments)
- `2`: Authentication Failed (Wrong Password)
- `3`: Archive Destroyed (Self-destruct triggered or previously destroyed)
