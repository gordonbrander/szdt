#[derive(Debug)]
pub enum Error {
    IoError(String, std::io::Error),
    SerializationError(String, serde_cbor::Error),
    Ed25519Error(String, ed25519_dalek::ed25519::Error),
    DecodingError(String),
    ValidationError(String),
    SignatureError(String),
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::IoError(_, err) => Some(err),
            Error::SerializationError(_, err) => Some(err),
            Error::Ed25519Error(_, err) => Some(err),
            Error::DecodingError(_) => None,
            Error::ValidationError(_) => None,
            Error::SignatureError(_) => None,
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IoError(msg, _) => write!(f, "IO error: {}", msg),
            Error::SerializationError(msg, _) => write!(f, "Serialization error: {}", msg),
            Error::Ed25519Error(msg, _) => write!(f, "Ed25519 error: {}", msg),
            Error::DecodingError(msg) => write!(f, "Decoding error: {}", msg),
            Error::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            Error::SignatureError(msg) => write!(f, "Signature error: {}", msg),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IoError(err.to_string(), err)
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

impl From<ed25519_dalek::ed25519::Error> for Error {
    fn from(err: ed25519_dalek::ed25519::Error) -> Self {
        Error::Ed25519Error(err.to_string(), err)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
