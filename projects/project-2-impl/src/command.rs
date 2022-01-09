use serde::{Deserialize, Serialize};

/// Enum representation of a command
#[derive(Debug, Serialize, Deserialize)]
pub enum Command {
    Set { key: String, value: String },
    Remove { key: String },
}
