use std::{collections::TryReserveError, convert::Infallible};
use thiserror::Error;

use crate::claim;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Decoding error: {0}")]
    CborDecode(#[from] serde_ipld_dagcbor::DecodeError<Infallible>),
    #[error("Encoding error: {0}")]
    CborEncode(#[from] serde_ipld_dagcbor::EncodeError<TryReserveError>),
    #[error("Claim error: {0}")]
    Claim(#[from] claim::Error),
    #[error("Value error")]
    Value(String),
    #[error("Manifest file entry missing for block")]
    ManifestFileEntryMissing(String),
}
