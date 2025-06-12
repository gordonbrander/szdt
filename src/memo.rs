use crate::ed25519_key_material::Ed25519KeyMaterial;
use crate::error::Error;
use crate::hash::Hash;
use crate::link::IntoLink;
use crate::util::now;
use crate::{did::DidKey, error::TimestampComparison};
use cbor4ii::core::Value;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Memo {
    /// Issuer (DID)
    pub iss: DidKey,
    /// Issued at (UNIX timestamp, seconds)
    pub iat: u64,
    /// Not valid before (UNIX timestamp, seconds)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nbf: Option<u64>,
    /// Expiration time (UNIX timestamp, seconds)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exp: Option<u64>,
    /// Blake3 hash of the previous version of the memo
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prev: Option<Hash>,
    /// Content type (MIME type)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ctype: Option<String>,
    /// Hash of the body of the memo
    pub body: Hash,
    /// Additional "non-blessed" fields
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sig: Option<Vec<u8>>,
}

impl Memo {
    /// Create a new memo with the given issuer and body
    pub fn new(iss: DidKey, body: Hash) -> Self {
        Self {
            iss,
            iat: now(),
            nbf: Some(now()),
            exp: None,
            prev: None,
            ctype: None,
            body,
            extra: HashMap::new(),
            sig: None,
        }
    }

    /// Sign the memo with the given key material, returning a signed memo
    pub fn sign(&mut self, key_material: &Ed25519KeyMaterial) -> Result<(), Error> {
        // Set any existing signature to None. This will prevent the field
        // from being serialized.
        self.sig = None;
        // Get link hash (hash of CBOR-encoded memo)
        let link = &self.into_link()?;
        let sig = key_material.sign(link.as_bytes())?;
        // Set the signature
        self.sig = Some(sig);
        Ok(())
    }

    /// Verify the signature of the memo, returning a result.
    /// In the case that a Memo is not signed, will return an error of `Error::MemoUnsigned`.
    pub fn verify(&self) -> Result<(), Error> {
        let Some(sig) = &self.sig else {
            return Err(Error::MemoUnsigned);
        };
        let key_material = Ed25519KeyMaterial::try_from(&self.iss)?;

        // Recreate the unsigned memo.
        let mut copy = self.clone();
        copy.sig = None;
        // Get the link for unsigned memo.
        let link = copy.into_link()?;
        // Verify the signature against the link.
        key_material.verify(link.as_bytes(), sig)?;
        Ok(())
    }

    /// Is claim expired?
    pub fn is_expired(&self, now_time: Option<u64>) -> bool {
        match self.exp {
            Some(exp) => exp < now_time.unwrap_or_else(now),
            None => false,
        }
    }

    /// Is claim too early?
    pub fn is_too_early(&self, now_time: Option<u64>) -> bool {
        match self.nbf {
            Some(nbf) => nbf > now_time.unwrap_or_else(now),
            None => false,
        }
    }

    /// Is memo valid?
    /// Checks if the memo is expired or too early, and verifies the signature.
    /// Unsigned memos are considered invalid.
    pub fn validate(&self, now_time: Option<u64>) -> Result<(), Error> {
        if self.is_expired(now_time) {
            return Err(Error::MemoExpError(TimestampComparison::new(
                self.exp, now_time,
            )));
        }
        if self.is_too_early(now_time) {
            return Err(Error::MemoNbfError(TimestampComparison::new(
                self.nbf, now_time,
            )));
        }
        self.verify()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::did::DidKey;
    use crate::ed25519_key_material::Ed25519KeyMaterial;
    use crate::hash::Hash;

    fn create_test_key() -> Ed25519KeyMaterial {
        Ed25519KeyMaterial::generate()
    }

    fn create_test_hash() -> Hash {
        Hash::from_bytes([
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
            25, 26, 27, 28, 29, 30, 31, 32,
        ])
    }

    #[test]
    fn test_memo_new() {
        let key = create_test_key();
        let did = DidKey::from(&key);
        let body_hash = create_test_hash();

        let memo = Memo::new(did.clone(), body_hash.clone());

        assert_eq!(memo.iss, did);
        assert_eq!(memo.body, body_hash);
        assert!(memo.nbf.is_some());
        assert!(memo.exp.is_none());
        assert!(memo.prev.is_none());
        assert!(memo.ctype.is_none());
    }

    #[test]
    fn test_memo_is_expired() {
        let key = create_test_key();
        let did = DidKey::from(&key);
        let body_hash = create_test_hash();
        let mut memo = Memo::new(did, body_hash);

        // Not expired when no expiration set
        assert!(!memo.is_expired(None));

        // Set expiration in the past
        memo.exp = Some(now() - 3600);
        assert!(memo.is_expired(None));

        // Set expiration in the future
        memo.exp = Some(now() + 3600);
        assert!(!memo.is_expired(None));
    }

    #[test]
    fn test_memo_is_too_early() {
        let key = create_test_key();
        let did = DidKey::from(&key);
        let body_hash = create_test_hash();
        let mut memo = Memo::new(did, body_hash);

        // Set nbf in the future
        memo.nbf = Some(now() + 3600);
        assert!(memo.is_too_early(None));

        // Set nbf in the past
        memo.nbf = Some(now() - 3600);
        assert!(!memo.is_too_early(None));

        // No nbf set
        memo.nbf = None;
        assert!(!memo.is_too_early(None));
    }

    #[test]
    fn test_memo_validate_unsigned() {
        let key = create_test_key();
        let did = DidKey::from(&key);
        let body_hash = create_test_hash();
        let memo = Memo::new(did, body_hash);

        // Valid memo
        assert!(memo.validate(None).is_err());
    }

    #[test]
    fn test_memo_sign_and_verify() {
        let key = create_test_key();
        let did = DidKey::from(&key);
        let body_hash = create_test_hash();
        let mut memo = Memo::new(did, body_hash.clone());

        memo.sign(&key).unwrap();

        assert!(memo.sig.is_some());

        memo.verify().unwrap();
    }

    #[test]
    fn test_signed_memo_validate() {
        let key = create_test_key();
        let did = DidKey::from(&key);
        let body_hash = create_test_hash();
        let mut memo = Memo::new(did, body_hash.clone());

        memo.sign(&key).unwrap();
        memo.validate(None).unwrap();
    }

    #[test]
    fn test_signed_memo_validate_expired() {
        let key = create_test_key();
        let did = DidKey::from(&key);
        let body_hash = create_test_hash();
        let mut memo = Memo::new(did, body_hash);

        memo.exp = Some(now() - 3600); // Expired
        memo.sign(&key).unwrap();

        assert!(memo.validate(None).is_err());
    }
}
