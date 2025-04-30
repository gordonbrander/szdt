use crate::did::DidKey;
use cid::Cid;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Jwt {
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "atr")]
pub enum Assertion {
    #[serde(rename = "authority")]
    Authority(AuthorityAssertion),
    #[serde(rename = "comment")]
    Comment(CommentAssertion),
    #[serde(rename = "dn")]
    Dn(DnAssertion),
    #[serde(rename = "petname")]
    Petname(PetnameAssertion),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AuthorityAssertion {
    pub ent: Cid,
    pub val: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DnAssertion {
    pub ent: Cid,
    pub val: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CommentAssertion {
    pub ent: Cid,
    pub val: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PetnameAssertion {
    pub ent: Cid,
    pub val: String,
}
