use crate::error::Error;
use serde::{de::DeserializeOwned, ser::Serialize};
use std::io::{BufRead, Write};

/// A specialized reader for deserializes SZDT archives.
/// SZDT archives are CBOR sequences with a particular shape.
pub struct CborSeqReader<R> {
    reader: R,
}

impl<R: BufRead> CborSeqReader<R> {
    pub fn new(reader: R) -> Self {
        Self { reader }
    }

    /// Deserialize next block
    pub fn read_block<T: DeserializeOwned>(&mut self) -> Result<T, Error> {
        let result: T = match serde_ipld_dagcbor::de::from_reader_once(&mut self.reader) {
            Ok(value) => value,
            Err(serde_ipld_dagcbor::DecodeError::Eof) => return Err(Error::Eof),
            Err(err) => return Err(Error::CborDecode(err.to_string())),
        };
        Ok(result)
    }

    /// Unwrap inner reader
    pub fn into_inner(self) -> R {
        self.reader
    }
}

/// Represents the metadata portion of an SZDT archive
pub struct CborSeqWriter<W> {
    writer: W,
}

impl<W: Write> CborSeqWriter<W> {
    pub fn new(writer: W) -> Self {
        Self { writer }
    }

    /// Serialize next block
    pub fn write_block<T: Serialize>(&mut self, block: &T) -> Result<(), Error> {
        serde_ipld_dagcbor::ser::to_writer(&mut self.writer, block)?;
        Ok(())
    }

    /// Unwrap inner writer
    pub fn into_inner(self) -> W {
        self.writer
    }

    pub fn flush(&mut self) -> Result<(), Error> {
        self.writer.flush()?;
        Ok(())
    }
}

impl<W: Write> Write for CborSeqWriter<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.writer.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}
