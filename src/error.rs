#[derive(Debug)]
pub struct Error {
    pub msg: String,
    pub kind: ErrorKind,
}

impl Error {
    pub fn new<S: Into<String>>(msg: S, kind: ErrorKind) -> Self {
        Error {
            msg: msg.into(),
            kind,
        }
    }
}

#[derive(Debug)]
pub enum ErrorKind {
    IoError(std::io::Error),
    SerializationError(serde_cbor::Error),
    Ed25519Error(ed25519_dalek::ed25519::Error),
    DecodingError,
    ValidationError,
    SignatureError,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {:?}", self.msg, self.kind)
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error {
            msg: error.to_string(),
            kind: ErrorKind::IoError(error),
        }
    }
}

impl From<serde_cbor::Error> for Error {
    fn from(error: serde_cbor::Error) -> Self {
        Error {
            msg: error.to_string(),
            kind: ErrorKind::SerializationError(error),
        }
    }
}

impl From<data_encoding::DecodeError> for Error {
    fn from(error: data_encoding::DecodeError) -> Self {
        Error {
            msg: error.to_string(),
            kind: ErrorKind::DecodingError,
        }
    }
}

impl From<ed25519_dalek::ed25519::Error> for Error {
    fn from(error: ed25519_dalek::ed25519::Error) -> Self {
        Error {
            msg: error.to_string(),
            kind: ErrorKind::Ed25519Error(error),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
