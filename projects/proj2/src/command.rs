use std::ops::Range;

use serde::{Deserialize, Serialize};

/// Enum representation of a command
#[derive(Debug, Serialize, Deserialize)]
pub enum Command {
    Set { key: String, value: String },
    Remove { key: String },
}

impl Command {
    pub fn set(key: String, value: String) -> Command {
        Command::Set { key, value }
    }

    pub fn remove(key: String) -> Command {
        Command::Remove { key }
    }
}

/// A struct that keeps track of a command's size in the log file.
/// Also used for indexing log entries
pub struct CommandPos {
    pub filename: u64,
    pub pos: u64,
    pub len: u64,
}

impl From<(u64, Range<u64>)> for CommandPos {
    fn from((filename, range): (u64, Range<u64>)) -> Self {
        CommandPos {
            filename,
            pos: range.start,
            len: range.end - range.start,
        }
    }
}
