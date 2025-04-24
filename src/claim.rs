use crate::{dasl_cid::DaslCid, did::DidKey};
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
    pub ent: DaslCid,
    pub val: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DnAssertion {
    pub ent: DaslCid,
    pub val: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CommentAssertion {
    pub ent: DaslCid,
    pub val: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PetnameAssertion {
    pub ent: DaslCid,
    pub val: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_authority_assertion_serializes_to_dag_json() {
        let cid = DaslCid::try_from("bafkreifzjut3te2nhyekklss27nh3k72ysco7y32koao5eei66wof36n5e")
            .unwrap();
        let timestamp = 1234567890;

        let assertion = Assertion::Authority(AuthorityAssertion {
            ent: cid.clone(),
            val: timestamp,
        });

        let serialized = serde_json::to_string(&assertion).unwrap();
        let json_value: serde_json::Value = serde_json::from_str(&serialized).unwrap();

        // Check that we have the expected fields
        assert!(json_value.is_object());
        assert_eq!(json_value["atr"], "authority");
        assert_eq!(json_value["ent"], cid.to_string());
        assert_eq!(json_value["val"], timestamp);
    }
}
