use super::error::Error;
use super::hash::Hash;
use crate::util::now;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    /// Hash for file
    pub hash: Hash,
    /// Length of file in bytes
    pub length: u64,
    /// Path for file (display name)
    pub path: PathBuf,
    /// Content type of file
    pub content_type: Option<String>,
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
        let len = file.read_to_end(&mut buf)?;
        let hash = Hash::new(&buf);
        entries.push(FileEntry::new(hash, len as u64, path));
    }
    Ok(entries)
}

impl FileEntry {
    pub fn new(hash: Hash, length: u64, path: PathBuf) -> Self {
        Self {
            hash,
            path,
            length,
            content_type: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkEntry {
    hash: Hash,
    urls: Vec<Url>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub prev: Option<LinkEntry>,
    pub next: Option<Url>,
    pub timestamp: u64,
    pub files: Vec<FileEntry>,
}

impl Manifest {
    pub fn new(files: Vec<FileEntry>, prev: Option<LinkEntry>, next: Option<Url>) -> Self {
        Self {
            files,
            prev,
            next,
            timestamp: now(),
        }
    }

    /// Serialize this to dag-cbor bytes suitable for signing
    pub fn into_signing_bytes(&self) -> Result<Vec<u8>, Error> {
        let bytes = serde_ipld_dagcbor::to_vec(self)?;
        Ok(bytes)
    }
}
