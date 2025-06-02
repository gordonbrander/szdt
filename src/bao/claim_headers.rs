use crate::claim::Claim;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimHeaders {
    pub claims: Vec<Claim>,
}
