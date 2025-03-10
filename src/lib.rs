use serde::{Deserialize, Serialize};
use serde_cbor;
use std::fs;
use std::io::Result;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn now_epoch_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Expected now to be greater than epoch")
        .as_secs()
}

/// Represents the contents of a file
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Hash)]
pub struct File {
    /// Suggested path for this file
    pub path: PathBuf,
    /// The raw file bytes
    pub content: Vec<u8>,
}

impl File {
    pub fn new(path: PathBuf, content: Vec<u8>) -> File {
        File { path, content }
    }

    pub fn read(path: PathBuf) -> Result<File> {
        let content = std::fs::read(&path)?;
        return Ok(File::new(path, content));
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Hash)]
pub struct Archive {
    /// A Unix timestamp representing the archive creation time
    pub created_at: u64,
    pub files: Vec<File>,
}

impl Archive {
    pub fn new(files: Vec<File>) -> Archive {
        Archive {
            created_at: now_epoch_secs(),
            files,
        }
    }

    pub fn from_paths(paths: &[PathBuf]) -> Result<Archive> {
        let mut files = Vec::new();

        for path in paths {
            let file = File::read(path.clone())?;
            files.push(file);
        }

        Ok(Archive {
            created_at: now_epoch_secs(),
            files,
        })
    }

    /// Write archive to CBOR
    pub fn write_cbor(&self, path: &Path) -> Result<()> {
        let cbor_file = fs::File::create(path)?;
        serde_cbor::to_writer(cbor_file, self).map_err(|e| std::io::Error::other(e))
    }

    /// Read archive from CBOR
    pub fn read_cbor(path: &Path) -> Result<Archive> {
        let cbor_file = fs::File::open(path)?;
        let archive: Archive =
            serde_cbor::from_reader(cbor_file).map_err(|e| std::io::Error::other(e))?;
        Ok(archive)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
