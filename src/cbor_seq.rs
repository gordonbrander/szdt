use crate::error::Error;
use serde::{de::DeserializeOwned, ser::Serialize};
use std::io::{BufRead, Write};
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncBufRead, AsyncRead, AsyncReadExt, ReadBuf};

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

/// Async version of CborSeqReader with internal buffering
pub struct AsyncCborSeqReader<R> {
    reader: R,
    buffer: Vec<u8>,
}

impl<R: AsyncBufRead + Unpin> AsyncCborSeqReader<R> {
    /// Create a new async CBOR sequence reader
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            buffer: Vec::new(),
        }
    }

    /// Read the next CBOR block asynchronously
    pub async fn read_block_async<T: DeserializeOwned>(&mut self) -> Result<T, Error> {
        loop {
            // First, try to decode from existing buffer
            if !self.buffer.is_empty() {
                let mut cursor = std::io::Cursor::new(&self.buffer);
                match serde_ipld_dagcbor::de::from_reader_once(&mut cursor) {
                    Ok(value) => {
                        // Successfully decoded one object
                        let consumed = cursor.position() as usize;
                        // Remove consumed data from buffer
                        self.buffer.drain(0..consumed);
                        return Ok(value);
                    }
                    Err(serde_ipld_dagcbor::DecodeError::Eof) => {
                        // Need more data, continue to read loop
                    }
                    Err(_err) => {
                        // Other decode error - try reading more data
                    }
                }
            }

            // Read more data into buffer
            let mut temp_buf = [0u8; 1024];
            match self.reader.read(&mut temp_buf).await {
                Ok(0) => {
                    // EOF - try one final decode attempt
                    if self.buffer.is_empty() {
                        return Err(Error::Eof);
                    }

                    let mut cursor = std::io::Cursor::new(&self.buffer);
                    match serde_ipld_dagcbor::de::from_reader_once(&mut cursor) {
                        Ok(value) => {
                            let consumed = cursor.position() as usize;
                            self.buffer.drain(0..consumed);
                            return Ok(value);
                        }
                        Err(serde_ipld_dagcbor::DecodeError::Eof) => return Err(Error::Eof),
                        Err(err) => return Err(Error::CborDecode(err.to_string())),
                    }
                }
                Ok(n) => {
                    self.buffer.extend_from_slice(&temp_buf[..n]);
                    // Continue loop to try decoding again
                }
                Err(err) => return Err(Error::Io(err)),
            }
        }
    }

    /// Get a reference to the inner reader
    pub fn get_ref(&self) -> &R {
        &self.reader
    }

    /// Get a mutable reference to the inner reader
    pub fn get_mut(&mut self) -> &mut R {
        &mut self.reader
    }

    /// Consume and return the inner reader
    pub fn into_inner(self) -> R {
        self.reader
    }
}

impl<R: AsyncRead + Unpin> AsyncRead for AsyncCborSeqReader<R> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.reader).poll_read(cx, buf)
    }
}

impl<R: AsyncRead + Unpin> AsyncRead for CborSeqReader<R> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.reader).poll_read(cx, buf)
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

    #[tokio::test]
    async fn test_async_read_single_block() {
        let test_data = TestData {
            id: 42,
            name: "async_test".to_string(),
            active: true,
        };

        // Write data synchronously first
        let mut buffer = Vec::new();
        let mut writer = CborSeqWriter::new(&mut buffer);
        writer.write_block(&test_data).unwrap();
        writer.flush().unwrap();

        // Create an async reader from the buffer
        let async_cursor = tokio::io::BufReader::new(&buffer[..]);
        let mut reader = AsyncCborSeqReader::new(async_cursor);

        let result: TestData = reader.read_block_async().await.unwrap();
        assert_eq!(test_data, result);
    }

    #[tokio::test]
    async fn test_async_read_multiple_blocks() {
        let test_data1 = TestData {
            id: 1,
            name: "first_async".to_string(),
            active: true,
        };
        let test_data2 = TestData {
            id: 2,
            name: "second_async".to_string(),
            active: false,
        };

        // Write multiple blocks
        let mut buffer = Vec::new();
        let mut writer = CborSeqWriter::new(&mut buffer);
        writer.write_block(&test_data1).unwrap();
        writer.write_block(&test_data2).unwrap();
        writer.flush().unwrap();

        // Create an async reader from the buffer
        let async_cursor = tokio::io::BufReader::new(&buffer[..]);
        let mut reader = AsyncCborSeqReader::new(async_cursor);

        let result1: TestData = reader.read_block_async().await.unwrap();
        let result2: TestData = reader.read_block_async().await.unwrap();

        assert_eq!(test_data1, result1);
        assert_eq!(test_data2, result2);
    }

    #[tokio::test]
    async fn test_async_read_eof_error() {
        let buffer: &[u8] = &[];
        let async_cursor = tokio::io::BufReader::new(buffer);
        let mut reader = AsyncCborSeqReader::new(async_cursor);

        let result: Result<TestData, Error> = reader.read_block_async().await;
        assert!(matches!(result, Err(Error::Eof)));
    }

    #[tokio::test]
    async fn test_async_read_eof_after_data() {
        let test_data = TestData {
            id: 123,
            name: "last_async".to_string(),
            active: false,
        };

        // Write one block
        let mut buffer = Vec::new();
        let mut writer = CborSeqWriter::new(&mut buffer);
        writer.write_block(&test_data).unwrap();
        writer.flush().unwrap();

        // Create an async reader from the buffer
        let async_cursor = tokio::io::BufReader::new(&buffer[..]);
        let mut reader = AsyncCborSeqReader::new(async_cursor);

        let result1: TestData = reader.read_block_async().await.unwrap();
        assert_eq!(test_data, result1);

        let result2: Result<TestData, Error> = reader.read_block_async().await;
        assert!(matches!(result2, Err(Error::Eof)));
    }

    #[tokio::test]
    async fn test_async_read_trait() {
        use tokio::io::AsyncReadExt;

        let test_data = TestData {
            id: 99,
            name: "trait_test".to_string(),
            active: true,
        };

        // Write data
        let mut buffer = Vec::new();
        let mut writer = CborSeqWriter::new(&mut buffer);
        writer.write_block(&test_data).unwrap();
        writer.flush().unwrap();

        // Test AsyncRead trait by reading raw bytes through the trait
        let async_cursor = tokio::io::BufReader::new(&buffer[..]);
        let mut reader = AsyncCborSeqReader::new(async_cursor);

        let mut read_buffer = Vec::new();
        reader.read_to_end(&mut read_buffer).await.unwrap();

        // Should read the same data that was written
        assert_eq!(read_buffer, buffer);
    }
}
