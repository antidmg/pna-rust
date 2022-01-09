use std::fs::File;
use std::fs::{self, OpenOptions};
use std::io::{BufWriter, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use crate::{Command, Result};

/// A key-value store
pub struct KvStore {
    path: PathBuf,
    writer: BufWriterWithPos<File>,
}

/// Wrapper type for a buffered writer that tracks our position in the file
pub struct BufWriterWithPos<W: Write + Seek> {
    writer: BufWriter<W>,
    pos: u64,
}

impl<W: Write + Seek> BufWriterWithPos<W> {
    fn new(mut w: W) -> Result<Self> {
        let pos = w.seek(SeekFrom::Current(0))?;
        Ok(BufWriterWithPos {
            writer: BufWriter::new(w),
            pos,
        })
    }
}

#[allow(unused_variables)]
impl KvStore {
    pub fn remove(&mut self, key: String) -> Result<()> {
        Ok(())
    }

    pub fn get(&self, key: String) -> Result<Option<String>> {
        Ok(Some(String::from("ok")))
    }

    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        // create a command struct
        let cmd = Command::Set { key, value };

        // TODO: flush the leftover buffered content if any
        // serde_json::to_writer(&mut self.writer, &cmd)?;
        // self.writer.flush()?;

        // TODO: update the index as well

        Ok(())
    }

    pub fn open(path: impl Into<std::path::PathBuf>) -> Result<KvStore> {
        let path = path.into();
        fs::create_dir_all(&path)?;
        let writer = get_log_file(&path)?;

        Ok(KvStore { writer, path })
    }
}

fn get_log_file(path: &Path) -> Result<BufWriterWithPos<File>> {
    let writer = BufWriterWithPos::new(
        OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&path)?,
    )?;

    Ok(writer)
}
