#[derive(Debug)]
pub enum Error {
    IoError(String, std::io::Error),
    SerializationError(String, serde_cbor::Error),
    Ed25519Error(String, ed25519_dalek::ed25519::Error),
    JsonError(serde_json::Error),
    DecodingError(String),
    ValidationError(String),
    SignatureVerificationError(String),
    ValueError(String),
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::IoError(_, err) => Some(err),
            Error::SerializationError(_, err) => Some(err),
            Error::Ed25519Error(_, err) => Some(err),
            Error::JsonError(err) => Some(err),
            Error::DecodingError(_) => None,
            Error::ValidationError(_) => None,
            Error::SignatureVerificationError(_) => None,
            Error::ValueError(_) => None,
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IoError(msg, _) => write!(f, "IO error: {}", msg),
            Error::SerializationError(msg, _) => write!(f, "Serialization error: {}", msg),
            Error::Ed25519Error(msg, _) => write!(f, "Ed25519 error: {}", msg),
            Error::JsonError(err) => write!(f, "JSON error: {}", err),
            Error::DecodingError(msg) => write!(f, "Decoding error: {}", msg),
            Error::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            Error::SignatureVerificationError(msg) => write!(f, "Signature error: {}", msg),
            Error::ValueError(msg) => write!(f, "Value error: {}", msg),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IoError(err.to_string(), err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::JsonError(err)
    }
}

impl From<serde_cbor::Error> for Error {
    fn from(err: serde_cbor::Error) -> Self {
        Error::SerializationError(err.to_string(), err)
    }
}

impl From<data_encoding::DecodeError> for Error {
    fn from(err: data_encoding::DecodeError) -> Self {
        Error::DecodingError(err.to_string())
    }
}

impl From<bs58::decode::Error> for Error {
    fn from(err: bs58::decode::Error) -> Self {
        Error::DecodingError(err.to_string())
    }
}

impl From<ed25519_dalek::ed25519::Error> for Error {
    fn from(err: ed25519_dalek::ed25519::Error) -> Self {
        Error::Ed25519Error(err.to_string(), err)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
