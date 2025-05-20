use crate::claim::Claim;
use cid::Cid;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CarClaimHeader {
    version: u64,
    pub roots: Vec<Cid>,
    pub claims: Vec<Claim>,
}

impl CarClaimHeader {
    pub fn new(roots: Vec<Cid>, claims: Vec<Claim>) -> Self {
        CarClaimHeader {
            version: 1,
            roots,
            claims,
        }
    }
}
