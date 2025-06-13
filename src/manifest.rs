use crate::memo::Memo;

use super::error::Error;
use super::hash::Hash;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

pub static CONTENT_TYPE: &str = "application/vnd.szdt.manifest+cbor";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ManifestEntry {
    /// Hash for file
    pub hash: Hash,
    /// Length of file in bytes
    pub length: u64,
    /// Path for file (display name)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

impl ManifestEntry {
    pub fn new(hash: Hash, length: u64, path: Option<String>) -> Self {
        Self { hash, path, length }
    }

    /// Construct a new manifest entry for a serializable value.
    pub fn for_value<T: Serialize>(value: T, path: Option<String>) -> Result<Self, Error> {
        let cbor_bytes = serde_ipld_dagcbor::to_vec(&value)?;
        let hash = Hash::new(&cbor_bytes);
        Ok(Self::new(hash, cbor_bytes.len() as u64, path))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Manifest {
    pub entries: Vec<ManifestEntry>,
}

impl Manifest {
    pub fn new(files: Vec<ManifestEntry>) -> Self {
        Self { entries: files }
    }

    /// Create a new manifest from a list of paths by reading the contents of each file.
    pub fn from_paths<I>(paths: I) -> Result<Self, Error>
    where
        I: Iterator<Item = PathBuf>,
    {
        let mut entries: Vec<ManifestEntry> = Vec::new();
        let mut buf = vec![];
        for path in paths {
            buf.truncate(0);
            let mut file = File::open(&path)?;
            file.read_to_end(&mut buf)?;

            let path_string = path.to_string_lossy().into_owned();
            let file_entry = ManifestEntry::for_value(&buf, Some(path_string.clone()))?;

            let mut memo = Memo::new(file_entry.hash.clone());
            memo.protected.path = Some(path_string.clone());
            let memo_entry = ManifestEntry::for_value(memo, None)?;
            entries.push(memo_entry);
            entries.push(file_entry);
        }
        Ok(Self { entries })
    }
}
