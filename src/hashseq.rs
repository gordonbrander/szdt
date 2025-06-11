use crate::hash::Hash;
use thiserror::Error;

pub struct HashSeq(Vec<u8>);

impl HashSeq {
    /// Create an empty hashseq
    pub fn empty() -> Self {
        Self(Vec::new())
    }

    /// Create a new sequence of hashes.
    pub fn new(bytes: Vec<u8>) -> Result<Self, Error> {
        if bytes.len() % 32 == 0 {
            Ok(Self(bytes))
        } else {
            Err(Error::InvalidBufferSize)
        }
    }

    pub fn append(&mut self, hash: Hash) -> () {
        self.0.extend_from_slice(hash.as_bytes());
    }

    /// Get the underlying byte representation
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Iterate over the hashes in this sequence.
    pub fn iter(&self) -> impl Iterator<Item = Hash> + '_ {
        self.0.chunks_exact(32).map(|chunk| {
            let hash: [u8; 32] = chunk.try_into().unwrap();
            hash.into()
        })
    }
}

impl serde::Serialize for HashSeq {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bytes(&self.0)
    }
}

impl<'de> serde::Deserialize<'de> for HashSeq {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_bytes(HashSeqVisitor)
    }
}

impl<I> From<I> for HashSeq
where
    I: Iterator<Item = Hash>,
{
    /// Construct hashseq from iterator of hashes
    fn from(iter: I) -> Self {
        let mut hashseq = HashSeq::empty();
        for hash in iter {
            hashseq.append(hash);
        }
        hashseq
    }
}

impl From<HashSeq> for Hash {
    /// Generate Hash for HashSeq
    fn from(value: HashSeq) -> Self {
        Hash::new(value.as_bytes())
    }
}

struct HashSeqVisitor;

impl<'de> serde::de::Visitor<'de> for HashSeqVisitor {
    type Value = HashSeq;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a byte slice representing a HashSeq")
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(HashSeq(v.to_vec()))
    }

    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(HashSeq(v))
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Invalid hash sequence buffer size")]
    InvalidBufferSize,
}
