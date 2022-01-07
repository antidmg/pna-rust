#[derive(Debug)]

/// Error type for KvStore
pub enum KvsError {}

/// Custom `Result` type for KvStore
pub type Result<T> = std::result::Result<T, KvsError>;
