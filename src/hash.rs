use crate::error::{Error, Result};
use sha2::{Digest, Sha256};

pub type Hash = [u8; 32];
pub type Multihash = [u8; 34];

/// Converts a Slice<u8> to a fixed-size array of 32 bytes.
pub fn to_hash(bytes: &[u8]) -> Result<Hash> {
    if bytes.len() != 32 {
        return Err(Error::ValueError(format!(
            "Expected 32 bytes, got {}",
            bytes.len()
        )));
    }

    // This is safe because we've verified the length is exactly 32
    let bytes_32 = bytes
        .try_into()
        .map_err(|_| Error::ValueError("Failed to convert vector to array".to_string()))?;

    Ok(bytes_32)
}

/// Encodes a SHA-256 hash as a multihash by adding the appropriate identifier bytes.
///
/// A multihash is a self-describing hash format that includes:
/// - A hash function identifier (0x12 for SHA-256)
/// - The length of the hash (32 bytes for SHA-256)
/// - The hash digest itself
pub fn sha256_to_multihash(sha256digest: &Hash) -> Multihash {
    // Create a new vector with capacity for the multihash
    let mut multihash = Vec::with_capacity(34); // 2 bytes prefix + 32 bytes hash

    // Add the multihash identifier for SHA-256 (0x12)
    multihash.push(0x12);

    // Add the length of the hash (32 bytes)
    multihash.push(32);

    // Add the actual hash bytes
    multihash.extend_from_slice(sha256digest);

    multihash
        .try_into()
        .expect("Vec should be convertible to multihash array")
}

/// Decodes a multihash into a SHA-256 hash.
///
/// A multihash is a self-describing hash format that includes:
/// - A hash function identifier (0x12 for SHA-256)
/// - The length of the hash (32 bytes for SHA-256)
/// - The hash digest itself
pub fn multihash_to_sha256(multihash: &[u8; 34]) -> Result<Hash> {
    // Ensure the multihash identifier is 0x12 (SHA-256)
    if multihash[0] != 0x12 {
        return Err(Error::ValueError(
            "SHA-256 multihash identifier must be 0x12".to_string(),
        ));
    }

    // Ensure the length is 32 bytes
    if multihash[1] != 32 {
        return Err(Error::ValueError(
            "SHA-256 multihash length must be 32 bytes".to_string(),
        ));
    }

    // Extract the hash bytes
    let hash = to_hash(&multihash[2..])?;

    Ok(hash)
}

pub fn sha256(bytes: &[u8]) -> Hash {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let hash = hasher.finalize();
    to_hash(hash.as_slice()).expect("Sha256 should return valid hash bytes")
}

/// Generate the SHA-256 multihash from bytes
pub fn sha256_multihash(bytes: &[u8]) -> Multihash {
    sha256_to_multihash(&sha256(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256_to_multihash() {
        // Create a mock SHA-256 hash (32 bytes of 0xAA)
        let hash = [0xAA; 32];

        // Convert to multihash
        let multihash = sha256_to_multihash(&hash);

        // Verify the result
        assert_eq!(multihash.len(), 34);
        assert_eq!(multihash[0], 0x12); // SHA-256 identifier
        assert_eq!(multihash[1], 32); // length
        assert_eq!(&multihash[2..], &hash); // hash content
    }

    #[test]
    fn test_multihash_to_sha256() {
        // Create a mock multihash
        let mut multihash = [0; 34];
        multihash[0] = 0x12; // SHA-256 identifier
        multihash[1] = 32; // length
        for i in 2..34 {
            multihash[i] = 0xBB; // Fill with test data
        }

        // Decode the multihash
        let hash = multihash_to_sha256(&multihash).unwrap();

        for byte in hash.iter() {
            assert_eq!(*byte, 0xBB);
        }
    }

    #[test]
    fn test_roundtrip() {
        // Create a mock SHA-256 hash
        let original_hash = [0xCC; 32];

        // Convert to multihash and back
        let multihash = sha256_to_multihash(&original_hash);
        let multihash_array: [u8; 34] = multihash.try_into().unwrap();
        let decoded_hash = multihash_to_sha256(&multihash_array).unwrap();

        // Verify the result
        assert_eq!(decoded_hash.len(), 32);
        assert_eq!(&decoded_hash[..], &original_hash[..]);
    }

    #[test]
    fn test_invalid_multihash_identifier() {
        // Create a multihash with an invalid identifier
        let mut invalid_multihash = [0; 34];
        invalid_multihash[0] = 0x11; // Not SHA-256
        invalid_multihash[1] = 32;

        // Attempt to decode
        let result = multihash_to_sha256(&invalid_multihash);

        // Verify it returns an error
        assert!(result.is_err());
        if let Err(Error::ValueError(msg)) = result {
            assert!(msg.contains("identifier"));
        } else {
            panic!("Expected ValueError about identifier");
        }
    }

    #[test]
    fn test_invalid_multihash_length() {
        // Create a multihash with an invalid length
        let mut invalid_multihash = [0; 34];
        invalid_multihash[0] = 0x12; // SHA-256
        invalid_multihash[1] = 31; // Wrong length

        // Attempt to decode
        let result = multihash_to_sha256(&invalid_multihash);

        // Verify it returns an error
        assert!(result.is_err());
        if let Err(Error::ValueError(msg)) = result {
            assert!(msg.contains("length"));
        } else {
            panic!("Expected ValueError about length");
        }
    }

    #[test]
    fn test_sha256_multihash() {
        // Test with a known string
        let input = b"hello world";
        let multihash = sha256_multihash(input);

        // Verify it's a proper multihash
        assert_eq!(multihash.len(), 34);
        assert_eq!(multihash[0], 0x12); // SHA-256 identifier
        assert_eq!(multihash[1], 32); // length

        // Verify the hash matches the expected SHA-256 hash of "hello world"
        // SHA-256 of "hello world" is b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9
        let expected_hash = [
            0xB9, 0x4D, 0x27, 0xB9, 0x93, 0x4D, 0x3E, 0x08, 0xA5, 0x2E, 0x52, 0xD7, 0xDA, 0x7D,
            0xAB, 0xFA, 0xC4, 0x84, 0xEF, 0xE3, 0x7A, 0x53, 0x80, 0xEE, 0x90, 0x88, 0xF7, 0xAC,
            0xE2, 0xEF, 0xCD, 0xE9,
        ];
        assert_eq!(&multihash[2..], expected_hash.as_slice());
    }
}
