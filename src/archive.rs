use crate::cid::{self, read_into_cid_v1_raw};
use crate::file::walk_files;
use cid::Cid;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::collections::TryReserveError;
use std::fs::File;
use std::path::Path;
use thiserror::Error;
use url::Url;

/// Archive manifest
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Archive {
    files: BTreeMap<String, Link>,
}

impl Archive {
    pub fn new() -> Self {
        Archive {
            files: BTreeMap::new(),
        }
    }

    /// Add a file entry by reading from the file system, generating a CID.
    /// Uses file path (relative to working directory) as the display name.
    pub fn add_file(&mut self, path: &Path) -> Result<(), Error> {
        let mut file = File::open(path)?;
        let cid = read_into_cid_v1_raw(&mut file)?;
        let dn = path.to_string_lossy().to_string();
        let link = Link {
            content: cid,
            location: Vec::new(),
        };
        self.files.insert(dn, link);
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

impl TryFrom<Archive> for Cid {
    type Error = Error;

    fn try_from(value: Archive) -> Result<Self, Self::Error> {
        let bytes = serde_ipld_dagcbor::to_vec(&value)?;
        let archive_cid = cid::read_into_cid_v1_cbor(&mut bytes.as_slice())?;
        Ok(archive_cid)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Link {
    content: Cid,
    location: Vec<Url>,
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
        let mut archive = Archive::new();

        // Create a mock file entry
        let link = Link {
            content: Cid::try_from("bafkreihdwdcefgh4dqkjv67uzcmw7ojee6xedzdetojuzjevtenxquvyku")
                .unwrap(),
            location: Vec::new(),
        };

        archive.files.insert("test-file".to_string(), link);

        // Convert archive to CID
        let cid = Cid::try_from(archive).unwrap();
        assert!(cid.to_string().starts_with("bafy"));
    }
}
