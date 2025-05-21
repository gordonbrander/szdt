use crate::did;
use crate::ed25519::{self, Ed25519KeyMaterial};
use crate::{did::DidKey, util::now};
use cid::Cid;
use serde::{Deserialize, Serialize};
use std::collections::TryReserveError;
use thiserror::Error;

/// An SZDT Claim.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Claim {
    payload: Payload,
    signature: Vec<u8>,
}

impl Claim {
    pub fn new(payload: Payload, signature: Vec<u8>) -> Self {
        Self { payload, signature }
    }

    pub fn payload(&self) -> &Payload {
        &self.payload
    }

    pub fn signature(&self) -> &Vec<u8> {
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
        let key_material = Ed25519KeyMaterial::try_from_public_key(public_key)?;
        let payload_bytes = Vec::try_from(self.payload())?;
        let result = key_material.verify(&payload_bytes, &self.signature())?;
        return Ok(result);
    }

    /// Is claim expired?
    pub fn is_expired(&self, now_time: Option<u64>) -> bool {
        match self.payload.exp {
            Some(exp) => exp < now_time.unwrap_or_else(now),
            None => false,
        }
    }

    /// Is claim too early?
    pub fn is_too_early(&self, now_time: Option<u64>) -> bool {
        match self.payload.nbf {
            Some(nbf) => nbf > now_time.unwrap_or_else(now),
            None => false,
        }
    }
}

/// Build a claim
#[derive(Clone, Debug)]
pub struct Builder {
    /// Issuer (DID)
    key_material: Ed25519KeyMaterial,
    /// Issued at (UNIX timestamp, seconds)
    iat: u64,
    /// Not valid before (UNIX timestamp, seconds)
    nbf: Option<u64>,
    /// Expiration time (UNIX timestamp, seconds)
    exp: Option<u64>,
    /// Assertions
    ast: Vec<Assertion>,
}

impl Builder {
    /// Build a claim, starting with the bytes of your secret key.
    pub fn new(private_key: &[u8]) -> Result<Self, Error> {
        let key_material = Ed25519KeyMaterial::try_from_private_key(private_key)?;
        Ok(Self {
            key_material,
            iat: now(),
            nbf: Some(now()),
            exp: None,
            ast: Vec::new(),
        })
    }

    pub fn iat(mut self, iat: u64) -> Self {
        self.iat = iat;
        self
    }

    pub fn nbf(mut self, nbf: u64) -> Self {
        self.nbf = Some(nbf);
        self
    }

    pub fn exp(mut self, exp: u64) -> Self {
        self.exp = Some(exp);
        self
    }

    pub fn ast(mut self, ast: Vec<Assertion>) -> Self {
        self.ast = ast;
        self
    }

    pub fn add_ast(mut self, ast: Assertion) -> Self {
        self.ast.push(ast);
        self
    }

    /// Sign and return the claim
    pub fn sign(self) -> Result<Claim, Error> {
        let pubkey = self.key_material.public_key();
        let did = DidKey::new(&pubkey)?;
        let payload = Payload {
            iss: did,
            iat: self.iat,
            nbf: self.nbf,
            exp: self.exp,
            ast: self.ast,
        };
        let payload_bytes: Vec<u8> = (&payload).try_into()?;
        let signature = self.key_material.sign(&payload_bytes)?;
        Ok(Claim { payload, signature })
    }
}

/// A signed claim
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
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

impl Payload {
    /// Create a new payload
    pub fn new(iss: DidKey, iat: u64) -> Self {
        Self {
            iss,
            iat,
            nbf: None,
            exp: None,
            ast: Vec::new(),
        }
    }
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
    #[serde(rename = "witness")]
    Witness(WitnessAssertion),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WitnessAssertion {
    pub cid: Cid,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Invalid signature: {0}")]
    Ed25519(#[from] ed25519::Error),
    #[error("dag-cbor encode error: {0}")]
    CborEncodeError(#[from] serde_ipld_dagcbor::EncodeError<TryReserveError>),
    #[error("DID error: {0}")]
    DidError(#[from] did::Error),
    #[error("Claim is too early")]
    Nbf,
    #[error("Claim expired")]
    Exp,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_sign_and_validate() {
        // Generate a key pair for testing
        let (_, privkey) = ed25519::generate_keypair();

        // Build and sign a claim
        let claim = Builder::new(&privkey).unwrap().sign().unwrap();

        // Validate the claim
        claim.validate(None).unwrap();
    }
}
