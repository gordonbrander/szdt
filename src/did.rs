use crate::base58btc;
use crate::ed25519;

/// The multicodec prefix for ed25519 public key is 0xed.
/// https://github.com/multiformats/multicodec/blob/master/table.csv
const MULTICODEC_ED25519_PUB_PREFIX: u8 = 0xed;

/// The prefix for did:key using Base58BTC encoding.
/// The multibase code for ed25519 public key is 'z'.
const DID_KEY_BASE58BTC_PREFIX: &str = "did:key:z";

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DidKey(ed25519::PublicKey);

impl DidKey {
    pub fn new(pubkey_bytes: &[u8]) -> Result<Self, Error> {
        let pubkey = ed25519::slice_to_public_key(pubkey_bytes)?;
        Ok(DidKey(pubkey))
    }

    pub fn pubkey(&self) -> &ed25519::PublicKey {
        &self.0
    }
}

impl TryFrom<&str> for DidKey {
    type Error = Error;

    /// Parse a did:key str encoding an ed25519 public key into a DidKey.
    fn try_from(did_key: &str) -> Result<Self, Error> {
        // Parse the did:key
        let base58_key = did_key
            .strip_prefix(DID_KEY_BASE58BTC_PREFIX)
            .ok_or(Error::UnsupportedBase)?;

        let decoded_bytes = base58btc::decode(base58_key)?;

        // Verify that the first byte corresponds to ED25519_PUB_PREFIX
        if decoded_bytes.is_empty() || decoded_bytes[0] != MULTICODEC_ED25519_PUB_PREFIX {
            return Err(Error::UnsupportedCodec);
        }

        // Extract the public key
        DidKey::new(&decoded_bytes[1..])
    }
}

impl From<&DidKey> for String {
    fn from(did_key: &DidKey) -> Self {
        // Convert public key to multibase encoded string
        let mut multicodec_bytes = vec![MULTICODEC_ED25519_PUB_PREFIX];
        multicodec_bytes.extend_from_slice(&did_key.0);

        // Encode with multibase (Base58BTC, prefix 'z')
        let multibase_encoded = base58btc::encode(multicodec_bytes);

        // Construct the did:key
        format!("{}{}", DID_KEY_BASE58BTC_PREFIX, multibase_encoded)
    }
}

#[derive(Debug)]
pub enum Error {
    Key(ed25519::Error),
    Base(String),
    UnsupportedBase,
    UnsupportedCodec,
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Key(msg) => write!(f, "Public key error: {}", msg),
            Error::Base(msg) => write!(f, "Base encoding/decoding error: {}", msg),
            Error::UnsupportedBase => {
                write!(f, "Unsupported base encoding. Only Base58BTC is supported.")
            }
            Error::UnsupportedCodec => write!(
                f,
                "Unsupported codec. Only Ed25519 public keys are supported."
            ),
        }
    }
}

impl From<ed25519::Error> for Error {
    fn from(err: ed25519::Error) -> Self {
        Error::Key(err)
    }
}

impl From<bs58::decode::Error> for Error {
    fn from(err: bs58::decode::Error) -> Self {
        Error::Base(err.to_string())
    }
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

        let did = DidKey(pubkey);
        let did_string = String::from(&did);
        let did2 = DidKey::try_from(did_string.as_str()).unwrap();

        assert_eq!(did, did2);
    }

    #[test]
    fn test_decode_invalid_did_key() {
        // Invalid prefix
        assert!(DidKey::try_from("did:invalid:z123").is_err());

        // Invalid encoding
        assert!(DidKey::try_from("did:key:INVALID").is_err());

        // Empty string
        assert!(DidKey::try_from("").is_err());
    }
}
