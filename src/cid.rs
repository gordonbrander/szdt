use sha2::{Digest, Sha256};

/// Represents a CIDv1 with SHA-256 hash using raw codec (0x55)
pub struct Cid(Vec<u8>);

impl Cid {
    pub fn from_sha256(sha256_hash: impl AsRef<[u8]>) -> Self {
        // Initialize a vector to hold the CID bytes
        // 4 bytes for version, codec, hash algo, length + 32 for hash
        let mut cid_bytes = Vec::with_capacity(36);

        // version 1
        cid_bytes.push(0x01);

        // raw codec (0x55)
        cid_bytes.push(0x55);

        // sha2-256 hash algorithm (0x12)
        cid_bytes.push(0x12);

        // hash length (32 bytes for sha256)
        cid_bytes.push(32);

        // append the hash itself
        cid_bytes.extend_from_slice(&sha256_hash.as_ref());

        Self(cid_bytes)
    }

    /// Create a CIDv1 from the bytes of a SHA-256 hash
    pub fn new(bytes: impl AsRef<[u8]>) -> Self {
        let sha256_hash = Sha256::digest(bytes.as_ref());
        Self::from_sha256(sha256_hash)
    }

    /// Converts the CID to a CIDv1 string representation
    /// See https://dasl.ing/cid.html
    pub fn to_cid_string(&self) -> String {
        // Convert to base32
        let mut encoded = String::with_capacity(self.0.len() * 2);

        // Add the "b" multibase prefix for base32
        encoded.push('b');

        // Encode the CID bytes using base32 lower
        encoded.push_str(&data_encoding::BASE32_NOPAD.encode(&self.0));

        // Return the encoded string (lowercase)
        encoded.to_lowercase()
    }
}

impl From<String> for Cid {
    fn from(s: String) -> Self {
        Self::new(s.as_bytes())
    }
}

impl From<&str> for Cid {
    fn from(s: &str) -> Self {
        Self::new(s.as_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cid_from_str() {
        let text = "hello world";
        let cid = Cid::from(text);

        // Verify the CID starts with the correct prefix
        assert_eq!(cid.0[0], 0x01); // version 1
        assert_eq!(cid.0[1], 0x55); // raw codec
        assert_eq!(cid.0[2], 0x12); // sha2-256
        assert_eq!(cid.0[3], 32); // hash length

        // Check total length (prefix + hash)
        assert_eq!(cid.0.len(), 36);
    }

    #[test]
    fn test_cid_from_bytes() {
        let bytes = b"test data";
        let cid = Cid::new(bytes);

        // Verify the structure is correct
        assert_eq!(cid.0[0], 0x01);
        assert_eq!(cid.0[1], 0x55);
        assert_eq!(cid.0[2], 0x12);
        assert_eq!(cid.0[3], 32);
        assert_eq!(cid.0.len(), 36);
    }

    #[test]
    fn test_cid_string_representation() {
        let text = "hello world";
        let cid = Cid::from(text);

        // Known value test - this hash is for "hello world"
        let expected_cid = "bafkreifzjut3te2nhyekklss27nh3k72ysco7y32koao5eei66wof36n5e";
        assert_eq!(cid.to_cid_string(), expected_cid);
    }

    #[test]
    fn test_different_inputs_yield_different_cids() {
        let cid1 = Cid::from("data1".to_string());
        let cid2 = Cid::from("data2".to_string());

        // Check that different inputs create different CIDs
        assert_ne!(cid1.0, cid2.0);
        assert_ne!(cid1.to_cid_string(), cid2.to_cid_string());
    }

    #[test]
    fn test_identical_inputs_yield_same_cids() {
        let cid1 = Cid::from("same data".to_string());
        let cid2 = Cid::from("same data".to_string());

        // Check that identical inputs create the same CID
        assert_eq!(cid1.0, cid2.0);
        assert_eq!(cid1.to_cid_string(), cid2.to_cid_string());
    }
}
