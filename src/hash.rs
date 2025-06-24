/// A Blake3 hash
use serde::{Deserialize, Serialize};
use std::io::Read;

/// Blake3 hash
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Hash(blake3::Hash);

impl Hash {
    /// Hash the provided bytes.
    pub fn new(buf: impl AsRef<[u8]>) -> Self {
        let val = blake3::hash(buf.as_ref());
        Hash(val)
    }

    /// Streaming hash the bytes returned by a reader
    pub fn from_reader<R: Read>(reader: R) -> Self {
        let mut hasher = blake3::Hasher::new();
        let mut buffer = [0; 1024];
        let mut reader = std::io::BufReader::new(reader);
        while let Ok(n) = reader.read(&mut buffer) {
            if n == 0 {
                break;
            }
            hasher.update(&buffer[..n]);
        }
        Hash(hasher.finalize())
    }

    /// Create a Hash from its raw bytes representation.
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(blake3::Hash::from_bytes(bytes))
    }

    /// Bytes of the hash.
    pub fn as_bytes(&self) -> &[u8; 32] {
        self.0.as_bytes()
    }
}

impl PartialOrd for Hash {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.0.as_bytes().cmp(other.0.as_bytes()))
    }
}

impl Ord for Hash {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.as_bytes().cmp(other.0.as_bytes())
    }
}

impl std::fmt::Display for Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Serialize for Hash {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Hash {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let inner = blake3::Hash::deserialize(deserializer)?;
        Ok(Hash(inner))
    }
}

impl From<[u8; 32]> for Hash {
    /// Convert hash bytes to hash
    fn from(value: [u8; 32]) -> Self {
        Self::from_bytes(value)
    }
}

impl From<Hash> for blake3::Hash {
    fn from(value: Hash) -> Self {
        value.0
    }
}

impl From<blake3::Hash> for Hash {
    fn from(value: blake3::Hash) -> Self {
        Hash(value)
    }
}
