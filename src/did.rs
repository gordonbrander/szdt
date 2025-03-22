use bs58;

/// The multicodec prefix for ed25519 public key is 0xed.
const ED25519_PUB_PREFIX: u8 = 0xed;

/// The prefix for did:key using Base58BTC encoding.
/// The multibase code for ed25519 public key is 'z'.
const DID_KEY_BASE58BTC_PREFIX: &str = "did:key:z";

/// Convert ed25519 public key bytes to a did:key.
pub fn encode_ed25519_did_key(public_key: &[u8; 32]) -> String {
    // Convert public key to multibase encoded string
    // For ed25519 public key, the multicodec prefix is 0xed
    // https://github.com/multiformats/multicodec/blob/master/table.csv
    let mut multicodec_bytes = vec![ED25519_PUB_PREFIX];
    multicodec_bytes.extend_from_slice(public_key);

    // Encode with multibase (Base58BTC, prefix 'z')
    let multibase_encoded = bs58::encode(multicodec_bytes).into_string();

    // Construct the did:key
    format!("{}{}", DID_KEY_BASE58BTC_PREFIX, multibase_encoded)
}

/// Convert a did:key string to ed25519 public key bytes.
pub fn decode_ed25519_did_key(did_key: &str) -> Option<[u8; 32]> {
    // Parse the did:key
    let did_key = did_key.strip_prefix(DID_KEY_BASE58BTC_PREFIX)?;
    let multibase_bytes = bs58::decode(did_key).into_vec().ok()?;

    // Extract the public key
    let public_key = multibase_bytes[1..].try_into().ok()?;

    Some(public_key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip_ed25519_did_key() {
        // Test vector
        let pubkey: [u8; 32] = [
            215, 90, 152, 1, 130, 177, 10, 183, 213, 75, 254, 211, 201, 100, 7, 58, 14, 225, 114,
            243, 218, 166, 35, 37, 175, 2, 26, 104, 247, 7, 81, 26,
        ];

        let did = encode_ed25519_did_key(&pubkey);
        let pubkey2 = decode_ed25519_did_key(&did).unwrap();

        assert_eq!(pubkey, pubkey2);
    }

    #[test]
    fn test_decode_invalid_did_key() {
        // Invalid prefix
        assert!(decode_ed25519_did_key("did:invalid:z123").is_none());

        // Invalid encoding
        assert!(decode_ed25519_did_key("did:key:INVALID").is_none());

        // Empty string
        assert!(decode_ed25519_did_key("").is_none());
    }
}
