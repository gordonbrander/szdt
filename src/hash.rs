use crate::error::{Error, Result};

/// Encodes a SHA-256 hash as a multihash by adding the appropriate identifier bytes.
///
/// A multihash is a self-describing hash format that includes:
/// - A hash function identifier (0x12 for SHA-256)
/// - The length of the hash (32 bytes for SHA-256)
/// - The hash digest itself
///
/// # Arguments
/// * `hash_bytes` - A byte slice containing the raw SHA-256 hash (should be 32 bytes)
///
/// # Returns
/// * `Vec<u8>` - The multihash-encoded bytes
/// ```
pub fn sha256_to_multihash(hash_bytes: &[u8; 32]) -> Vec<u8> {
    // Create a new vector with capacity for the multihash
    let mut multihash = Vec::with_capacity(34); // 2 bytes prefix + 32 bytes hash

    // Add the multihash identifier for SHA-256 (0x12)
    multihash.push(0x12);

    // Add the length of the hash (32 bytes)
    multihash.push(32);

    // Add the actual hash bytes
    multihash.extend_from_slice(hash_bytes);

    multihash
}

/// Decodes a multihash into a SHA-256 hash.
///
/// A multihash is a self-describing hash format that includes:
/// - A hash function identifier (0x12 for SHA-256)
/// - The length of the hash (32 bytes for SHA-256)
/// - The hash digest itself
///
/// # Arguments
/// * `multihash` - A byte slice containing the multihash-encoded bytes (should be 34 bytes)
///
/// # Returns
/// * `Result<Vec<u8>>` - The decoded SHA-256 hash bytes or an error if the multihash is invalid
/// ```
pub fn multihash_to_sha256(multihash: &[u8; 34]) -> Result<Vec<u8>> {
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
    let hash_bytes = &multihash[2..];

    Ok(hash_bytes.to_vec())
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

        // Verify the result
        assert_eq!(hash.len(), 32);
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
}
