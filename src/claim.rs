use crate::ed25519::{self, SignatureBytes};
use crate::{did::DidKey, util::now_epoch_secs};
use cid::Cid;
use data_encoding::BASE64;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub struct Builder {
    payload: Payload,
}

impl Builder {
    pub fn new(payload: Payload) -> Self {
        Self { payload }
    }

    /// Sign payload.
    /// Returns a result for Claim.
    pub fn sign(self, secret_key: ed25519::SecretKey) -> Result<Claim, Error> {
        let header = Header::new_ed25519();
        let header_base64 = header.jwt_base64_encode()?;
        let payload_base64 = self.payload.jwt_base64_encode()?;
        // Construct the first portion of the JWt
        let data_to_sign = format!("{header_base64}.{payload_base64}")
            .as_bytes()
            .to_vec();
        let signature = ed25519::sign(&data_to_sign, &secret_key);
        Ok(Claim {
            header,
            payload: self.payload,
            signed_data: data_to_sign,
            signature,
        })
    }
}

/// An SZDT Claim. Claims are valid JWTs, and canonicalized to JWT form for signing.
/// Claim provides the structured data in a claim, but does not validate the claim
/// on construction. To validate the validity of the claim, you can call `claim.validate()`.
pub struct Claim {
    header: Header,
    payload: Payload,
    signed_data: Vec<u8>,
    signature: ed25519::SignatureBytes,
}

impl Claim {
    pub fn header(&self) -> &Header {
        &self.header
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
        let result = ed25519::verify(&self.signed_data, &self.signature, public_key)?;
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

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Header {
    pub alg: String,
    pub typ: String,
}

impl Header {
    pub fn new_ed25519() -> Self {
        Self {
            alg: "EdDSA".to_string(),
            typ: "JWT".to_string(),
        }
    }

    /// Encode as base64-encoded dag-json
    pub fn jwt_base64_encode(&self) -> Result<String, Error> {
        let dag_json = serde_ipld_dagjson::to_vec(self)?;
        Ok(BASE64.encode(&dag_json))
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
    pub assertions: Vec<Assertion>,
}

impl Payload {
    /// Encode as base64-encoded dag-json
    pub fn jwt_base64_encode(&self) -> Result<String, Error> {
        let dag_json = serde_ipld_dagjson::to_vec(self)?;
        Ok(BASE64.encode(&dag_json))
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
    #[error("Error encoding to dag-json: {0}")]
    DagJsonEncode(#[from] serde_ipld_dagjson::EncodeError),
}
