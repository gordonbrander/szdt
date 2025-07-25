use crate::did;
use crate::ed25519;
use crate::nickname;
use std::{collections::TryReserveError, convert::Infallible};
use thiserror::Error;

#[derive(Debug)]
pub struct TimestampComparison {
    pub timestamp: Option<u64>,
    pub now: Option<u64>,
}

impl TimestampComparison {
    pub fn new(timestamp: Option<u64>, now: Option<u64>) -> Self {
        TimestampComparison { timestamp, now }
    }
}

impl std::fmt::Display for TimestampComparison {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "(timestamp: {}, now: {} )",
            self.timestamp
                .map(|ts| ts.to_string())
                .unwrap_or("None".to_string()),
            self.now
                .map(|ts| ts.to_string())
                .unwrap_or("None".to_string())
        )
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("CBOR decoding error: {0}")]
    CborDecode(String),
    #[error("CBOR encoding error: {0}")]
    CborEncode(String),
    #[error("ed25519 error: {0}")]
    Ed25519(#[from] ed25519::Error),
    #[error("DID error: {0}")]
    Did(#[from] did::Error),
    #[error("BIP39 error: {0}")]
    Bip39(#[from] bip39::Error),
    #[error("Private key missing: {0}")]
    PrivateKeyMissing(String),
    #[error("Data integrity error: {0}")]
    IntegrityError(String),
    #[error("Memo issuer DID is missing")]
    MemoIssMissing,
    #[error("Memo is unsigned")]
    MemoUnsigned,
    #[error("Memo is too early (nbf time didn't validate): {0}")]
    MemoNbfError(TimestampComparison),
    #[error("Memo has expired (exp time didn't validate): {0}")]
    MemoExpError(TimestampComparison),
    #[error("Nickname error: {0}")]
    NicknameError(#[from] nickname::NicknameError),
    #[error("EOF")]
    Eof,
}

impl From<serde_cbor_core::DecodeError<std::io::Error>> for Error {
    fn from(err: serde_cbor_core::DecodeError<std::io::Error>) -> Self {
        Error::CborDecode(err.to_string())
    }
}

impl From<serde_cbor_core::DecodeError<Infallible>> for Error {
    fn from(err: serde_cbor_core::DecodeError<Infallible>) -> Self {
        Error::CborDecode(err.to_string())
    }
}

impl From<serde_cbor_core::EncodeError<TryReserveError>> for Error {
    fn from(err: serde_cbor_core::EncodeError<TryReserveError>) -> Self {
        Error::CborEncode(err.to_string())
    }
}

impl From<serde_cbor_core::EncodeError<std::io::Error>> for Error {
    fn from(err: serde_cbor_core::EncodeError<std::io::Error>) -> Self {
        Error::CborEncode(err.to_string())
    }
}
