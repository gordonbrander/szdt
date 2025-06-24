use serde::de::{self, Unexpected, Visitor};
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

    /// Construct a hash from a byte array representing the hash.
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(blake3::Hash::from_bytes(bytes))
    }

    /// Bytes of the hash.
    pub fn as_bytes(&self) -> &[u8; 32] {
        self.0.as_bytes()
    }

    /// Construct a hash from a slice representing the hash bytes
    pub fn from_slice(bytes: &[u8]) -> Result<Self, std::array::TryFromSliceError> {
        let byte_array: [u8; 32] = bytes.try_into()?;
        Ok(Self::from_bytes(byte_array))
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
        serializer.serialize_bytes(self.0.as_bytes())
    }
}

impl<'de> Deserialize<'de> for Hash {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_bytes(HashVisitor)
    }
}

struct HashVisitor;

impl<'de> Visitor<'de> for HashVisitor {
    type Value = Hash;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a byte string representing a hash")
    }

    fn visit_bytes<E>(self, value: &[u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let hash = Hash::from_slice(value).map_err(|_| {
            de::Error::invalid_value(Unexpected::Bytes(value), &"a byte array of length 32")
        })?;
        Ok(hash)
    }

    fn visit_byte_buf<E>(self, value: Vec<u8>) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let hash = Hash::from_slice(&value).map_err(|_| {
            de::Error::invalid_value(Unexpected::Bytes(&value), &"a byte array of length 32")
        })?;
        Ok(hash)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_serializes_as_cbor_byte_string() {
        let hash = Hash::new(b"test data");
        let serialized = serde_ipld_dagcbor::to_vec(&hash).expect("Failed to serialize hash");

        // CBOR byte strings with length 32 should start with 0x58 0x20
        // 0x58 = major type 2 (byte string) with additional info 24 (1-byte length follows)
        // 0x20 = 32 in decimal (the length of blake3 hash)
        assert_eq!(
            serialized[0], 0x58,
            "First byte should indicate CBOR byte string with 1-byte length"
        );
        assert_eq!(serialized[1], 32, "Second byte should be the length (32)");
        assert_eq!(
            serialized.len(),
            34,
            "Total length should be 2 header bytes + 32 data bytes"
        );

        // Verify the hash bytes are included
        assert_eq!(&serialized[2..], hash.as_bytes(), "Hash bytes should match");
    }

    #[test]
    fn test_hash_serialize_roundtrip() {
        let original_hash = Hash::new(b"roundtrip test data");

        // Serialize
        let serialized = serde_ipld_dagcbor::to_vec(&original_hash).expect("Failed to serialize");

        // Deserialize
        let deserialized_hash: Hash =
            serde_ipld_dagcbor::from_slice(&serialized).expect("Failed to deserialize");

        assert_eq!(
            original_hash, deserialized_hash,
            "Hash should roundtrip correctly"
        );
    }
}
