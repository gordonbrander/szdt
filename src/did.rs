use crate::error::{Error, Result};
use bs58;

/// The multicodec prefix for ed25519 public key is 0xed.
/// https://github.com/multiformats/multicodec/blob/master/table.csv
const ED25519_PUB_PREFIX: u8 = 0xed;

/// The prefix for did:key using Base58BTC encoding.
/// The multibase code for ed25519 public key is 'z'.
const DID_KEY_BASE58BTC_PREFIX: &str = "did:key:z";

/// Encode bytes using Base58BTC encoding.
pub fn encode_base58btc<I>(bytes: I) -> String
where
    I: AsRef<[u8]>,
{
    bs58::encode(bytes).into_string()
}

/// Decode bytes from Base58BTC encoding.
pub fn decode_base58btc(s: &str) -> Result<Vec<u8>> {
    let bytes = bs58::decode(s).into_vec()?;
    Ok(bytes)
}

/// Convert ed25519 public key bytes to a did:key.
pub fn encode_ed25519_did_key(public_key: &[u8; 32]) -> String {
    // Convert public key to multibase encoded string
    let mut multicodec_bytes = vec![ED25519_PUB_PREFIX];
    multicodec_bytes.extend_from_slice(public_key);

    // Encode with multibase (Base58BTC, prefix 'z')
    let multibase_encoded = encode_base58btc(multicodec_bytes);

    // Construct the did:key
    format!("{}{}", DID_KEY_BASE58BTC_PREFIX, multibase_encoded)
}

/// Convert a did:key string to ed25519 public key bytes.
pub fn decode_ed25519_did_key(did_key: &str) -> Result<[u8; 32]> {
    // Parse the did:key
    let base58_key = did_key
        .strip_prefix(DID_KEY_BASE58BTC_PREFIX)
        .ok_or(Error::DecodingError(
            "did is not a valid did:key. Only did:key is supported.".to_string(),
        ))?;

    let multibase_bytes = decode_base58btc(base58_key)?;

    // Verify that the first byte corresponds to ED25519_PUB_PREFIX
    if multibase_bytes.is_empty() || multibase_bytes[0] != ED25519_PUB_PREFIX {
        return Err(Error::DecodingError(
            "Key is not Ed25519. Only did:keys with Ed25519 multibase prefix are supported."
                .to_string(),
        ));
    }

    // Extract the public key
    let Ok(public_key) = multibase_bytes[1..].try_into() else {
        return Err(Error::DecodingError(
            "Unable to extract public key bytes".to_string(),
        ));
    };

    Ok(public_key)
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
        assert!(decode_ed25519_did_key("did:invalid:z123").is_err());

        // Invalid encoding
        assert!(decode_ed25519_did_key("did:key:INVALID").is_err());

        // Empty string
        assert!(decode_ed25519_did_key("").is_err());
    }
}
