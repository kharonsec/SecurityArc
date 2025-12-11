//! SecureArc CLI Tool

use clap::{Parser, Subcommand};
use securearc_core::archive::{ArchiveConfig, ArchiveReader, ArchiveWriter};
use securearc_core::format::{CompressionAlgorithm, EncryptionAlgorithm};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "securearc-cli")]
#[command(about = "SecureArc - Self-Destructing Encrypted Archive Tool")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new SecureArc archive
    Create {
        /// Output archive file
        #[arg(short, long)]
        output: PathBuf,
        /// Input files or directories
        files: Vec<PathBuf>,
        /// Maximum password attempts before destruction
        #[arg(short = 'm', long, default_value = "5")]
        max_attempts: u32,
        /// Encryption algorithm (aes256 or chacha20)
        #[arg(short = 'e', long, default_value = "aes256")]
        encryption: String,
        /// Compression algorithm (lzma2, zstd, brotli, none)
        #[arg(short = 'c', long, default_value = "lzma2")]
        compression: String,
        /// Password (if not provided, will prompt interactively)
        #[arg(short = 'p', long)]
        password: Option<String>,
    },
    /// Extract files from a SecureArc archive
    Extract {
        /// Archive file to extract
        archive: PathBuf,
        /// Output directory
        #[arg(short, long, default_value = ".")]
        output: PathBuf,
        /// Password (if not provided, will prompt interactively)
        #[arg(short = 'p', long)]
        password: Option<String>,
    },
    /// List files in a SecureArc archive
    List {
        /// Archive file
        archive: PathBuf,
        /// Password (if not provided, will prompt interactively)
        #[arg(short = 'p', long)]
        password: Option<String>,
    },
    /// Display archive information
    Info {
        /// Archive file
        archive: PathBuf,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Create {
            output,
            files,
            max_attempts,
            encryption,
            compression,
            password,
        } => {
            create_archive(output, files, max_attempts, encryption, compression, password)?;
        }
        Commands::Extract { archive, output, password } => {
            extract_archive(archive, output, password)?;
        }
        Commands::List { archive, password } => {
            list_archive(archive, password)?;
        }
        Commands::Info { archive } => {
            info_archive(archive)?;
        }
    }

    Ok(())
}

fn create_archive(
    output: PathBuf,
    files: Vec<PathBuf>,
    max_attempts: u32,
    encryption: String,
    compression: String,
    password: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    if files.is_empty() {
        return Err("No files specified".into());
    }

    // Parse encryption algorithm
    let encryption_algorithm = match encryption.to_lowercase().as_str() {
        "aes256" | "aes" => EncryptionAlgorithm::Aes256Gcm,
        "chacha20" | "chacha" => EncryptionAlgorithm::ChaCha20Poly1305,
        _ => return Err(format!("Unknown encryption algorithm: {}", encryption).into()),
    };

    // Parse compression algorithm
    let compression_algorithm = match compression.to_lowercase().as_str() {
        "lzma2" | "lzma" => CompressionAlgorithm::Lzma2,
        "zstd" => CompressionAlgorithm::Zstd,
        "brotli" => CompressionAlgorithm::Brotli,
        "none" => CompressionAlgorithm::None,
        _ => return Err(format!("Unknown compression algorithm: {}", compression).into()),
    };

    let config = ArchiveConfig {
        max_attempts,
        encryption_algorithm,
        compression_algorithm,
        ..Default::default()
    };

    let mut writer = ArchiveWriter::new(config);

    // Add files
    let pb = indicatif::ProgressBar::new(files.len() as u64);
    pb.set_style(indicatif::ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
        .unwrap()
        .progress_chars("#>-"));

    for file in &files {
        if !file.exists() {
            eprintln!("Warning: File not found: {:?}", file);
            continue;
        }

        let archive_path = file.file_name()
            .map(|n| PathBuf::from(n))
            .unwrap_or_else(|| file.clone());
        
        writer.add_file(file, archive_path)?;
        pb.inc(1);
    }
    pb.finish_with_message("Files processed");

    // Get password
    let password = if let Some(pwd) = password {
        pwd
    } else {
        let pwd = rpassword::prompt_password("Enter password: ")?;
        let password_confirm = rpassword::prompt_password("Confirm password: ")?;
        
        if pwd != password_confirm {
            return Err("Passwords do not match".into());
        }
        pwd
    };

    println!("Writing archive...");
    let spinner = indicatif::ProgressBar::new_spinner();
    spinner.set_style(indicatif::ProgressStyle::default_spinner().template("{spinner:.blue} {msg}").unwrap()); 
    spinner.enable_steady_tick(std::time::Duration::from_millis(100));
    spinner.set_message("Encrypting and saving...");
    
    writer.write_to_file(&output, password.as_bytes())?;
    spinner.finish_and_clear();
    
    println!("Archive created successfully: {:?}", output);
    Ok(())
}

fn extract_archive(
    archive: PathBuf,
    output: PathBuf,
    password: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut reader = ArchiveReader::open(&archive)?;
    
    // Show archive info
    let info = reader.get_info();
    println!("Remaining attempts: {}", info.remaining_attempts);
    if info.remaining_attempts <= 2 {
        eprintln!("Warning: Low remaining attempts!");
    }

    // Get password
    let password = password.unwrap_or_else(|| {
        rpassword::prompt_password("Enter password: ").unwrap_or_default()
    });
    
    let spinner = indicatif::ProgressBar::new_spinner();
    spinner.set_style(indicatif::ProgressStyle::default_spinner().template("{spinner:.blue} {msg}").unwrap());
    spinner.enable_steady_tick(std::time::Duration::from_millis(100));
    spinner.set_message("Verifying and unlocking...");
    
    reader.unlock(password.as_bytes())?;
    spinner.finish_with_message("Unlocked");
    
    // Create output directory
    std::fs::create_dir_all(&output)?;
    
    // List and extract files
    let files = reader.list_files()?;
    
    let pb = indicatif::ProgressBar::new(files.len() as u64);
    pb.set_style(indicatif::ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}")
        .unwrap()
        .progress_chars("#>-"));
        
    for file in &files {
        pb.set_message(format!("{:?}", file.file_name().unwrap_or_default()));
        let output_path = output.join(file);
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        reader.extract_file(file, &output_path)?;
        pb.inc(1);
    }
    pb.finish_with_message("Extraction complete!");
    
    Ok(())
}

fn list_archive(archive: PathBuf, password: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let mut reader = ArchiveReader::open(&archive)?;
    
    let password = password.unwrap_or_else(|| {
        rpassword::prompt_password("Enter password: ").unwrap_or_default()
    });
    reader.unlock(password.as_bytes())?;
    
    let files = reader.list_files()?;
    println!("Files in archive:");
    for file in &files {
        println!("  {:?}", file);
    }
    
    Ok(())
}

fn info_archive(archive: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let reader = ArchiveReader::open(&archive)?;
    let info = reader.get_info();
    
    println!("Archive Information:");
    println!("  Max attempts: {}", info.max_attempts);
    println!("  Current attempts: {}", info.current_attempts);
    println!("  Remaining attempts: {}", info.remaining_attempts);
    println!("  Destroyed: {}", info.destroyed);
    println!("  File count: {}", info.file_count);
    
    if info.remaining_attempts <= 2 {
        eprintln!("Warning: Low remaining attempts!");
    }
    
    Ok(())
}

