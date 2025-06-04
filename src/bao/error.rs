use std::{collections::TryReserveError, convert::Infallible};
use thiserror::Error;

use crate::claim;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Decoding error: {0}")]
    CborDecode(String),
    #[error("Encoding error: {0}")]
    CborEncode(String),
    #[error("Claim error: {0}")]
    Claim(#[from] claim::Error),
    #[error("Value error")]
    Value(String),
    #[error("Archive integrity error: {0}")]
    IntegrityError(String),
    #[error("EOF")]
    Eof,
}

impl From<serde_ipld_dagcbor::DecodeError<std::io::Error>> for Error {
    fn from(err: serde_ipld_dagcbor::DecodeError<std::io::Error>) -> Self {
        Error::CborDecode(err.to_string())
    }
}

impl From<serde_ipld_dagcbor::DecodeError<Infallible>> for Error {
    fn from(err: serde_ipld_dagcbor::DecodeError<Infallible>) -> Self {
        Error::CborDecode(err.to_string())
    }
}

impl From<serde_ipld_dagcbor::EncodeError<TryReserveError>> for Error {
    fn from(err: serde_ipld_dagcbor::EncodeError<TryReserveError>) -> Self {
        Error::CborEncode(err.to_string())
    }
}

impl From<serde_ipld_dagcbor::EncodeError<std::io::Error>> for Error {
    fn from(err: serde_ipld_dagcbor::EncodeError<std::io::Error>) -> Self {
        Error::CborEncode(err.to_string())
    }
}
