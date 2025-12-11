# SecureArc Installation Guide

This guide provides installation instructions for SecureArc on all supported platforms. For user documentation, see [User Guide](USER_GUIDE.md).

## Prerequisites: Installing Rust

SecureArc is written in Rust, so you need to install the Rust toolchain first.

### Windows Installation

#### Option 1: Using rustup (Recommended)

1. **Download rustup-init.exe**
   - Visit: https://rustup.rs/
   - Or download directly: https://win.rustup.rs/x86_64
   - Save the file (e.g., to your Downloads folder)

2. **Run the installer**
   ```powershell
   # Navigate to Downloads (or wherever you saved it)
   cd $env:USERPROFILE\Downloads
   
   # Run the installer
   .\rustup-init.exe
   ```

3. **Follow the installer prompts**
   - Press Enter to proceed with default installation
   - The installer will:
     - Install Rust to `%USERPROFILE%\.cargo`
     - Add Rust to your PATH
     - Install `rustc`, `cargo`, and `rustup`

4. **Restart your terminal/PowerShell**
   - Close and reopen PowerShell/VS Code terminal
   - This ensures PATH changes take effect

5. **Verify installation**
   ```powershell
   rustc --version
   cargo --version
   ```

#### Option 2: Using Chocolatey (if you have it)

```powershell
choco install rust
```

#### Option 3: Using Scoop (if you have it)

```powershell
scoop install rust
```

### Troubleshooting PATH Issues

If `cargo` is still not recognized after installation:

1. **Check if Rust is installed but not in PATH**
   ```powershell
   # Check if cargo exists in user directory
   Test-Path "$env:USERPROFILE\.cargo\bin\cargo.exe"
   ```

2. **Add to PATH manually** (if needed)
   ```powershell
   # Add to user PATH permanently
   [Environment]::SetEnvironmentVariable(
       "Path",
       [Environment]::GetEnvironmentVariable("Path", "User") + ";$env:USERPROFILE\.cargo\bin",
       "User"
   )
   
   # Reload PATH in current session
   $env:Path += ";$env:USERPROFILE\.cargo\bin"
   ```

3. **Verify PATH was added**
   ```powershell
   $env:Path -split ';' | Select-String -Pattern 'cargo'
   ```

### After Rust Installation

Once Rust is installed, you can build and test SecureArc:

```powershell
# Navigate to project directory (replace with your actual path)
cd path\to\SecurityArc

# Build the project
cargo build

# Run tests
cargo test

# Build in release mode
cargo build --release
```

## Alternative: Using WSL (Windows Subsystem for Linux)

If you prefer a Linux-like environment:

1. **Install WSL** (if not already installed)
   ```powershell
   wsl --install
   ```

2. **Install Rust in WSL**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source $HOME/.cargo/env
   ```

3. **Use WSL terminal for development**
   - Open WSL terminal
   - Navigate to your project directory (e.g., `/mnt/c/path/to/SecurityArc`)
   - Run cargo commands normally

## System Requirements

- **Windows**: Windows 7 or later (64-bit recommended)
- **RAM**: 2GB minimum (4GB+ recommended for compilation)
- **Disk Space**: ~1GB for Rust toolchain + dependencies
- **Internet**: Required for downloading dependencies

## Next Steps

After installing Rust:

1. Verify installation: `cargo --version`
2. Build the project: `cargo build`
3. Run tests: `cargo test`
4. See [Testing Guide](testing-guide.md) for detailed testing instructions

