//! Compression algorithm implementations

use crate::format::CompressionAlgorithm as FormatCompression;
use crate::SecureArcError;
use brotli::enc::BrotliEncoderParams;
use std::io::{Read, Write};

/// Compression error type
#[derive(Debug, thiserror::Error)]
pub enum CompressionError {
    #[error("Compression failed: {0}")]
    CompressionFailed(String),
    #[error("Decompression failed: {0}")]
    DecompressionFailed(String),
}

/// Compress data using the specified algorithm
pub fn compress_data(
    data: &[u8],
    algorithm: FormatCompression,
) -> Result<Vec<u8>, SecureArcError> {
    match algorithm {
        FormatCompression::None => Ok(data.to_vec()),
        FormatCompression::Lzma2 => {
            use lzma_rs::lzma_compress;
            let mut output = Vec::new();
            lzma_compress(&mut std::io::Cursor::new(data), &mut output)
                .map_err(|e| SecureArcError::CompressionError(format!("LZMA2 compression failed: {}", e)))?;
            Ok(output)
        }
        FormatCompression::Zstd => {
            zstd::encode_all(data, 3) // Level 3 compression
                .map_err(|e| SecureArcError::CompressionError(format!("Zstd compression failed: {}", e)))
        }
        FormatCompression::Brotli => {
            let params = BrotliEncoderParams {
                quality: 6, // Default quality
                ..Default::default()
            };
            let mut output = Vec::new();
            {
                let mut writer = brotli::CompressorWriter::with_params(
                    &mut output,
                    4096, // Buffer size
                    &params,
                );
                writer
                    .write_all(data)
                    .map_err(|e| SecureArcError::CompressionError(format!("Brotli compression failed: {}", e)))?;
            }
            Ok(output)
        }
    }
}

/// Decompress data using the specified algorithm
pub fn decompress_data(
    compressed_data: &[u8],
    algorithm: FormatCompression,
) -> Result<Vec<u8>, SecureArcError> {
    match algorithm {
        FormatCompression::None => Ok(compressed_data.to_vec()),
        FormatCompression::Lzma2 => {
            use lzma_rs::lzma_decompress;
            let mut output = Vec::new();
            lzma_decompress(&mut std::io::Cursor::new(compressed_data), &mut output)
                .map_err(|e| SecureArcError::CompressionError(format!("LZMA2 decompression failed: {}", e)))?;
            Ok(output)
        }
        FormatCompression::Zstd => {
            zstd::decode_all(compressed_data)
                .map_err(|e| SecureArcError::CompressionError(format!("Zstd decompression failed: {}", e)))
        }
        FormatCompression::Brotli => {
            let mut output = Vec::new();
            brotli::Decompressor::new(std::io::Cursor::new(compressed_data), 4096)
                .read_to_end(&mut output)
                .map_err(|e| SecureArcError::CompressionError(format!("Brotli decompression failed: {}", e)))?;
            Ok(output)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lzma2_compression() {
        let data = b"Hello, SecureArc! This is a test string for compression.";
        let compressed = compress_data(data, FormatCompression::Lzma2).unwrap();
        let decompressed = decompress_data(&compressed, FormatCompression::Lzma2).unwrap();
        assert_eq!(data, decompressed.as_slice());
    }

    #[test]
    fn test_zstd_compression() {
        let data = b"Hello, SecureArc! This is a test string for compression.";
        let compressed = compress_data(data, FormatCompression::Zstd).unwrap();
        let decompressed = decompress_data(&compressed, FormatCompression::Zstd).unwrap();
        assert_eq!(data, decompressed.as_slice());
    }

    #[test]
    fn test_brotli_compression() {
        let data = b"Hello, SecureArc! This is a test string for compression.";
        let compressed = compress_data(data, FormatCompression::Brotli).unwrap();
        let decompressed = decompress_data(&compressed, FormatCompression::Brotli).unwrap();
        assert_eq!(data, decompressed.as_slice());
    }
}

