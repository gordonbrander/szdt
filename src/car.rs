use crate::byte_counter_reader::ByteCounterReader;
use crate::cid::{read_into_cid_v1_cbor, read_into_cid_v1_raw};
use crate::multihash::{self, read_into_multihash};
use crate::varint::{self, read_varint_usize, write_usize_varint};
use cid::Cid;
use serde::{Deserialize, Serialize, de, de::DeserializeOwned, ser};
use serde_ipld_dagcbor;
use std::io::{self, Read, Write};
use thiserror::Error;

pub struct CarReader<R: Read, H> {
    header: H,
    reader: R,
}

impl<R: Read, H> CarReader<R, H> {
    pub fn header(&self) -> &H {
        &self.header
    }
}

impl<R: Read, H: de::DeserializeOwned> CarReader<R, H> {
    /// Read bytes into a Car file.
    pub fn read_from(mut reader: R) -> Result<Self, Error> {
        let header: H = read_header(&mut reader)?;
        return Ok(Self { header, reader });
    }
}

impl<R: Read, H: DeserializeOwned> Iterator for CarReader<R, H> {
    type Item = Result<CarBlock, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        // Try to read the next block
        match CarBlock::read_from(&mut self.reader) {
            Ok(block) => Some(Ok(block)),
            Err(Error::Io(e)) if e.kind() == io::ErrorKind::UnexpectedEof => None,
            Err(e) => Some(Err(e)),
        }
    }
}

pub struct CarWriter<W: Write> {
    writer: W,
}

impl<W: Write> CarWriter<W> {
    /// Create a new `CarWriter` instance, writing the header to the writer.
    pub fn new<H: ser::Serialize>(mut writer: W, header: H) -> Result<Self, Error> {
        // Immediately write the header to the writer
        write_header(&mut writer, &header)?;
        Ok(CarWriter { writer })
    }

    pub fn inner(&mut self) -> &mut W {
        &mut self.writer
    }

    pub fn write_block(&mut self, block: &CarBlock) -> Result<usize, Error> {
        block.write_into(&mut self.writer)
    }
}

/// Read the header portion of a CAR file.
/// This function consumes the header bytes of the CAR file from the reader.
/// Reader may be subsequently passed to functions which read the body blocks of the CAR file.
fn read_header<R: io::Read, T: DeserializeOwned>(reader: &mut R) -> Result<T, Error> {
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
fn write_header<W: io::Write, T: Serialize>(writer: &mut W, header: &T) -> Result<usize, Error> {
    let header_cbor =
        serde_ipld_dagcbor::to_vec(header).map_err(|e| Error::Serialization(e.to_string()))?;
    let written = varint::write_usize_varint(writer, header_cbor.len())?;
    writer.write_all(&header_cbor)?;
    Ok(written + header_cbor.len())
}

/// The CAR header of an SZDT archive.
/// In addition to `version` and `roots`, this header also includes metadata
/// related to the SZDT archive.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CarHeader<T> {
    version: u64,
    pub roots: Vec<Cid>,
    #[serde(flatten)]
    pub meta: T,
}

impl<T> CarHeader<T> {
    /// Construct a new CarHeader
    pub fn new_v1(roots: Vec<Cid>, meta: T) -> Self {
        CarHeader {
            version: 1,
            roots,
            meta,
        }
    }
}

/// A single block of data in a CAR file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CarBlock {
    cid: Cid,
    body: Vec<u8>,
}

impl CarBlock {
    /// Construct a new CarBlock
    /// This method cryptographically verifies the contents of the block by
    /// reconstructing the CID from the data and comparing it to the provided CID.
    pub fn new(cid: Cid, body: Vec<u8>) -> Result<Self, Error> {
        let mut reader = body.as_slice();
        let hash = read_into_multihash(&mut reader)?;
        let actual_cid = Cid::new_v1(cid.codec(), hash);
        if actual_cid != cid {
            return Err(Error::InvalidBlock(format!(
                "CID doesn't match.\n\tExpected: {}\n\tActual: {}",
                cid, actual_cid
            )));
        }
        Ok(CarBlock { cid, body })
    }

