use crate::cid::{self, read_into_cid_v1_raw};
use crate::file::walk_files;
use cid::Cid;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::collections::TryReserveError;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use thiserror::Error;
use url::Url;

/// Archive manifest
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
    files: BTreeMap<PathBuf, Link>,
}

impl Manifest {
    pub fn new() -> Self {
        Manifest {
            files: BTreeMap::new(),
        }
    }

    /// Create an archive from a directory
    pub fn from_dir(dir: &Path) -> Result<Self, Error> {
        let mut archive = Manifest::new();
        archive.add_dir(dir)?;
        Ok(archive)
    }

    /// Add a file entry by reading from the file system, generating a CID.
    /// Uses file path (relative to working directory) as the display name.
    pub fn add_file(&mut self, path: &Path) -> Result<(), Error> {
        let mut file = File::open(path)?;
        let link = Link::read_from(&mut file)?;
        self.files.insert(path.to_owned(), link);
        Ok(())
    }

    /// Add all files in a directory (recursive)
    pub fn add_dir(&mut self, dir: &Path) -> Result<(), Error> {
        for path in walk_files(dir)? {
            self.add_file(&path)?;
        }
        Ok(())
    }
}

impl TryFrom<Manifest> for Cid {
    type Error = Error;

    fn try_from(value: Manifest) -> Result<Self, Self::Error> {
        let bytes = serde_ipld_dagcbor::to_vec(&value)?;
        let archive_cid = cid::read_into_cid_v1_cbor(&mut bytes.as_slice())?;
        Ok(archive_cid)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Link {
    pub content: Cid,
    pub location: Vec<Url>,
}

impl Link {
    pub fn new(content: Cid, location: Vec<Url>) -> Self {
        Link { content, location }
    }

    /// Read link from reader
    pub fn read_from<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let cid = read_into_cid_v1_raw(reader)?;
        let link = Link::new(cid, Vec::new());
        Ok(link)
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("CID error: {0}")]
    Cid(#[from] cid::Error),
    #[error("CBOR encode error: {0}")]
    CborEncode(#[from] serde_ipld_dagcbor::EncodeError<TryReserveError>),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_from_archive_to_cid() {
        let mut archive = Manifest::new();

        // Create a mock file entry
        let link = Link {
            content: Cid::try_from("bafkreihdwdcefgh4dqkjv67uzcmw7ojee6xedzdetojuzjevtenxquvyku")
                .unwrap(),
            location: Vec::new(),
        };

        archive.files.insert("test-file".into(), link);

        // Convert archive to CID
        let cid = Cid::try_from(archive).unwrap();
        assert!(cid.to_string().starts_with("bafy"));
    }
}
