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
    let header: T = serde_ipld_dagcbor::from_slice(&header_buffer)
        .map_err(|e| Error::Serialization(e.to_string()))?;
    Ok(header)
}

/// Write the header portion of a CAR file.
/// This function writes the header bytes of the CAR file to the writer.
/// Writer may be subsequently passed to functions which write the body blocks of the CAR file.
/// Returns the number of bytes written, including the varint length.
pub fn write_header<W: io::Write, T: serde::Serialize>(
    writer: &mut W,
    header: &T,
) -> Result<usize, Error> {
    let header_cbor =
        serde_ipld_dagcbor::to_vec(header).map_err(|e| Error::Serialization(e.to_string()))?;
    let written = varint::write_usize_varint(writer, header_cbor.len())?;
    writer.write_all(&header_cbor)?;
    Ok(written + header_cbor.len())
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CarBlock {
    pub cid: DaslCid,
    pub data: Vec<u8>,
}

impl CarBlock {
    /// Construct a new CarBlock
    /// This method cryptographically verifies the contents of the block by
    /// reconstructing the CID from the data and comparing it to the provided CID.
    pub fn new(cid: DaslCid, data: Vec<u8>) -> Result<Self, Error> {
        let actual_cid = DaslCid::hash(&mut data.as_slice(), cid.codec())?;
        if actual_cid != cid {
            return Err(Error::InvalidBlock(format!(
                "CID doesn't match.\n\tExpected: {}\n\tActual: {}",
                cid, actual_cid
            )));
        }
        Ok(CarBlock { cid, data })
    }

    /// Read a single body block from a CAR file
    pub fn read<R: io::Read>(reader: &mut R) -> Result<Self, Error> {
        // Read size
        let block_size = read_varint_usize(reader)?;
        // Wrap reader in byte counter reader
        let mut read_counter = ByteCounterReader::new(reader);
        // Read the cid
        let cid = DaslCid::read_cid(&mut read_counter)?;
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
    Serialization(String),
    InvalidBlock(String),
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
            Error::Serialization(err) => write!(f, "Serialization error: {}", err),
            Error::InvalidBlock(msg) => write!(f, "Invalid block: {}", msg),
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
            Error::Serialization(_) => None,
            Error::InvalidBlock(_) => None,
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use std::io::Cursor;

    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
    struct TestHeader {
        version: u64,
        roots: Vec<DaslCid>,
    }

    #[test]
    fn test_write_header_read_header_roundtrip() {
        // Create a test header in CBOR format
        // For simplicity, we're creating a CAR v1 header with an empty roots array
        let header = TestHeader {
            version: 1,
            roots: vec![],
        };

        // Prepare the full input with a varint for the header length followed by the header
        let mut input = Vec::new();
        let written = write_header(&mut input, &header).unwrap();

        // Read the header back
        let mut reader = Cursor::new(input);
        let header2: TestHeader = read_header(&mut reader).unwrap();

        // Verify the result
        assert_eq!(header, header2);

        // Verify reader position is at the end of the header
        assert_eq!(reader.position() as usize, written);
    }

    #[test]
    fn test_read_header_reading_empty_buffer_is_error() {
        // Create an empty reader to simulate IO error
        let mut reader = Cursor::new(Vec::<u8>::new());
        let result: Result<TestHeader, Error> = read_header(&mut reader);
        assert!(result.is_err());
    }
}
