use crate::error::Error;
use crate::hash::Hash;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

pub static CONTENT_TYPE: &str = "application/vnd.szdt.manifest+cbor";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResourceHeaders {
    /// Hash for resource
    pub src: Hash,
    /// Length of resource in bytes
    pub length: u64,
    /// Path for resource
    pub path: PathBuf,
    /// Content type (MIME type) of resource (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "content-type")]
    pub content_type: Option<String>,
}

impl ResourceHeaders {
    /// Construct a new manifest entry for a serializable value.
    pub fn for_value<T: Serialize>(
        value: T,
        path: PathBuf,
        content_type: Option<String>,
    ) -> Result<Self, Error> {
        let cbor_bytes = serde_ipld_dagcbor::to_vec(&value)?;
        let src = Hash::new(&cbor_bytes);
        Ok(Self {
            src,
            length: cbor_bytes.len() as u64,
            path,
            content_type,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Manifest {
    pub resources: Vec<ResourceHeaders>,
}

impl Manifest {
    /// Create a new manifest from a list of paths by reading the contents of each file.
    pub fn from_paths<I>(paths: I, base: &Path) -> Result<Self, Error>
    where
        I: Iterator<Item = PathBuf>,
    {
        let mut resources: Vec<ResourceHeaders> = Vec::new();
        for path in paths {
            let mut bytes = vec![];
            let mut file = File::open(&path)?;
            file.read_to_end(&mut bytes)?;
            let relative_path = path.strip_prefix(&base)?.to_path_buf();
            let cbor_bytes = cbor4ii::core::Value::Bytes(bytes);
            let headers = ResourceHeaders::for_value(&cbor_bytes, relative_path, None)?;
            resources.push(headers);
        }
        Ok(Self { resources })
    }

    pub fn keyed_by_path(self) -> HashMap<PathBuf, ResourceHeaders> {
        let mut index = HashMap::new();
        for resource in self.resources {
            index.insert(resource.path.clone(), resource);
        }
        index
    }

    /// Get a hashmap resources, keyed by
    pub fn keyed_by_src(self) -> HashMap<Hash, ResourceHeaders> {
        let mut index = HashMap::new();
        for resource in self.resources {
            index.insert(resource.src.clone(), resource);
        }
        index
    }
}