    /// Constructs a new CarBlock from raw data
    pub fn from_raw(body: Vec<u8>) -> Self {
        let cid = read_into_cid_v1_raw(&mut body.as_slice()).expect("Should be able to read vec");
        CarBlock { cid, body }
    }

    /// Serializes value as dcbor42, and creates a new CarBlock with dcbor42 CID
    pub fn from_serializable<T: Serialize>(value: &T) -> Result<Self, Error> {
        let body =
            serde_ipld_dagcbor::to_vec(value).map_err(|e| Error::Serialization(e.to_string()))?;
        let cid = read_into_cid_v1_cbor(&mut body.as_slice())?;
        Ok(CarBlock { cid, body })
    }

    /// Read a single body block from a CAR file
    pub fn read_from<R: io::Read>(reader: &mut R) -> Result<Self, Error> {
        // Read size
        let block_size = read_varint_usize(reader)?;
        // Wrap reader in byte counter reader
        let mut read_counter = ByteCounterReader::new(reader);
        // Read the cid
        let cid = Cid::read_bytes(&mut read_counter)?;
        // Get the number of bytes read while reading the cid
        let read_size = read_counter.read_size();
        // Allocate memory for the data (the block length minus the CID length)
        let mut data = vec![0; block_size - read_size];
        // Read data portion
        read_counter.read_exact(&mut data)?;
        Ok(Self::new(cid, data)?)
    }

    /// Write a single body block to a writer
    pub fn write_into<W: io::Write>(&self, writer: &mut W) -> Result<usize, Error> {
        let cid_bytes = &self.cid.to_bytes();
        let total_len = cid_bytes.len() + self.body.len();
        // Write the length of the CID and data
        let written = write_usize_varint(writer, total_len)?;
        // Write CID
        writer.write_all(&cid_bytes)?;
        writer.write_all(&self.body)?;
        Ok(written + total_len)
    }

    /// Get the CID of the block
    pub fn cid(&self) -> &Cid {
        &self.cid
    }

    /// Get the data of the block
    pub fn body(&self) -> &Vec<u8> {
        &self.body
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Error decoding unsigned-varint: {0}")]
    UnsignedVarIntDecode(unsigned_varint::decode::Error),
    #[error("CID error: {0}")]
    Cid(cid::Error),
    #[error("Multihash error: {0}")]
    Multihash(multihash::Error),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Invalid block: {0}")]
    InvalidBlock(String),
    #[error("Other error: {0}")]
    Other(String),
}

impl From<crate::cid::Error> for Error {
    fn from(err: crate::cid::Error) -> Self {
        match err {
            crate::cid::Error::Io(err) => Self::Io(err),
            crate::cid::Error::Multihash(err) => Self::Multihash(err),
        }
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

impl From<cid::Error> for Error {
    fn from(err: cid::Error) -> Self {
        match err {
            cid::Error::Io(err) => Self::Io(err),
            _ => Self::Cid(err),
        }
    }
}

impl From<multihash::Error> for Error {
    fn from(err: multihash::Error) -> Self {
        match err {
            multihash::Error::Io(err) => Self::Io(err),
            _ => Self::Multihash(err),
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
        roots: Vec<Cid>,
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
        write_header(&mut input, &header).unwrap();

        // Read the header back
        let mut reader = Cursor::new(input);
        let header2: TestHeader = read_header(&mut reader).unwrap();

        // Verify the result
        assert_eq!(header, header2);
    }

    #[test]
    fn test_read_header_reading_empty_buffer_is_error() {
        // Create an empty reader to simulate IO error
        let mut reader = Cursor::new(Vec::<u8>::new());
        let result: Result<TestHeader, Error> = read_header(&mut reader);
        assert!(result.is_err());
    }
}
