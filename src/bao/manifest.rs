use super::hash::Hash;
use crate::util::now;
use serde::{Deserialize, Serialize};
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
}
