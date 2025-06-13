use crate::ed25519_key_material::Ed25519KeyMaterial;
use crate::error::Error;
use crate::hash::Hash;
use crate::link::ToLink;
use crate::util::now;
use crate::{did::DidKey, error::TimestampComparison};
use cbor4ii::core::Value;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Unprotected headers for a memo.
/// Contains metadata that is not signed and can be freely modified.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UnprotectedHeaders {
    /// Ed25519 signature over protected memo fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sig: Option<Vec<u8>>,
    /// Additional fields
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

impl Default for UnprotectedHeaders {
    fn default() -> Self {
        Self {
            sig: None,
            extra: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProtectedHeaders {
    /// Issuer (DID)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iss: Option<DidKey>,
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
    #[serde(rename = "content-type")]
    pub ctype: Option<String>,
    /// File path within archive
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// Blake3 hash of the memo body
    pub body: Hash,
    /// Additional fields
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

impl ProtectedHeaders {
    /// Create new headers with the given issuer and body hash
    pub fn new(body: Hash) -> Self {
        Self {
            iss: None,
            iat: now(),
            nbf: Some(now()),
            exp: None,
            prev: None,
            ctype: None,
            path: None,
            body,
            extra: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Memo {
    /// Unsigned headers
    pub unprotected: UnprotectedHeaders,
    /// Headers protected by signature
    pub protected: ProtectedHeaders,
}

impl Memo {
    /// Create a new memo with the given hash for the body content.
    pub fn new(body: Hash) -> Self {
        Self {
            unprotected: UnprotectedHeaders::default(),
            protected: ProtectedHeaders::new(body),
        }
    }

    /// Create a memo that notionally wraps the given body content.
    /// Content will be serialized to CBOR/c and hashed.
    pub fn for_body<T: Serialize>(body: T) -> Result<Self, Error> {
        Ok(Self::new(body.to_link()?))
    }

    /// Create a memo wrapping empty body content
    pub fn empty() -> Self {
        Self::new(Hash::new(&[]))
    }

    /// Sign the headers with the given key material
    pub fn sign(&mut self, key_material: &Ed25519KeyMaterial) -> Result<(), Error> {
        // Set the issuer DID on the protected headers
        self.protected.iss = Some(key_material.did());
        let protected_hash = &self.protected.to_link()?;

        // Sign
        let sig = key_material.sign(protected_hash.as_bytes())?;

        // Set the signature
        self.unprotected.sig = Some(sig);
        Ok(())
    }

    /// Verify the memo signature, returning a result.
    /// In the case that memo is not signed, will return an error of `Error::MemoUnsigned`.
    pub fn verify(&self) -> Result<(), Error> {
        let Some(iss) = &self.protected.iss else {
            return Err(Error::MemoIssMissing);
        };

        let Some(sig) = &self.unprotected.sig else {
            return Err(Error::MemoUnsigned);
        };

        let key_material = Ed25519KeyMaterial::try_from(iss)?;

        // Construct the signing bytes
        let protected_hash = self.protected.to_link()?;
        // Verify the signature against the signing bytes.
        key_material.verify(protected_hash.as_bytes(), sig)?;
        Ok(())
    }

    /// Is expired?
    pub fn is_expired(&self, now_time: Option<u64>) -> bool {
        match self.protected.exp {
            Some(exp) => exp < now_time.unwrap_or_else(now),
            None => false,
        }
    }

    /// Is too early?
    pub fn is_too_early(&self, now_time: Option<u64>) -> bool {
        match self.protected.nbf {
            Some(nbf) => nbf > now_time.unwrap_or_else(now),
            None => false,
        }
    }

    /// Is memo valid?
    /// Checks if expired or too early, and verifies the signature.
    /// Unsigned memos are considered invalid (untrusted).
    pub fn validate(&self, now_time: Option<u64>) -> Result<(), Error> {
        if self.is_expired(now_time) {
            return Err(Error::MemoExpError(TimestampComparison::new(
                self.protected.exp,
                now_time,
            )));
        }
        if self.is_too_early(now_time) {
            return Err(Error::MemoNbfError(TimestampComparison::new(
                self.protected.nbf,
                now_time,
            )));
        }
        self.verify()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ed25519_key_material::Ed25519KeyMaterial;
    use crate::hash::Hash;

    fn create_test_key() -> Ed25519KeyMaterial {
        Ed25519KeyMaterial::generate()
    }

    fn create_test_body() -> Vec<u8> {
        b"Hello World".to_vec()
    }

    #[test]
    fn test_headers_new() {
        let body = create_test_body();
        let body_hash = Hash::new(&body);

        let headers = ProtectedHeaders::new(body_hash.clone());

        assert!(headers.iss.is_none());
        assert_eq!(headers.body, body_hash);
        assert!(headers.nbf.is_some());
        assert!(headers.exp.is_none());
        assert!(headers.prev.is_none());
        assert!(headers.ctype.is_none());
        assert!(headers.path.is_none());
    }

    #[test]
    fn test_memo_new() {
        let body_content = "Hello World";
        let memo = Memo::for_body(&body_content).unwrap();

        let cbor_bytes = serde_ipld_dagcbor::to_vec(body_content).unwrap();
        assert_eq!(memo.protected.body, Hash::new(&cbor_bytes));
    }

    #[test]
    fn test_memo_is_expired() {
        let body = create_test_body();

        let mut memo = Memo::for_body(&body).unwrap();

        // Not expired when no expiration set
        assert!(!memo.is_expired(None));

        // Set expiration in the past
        memo.protected.exp = Some(now() - 3600);
        assert!(memo.is_expired(None));

        // Set expiration in the future
        memo.protected.exp = Some(now() + 3600);
        assert!(!memo.is_expired(None));
    }

    #[test]
    fn test_headers_is_too_early() {
        let body = create_test_body();

        let mut memo = Memo::for_body(&body).unwrap();

        // Set nbf in the future
        memo.protected.nbf = Some(now() + 3600);
        assert!(memo.is_too_early(None));

        // Set nbf in the past
        memo.protected.nbf = Some(now() - 3600);
        assert!(!memo.is_too_early(None));

        // No nbf set
        memo.protected.nbf = None;
        assert!(!memo.is_too_early(None));
    }

    #[test]
    fn test_memo_validate_unsigned() {
        let body_content = b"Hello World".to_vec();
        let memo = Memo::for_body(&body_content).unwrap();

        // Unsigned memo should be invalid
        assert!(memo.validate(None).is_err());
    }

    #[test]
    fn test_memo_sign_and_verify() {
        let key = create_test_key();
        let body = create_test_body();
        let mut memo = Memo::for_body(&body).unwrap();

        memo.sign(&key).unwrap();

        assert!(memo.unprotected.sig.is_some());

        memo.verify().unwrap();
    }

    #[test]
    fn test_signed_memo_validate() {
        let key = create_test_key();
        let body_content = b"Hello World".to_vec();
        let mut memo = Memo::for_body(&body_content).unwrap();

        memo.sign(&key).unwrap();
        memo.validate(None).unwrap();
    }

    #[test]
    fn test_signed_memo_validate_expired() {
        let key = create_test_key();
        let body_content = b"Hello World".to_vec();
        let mut memo = Memo::for_body(&body_content).unwrap();

        memo.protected.exp = Some(now() - 3600); // Expired
        memo.sign(&key).unwrap();

        assert!(memo.validate(None).is_err());
    }
}
