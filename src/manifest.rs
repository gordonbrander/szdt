use super::error::Error;
use super::hash::Hash;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

pub static CONTENT_TYPE: &str = "application/vnd.szdt.manifest+cbor";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileEntry {
    /// Hash for file
    pub hash: Hash,
    /// Length of file in bytes
    pub length: u64,
    /// Path for file (display name)
    pub path: PathBuf,
}

impl FileEntry {
    pub fn new(hash: Hash, length: u64, path: PathBuf) -> Self {
        Self { hash, path, length }
    }
}

/// Read file entries from a list of paths
pub fn read_file_entries<I>(paths: I) -> Result<Vec<FileEntry>, Error>
where
    I: Iterator<Item = PathBuf>,
{
    let mut entries: Vec<FileEntry> = Vec::new();
    let mut buf = vec![];
    for path in paths {
        buf.truncate(0);
        let mut file = File::open(&path)?;
        file.read_to_end(&mut buf)?;
        let cbor_wrapped_bytes = serde_ipld_dagcbor::to_vec(&buf)?;
        let hash = Hash::new(&cbor_wrapped_bytes);
        entries.push(FileEntry::new(hash, cbor_wrapped_bytes.len() as u64, path));
    }
    Ok(entries)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Manifest {
    pub files: Vec<FileEntry>,
}

impl Manifest {
    pub fn new(files: Vec<FileEntry>) -> Self {
        Self { files }
    }
}
