use crate::byte_counter_reader::ByteCounterReader;
use crate::dasl_cid::{self, DaslCid};
use crate::varint::{self, read_varint_usize};
use serde::de;
use serde_ipld_dagcbor;
use std::io::{self, Read};

/// Read the header portion of a CAR file.
/// This function consumes the header bytes of the CAR file from the reader.
/// Reader may be subsequently passed to functions which read the body blocks of the CAR file.
pub fn read_header<R: io::Read, T: de::DeserializeOwned>(reader: &mut R) -> Result<T, Error> {
    let header_length = read_varint_usize(reader)?;
    // Create a `header_length` buffer and read bytes from the header block
    let mut header_buffer = vec![0; header_length];
    reader.read_exact(&mut header_buffer)?;
    let header: T = serde_ipld_dagcbor::from_slice(&header_buffer).unwrap();
    Ok(header)
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CarBlock {
    pub cid: DaslCid,
    pub data: Vec<u8>,
}

impl CarBlock {
    pub fn new(cid: DaslCid, data: Vec<u8>) -> Self {
        CarBlock { cid, data }
    }

    /// Read a single body block from a CAR file
    pub fn read<R: io::Read>(reader: &mut R) -> Result<Self, Error> {
        // Read size
        let block_size = read_varint_usize(reader)?;
        // Wrap reader in byte counter reader
        let mut read_counter = ByteCounterReader::new(reader);
        // Read the cid
        let cid = DaslCid::read_binary_cid(&mut read_counter)?;
        // Get the number of bytes read while reading the cid
        let read_size = read_counter.read_size();
        // Allocate memory for the data (the block length minus the CID length)
        let mut data = vec![0; block_size - read_size];
        // Read data portion
        read_counter.read_exact(&mut data)?;
        Ok(Self { cid, data })
    }
}

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    UnsignedVarIntDecode(unsigned_varint::decode::Error),
    Cid(dasl_cid::Error),
    Other(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Io(err) => write!(f, "IO error: {}", err),
            Error::UnsignedVarIntDecode(err) => {
                write!(f, "UnsignedVarIntDecodeError: {}", err)
            }
            Error::Cid(err) => write!(f, "CID error: {}", err),
            Error::Other(msg) => write!(f, "Other error: {}", msg),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(err) => Some(err),
            Error::UnsignedVarIntDecode(err) => Some(err),
            Error::Cid(err) => Some(err),
            Error::Other(_) => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<varint::Error> for Error {
    fn from(err: varint::Error) -> Self {
        match err {
            varint::Error::Io(err) => Error::Io(err),
            varint::Error::UnsignedVarIntDecode(err) => Error::UnsignedVarIntDecode(err),
            varint::Error::Other(msg) => Error::Other(msg),
        }
    }
}

impl From<dasl_cid::Error> for Error {
    fn from(err: dasl_cid::Error) -> Self {
        match err {
            dasl_cid::Error::Io(err) => Error::Io(err),
            dasl_cid::Error::UnsignedVarIntDecode(err) => Error::UnsignedVarIntDecode(err),
            _ => Error::Cid(err),
        }
    }
}
