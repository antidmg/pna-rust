use serde::{Deserialize, Serialize};

/// Enum representation of a command
#[derive(Debug, Serialize, Deserialize)]
pub enum Command {
    Set { key: String, value: String },
    Remove { key: String },
}

/// A struct that keeps track of a command's size in the log file.
/// Also used for indexing log entries
pub struct CommandPos {
    pub filename: u64,
    pub pos: u64,
    pub len: u64,
}
