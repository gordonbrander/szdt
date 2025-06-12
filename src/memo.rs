use crate::ed25519_key_material::Ed25519KeyMaterial;
use crate::error::Error;
use crate::hash::Hash;
use crate::link::IntoLink;
use crate::util::now;
use crate::{did::DidKey, error::TimestampComparison};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Memo {
    /// Issuer (DID)
    pub iss: DidKey,
    /// Issued at (UNIX timestamp, seconds)
    pub iat: u64,
    /// Not valid before (UNIX timestamp, seconds)
    pub nbf: Option<u64>,
    /// Expiration time (UNIX timestamp, seconds)
    pub exp: Option<u64>,
    /// Blake3 hash of the previous version of the memo
    pub prev: Option<Hash>,
    pub ctype: Option<String>,
    /// Hash of the body of the memo
    pub body: Hash,
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
        }
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

    /// Is claim valid?
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
        Ok(())
    }

    /// Get the content type
    pub fn ctype(&self) -> Option<&str> {
        self.ctype.as_deref()
    }

    /// Set the content type
    pub fn set_ctype(&mut self, ctype: Option<String>) {
        self.ctype = ctype;
    }

    /// Get the `prev` field, representing the hash of the previous memo
    pub fn prev(&self) -> Option<&Hash> {
        self.prev.as_ref()
    }

    /// Set the `prev` field, representing the hash of the previous memo
    pub fn set_prev(&mut self, prev: Option<Hash>) {
        self.prev = prev;
    }

    /// Sign the memo with the given key material, returning a signed memo
    pub fn sign(self, key_material: &Ed25519KeyMaterial) -> Result<SignedMemo, Error> {
        let link = &self.into_link()?;
        let sig = key_material.sign(link.as_bytes())?;
        Ok(SignedMemo {
            iss: self.iss,
            iat: self.iat,
            nbf: self.nbf,
            exp: self.exp,
            prev: self.prev,
            ctype: self.ctype,
            body: self.body,
            sig,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SignedMemo {
    /// Issuer (DID)
    pub iss: DidKey,
    /// Issued at (UNIX timestamp, seconds)
    pub iat: u64,
    /// Not valid before (UNIX timestamp, seconds)
    pub nbf: Option<u64>,
    /// Expiration time (UNIX timestamp, seconds)
    pub exp: Option<u64>,
    /// Blake3 hash of the previous version of the memo
    pub prev: Option<Hash>,
    /// Content type of the memo body
    pub ctype: Option<String>,
    /// Hash of the body of the memo
    pub body: Hash,
    /// Signature over the link of the rest of the memo
    pub sig: Vec<u8>,
}

impl SignedMemo {
    /// Get the unsigned memo data
    pub fn into_payload(self) -> Memo {
        Memo {
            iss: self.iss,
            iat: self.iat,
            nbf: self.nbf,
            exp: self.exp,
            prev: self.prev,
            ctype: self.ctype,
            body: self.body,
        }
    }

    /// Verify the signature of the memo
    pub fn verify(self) -> Result<Memo, Error> {
        let key_material = Ed25519KeyMaterial::try_from(&self.iss)?;
        let sig = self.sig.clone();
        let memo = self.into_payload();
        let link = memo.into_link()?;
        key_material.verify(link.as_bytes(), &sig)?;
        Ok(memo)
    }

    /// Verify the signature of the memo and validate the memo
    pub fn validate(self, now_time: Option<u64>) -> Result<Memo, Error> {
        let memo = self.verify()?;
        memo.validate(now_time)?;
        Ok(memo)
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
    fn test_memo_validate() {
        let key = create_test_key();
        let did = DidKey::from(&key);
        let body_hash = create_test_hash();
        let mut memo = Memo::new(did, body_hash);

        // Valid memo
        assert!(memo.validate(None).is_ok());

        // Expired memo
        memo.exp = Some(now() - 3600);
        assert!(memo.validate(None).is_err());

        // Reset expiration and set nbf in future
        memo.exp = None;
        memo.nbf = Some(now() + 3600);
        assert!(memo.validate(None).is_err());
    }

    #[test]
    fn test_memo_ctype_methods() {
        let key = create_test_key();
        let did = DidKey::from(&key);
        let body_hash = create_test_hash();
        let mut memo = Memo::new(did, body_hash);

        assert_eq!(memo.ctype(), None);

        memo.set_ctype(Some("text/plain".to_string()));
        assert_eq!(memo.ctype(), Some("text/plain"));

        memo.set_ctype(None);
        assert_eq!(memo.ctype(), None);
    }

    #[test]
    fn test_memo_prev_methods() {
        let key = create_test_key();
        let did = DidKey::from(&key);
        let body_hash = create_test_hash();
        let mut memo = Memo::new(did, body_hash);

        assert_eq!(memo.prev(), None);

        let prev_hash = create_test_hash();
        memo.set_prev(Some(prev_hash.clone()));
        assert_eq!(memo.prev(), Some(&prev_hash));

        memo.set_prev(None);
        assert_eq!(memo.prev(), None);
    }

    #[test]
    fn test_memo_sign_and_verify() {
        let key = create_test_key();
        let did = DidKey::from(&key);
        let body_hash = create_test_hash();
        let memo = Memo::new(did, body_hash.clone());

        let signed_memo = memo.sign(&key).unwrap();

        assert_eq!(signed_memo.body, body_hash);
        assert!(!signed_memo.sig.is_empty());

        let verified_memo = signed_memo.verify().unwrap();
        assert_eq!(verified_memo.body, body_hash);
    }

    #[test]
    fn test_signed_memo_into_payload() {
        let key = create_test_key();
        let did = DidKey::from(&key);
        let body_hash = create_test_hash();
        let memo = Memo::new(did.clone(), body_hash.clone());

        let signed_memo = memo.sign(&key).unwrap();
        let payload = signed_memo.into_payload();

        assert_eq!(payload.iss, did);
        assert_eq!(payload.body, body_hash);
    }

    #[test]
    fn test_signed_memo_validate() {
        let key = create_test_key();
        let did = DidKey::from(&key);
        let body_hash = create_test_hash();
        let memo = Memo::new(did, body_hash.clone());

        let signed_memo = memo.sign(&key).unwrap();
        let validated_memo = signed_memo.validate(None).unwrap();

        assert_eq!(validated_memo.body, body_hash);
    }

    #[test]
    fn test_signed_memo_validate_expired() {
        let key = create_test_key();
        let did = DidKey::from(&key);
        let body_hash = create_test_hash();
        let mut memo = Memo::new(did, body_hash);

        memo.exp = Some(now() - 3600); // Expired
        let signed_memo = memo.sign(&key).unwrap();

        assert!(signed_memo.validate(None).is_err());
    }
}
