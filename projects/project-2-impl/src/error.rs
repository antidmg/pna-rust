use std::io;

/// Error type for KvStore
#[derive(Debug)]
pub enum KvsError {
    // I/O error
    Io(io::Error),
    /// Serialization error
    Serde(serde_json::Error),
}

impl From<io::Error> for KvsError {
    fn from(err: io::Error) -> KvsError {
        KvsError::Io(err)
    }
}

impl From<serde_json::Error> for KvsError {
    fn from(err: serde_json::Error) -> KvsError {
        KvsError::Serde(err)
    }
}

/// Custom `Result` type for KvStore
pub type Result<T> = std::result::Result<T, KvsError>;
