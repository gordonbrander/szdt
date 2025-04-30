use std::io::{self, Read};

/// A wrapper around a reader that counts the number of bytes read.
pub(crate) struct ByteCounterReader<R> {
    reader: R,
    /// The number of bytes that have been read so far.
    read_size: usize,
}

impl<R> ByteCounterReader<R> {
    /// Creates a new `ByteCountingReader` wrapping the given reader.
    pub(crate) fn new(reader: R) -> Self {
        ByteCounterReader {
            reader,
            read_size: 0,
        }
    }

    /// Returns the number of bytes that have been read so far.
    pub(crate) fn read_size(&self) -> usize {
        self.read_size
    }
}

impl<R: Read> Read for ByteCounterReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let bytes = self.reader.read(buf)?;
        self.read_size += bytes;
        Ok(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufReader, Cursor};

    #[test]
    fn test_byte_counter_reader_counts_the_bytes() {
        let data = b"hello world";
        let cursor = Cursor::new(data);
        let mut reader = ByteCounterReader::new(cursor);

        // Read a few bytes
        let mut buf = [0u8; 5];
        let bytes_read = reader.read(&mut buf).unwrap();
        assert_eq!(bytes_read, 5);
        assert_eq!(&buf, b"hello");
        assert_eq!(reader.read_size(), 5);

        // Read the rest
        let mut buf = [0u8; 10];
        let bytes_read = reader.read(&mut buf).unwrap();
        assert_eq!(bytes_read, 6);
        assert_eq!(&buf[..bytes_read], b" world");
        assert_eq!(reader.read_size(), 11);

        // Try to read more (should return 0 bytes read)
        let mut buf = [0u8; 5];
        let bytes_read = reader.read(&mut buf).unwrap();
        assert_eq!(bytes_read, 0);
        assert_eq!(reader.read_size(), 11);
    }

    #[test]
    fn test_empty_byte_counter_reader_counts_zero() {
        let data = b"";
        let cursor = Cursor::new(data);
        let mut reader = ByteCounterReader::new(cursor);

        let mut buf = [0u8; 5];
        let bytes_read = reader.read(&mut buf).unwrap();
        assert_eq!(bytes_read, 0);
        assert_eq!(reader.read_size(), 0);
    }

    #[test]
    fn test_byte_counter_reader_plays_well_with_bufreader() {
        let data = b"hello world";
        let cursor = Cursor::new(data);
        let bufcursor = BufReader::new(cursor);
        let mut reader = ByteCounterReader::new(bufcursor);

        let mut buf = [0u8; 5];
        let bytes_read = reader.read(&mut buf).unwrap();
        assert_eq!(bytes_read, 5);
        assert_eq!(&buf, b"hello");
        assert_eq!(reader.read_size(), 5);

        let mut buf = [0u8; 10];
        let bytes_read = reader.read(&mut buf).unwrap();
        assert_eq!(bytes_read, 6);
        assert_eq!(&buf[..bytes_read], b" world");
        assert_eq!(reader.read_size(), 11);

        let mut buf = [0u8; 5];
        let bytes_read = reader.read(&mut buf).unwrap();
        assert_eq!(bytes_read, 0);
        assert_eq!(reader.read_size(), 11);
    }
}
