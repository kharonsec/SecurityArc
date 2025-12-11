//! Archive reading and writing functionality

pub mod error;
pub mod reader;
pub mod writer;

pub use error::SecureArcError;
pub use reader::{ArchiveReader, ArchiveInfo};
pub use writer::{ArchiveWriter, ArchiveConfig};

