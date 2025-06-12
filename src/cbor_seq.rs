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

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use std::io::Cursor;

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestData {
        id: u32,
        name: String,
        active: bool,
    }

    #[test]
    fn test_write_and_read_single_block() {
        let test_data = TestData {
            id: 42,
            name: "test".to_string(),
            active: true,
        };

        // Write data
        let mut buffer = Vec::new();
        let mut writer = CborSeqWriter::new(&mut buffer);
        writer.write_block(&test_data).unwrap();
        writer.flush().unwrap();

        // Read data back
        let cursor = Cursor::new(buffer);
        let mut reader = CborSeqReader::new(cursor);
        let result: TestData = reader.read_block().unwrap();

        assert_eq!(test_data, result);
    }

    #[test]
    fn test_write_and_read_multiple_blocks() {
        let test_data1 = TestData {
            id: 1,
            name: "first".to_string(),
            active: true,
        };
        let test_data2 = TestData {
            id: 2,
            name: "second".to_string(),
            active: false,
        };

        // Write multiple blocks
        let mut buffer = Vec::new();
        let mut writer = CborSeqWriter::new(&mut buffer);
        writer.write_block(&test_data1).unwrap();
        writer.write_block(&test_data2).unwrap();
        writer.flush().unwrap();

        // Read blocks back
        let cursor = Cursor::new(buffer);
        let mut reader = CborSeqReader::new(cursor);

        let result1: TestData = reader.read_block().unwrap();
        let result2: TestData = reader.read_block().unwrap();

        assert_eq!(test_data1, result1);
        assert_eq!(test_data2, result2);
    }

    #[test]
    fn test_read_eof_error() {
        let buffer = Vec::new();
        let cursor = Cursor::new(buffer);
        let mut reader = CborSeqReader::new(cursor);

        let result: Result<TestData, Error> = reader.read_block();
        assert!(matches!(result, Err(Error::Eof)));
    }

    #[test]
    fn test_read_eof_after_data() {
        let test_data = TestData {
            id: 123,
            name: "last".to_string(),
            active: false,
        };

        // Write one block
        let mut buffer = Vec::new();
        let mut writer = CborSeqWriter::new(&mut buffer);
        writer.write_block(&test_data).unwrap();
        writer.flush().unwrap();

        // Read the block and then try to read another
        let cursor = Cursor::new(buffer);
        let mut reader = CborSeqReader::new(cursor);

        let result1: TestData = reader.read_block().unwrap();
        assert_eq!(test_data, result1);

        let result2: Result<TestData, Error> = reader.read_block();
        assert!(matches!(result2, Err(Error::Eof)));
    }

    #[test]
    fn test_writer_into_inner() {
        let buffer = Vec::new();
        let writer = CborSeqWriter::new(buffer);
        let inner = writer.into_inner();
        assert_eq!(inner.len(), 0);
    }

    #[test]
    fn test_reader_into_inner() {
        let buffer = vec![1, 2, 3, 4];
        let cursor = Cursor::new(buffer.clone());
        let reader = CborSeqReader::new(cursor);
        let inner = reader.into_inner();
        assert_eq!(inner.into_inner(), buffer);
    }

    #[test]
    fn test_writer_write_trait() {
        let mut buffer = Vec::new();
        let mut writer = CborSeqWriter::new(&mut buffer);

        let data = b"hello world";
        let bytes_written = writer.write(data).unwrap();
        writer.flush().unwrap();

        assert_eq!(bytes_written, data.len());
        assert_eq!(buffer, data);
    }
}
