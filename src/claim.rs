use crate::ed25519::{self, SignatureBytes};
use crate::{did::DidKey, util::now_epoch_secs};
use cid::Cid;
use serde::{Deserialize, Serialize};
use std::collections::TryReserveError;
use thiserror::Error;

/// An SZDT Claim.
pub struct Claim {
    payload: Payload,
    signature: ed25519::SignatureBytes,
}

impl Claim {
    pub fn new(payload: Payload, signature: ed25519::SignatureBytes) -> Self {
        Self { payload, signature }
    }

    pub fn sign(payload: Payload, secret_key: &ed25519::SecretKey) -> Result<Self, Error> {
        let payload_bytes: Vec<u8> = (&payload).try_into()?;
        let signature = ed25519::sign(&payload_bytes, &secret_key);
        Ok(Self::new(payload, signature))
    }

    pub fn payload(&self) -> &Payload {
        &self.payload
    }

    pub fn signature(&self) -> &SignatureBytes {
        &self.signature
    }

    /// Is claim valid? Checks signature and other properties, such as exp.
    pub fn validate(&self, now_time: Option<u64>) -> Result<(), Error> {
        if self.is_too_early(now_time) {
            return Err(Error::Nbf);
        }
        if self.is_expired(now_time) {
            return Err(Error::Exp);
        }
        self.check_signature()?;
        return Ok(());
    }

    /// Check if signature is valid for claim
    pub fn check_signature(&self) -> Result<(), Error> {
        let public_key = self.payload.iss.pubkey();
        let payload_bytes = self.payload().try_into()?;
        let result = ed25519::verify(&payload_bytes, &self.signature, public_key)?;
        return Ok(result);
    }

    /// Is claim expired?
    pub fn is_expired(&self, now_time: Option<u64>) -> bool {
        match self.payload.exp {
            Some(exp) => exp < now_time.unwrap_or_else(now_epoch_secs),
            None => false,
        }
    }

    /// Is claim too early?
    pub fn is_too_early(&self, now_time: Option<u64>) -> bool {
        match self.payload.nbf {
            Some(nbf) => nbf > now_time.unwrap_or_else(now_epoch_secs),
            None => false,
        }
    }
}

/// A signed claim
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Payload {
    /// Issuer (DID)
    pub iss: DidKey,
    /// Issued at (UNIX timestamp, seconds)
    pub iat: u64,
    /// Not valid before (UNIX timestamp, seconds)
    pub nbf: Option<u64>,
    /// Expiration time (UNIX timestamp, seconds)
    pub exp: Option<u64>,
    /// Assertions
    pub ast: Vec<Assertion>,
}

impl TryFrom<&Payload> for Vec<u8> {
    type Error = Error;

    fn try_from(value: &Payload) -> Result<Self, Self::Error> {
        let bytes = serde_ipld_dagcbor::to_vec(value)?;
        Ok(bytes)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum Assertion {
    #[serde(rename = "authority")]
    Authority(AuthorityAssertion),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AuthorityAssertion {
    pub cid: Cid,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Claim is too early")]
    Nbf,
    #[error("Claim expired")]
    Exp,
    #[error("Invalid signature: {0}")]
    Ed25519(#[from] ed25519::Error),
    #[error("dag-cbor encode error: {0}")]
    CborEncodeError(#[from] serde_ipld_dagcbor::EncodeError<TryReserveError>),
}
