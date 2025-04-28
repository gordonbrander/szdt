use crate::multiformats::MULTIHASH_SHA256;
use multihash::Multihash;
use sha2::{Digest, Sha256};
use std::io::{self, Read};

pub type Sha256Digest = [u8; 32];
pub type Multihash64 = Multihash<64>;

/// Streaming read and hash bytes into a SHA-256 hash.
pub fn read_into_sha256<R: Read>(reader: &mut R) -> Result<Sha256Digest, Error> {
    let mut hasher = Sha256::new();
    io::copy(reader, &mut hasher)?;
    let hash = hasher.finalize();
    try_into_hash(hash.as_slice())
}

/// Streaming read and hash bytes into a SHA-256 multihash.
pub fn read_into_multihash<R: Read>(reader: &mut R) -> Result<Multihash64, Error> {
    let hash = read_into_sha256(reader)?;
    into_multihash(&hash)
}

/// Converts a Slice<u8> to a fixed-size array of 32 bytes.
pub fn try_into_hash(bytes: &[u8]) -> Result<Sha256Digest, Error> {
    let bytes_32 = bytes
        .try_into()
        .map_err(|_| Error::ValueError("Failed to convert slice into 32-byte array".to_string()))?;
    Ok(bytes_32)
}

pub fn into_multihash(hash: &[u8]) -> Result<Multihash64, Error> {
    let multihash = Multihash::<64>::wrap(MULTIHASH_SHA256, hash)?;
    Ok(multihash)
}

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Multihash(multihash::Error),
    ValueError(String),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Io(err) => write!(f, "I/O error: {}", err),
            Error::Multihash(err) => write!(f, "Multihash error: {}", err),
            Error::ValueError(msg) => write!(f, "Value error: {}", msg),
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<multihash::Error> for Error {
    fn from(err: multihash::Error) -> Self {
        Error::Multihash(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_into_hash() {
        let data = [0u8; 32];
        let result = try_into_hash(&data);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), data);

        let data_short = [0u8; 31];
        let result = try_into_hash(&data_short);
        assert!(result.is_err());

        let data_long = [0u8; 33];
        let result = try_into_hash(&data_long);
        assert!(result.is_err());
    }

    #[test]
    fn test_into_multihash() {
        let data = [0u8; 32];
        let result = into_multihash(&data);
        assert!(result.is_ok());

        let multihash = result.unwrap();
        assert_eq!(multihash.code(), MULTIHASH_SHA256);
        assert_eq!(multihash.digest(), &data);
    }

    #[test]
    fn test_read_into_sha256() {
        let data = b"hello world";
        let result = read_into_sha256(&mut data.as_slice());
        assert!(result.is_ok());
    }

    #[test]
    fn test_read_into_multihash() {
        let data = b"hello world";
        let mut cursor = std::io::Cursor::new(data);

        let result = read_into_multihash(&mut cursor);
        assert!(result.is_ok());

        let multihash = result.unwrap();
        assert_eq!(multihash.code(), MULTIHASH_SHA256);
    }
}
