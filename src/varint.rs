use std::io;
use thiserror::Error;

/// Read a leb128 (unsigned-varint) as a usize from a reader.
pub fn read_varint_usize(reader: &mut impl std::io::Read) -> Result<usize, Error> {
    let size = unsigned_varint::io::read_usize(reader)?;
    Ok(size)
}

/// Write a usize as a leb128 (unsigned-varint) to a writer.
/// Returns the number of bytes written.
pub fn write_usize_varint(writer: &mut impl std::io::Write, value: usize) -> Result<usize, Error> {
    let mut buf = unsigned_varint::encode::usize_buffer();
    let to_write = unsigned_varint::encode::usize(value, &mut buf);
    writer.write_all(&to_write)?;
    Ok(to_write.len())
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
    #[error("Error decoding unsigned varint: {0}")]
    UnsignedVarIntDecode(unsigned_varint::decode::Error),
    #[error("Error: {0}")]
    Other(String),
}

impl From<unsigned_varint::io::ReadError> for Error {
    fn from(err: unsigned_varint::io::ReadError) -> Self {
        match err {
            unsigned_varint::io::ReadError::Io(err) => Error::Io(err),
            unsigned_varint::io::ReadError::Decode(err) => Error::UnsignedVarIntDecode(err),
            _ => Error::Other(format!("Unknown error: {}", err)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_read_varint_usize() {
        // Test case 1: Single byte varint (value 1)
        let data = vec![0x01];
        let mut reader = Cursor::new(data);
        let result = read_varint_usize(&mut reader).unwrap();
        assert_eq!(result, 1);

        // Test case 2: Two byte varint (value 128)
        let data = vec![0x80, 0x01];
        let mut reader = Cursor::new(data);
        let result = read_varint_usize(&mut reader).unwrap();
        assert_eq!(result, 128);

        // Test case 3: Multi-byte varint (value 300)
        let data = vec![0xAC, 0x02];
        let mut reader = Cursor::new(data);
        let result = read_varint_usize(&mut reader).unwrap();
        assert_eq!(result, 300);

        // Test case 4: Empty reader should fail
        let data = vec![];
        let mut reader = Cursor::new(data);
        let result = read_varint_usize(&mut reader);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_varint_usize_consumes_only_needed_bytes() {
        // Prepare data with a varint followed by other data
        let data = vec![0x42, 0xFF, 0xFF]; // 0x42 is a single-byte varint (66), followed by other bytes
        let mut reader = Cursor::new(data);

        // Read the varint
        let result = read_varint_usize(&mut reader).unwrap();

        // Verify the correct value was read
        assert_eq!(result, 66);

        // Check that only one byte was consumed by checking the position
        assert_eq!(reader.position(), 1);

        // Test with a multi-byte varint
        let data = vec![0x80, 0x01, 0xFF, 0xFF]; // Two-byte varint (128) followed by other bytes
        let mut reader = Cursor::new(data);

        let result = read_varint_usize(&mut reader).unwrap();

        assert_eq!(result, 128);
        assert_eq!(reader.position(), 2); // Should have consumed exactly 2 bytes
    }

    #[test]
    fn test_write_usize_varint() {
        let mut buf: Vec<u8> = Vec::new();
        write_usize_varint(&mut buf, 128).unwrap();
        assert_eq!(buf, vec![0x80, 0x01]);
    }
}
