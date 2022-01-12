use std::collections::BTreeMap;
use std::fs::File;
use std::fs::{self, OpenOptions};
use std::io::{self, BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::{collections::HashMap, ffi::OsStr};

use serde_json::Deserializer;

use crate::command::CommandPos;
use crate::{Command, KvsError, Result};

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

impl<W: Write + Seek> Write for BufWriterWithPos<W> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let len = self.writer.write(bytes)?;
        self.pos += len as u64;
        Ok(len)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

pub struct BufReaderWithPos<File> {
    reader: BufReader<File>,
    pos: u64,
}

impl<R: Read + Seek> BufReaderWithPos<R> {
    pub fn new(mut r: R) -> Result<Self> {
        let pos = r.seek(SeekFrom::Current(0))?;
        Ok(BufReaderWithPos {
            reader: BufReader::new(r),
            pos,
        })
    }
}

impl<R: Read + Seek> Read for BufReaderWithPos<R> {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        let len = self.reader.read(bytes)?;
        self.pos += len as u64;

        Ok(len)
    }
}

impl<R: Read + Seek> Seek for BufReaderWithPos<R> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.pos = self.reader.seek(pos)?;
        Ok(self.pos)
    }
}

/// A key-value store
pub struct KvStore {
    path: PathBuf,
    writer: BufWriterWithPos<File>,
    readers: HashMap<u64, BufReaderWithPos<File>>,
    index: BTreeMap<String, CommandPos>,
    current_filename: u64,
    uncompacted: u64,
}

#[allow(unused_variables)]
impl KvStore {
    pub fn remove(&mut self, key: String) -> Result<()> {
        if self.index.contains_key(&key) {
            let cmd = Command::Remove { key };
            serde_json::to_writer(&mut self.writer, &cmd)?;
            self.writer.flush()?;

            // remove the key from the index
            if let Command::Remove { key } = cmd {
                self.index.remove(&key).expect("key not found");
            }
            Ok(())
        } else {
            println!("Key not found");
            Err(KvsError::KeyNotFound)
        }
    }

    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        if let Some(cmd_pos) = self.index.get(&key) {
            let reader = self
                .readers
                .get_mut(&cmd_pos.filename)
                .expect("Cannot find log reader");
            reader.seek(SeekFrom::Start(cmd_pos.pos))?;
            let cmd_reader = reader.take(cmd_pos.len);
            if let Command::Set { value, .. } = serde_json::from_reader(cmd_reader)? {
                Ok(Some(value))
            } else {
                Err(KvsError::UnexpectedCommandType)
            }
        } else {
            Ok(None)
        }
    }

    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        // create a command struct
        let cmd = Command::set(key, value);
        let old_pos = self.writer.pos;

        // Writing increments the pos pointer
        serde_json::to_writer(&mut self.writer, &cmd)?;
        self.writer.flush()?;

        if let Command::Set { key, .. } = cmd {
            if let Some(old_cmd) = self.index.insert(
                key,
                (self.current_filename, old_pos..self.writer.pos).into(),
            ) {
                // TODO
            }
        }

        Ok(())
    }

    /// Creates an instance of `KvStore` for a given path.
    ///
    /// Creates a new directory if there is none for the given path .
    ///
    /// # Errors
    ///
    /// Propagates I/O or Serde errors during the log replay.
    pub fn open(path: impl Into<std::path::PathBuf>) -> Result<KvStore> {
        let path = path.into();
        fs::create_dir_all(&path)?;

        let mut readers = HashMap::new();
        let mut index = BTreeMap::new();

        let gen_list = get_sorted_filenames(&path)?;
        let mut uncompacted = 0;

        for &gen in &gen_list {
            let mut reader = BufReaderWithPos::new(File::open(log_path(&path, gen))?)?;
            uncompacted += load_index(gen, &mut reader, &mut index)?;
            readers.insert(gen, reader);
        }

        let current_gen = gen_list.last().unwrap_or(&0) + 1;
        let writer = get_log_file(&path, current_gen, &mut readers)?;

        Ok(KvStore {
            path,
            readers,
            writer,
            current_filename: current_gen,
            index,
            uncompacted,
        })
    }
}

fn log_path(dir: &Path, filename: u64) -> PathBuf {
    dir.join(format!("{}.log", filename))
}

/// Reads the entries in the current directory, filters out ones that aren't .log files,
/// and sorts the resulting list.
///
/// The log entries we are interested in are incrementing integers. (e.g. 1.log, 2.log, etc.).
fn get_sorted_filenames(path: &Path) -> Result<Vec<u64>> {
    let mut list: Vec<u64> = fs::read_dir(path)?
        .flat_map(|item| -> Result<_> { Ok(item?.path()) })
        .filter(|path| path.is_file() && path.extension() == Some("log".as_ref()))
        .flat_map(|path| {
            path.file_name()
                .and_then(OsStr::to_str)
                .map(|s| s.trim_end_matches(".log")) // strip the .log from the end
                .map(str::parse::<u64>)
        })
        .flatten()
        .collect();

    list.sort_unstable();

    Ok(list)
}

/// Create a log file given the path of the current working directory.
fn get_log_file(
    path: &Path,
    filename: u64,
    readers: &mut HashMap<u64, BufReaderWithPos<File>>,
) -> Result<BufWriterWithPos<File>> {
    let path = log_path(path, filename);
    let writer = BufWriterWithPos::new(
        OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&path)?,
    )?;

    // create the reader for this filename
    readers.insert(filename, BufReaderWithPos::new(File::open(&path)?)?);

    Ok(writer)
}

/// Load the entire log file for the given reader into the index.
fn load_index(
    gen: u64,
    reader: &mut BufReaderWithPos<File>,
    index: &mut BTreeMap<String, CommandPos>,
) -> Result<u64> {
    let mut pos = reader.seek(SeekFrom::Start(0))?;
    let mut stream = Deserializer::from_reader(reader).into_iter::<Command>();
    while let Some(cmd) = stream.next() {
        let new_pos = stream.byte_offset() as u64;
        match cmd? {
            Command::Set { key, .. } => {
                index.insert(key, (gen, pos..new_pos).into());
            }

            Command::Remove { key } => {
                index.remove(&key);
            }
        }
        pos = new_pos;
    }

    Ok(0)
}
