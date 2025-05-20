use crate::cid::{self, read_into_cid_v1_raw};
use cid::Cid;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
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
    pub fn new(files: BTreeMap<String, Link>) -> Self {
        Archive { files }
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
}
