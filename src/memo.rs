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
