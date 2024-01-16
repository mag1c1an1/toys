use std::collections::BTreeMap;
use std::fs;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, BufWriter, Seek, SeekFrom, Write};
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use serde_json::Deserializer;
use crate::{KvsError, Result};

#[derive(Debug)]
pub struct KvStore {
    path: PathBuf,
    // writer: BufWriterWithPos<File>,
    // reader: BufReader<File>,
    // index: BTreeMap<String, CommandPos>,
    inner: BTreeMap<String, String>,
}

#[derive(Debug)]
struct BufWriterWithPos<W: Write + Seek> {
    writer: BufWriter<W>,
    pos: u64,
}

#[derive(Debug, Serialize, Deserialize)]
enum Command {
    Set {
        key: String,
        value: String,
    },
    Remove { key: String },
}

#[derive(Debug)]
struct CommandPos {
    pos: u64,
}

impl From<u64> for CommandPos {
    fn from(value: u64) -> Self {
        Self { pos: value }
    }
}


impl KvStore {
    pub fn open(path: impl Into<PathBuf>) -> Result<Self> {
        let mut path = path.into();
        fs::create_dir_all(&path)?;
        path.push("log.db");
        let inner = BTreeMap::new();
        let mut kv = Self {
            path,
            inner,
        };
        KvStore::replay(&mut kv)?;
        Ok(kv)
    }

    fn replay(kv: &mut KvStore) -> Result<()> {
        let f = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&kv.path)?;
        let reader = BufReader::new(f);
        let stream = Deserializer::from_reader(reader).into_iter::<Command>();
        for cmd in stream {
            match cmd? {
                Command::Set { key, value } => {
                    kv.inner.insert(key, value);
                }
                Command::Remove { key } => {
                    kv.inner.remove(&key);
                }
            }
        }
        Ok(())
    }

    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        let cmd = Command::Set { key, value };
        let mut writer = BufWriter::new(OpenOptions::new()
            .append(true)
            .create(true)
            .open(&self.path)?);
        serde_json::to_writer(&mut writer, &cmd)?;
        writer.flush()?;
        if let Command::Set { key, value } = cmd {
            self.inner.insert(key, value);
        }
        self.compaction()?;
        Ok(())
    }

    pub fn get(&self, key: String) -> Result<Option<String>> {
        match self.inner.get(&key) {
            None => {
                println!("Key not found");
                Ok(None)
            }
            Some(s) => Ok(Some(s.to_string())),
        }
    }
    pub fn remove(&mut self, key: String) -> Result<()> {
        if self.inner.get(&key).is_none() {
            println!("Key not found");
            return Err(KvsError::KeyNotFound);
        }
        let cmd = Command::Remove { key };
        let mut writer = BufWriter::new(OpenOptions::new().
            create(true)
            .append(true)
            .open(&self.path)?);
        serde_json::to_writer(&mut writer, &cmd)?;
        writer.flush()?;
        if let Command::Remove {key} = cmd {
           self.inner.remove(&key);
        }
        Ok(())
    }
    fn compaction(&mut self) -> Result<()> {
        fs::remove_file(&self.path)?;
        let mut writer = BufWriter::new(OpenOptions::new()
            .append(true)
            .create(true)
            .open(&self.path)?);
        let mut m = BTreeMap::new();
        for entry in &self.inner {
            let cmd = Command::Set { key:entry.0.clone(), value:entry.1.clone() };
            serde_json::to_writer(&mut writer, &cmd)?;
            writer.flush()?;
            if let Command::Set { key, value } = cmd {
                m.insert(key, value);
            }
        }
        self.inner = m;
       Ok(())
    }
}

impl<W> BufWriterWithPos<W> where W: Write + Seek {
    fn new(mut inner: W) -> Result<Self> {
        let pos = inner.stream_position()?;
        Ok(Self {
            writer: BufWriter::new(inner),
            pos,
        })
    }
}

impl<W: Write + Seek> Write for BufWriterWithPos<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let len = self.writer.write(buf)?;
        self.pos += len as u64;
        Ok(len)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}

impl<W: Write + Seek> Seek for BufWriterWithPos<W> {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.pos = self.writer.seek(pos)?;
        Ok(self.pos)
    }
}


#[cfg(test)]
mod tests {
    use std::env::current_dir;
    use super::*;

    #[test]
    fn serde_test() {
        let mut kvs = KvStore::open(current_dir().unwrap()).unwrap();
        kvs.set("String".to_string(), "String".to_string()).expect("TODO: panic message");
        kvs.set("Value".to_string(), "Value".to_string()).expect("TODO: panic message");
    }

    #[test]
    fn tmp_test() -> Result<()> {
        let mut store = KvStore::open(current_dir()?)?;

        store.set("key1".to_owned(), "value1".to_owned())?;
        store.set("key2".to_owned(), "value2".to_owned())?;

        assert_eq!(store.get("key1".to_owned())?, Some("value1".to_owned()));
        assert_eq!(store.get("key2".to_owned())?, Some("value2".to_owned()));

        // Open from disk again and check persistent data.
        drop(store);
        let mut store = KvStore::open(current_dir()?)?;
        assert_eq!(store.get("key1".to_owned())?, Some("value1".to_owned()));
        assert_eq!(store.get("key2".to_owned())?, Some("value2".to_owned()));
        Ok(())
    }
}
