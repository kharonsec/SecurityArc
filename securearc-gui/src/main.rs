//! SecureArc GUI Application

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use securearc_core::archive::{ArchiveConfig, ArchiveReader, ArchiveWriter};
use securearc_core::format::{CompressionAlgorithm, EncryptionAlgorithm};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateArchiveRequest {
    output_path: String,
    files: Vec<String>,
    password: String,
    max_attempts: u32,
    encryption: String,
    compression: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExtractArchiveRequest {
    archive_path: String,
    output_path: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ListArchiveRequest {
    archive_path: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ArchiveInfoResponse {
    max_attempts: u32,
    current_attempts: u32,
    remaining_attempts: u32,
    destroyed: bool,
    file_count: usize,
    files: Vec<String>,
}

#[derive(Clone, Serialize)]
struct ProgressPayload {
    current: usize,
    total: usize,
    filename: String,
    status: String,
}

#[tauri::command]
fn create_archive(window: tauri::Window, request: CreateArchiveRequest) -> Result<(), String> {
    let encryption_algorithm = match request.encryption.to_lowercase().as_str() {
        "aes256" | "aes" => EncryptionAlgorithm::Aes256Gcm,
        "chacha20" | "chacha" => EncryptionAlgorithm::ChaCha20Poly1305,
        _ => {
            return Err(format!(
                "Unknown encryption algorithm: {}",
                request.encryption
            ))
        }
    };

    let compression_algorithm = match request.compression.to_lowercase().as_str() {
        "lzma2" | "lzma" => CompressionAlgorithm::Lzma2,
        "zstd" => CompressionAlgorithm::Zstd,
        "brotli" => CompressionAlgorithm::Brotli,
        "none" => CompressionAlgorithm::None,
        _ => {
            return Err(format!(
                "Unknown compression algorithm: {}",
                request.compression
            ))
        }
    };

    let config = ArchiveConfig {
        max_attempts: request.max_attempts,
        encryption_algorithm,
        compression_algorithm,
        ..Default::default()
    };

    let mut writer = ArchiveWriter::new(config);
    let total_files = request.files.len(); // Approximate, update dynamically if possible or just use files processed count

    for (i, file_path) in request.files.iter().enumerate() {
        let path = PathBuf::from(file_path);
        if !path.exists() {
            return Err(format!("File not found: {}", file_path));
        }

        if path.is_dir() {
            for entry in walkdir::WalkDir::new(&path) {
                let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
                if entry.file_type().is_file() {
                    let file_path = entry.path();

                    // Create archive path relative to the selected folder's parent
                    // e.g. selecting /foo/bar and archiving /foo/bar/baz.txt -> bar/baz.txt
                    let parent = path.parent().unwrap_or(&path);
                    let archive_path = file_path
                        .strip_prefix(parent)
                        .unwrap_or_else(|_| {
                            file_path.file_name().map(Path::new).unwrap_or(file_path)
                        })
                        .to_path_buf();

                    // Emit progress
                    let _ = window.emit(
                        "create-progress",
                        ProgressPayload {
                            current: i + 1, // Only tracking top level items for now in 'total', imperfect but functional
                            total: total_files,
                            filename: file_path
                                .file_name()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .to_string(),
                            status: "processing".to_string(),
                        },
                    );

                    writer
                        .add_file(file_path, archive_path)
                        .map_err(|e| format!("Failed to add file: {}", e))?;
                }
            }
        } else {
            let archive_path = path
                .file_name()
                .map(PathBuf::from)
                .unwrap_or_else(|| path.clone());

            // Emit start progress for this file
            let _ = window.emit(
                "create-progress",
                ProgressPayload {
                    current: i + 1,
                    total: total_files,
                    filename: path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string(),
                    status: "processing".to_string(),
                },
            );

            writer
                .add_file(&path, archive_path)
                .map_err(|e| format!("Failed to add file: {}", e))?;
        }
    }

    writer
        .write_to_file(
            PathBuf::from(&request.output_path),
            request.password.as_bytes(),
        )
        .map_err(|e| format!("Failed to create archive: {}", e))?;

    Ok(())
}

#[tauri::command]
fn extract_archive(window: tauri::Window, request: ExtractArchiveRequest) -> Result<(), String> {
    let mut reader = ArchiveReader::open(PathBuf::from(&request.archive_path))
        .map_err(|e| format!("Failed to open archive: {}", e))?;

    reader
        .unlock(request.password.as_bytes())
        .map_err(|e| format!("Failed to unlock archive: {}", e))?;

    let files = reader
        .list_files()
        .map_err(|e| format!("Failed to list files: {}", e))?;

    std::fs::create_dir_all(&request.output_path)
        .map_err(|e| format!("Failed to create output directory: {}", e))?;

    let total_files = files.len();

    for (i, file) in files.iter().enumerate() {
        let output_path = PathBuf::from(&request.output_path).join(file);
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        }

        // Emit progress
        let _ = window.emit(
            "extract-progress",
            ProgressPayload {
                current: i + 1,
                total: total_files,
                filename: file.to_string_lossy().to_string(),
                status: "extracting".to_string(),
            },
        );

        reader
            .extract_file(file, &output_path)
            .map_err(|e| format!("Failed to extract file: {}", e))?;
    }

    Ok(())
}

#[tauri::command]
fn list_archive(request: ListArchiveRequest) -> Result<ArchiveInfoResponse, String> {
    let mut reader = ArchiveReader::open(PathBuf::from(&request.archive_path))
        .map_err(|e| format!("Failed to open archive: {}", e))?;

    reader
        .unlock(request.password.as_bytes())
        .map_err(|e| format!("Failed to unlock archive: {}", e))?;

    let info = reader.get_info();
    let files = reader
        .list_files()
        .map_err(|e| format!("Failed to list files: {}", e))?;

    Ok(ArchiveInfoResponse {
        max_attempts: info.max_attempts,
        current_attempts: info.current_attempts,
        remaining_attempts: info.remaining_attempts,
        destroyed: info.destroyed,
        file_count: info.file_count,
        files: files.iter().map(|p| p.display().to_string()).collect(),
    })
}

#[tauri::command]
fn get_archive_info(archive_path: String) -> Result<ArchiveInfoResponse, String> {
    let reader = ArchiveReader::open(PathBuf::from(&archive_path))
        .map_err(|e| format!("Failed to open archive: {}", e))?;

    let info = reader.get_info();

    Ok(ArchiveInfoResponse {
        max_attempts: info.max_attempts,
        current_attempts: info.current_attempts,
        remaining_attempts: info.remaining_attempts,
        destroyed: info.destroyed,
        file_count: info.file_count,
        files: Vec::new(), // Files require password to list
    })
}

use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            // Check for command line arguments (file open)
            let args: Vec<String> = std::env::args().collect();
            if args.len() > 1 {
                let file_path = args[1].clone();
                // Emit event to frontend after a slight delay to ensure window is ready
                // or just emit. Frontend listeners set up in useEffect should catch it if mounted.
                // Better: Emit once window is ready.
                let app_handle = app.handle();
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_millis(1000)); // Wait for frontend
                    let _ = app_handle.emit_all("open-file", file_path);
                });
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            create_archive,
            extract_archive,
            list_archive,
            get_archive_info
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
