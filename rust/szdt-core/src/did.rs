use crate::base58btc;
use crate::ed25519;
use ed25519_dalek::PUBLIC_KEY_LENGTH;
use serde::{Deserialize, Serialize};
use thiserror::Error;

type PublicKey = [u8; PUBLIC_KEY_LENGTH];

/// The multicodec prefix for ed25519 public key is 0xed01.
/// https://github.com/multiformats/multicodec/blob/master/table.csv
const MULTICODEC_ED25519_PUB_PREFIX: &[u8] = &[0xed, 0x01];

/// The prefix for did:key using Base58BTC encoding.
/// The multibase code for ed25519 public key is 'z'.
const DID_KEY_BASE58BTC_PREFIX: &str = "did:key:z";

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DidKey(PublicKey);

impl DidKey {
    /// Create a new DidKey from a public key.
    pub fn new(pubkey_bytes: &[u8]) -> Result<Self, Error> {
        let pubkey = ed25519::to_public_key(pubkey_bytes)?;
        Ok(DidKey(pubkey))
    }

    /// Parse a did:key URL string into a DidKey.
    pub fn parse(did_key_url: &str) -> Result<Self, Error> {
        // Parse the did:key
        let base58_key = did_key_url
            .strip_prefix(DID_KEY_BASE58BTC_PREFIX)
            .ok_or(Error::Base(
                "Unsupported base encoding. Only Base58BTC is supported.".to_string(),
            ))?;

        let decoded_bytes = base58btc::decode(base58_key)?;

        // Strip the ED25519_PUB_PREFIX, and return an error if the prefix is not present
        let Some(key_bytes) = decoded_bytes.strip_prefix(MULTICODEC_ED25519_PUB_PREFIX) else {
            return Err(Error::UnsupportedCodec(
                "Only Ed25519 public keys are supported.".to_string(),
            ));
        };
        // Extract the public key
        DidKey::new(key_bytes)
    }

    pub fn public_key(&self) -> &ed25519::PublicKey {
        &self.0
    }
}

impl std::fmt::Display for DidKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Convert public key to multibase encoded string
        let mut multicodec_bytes = MULTICODEC_ED25519_PUB_PREFIX.to_vec();
        multicodec_bytes.extend_from_slice(&self.0);

        // Encode with multibase (Base58BTC, prefix 'z')
        let multibase_encoded = base58btc::encode(multicodec_bytes);

        // Construct the did:key
        write!(f, "{DID_KEY_BASE58BTC_PREFIX}{multibase_encoded}")
    }
}

impl From<&DidKey> for String {
    fn from(did_key: &DidKey) -> Self {
        did_key.to_string()
    }
}

impl TryFrom<&str> for DidKey {
    type Error = Error;

    /// Parse a did:key str encoding an ed25519 public key into a DidKey.
    fn try_from(did_key_url: &str) -> Result<Self, Error> {
        DidKey::parse(did_key_url)
    }
}

impl TryFrom<String> for DidKey {
    type Error = Error;

    /// Parse a did:key str encoding an ed25519 public key into a DidKey.
    fn try_from(did_key_url: String) -> Result<Self, Error> {
        DidKey::parse(&did_key_url)
    }
}

impl Serialize for DidKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&String::from(self))
    }
}

impl<'de> Deserialize<'de> for DidKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        DidKey::parse(&s).map_err(|e| serde::de::Error::custom(e.to_string()))
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Public key error: {0}")]
    Key(#[from] ed25519::Error),
    #[error("Base encoding/decoding error: {0}")]
    Base(String),
    #[error("Unsupported codec: {0}")]
    UnsupportedCodec(String),
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
    fn test_roundtrip_did_key() {
        // Test vector
        let pubkey: [u8; 32] = [
            215, 90, 152, 1, 130, 177, 10, 183, 213, 75, 254, 211, 201, 100, 7, 58, 14, 225, 114,
            243, 218, 166, 35, 37, 175, 2, 26, 104, 247, 7, 81, 26,
        ];

        let did = DidKey::new(&pubkey).unwrap();
        let did_string = String::from(&did);
        let did2 = DidKey::try_from(did_string.as_str()).unwrap();

        assert_eq!(did, did2);
    }

    #[test]
    fn test_roundtrip_did_url() {
        let did_url = "did:key:z6MkjxXr49JYNRDagDRVTNJKj17vTcmxwPb1KybzeVUM13qs";
        let did = DidKey::parse(did_url).unwrap();
        let did_url_2 = did.to_string();
        assert_eq!(did_url, did_url_2);
    }

    #[test]
    fn test_did_key_string_magic_prefix() {
        let pubkey: [u8; 32] = [
            215, 90, 152, 1, 130, 177, 10, 183, 213, 75, 254, 211, 201, 100, 7, 58, 14, 225, 114,
            243, 218, 166, 35, 37, 175, 2, 26, 104, 247, 7, 81, 26,
        ];

        let did = DidKey::new(&pubkey).unwrap();

        assert!(did.to_string().starts_with("did:key:z6Mk"));
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
