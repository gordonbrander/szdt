use crate::claim::Claim;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClaimHeaders {
    pub claims: Vec<Claim>,
}

impl Default for ClaimHeaders {
    fn default() -> Self {
        Self { claims: Vec::new() }
    }
}
