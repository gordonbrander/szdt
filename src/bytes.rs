use serde::de::{self, Visitor};
use serde::{Deserialize, Serialize};

/// Wrapper for byte vectors that will ensure they are serialized as byte strings
/// and not arrays.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Bytes(pub Vec<u8>);

impl Bytes {
    pub fn into_inner(self) -> Vec<u8> {
        self.0
    }
}

impl Serialize for Bytes {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bytes(&self.0)
    }
}

impl<'de> Deserialize<'de> for Bytes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_bytes(BytesVisitor)
    }
}

struct BytesVisitor;

impl<'de> Visitor<'de> for BytesVisitor {
    type Value = Bytes;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a byte array")
    }

    fn visit_bytes<E>(self, value: &[u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Bytes(value.to_vec()))
    }

    fn visit_byte_buf<E>(self, value: Vec<u8>) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Bytes(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_ipld_dagcbor;

    #[test]
    fn test_bytes_serialized_as_byte_string() {
        let bytes = Bytes(vec![1, 2, 3, 4, 5]);
        let serialized = serde_ipld_dagcbor::to_vec(&bytes).unwrap();

        // Check that the first byte indicates a byte string (major type 2)
        // In CBOR, major type 2 is for byte strings, encoded as 0b010xxxxx
        assert_eq!(
            serialized[0] & 0xE0,
            0x40,
            "Should be serialized as byte string (major type 2)"
        );

        // Verify round-trip deserialization works
        let deserialized: Bytes = serde_ipld_dagcbor::from_slice(&serialized).unwrap();
        assert_eq!(bytes, deserialized);
    }
}
