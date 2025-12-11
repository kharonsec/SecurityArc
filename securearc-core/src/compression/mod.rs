//! Compression algorithm implementations

pub mod algorithms;

pub use algorithms::{compress_data, decompress_data, CompressionError};
